//! Coordinate-sorted BAM pileup engine — a Rust port of htslib's `bam_plp`
//! (htslib `sam.c`: `bam_plp_push` / `bam_plp64_next` / `resolve_cigar2` /
//! `overlap_push` / `tweak_overlap_quality`).
//!
//! Given a stream of coordinate-sorted records the engine yields one
//! [`Column`] per covered reference position, each carrying the reads
//! overlapping that position together with their CIGAR-resolved per-read state
//! (`qpos`, `is_del`, `is_refskip`, `indel`, `is_head`, `is_tail`). This is the
//! exact state `samtools mpileup` and `samtools consensus` consume to build a
//! pileup column, so both tools build on this one engine (Layer A; B never
//! depends on B).
//!
//! Records are read via [`rsomics_bamio::raw::RawRecord`] — no decode of
//! seq/cigar/qual into noodles types. Each buffered read caches its decoded
//! CIGAR once so the per-position cigar walk is O(1) amortised, matching
//! htslib's pointer-indexed `resolve_cigar2`. The one mutation the engine
//! performs is overlapping-mate quality removal
//! ([`PileupEngine::feed`] → `tweak_overlap_quality`), written back into the
//! record's raw payload exactly as htslib does, so a consumer reading
//! `quality_scores()` afterwards sees the values samtools would.
//!
//! Structurally mirrors htslib: a buffer of active reads with an emit cursor
//! (`tid`, `pos`) advancing one position at a time, per-read cigar walk state
//! carried across positions, and a name → buffered-read map so the second mate
//! of a proper pair tweaks the first's qualities on arrival.

use std::collections::HashMap;

use rsomics_bamio::raw::RawRecord;

// SAM/BAM FLAG bits (SAMv1 §1.4).
const FLAG_PAIRED: u16 = 0x1;
const FLAG_PROPER_PAIR: u16 = 0x2;
const FLAG_UNMAPPED: u16 = 0x4;
const FLAG_MATE_UNMAPPED: u16 = 0x8;

// CIGAR op codes (BAM packed encoding, low nibble): M=0 I=1 D=2 N=3 S=4 H=5 P=6 ==7 X=8.
const CIGAR_MATCH: u8 = 0;
const CIGAR_INS: u8 = 1;
const CIGAR_DEL: u8 = 2;
const CIGAR_REF_SKIP: u8 = 3;
const CIGAR_SOFT_CLIP: u8 = 4;
const CIGAR_HARD_CLIP: u8 = 5;
const CIGAR_PAD: u8 = 6;
const CIGAR_EQUAL: u8 = 7;
const CIGAR_DIFF: u8 = 8;

fn is_match_op(op: u8) -> bool {
    matches!(op, CIGAR_MATCH | CIGAR_EQUAL | CIGAR_DIFF)
}

fn consumes_ref(op: u8) -> bool {
    matches!(
        op,
        CIGAR_MATCH | CIGAR_DEL | CIGAR_REF_SKIP | CIGAR_EQUAL | CIGAR_DIFF
    )
}

/// Per-read CIGAR walk state, carried across emit positions so `resolve_cigar2`
/// advances incrementally (htslib `cstate_t`: `k`/`x`/`y`). `k` is the index of
/// the CIGAR op currently being processed, `x` its reference start, `y` its
/// query start; `k == -1` means the read has not been processed yet.
#[derive(Clone, Copy, Debug)]
struct CigarState {
    k: i64,
    x: i64,
    y: i64,
}

impl Default for CigarState {
    fn default() -> Self {
        Self { k: -1, x: 0, y: 0 }
    }
}

/// One read's resolved state at a given pileup position. The base/quality are
/// read from the [`RawRecord`] by index: at a match position `seq_nibble(qpos)`
/// / `quality_scores()[qpos]`; at a deletion (`is_del`) `qpos` points at the
/// last query base before the gap. `indel > 0` is an insertion of that length
/// immediately after this position; `indel < 0` is a deletion of `-indel`
/// reference bases immediately after this position.
#[derive(Clone, Copy, Debug)]
pub struct PileupRead {
    /// Index of the read in the column's `records` slice.
    pub read_index: usize,
    pub qpos: usize,
    pub is_del: bool,
    pub is_refskip: bool,
    pub is_head: bool,
    pub is_tail: bool,
    pub indel: i64,
}

/// A buffered read in the engine's active set.
struct Node {
    rec: RawRecord,
    tid: i32,
    /// First reference position the read covers (htslib `lbnode_t::beg`, the
    /// 0-based alignment start).
    beg: i64,
    /// Exclusive reference end (htslib `lbnode_t::end`, `bam_endpos`).
    end: i64,
    /// Decoded CIGAR `(op, len)` cached once at feed time so the per-position
    /// walk indexes in O(1) rather than re-walking the packed bytes.
    cigar: Vec<(u8, i64)>,
    state: CigarState,
}

/// `bam_endpos`: 0-based exclusive reference end = pos + reference span, with a
/// 1-bp floor for an empty/clip-only CIGAR (htslib).
fn end_pos(beg: i64, cigar: &[(u8, i64)]) -> i64 {
    let span: i64 = cigar
        .iter()
        .filter(|(op, _)| consumes_ref(*op))
        .map(|(_, len)| len)
        .sum();
    beg + if span > 0 { span } else { 1 }
}

/// Engine options. The pileup itself only needs the overlap toggle and the
/// orphan/flag filters; base-quality and mapq-char thresholds are applied by the
/// consumer (they affect the encoded column, not which positions exist), exactly
/// as in `bam_plcmd.c` where `min_baseQ` gates per-base inside the output loop.
#[derive(Clone, Debug)]
pub struct PileupOpts {
    /// Remove overlapping-mate base qualities (htslib `MPLP_SMART_OVERLAPS`,
    /// `samtools mpileup` default ON).
    pub smart_overlaps: bool,
    /// Skip PAIRED reads that are not PROPER_PAIR (htslib `MPLP_NO_ORPHAN`,
    /// `samtools mpileup` default ON).
    pub no_orphan: bool,
    /// Minimum MAPQ; reads below are skipped (htslib `min_mq`, default 0).
    pub min_mapq: u8,
    /// FLAG bits any of which exclude a read (htslib `rflag_filter`, default
    /// `UNMAP|SECONDARY|QCFAIL|DUP` = 0x704).
    pub rflag_filter: u16,
    /// FLAG bits all-absent → exclude (htslib `rflag_require`, default 0 = off).
    pub rflag_require: u16,
}

impl Default for PileupOpts {
    fn default() -> Self {
        Self {
            smart_overlaps: true,
            no_orphan: true,
            min_mapq: 0,
            rflag_filter: 0x704,
            rflag_require: 0,
        }
    }
}

/// A pileup column: every read overlapping reference position `pos` on `tid`,
/// with its resolved per-read state. `records[i]` is the record for
/// `reads[i].read_index` (i.e. `read_index == i`); records borrow the engine's
/// buffer, so no per-position copy is made (qualities already reflect any
/// overlap tweak). The two slices are index-aligned.
pub struct Column<'a> {
    pub tid: i32,
    pub pos: i64,
    pub records: Vec<&'a RawRecord>,
    pub reads: Vec<PileupRead>,
}

/// Streaming pileup over a coordinate-sorted record source.
///
/// `feed` pushes one record (applying read-level filters and overlap
/// bookkeeping); `next_column` drains ready columns. The canonical driver is
/// [`run`]: feed until the buffer's furthest start passes the emit cursor, drain
/// ready columns, repeat; at end of input `finish` then drain the tail.
pub struct PileupEngine {
    opts: PileupOpts,
    nodes: Vec<Node>,
    seeded: bool,
    /// Emit cursor (htslib `iter->tid` / `iter->pos`).
    tid: i32,
    pos: i64,
    /// Furthest (tid, pos) start of any fed read (htslib `max_tid`/`max_pos`).
    max_tid: i32,
    max_pos: i64,
    is_eof: bool,
    /// name → index in `nodes` of the still-awaited first mate of a pair.
    overlaps: HashMap<Vec<u8>, usize>,
    /// Reusable scratch: the `nodes` indices of the reads covering the column
    /// most recently produced by [`step`], aligned with the emitted `reads`.
    column_idx: Vec<usize>,
}

impl PileupEngine {
    pub fn new(opts: PileupOpts) -> Self {
        Self {
            opts,
            nodes: Vec::new(),
            seeded: false,
            tid: 0,
            pos: 0,
            max_tid: 0,
            max_pos: 0,
            is_eof: false,
            overlaps: HashMap::new(),
            column_idx: Vec::new(),
        }
    }

    fn passes_filters(&self, rec: &RawRecord) -> bool {
        let flag = rec.flags();
        // bam_plp_push skips tid<0 / FUNMAP; mplp_func then applies the flag
        // filters, mapq and orphan rule (bam_plcmd.c:413-458) in this order.
        if rec.reference_sequence_id() < 0 || flag & FLAG_UNMAPPED != 0 {
            return false;
        }
        if self.opts.rflag_require != 0 && flag & self.opts.rflag_require == 0 {
            return false;
        }
        if self.opts.rflag_filter != 0 && flag & self.opts.rflag_filter != 0 {
            return false;
        }
        if rec.mapping_quality() < self.opts.min_mapq {
            return false;
        }
        if self.opts.no_orphan && flag & FLAG_PAIRED != 0 && flag & FLAG_PROPER_PAIR == 0 {
            return false;
        }
        true
    }

    /// Push one record into the active set (htslib `bam_plp_push`). Filtered-out
    /// records still drop their overlap bookkeeping, matching htslib's
    /// `overlap_remove` on skip.
    pub fn feed(&mut self, rec: RawRecord) {
        if !self.passes_filters(&rec) {
            if self.opts.smart_overlaps {
                self.overlaps.remove(rec.name());
            }
            return;
        }
        let tid = rec.reference_sequence_id();
        let beg = i64::from(rec.alignment_start());
        let cigar: Vec<(u8, i64)> = rec.cigar_ops().map(|(o, l)| (o, i64::from(l))).collect();
        let end = end_pos(beg, &cigar);

        let idx = self.nodes.len();
        self.nodes.push(Node {
            rec,
            tid,
            beg,
            end,
            cigar,
            state: CigarState::default(),
        });

        self.max_tid = tid;
        self.max_pos = beg;
        if !self.seeded {
            self.tid = tid;
            self.pos = beg;
            self.seeded = true;
        }

        if self.opts.smart_overlaps {
            self.overlap_push(idx);
        }
    }

    /// htslib `overlap_push`: when the second mate of a proper pair arrives,
    /// tweak the first mate's overlapping-base qualities.
    fn overlap_push(&mut self, idx: usize) {
        let node = &self.nodes[idx];
        let flag = node.rec.flags();
        if flag & FLAG_MATE_UNMAPPED != 0 || flag & FLAG_PROPER_PAIR == 0 {
            return;
        }
        let mtid = node.rec.mate_reference_sequence_id();
        let isize = i64::from(node.rec.template_length());
        let l_qseq = node.rec.sequence_len() as i64;
        let mpos = i64::from(node.rec.mate_alignment_start());
        if (mtid >= 0 && node.tid != mtid) || (isize.abs() >= 2 * l_qseq && mpos >= node.end) {
            return;
        }
        let name = node.rec.name().to_vec();
        match self.overlaps.get(&name).copied() {
            None => {
                let beg = node.beg;
                if mpos >= beg || (flag & FLAG_PAIRED != 0 && mpos == -1) {
                    self.overlaps.insert(name, idx);
                }
            }
            Some(first_idx) => {
                self.overlaps.remove(&name);
                self.tweak_overlap(first_idx, idx);
            }
        }
    }

    /// Apply [`tweak_overlap_quality`] to the two mate nodes. Splits the slice so
    /// both can be borrowed mutably at once.
    fn tweak_overlap(&mut self, a_idx: usize, b_idx: usize) {
        let (lo, hi) = (a_idx.min(b_idx), a_idx.max(b_idx));
        let (left, right) = self.nodes.split_at_mut(hi);
        let (a, b) = if a_idx < b_idx {
            (&mut left[lo], &mut right[0])
        } else {
            (&mut right[0], &mut left[lo])
        };
        tweak_overlap_quality(a, b);
    }

    /// Signal that no more records will be fed (htslib `bam_plp_push(NULL)`).
    pub fn finish(&mut self) {
        self.is_eof = true;
    }

    /// Resolve the next ready column into `reads` (cleared first) and return its
    /// `(tid, pos)`, or `None` when the buffer cannot yet advance (more input
    /// needed) or is exhausted at EOF. Mirrors htslib `bam_plp64_next`: prune
    /// ended reads, collect those covering the cursor, resolve each one's cigar,
    /// advance the cursor. After this returns `Some`, `reads[i].read_index` is
    /// `i` into [`record`](Self::record)'s 0-based live order — use [`record`] to
    /// borrow each read's record without copying.
    fn step(&mut self, reads: &mut Vec<PileupRead>) -> Option<(i32, i64)> {
        loop {
            if self.is_eof && self.nodes.is_empty() {
                return None;
            }
            let can_emit = self.is_eof
                || self.max_tid > self.tid
                || (self.max_tid == self.tid && self.max_pos > self.pos);
            if !can_emit {
                return None;
            }

            let cur_tid = self.tid;
            let cur_pos = self.pos;
            let pruned_any = self
                .nodes
                .iter()
                .any(|node| node.tid < cur_tid || (node.tid == cur_tid && node.end <= cur_pos));
            if pruned_any {
                self.prune(cur_tid, cur_pos);
            }

            reads.clear();
            self.column_idx.clear();
            // The cursor's covering reads are resolved in buffer order; their
            // `nodes` indices are recorded in `column_idx` so `record(i)` is O(1)
            // and `advance_cursor` (which never reorders nodes) keeps them valid.
            let mut covering = 0usize;
            for (slot, node) in self.nodes.iter_mut().enumerate() {
                if node.tid == cur_tid && node.beg <= cur_pos {
                    let resolved = resolve_cigar2(node, cur_pos, covering);
                    reads.push(resolved);
                    self.column_idx.push(slot);
                    covering += 1;
                }
            }

            let have_reads = !reads.is_empty();
            self.advance_cursor();

            if have_reads {
                return Some((cur_tid, cur_pos));
            }
            if self.is_eof && self.nodes.is_empty() {
                return None;
            }
        }
    }

    /// The record of the `i`-th read in the most recently stepped column,
    /// borrowing the engine's buffer (no copy).
    fn record(&self, i: usize) -> &RawRecord {
        &self.nodes[self.column_idx[i]].rec
    }

    /// Drop reads ending at or before `(tid, pos)`, keeping `overlaps` indices
    /// consistent with the compacted arena. `new_index[old] = Some(new)` for a
    /// kept read, `None` for a dropped one, so the overlap map can be remapped.
    fn prune(&mut self, tid: i32, pos: i64) {
        let mut new_index: Vec<Option<usize>> = Vec::with_capacity(self.nodes.len());
        let mut write = 0;
        for node in &self.nodes {
            let ended = node.tid < tid || (node.tid == tid && node.end <= pos);
            new_index.push((!ended).then(|| {
                let ni = write;
                write += 1;
                ni
            }));
        }
        // Compact the kept nodes to the front in their original order.
        let mut keep_iter = new_index.iter();
        self.nodes.retain(|_| keep_iter.next().unwrap().is_some());

        if self.opts.smart_overlaps && !self.overlaps.is_empty() {
            self.overlaps
                .retain(|_, idx| match new_index.get(*idx).copied().flatten() {
                    Some(ni) => {
                        *idx = ni;
                        true
                    }
                    None => false,
                });
        }
    }

    /// Recompute the head (lowest-coordinate live read) and step the emit cursor
    /// (htslib `bam_plp64_next` tail).
    fn advance_cursor(&mut self) {
        let head = self
            .nodes
            .iter()
            .min_by(|a, b| (a.tid, a.beg).cmp(&(b.tid, b.beg)));
        match head {
            Some(h) => {
                if self.tid < h.tid {
                    self.tid = h.tid;
                    self.pos = h.beg;
                } else if self.pos < h.beg {
                    self.pos = h.beg;
                } else {
                    self.pos += 1;
                }
            }
            None => self.pos += 1,
        }
    }
}

/// htslib `resolve_cigar2`: advance the read's cigar walk to `pos` and fill in
/// its per-position state. `read_index` is the caller's slot in the column.
fn resolve_cigar2(node: &mut Node, pos: i64, read_index: usize) -> PileupRead {
    let cigar = &node.cigar;
    let n_cigar = cigar.len();
    let core_pos = node.beg;
    let s = &mut node.state;

    if s.k == -1 {
        s.x = core_pos;
        s.y = 0;
        if n_cigar == 1 {
            if is_match_op(cigar[0].0) {
                s.k = 0;
            }
        } else {
            let mut k = 0;
            while k < n_cigar {
                let (op, l) = cigar[k];
                if matches!(
                    op,
                    CIGAR_MATCH | CIGAR_DEL | CIGAR_REF_SKIP | CIGAR_EQUAL | CIGAR_DIFF
                ) {
                    break;
                } else if matches!(op, CIGAR_INS | CIGAR_SOFT_CLIP) {
                    s.y += l;
                }
                k += 1;
            }
            s.k = k as i64;
        }
    } else {
        let k = s.k as usize;
        let (this_op, this_l) = cigar[k];
        if pos - s.x >= this_l {
            let (next_op, _) = cigar[k + 1];
            if matches!(
                next_op,
                CIGAR_MATCH | CIGAR_DEL | CIGAR_REF_SKIP | CIGAR_EQUAL | CIGAR_DIFF
            ) {
                if is_match_op(this_op) {
                    s.y += this_l;
                }
                s.x += this_l;
                s.k += 1;
            } else {
                if is_match_op(this_op) {
                    s.y += this_l;
                }
                s.x += this_l;
                let mut kk = k + 1;
                while kk < n_cigar {
                    let (op, l) = cigar[kk];
                    if matches!(
                        op,
                        CIGAR_MATCH | CIGAR_DEL | CIGAR_REF_SKIP | CIGAR_EQUAL | CIGAR_DIFF
                    ) {
                        break;
                    } else if matches!(op, CIGAR_INS | CIGAR_SOFT_CLIP) {
                        s.y += l;
                    }
                    kk += 1;
                }
                s.k = kk as i64;
            }
        }
    }

    let k = s.k as usize;
    let (op, l) = cigar[k];
    let mut is_del = false;
    let mut is_refskip = false;
    let mut indel: i64 = 0;
    let mut qpos: usize = 0;

    if s.x + l - 1 == pos && k + 1 < n_cigar {
        let (op2, l2) = cigar[k + 1];
        if op2 == CIGAR_DEL && op != CIGAR_DEL {
            indel = -l2;
            let mut kk = k + 2;
            while kk < n_cigar {
                let (o, ll) = cigar[kk];
                if o == CIGAR_DEL {
                    indel -= ll;
                } else {
                    break;
                }
                kk += 1;
            }
        } else if op2 == CIGAR_INS {
            indel = l2;
            let mut kk = k + 2;
            while kk < n_cigar {
                let (o, ll) = cigar[kk];
                if o == CIGAR_INS {
                    indel += ll;
                } else if o != CIGAR_PAD {
                    break;
                }
                kk += 1;
            }
        } else if op2 == CIGAR_PAD && k + 2 < n_cigar {
            let mut l3 = 0;
            let mut kk = k + 2;
            while kk < n_cigar {
                let (o, ll) = cigar[kk];
                if o == CIGAR_INS {
                    l3 += ll;
                } else if matches!(
                    o,
                    CIGAR_DEL | CIGAR_MATCH | CIGAR_REF_SKIP | CIGAR_EQUAL | CIGAR_DIFF
                ) {
                    break;
                }
                kk += 1;
            }
            if l3 > 0 {
                indel = l3;
            }
        }
    }

    if is_match_op(op) {
        qpos = (s.y + (pos - s.x)) as usize;
    } else if op == CIGAR_DEL || op == CIGAR_REF_SKIP {
        is_del = true;
        qpos = s.y as usize;
        is_refskip = op == CIGAR_REF_SKIP;
    }

    PileupRead {
        read_index,
        qpos,
        is_del,
        is_refskip,
        is_head: pos == core_pos,
        is_tail: pos == node.end - 1,
        indel,
    }
}

// ---------------------------------------------------------------------------
// Overlapping-mate quality removal (htslib tweak_overlap_quality + helpers).
// ---------------------------------------------------------------------------

/// htslib `__ac_X31_hash_string`: a 32-bit string hash (`h = h*31 + c`, wrapping).
fn x31_hash_string(s: &[u8]) -> u32 {
    if s.is_empty() {
        return 0;
    }
    let mut h = u32::from(s[0]);
    if h != 0 {
        for &c in &s[1..] {
            h = h.wrapping_shl(5).wrapping_sub(h).wrapping_add(u32::from(c));
        }
    }
    h
}

/// htslib `__ac_Wang_hash`: a 32-bit integer mixer (wrapping).
fn wang_hash(mut key: u32) -> u32 {
    key = key.wrapping_add(!key.wrapping_shl(15));
    key ^= key >> 10;
    key = key.wrapping_add(key.wrapping_shl(3));
    key ^= key >> 6;
    key = key.wrapping_add(!key.wrapping_shl(11));
    key ^= key >> 16;
    key
}

/// A standalone cigar walk cursor over a record, mirroring htslib's
/// `cigar`/`icig`/`iseq`/`iref` quartet but cached against a decoded cigar slice.
/// `ck` is the current op index, `icig` the offset into it, `iseq` the query
/// index, `iref` the reference offset from the read start.
struct IrefCursor<'a> {
    cigar: &'a [(u8, i64)],
    ck: usize,
    icig: i64,
    iseq: i64,
    iref: i64,
}

impl<'a> IrefCursor<'a> {
    /// htslib `cigar_iref2iseq_set`: position at the first M base at/after
    /// reference offset `target_iref`. Returns false if not covered.
    fn set(cigar: &'a [(u8, i64)], target_iref: i64) -> Option<Self> {
        let mut pos = target_iref;
        if pos < 0 {
            return None;
        }
        let mut ck = 0;
        let mut iseq = 0i64;
        let mut iref = 0i64;
        while ck < cigar.len() {
            let (op, ncig) = cigar[ck];
            match op {
                CIGAR_SOFT_CLIP => {
                    ck += 1;
                    iseq += ncig;
                }
                CIGAR_HARD_CLIP | CIGAR_PAD => {
                    ck += 1;
                }
                CIGAR_MATCH | CIGAR_EQUAL | CIGAR_DIFF => {
                    pos -= ncig;
                    if pos < 0 {
                        let icig = ncig + pos;
                        iseq += icig;
                        iref += icig;
                        return Some(Self {
                            cigar,
                            ck,
                            icig,
                            iseq,
                            iref,
                        });
                    }
                    ck += 1;
                    iseq += ncig;
                    iref += ncig;
                }
                CIGAR_INS => {
                    ck += 1;
                    iseq += ncig;
                }
                CIGAR_DEL | CIGAR_REF_SKIP => {
                    pos -= ncig;
                    if pos < 0 {
                        pos = 0;
                    }
                    ck += 1;
                    iref += ncig;
                }
                _ => return None,
            }
        }
        None
    }

    /// htslib `cigar_iref2iseq_next`: step to the next M base. Returns false at
    /// end of cigar (`iseq`/`iref` set to -1).
    fn next(&mut self) -> bool {
        while self.ck < self.cigar.len() {
            let (op, ncig) = self.cigar[self.ck];
            match op {
                CIGAR_MATCH | CIGAR_EQUAL | CIGAR_DIFF => {
                    if self.icig >= ncig - 1 {
                        self.icig = -1;
                        self.ck += 1;
                        continue;
                    }
                    self.iseq += 1;
                    self.icig += 1;
                    self.iref += 1;
                    return true;
                }
                CIGAR_DEL | CIGAR_REF_SKIP => {
                    self.ck += 1;
                    self.iref += ncig;
                    self.icig = -1;
                }
                CIGAR_INS => {
                    self.ck += 1;
                    self.iseq += ncig;
                    self.icig = -1;
                }
                CIGAR_SOFT_CLIP => {
                    self.ck += 1;
                    self.iseq += ncig;
                    self.icig = -1;
                }
                CIGAR_HARD_CLIP | CIGAR_PAD => {
                    self.ck += 1;
                    self.icig = -1;
                }
                _ => return false,
            }
        }
        self.iseq = -1;
        self.iref = -1;
        false
    }

    /// Whether the op the cursor just advanced past was a deletion (htslib peeks
    /// `cigar[-1]` for the del catch-up).
    fn prev_was_del(&self) -> bool {
        self.ck > 0 && self.cigar[self.ck - 1].0 == CIGAR_DEL
    }
}

/// htslib `tweak_overlap_quality`: adjust the qualities of the overlapping bases
/// of mate reads `a` (left) and `b` (right). Matching bases give one mate the
/// summed quality (capped at 200) and zero the other; the keeper is chosen by
/// the name hash. Mismatches zero the lower-quality base and scale the higher by
/// 0.8. Deletions in one mate catch the other up, scaling/zeroing as it goes.
fn tweak_overlap_quality(a: &mut Node, b: &mut Node) {
    let a_pos = a.beg;
    let b_pos = b.beg;
    let iref0 = b_pos;

    let Some(mut a_cur) = IrefCursor::set(&a.cigar, iref0 - a_pos) else {
        return;
    };
    let Some(mut b_cur) = IrefCursor::set(&b.cigar, iref0 - b_pos) else {
        return;
    };

    let hash_a = wang_hash(x31_hash_string(a.rec.name()));
    let (amul, bmul): (i32, i32) = if hash_a & 1 != 0 { (1, 0) } else { (0, 1) };

    let a_len = a.rec.sequence_len() as i64;
    let b_len = b.rec.sequence_len() as i64;
    let mut iref = iref0;

    loop {
        while a_cur.iref >= 0 && a_cur.iref < iref - a_pos {
            if !a_cur.next() {
                return;
            }
        }
        if a_cur.iref < 0 {
            return;
        }
        while b_cur.iref >= 0 && b_cur.iref < iref - b_pos {
            if !b_cur.next() {
                return;
            }
        }
        if b_cur.iref < 0 {
            return;
        }

        if iref < a_cur.iref + a_pos {
            iref = a_cur.iref + a_pos;
        }
        if iref < b_cur.iref + b_pos {
            iref = b_cur.iref + b_pos;
        }
        iref += 1;

        // Deletion catch-up (htslib): when the two mates land on different
        // reference positions because one had a deletion, walk the lagging mate
        // forward, scaling/zeroing as for a mismatch, until they realign.
        if a_cur.iref + a_pos != b_cur.iref + b_pos {
            if a_cur.iref + a_pos < b_cur.iref + b_pos && b_cur.prev_was_del() {
                loop {
                    let q = a.rec.quality_scores_mut()[a_cur.iseq as usize];
                    a.rec.quality_scores_mut()[a_cur.iseq as usize] =
                        if amul != 0 { (q as f64 * 0.8) as u8 } else { 0 };
                    if !a_cur.next() {
                        return;
                    }
                    if a_cur.iref + a_pos >= b_cur.iref + b_pos {
                        break;
                    }
                }
            } else if a_cur.prev_was_del() {
                loop {
                    let q = b.rec.quality_scores_mut()[b_cur.iseq as usize];
                    b.rec.quality_scores_mut()[b_cur.iseq as usize] =
                        if bmul != 0 { (q as f64 * 0.8) as u8 } else { 0 };
                    if !b_cur.next() {
                        return;
                    }
                    if b_cur.iref + b_pos >= a_cur.iref + a_pos {
                        break;
                    }
                }
            } else {
                continue;
            }
        }

        if a_cur.iseq > a_len || b_cur.iseq > b_len {
            return;
        }

        let a_base = a.rec.seq_nibble(a_cur.iseq as usize);
        let b_base = b.rec.seq_nibble(b_cur.iseq as usize);
        let a_q = i32::from(a.rec.quality_scores_mut()[a_cur.iseq as usize]);
        let b_q = i32::from(b.rec.quality_scores_mut()[b_cur.iseq as usize]);

        if a_base == b_base {
            let qual = (a_q + b_q).min(200);
            a.rec.quality_scores_mut()[a_cur.iseq as usize] = (amul * qual) as u8;
            b.rec.quality_scores_mut()[b_cur.iseq as usize] = (bmul * qual) as u8;
        } else if a_q > b_q {
            a.rec.quality_scores_mut()[a_cur.iseq as usize] = (0.8 * a_q as f64) as u8;
            b.rec.quality_scores_mut()[b_cur.iseq as usize] = 0;
        } else if a_q < b_q {
            b.rec.quality_scores_mut()[b_cur.iseq as usize] = (0.8 * b_q as f64) as u8;
            a.rec.quality_scores_mut()[a_cur.iseq as usize] = 0;
        } else {
            a.rec.quality_scores_mut()[a_cur.iseq as usize] =
                (amul as f64 * 0.8 * a_q as f64) as u8;
            b.rec.quality_scores_mut()[b_cur.iseq as usize] =
                (bmul as f64 * 0.8 * b_q as f64) as u8;
        }
    }
}

impl PileupEngine {
    /// Drain every column whose position is now safe to emit, calling `emit` for
    /// each. Reuses an internal `reads` buffer across columns. A column borrows
    /// the engine's record buffer for the duration of the `emit` call.
    fn drain<F, E>(&mut self, reads: &mut Vec<PileupRead>, emit: &mut F) -> Result<(), E>
    where
        F: FnMut(&Column) -> Result<(), E>,
    {
        while let Some((tid, pos)) = self.step(reads) {
            let records: Vec<&RawRecord> = (0..reads.len()).map(|i| self.record(i)).collect();
            let col = Column {
                tid,
                pos,
                records,
                reads: std::mem::take(reads),
            };
            emit(&col)?;
            *reads = col.reads;
        }
        Ok(())
    }
}

/// Drive the engine over a record source to completion, calling `emit` for each
/// pileup column in coordinate order. `next_record` returns `Ok(None)` at end of
/// input. The canonical consumer loop; mpileup and consensus differ only in
/// `emit`.
pub fn run<F, E>(
    engine: &mut PileupEngine,
    mut next_record: impl FnMut() -> Result<Option<RawRecord>, E>,
    mut emit: F,
) -> Result<(), E>
where
    F: FnMut(&Column) -> Result<(), E>,
{
    let mut reads: Vec<PileupRead> = Vec::new();
    while let Some(rec) = next_record()? {
        engine.feed(rec);
        engine.drain(&mut reads, &mut emit)?;
    }
    engine.finish();
    engine.drain(&mut reads, &mut emit)?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Build a minimal single-block raw BAM record on contig 0 at `pos` (0-based)
    /// with the given flags, cigar ops `(op,len)` and ASCII bases (qualities all
    /// 40, mate fields unset). Round-trips through `read_record` so the field
    /// offsets are exercised the same way the production path constructs records.
    fn make_record(pos: i32, flags: u16, cigar: &[(u8, u32)], bases: &[u8]) -> RawRecord {
        let name = b"r\0";
        let mut payload = Vec::new();
        payload.extend_from_slice(&0i32.to_le_bytes()); // refID = 0
        payload.extend_from_slice(&pos.to_le_bytes());
        payload.push(name.len() as u8); // l_read_name
        payload.push(60); // mapq
        payload.extend_from_slice(&0u16.to_le_bytes()); // bin
        payload.extend_from_slice(&(cigar.len() as u16).to_le_bytes()); // n_cigar
        payload.extend_from_slice(&flags.to_le_bytes());
        payload.extend_from_slice(&(bases.len() as u32).to_le_bytes()); // l_seq
        payload.extend_from_slice(&(-1i32).to_le_bytes()); // next_refID
        payload.extend_from_slice(&(-1i32).to_le_bytes()); // next_pos
        payload.extend_from_slice(&0i32.to_le_bytes()); // tlen
        payload.extend_from_slice(name);
        for &(op, len) in cigar {
            payload.extend_from_slice(&((len << 4) | u32::from(op)).to_le_bytes());
        }
        let nt16 = |b: u8| match b {
            b'A' => 1u8,
            b'C' => 2,
            b'G' => 4,
            b'T' => 8,
            _ => 15,
        };
        for pair in bases.chunks(2) {
            let hi = nt16(pair[0]);
            let lo = if pair.len() == 2 { nt16(pair[1]) } else { 0 };
            payload.push((hi << 4) | lo);
        }
        payload.extend(std::iter::repeat_n(40u8, bases.len()));

        let mut framed = Vec::new();
        framed.extend_from_slice(&(payload.len() as u32).to_le_bytes());
        framed.extend_from_slice(&payload);
        let mut rec = RawRecord::default();
        let mut cur = std::io::Cursor::new(framed);
        rsomics_bamio::raw::read_record(&mut cur, &mut rec).unwrap();
        rec
    }

    /// `(qpos, is_del, indel)` for each read in a column.
    type ReadSummary = (usize, bool, i64);
    /// `(position, reads)` for each emitted column.
    type ColumnSummary = (i64, Vec<ReadSummary>);

    fn collect_columns(recs: Vec<RawRecord>, opts: PileupOpts) -> Vec<ColumnSummary> {
        let mut engine = PileupEngine::new(opts);
        let mut it = recs.into_iter();
        let mut out = Vec::new();
        run::<_, ()>(
            &mut engine,
            || Ok(it.next()),
            |col| {
                let reads = col
                    .reads
                    .iter()
                    .map(|r| (r.qpos, r.is_del, r.indel))
                    .collect();
                out.push((col.pos, reads));
                Ok(())
            },
        )
        .unwrap();
        out
    }

    #[test]
    fn single_match_read_covers_each_position() {
        let rec = make_record(4, 0, &[(CIGAR_MATCH, 5)], b"ACGTA");
        let cols = collect_columns(vec![rec], PileupOpts::default());
        let positions: Vec<i64> = cols.iter().map(|(p, _)| *p).collect();
        assert_eq!(positions, vec![4, 5, 6, 7, 8]);
        for (i, (_, reads)) in cols.iter().enumerate() {
            assert_eq!(reads.len(), 1);
            assert_eq!(reads[0], (i, false, 0));
        }
    }

    #[test]
    fn deletion_marks_is_del_and_indel() {
        // 2M2D2M at pos 0: pos 1 (last match before del) carries indel=-2; pos
        // 2,3 are is_del; pos 4 resumes matching at query base 2.
        let rec = make_record(
            0,
            0,
            &[(CIGAR_MATCH, 2), (CIGAR_DEL, 2), (CIGAR_MATCH, 2)],
            b"ACGT",
        );
        let cols = collect_columns(vec![rec], PileupOpts::default());
        let by_pos: std::collections::HashMap<i64, (usize, bool, i64)> =
            cols.iter().map(|(p, r)| (*p, r[0])).collect();
        assert_eq!(by_pos[&1].2, -2, "base before deletion carries indel=-2");
        assert!(by_pos[&2].1, "first deleted position is_del");
        assert!(by_pos[&3].1, "second deleted position is_del");
        assert!(!by_pos[&4].1, "post-deletion match is not a deletion");
        assert_eq!(
            by_pos[&4].0, 2,
            "post-deletion qpos resumes at query base 2"
        );
    }

    #[test]
    fn insertion_marks_indel_on_preceding_base() {
        // 2M2I2M at pos 0: pos 1 (last match before insert) carries indel=+2; the
        // 2I consumes no reference, so positions are 0,1,2,3.
        let rec = make_record(
            0,
            0,
            &[(CIGAR_MATCH, 2), (CIGAR_INS, 2), (CIGAR_MATCH, 2)],
            b"ACGTAC",
        );
        let cols = collect_columns(vec![rec], PileupOpts::default());
        let by_pos: std::collections::HashMap<i64, (usize, bool, i64)> =
            cols.iter().map(|(p, r)| (*p, r[0])).collect();
        assert_eq!(by_pos[&1].2, 2, "base before insertion carries indel=+2");
        let positions: Vec<i64> = cols.iter().map(|(p, _)| *p).collect();
        assert_eq!(positions, vec![0, 1, 2, 3]);
    }

    #[test]
    fn orphan_filter_drops_improper_pair() {
        let rec = make_record(0, FLAG_PAIRED, &[(CIGAR_MATCH, 3)], b"ACG");
        let cols = collect_columns(vec![rec], PileupOpts::default());
        assert!(cols.is_empty(), "orphan should be filtered out");

        let rec2 = make_record(0, FLAG_PAIRED, &[(CIGAR_MATCH, 3)], b"ACG");
        let keep = PileupOpts {
            no_orphan: false,
            ..PileupOpts::default()
        };
        let cols2 = collect_columns(vec![rec2], keep);
        assert_eq!(cols2.len(), 3, "with no_orphan off the read is kept");
    }
}
