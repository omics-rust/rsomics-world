//! Per-CpG methylation extraction from bisulfite-aligned BAM.
//!
//! Implements MethylDackel `extract` default mode: for each CpG position in
//! the reference, count methylated (cytosine retained) vs unmethylated
//! (C→T converted) calls across all reads that cover that position, then
//! emit a bedGraph line.
//!
//! ## Bisulfite strand determination (`getStrand`, MethylDackel `common.c`)
//!
//! 1 = OT (original top), 2 = OB (original bottom),
//! 3 = CTOT (complementary to OT), 4 = CTOB (complementary to OB).
//!
//! Priority: XG aux tag (`C`→CT-conversion strand, `G`→GA-conversion strand)
//! combined with read flags. Without XG: paired reads use FLAG orientation;
//! unpaired reads: REVERSE flag → OB, else OT.
//!
//! ## Methylation calling (`updateMetrics`, MethylDackel `common.c`)
//!
//! OT/CTOT (strand & 1 != 0): C(nt16=2)→methylated, T(nt16=8)→unmethylated.
//! OB/CTOB (strand & 1 == 0): G(nt16=4)→methylated, A(nt16=1)→unmethylated.
//!
//! ## Default filters (MethylDackel `extract.c`, `extractCalls` init)
//!
//! minMapq=10, minPhred=5, ignoreFlags=0xF00, requireFlags=0, minDepth=1,
//! keepDupes=false, keepSingleton=false, keepDiscordant=false.
//!
//! ## Overlap handling (`cust_tweak_overlap_quality`, MethylDackel `overlaps.c`)
//!
//! MethylDackel's custom overlap handler (distinct from htslib's):
//! - Mismatch, a_qual > b_qual: `a_qual -= b_qual; b_qual = 0`
//! - Mismatch, b_qual > a_qual: `b_qual -= a_qual; a_qual = 0`
//! - Mismatch, equal qual: both → 0
//! - Match, a_qual > b_qual: `a_qual += 0.2*a_qual; b_qual = 0` (capped at 255)
//! - Match, a_qual <= b_qual: `b_qual += 0.2*b_qual; a_qual = 0`
//!
//! Reads on opposite bisulfite strands are not overlap-adjusted.
//!
//! ## Output format (bedGraph, MethylDackel `extract.c` `writeCall`)
//!
//! Header: `track type="bedGraph" description="<prefix> CpG methylation levels"\n`
//! Data: `<chrom>\t<start>\t<end>\t<pct_int>\t<nmeth>\t<nunmeth>\n`
//! where start is 0-based, end = start+1, pct = (int)(100.0*nmeth/(nmeth+nunmeth)).
//! Positions with nmeth+nunmeth==0 after filter are not emitted (minDepth default 1).

#![allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]

use std::collections::HashMap;
use std::io::{BufWriter, Write};
use std::num::NonZero;
use std::path::Path;
use std::sync::Arc;

use rsomics_bamio::raw::{RawRecord, read_record};
use rsomics_common::{Result, RsomicsError};
use serde::Serialize;

// SAM/BAM FLAG bits (SAMv1 §1.4).
const FLAG_PAIRED: u16 = 0x1;
const FLAG_PROPER_PAIR: u16 = 0x2;
const FLAG_UNMAPPED: u16 = 0x4;
const FLAG_MATE_UNMAPPED: u16 = 0x8;
const FLAG_REVERSE: u16 = 0x10;
const FLAG_READ1: u16 = 0x40;
const FLAG_READ2: u16 = 0x80;
const FLAG_SECONDARY: u16 = 0x100;
const FLAG_QCFAIL: u16 = 0x200;
const FLAG_DUP: u16 = 0x400;
const FLAG_SUPPLEMENTARY: u16 = 0x800;

/// Default FLAGS to ignore (secondary|qcfail|dup|supplementary).
pub const DEFAULT_IGNORE_FLAGS: u16 = FLAG_SECONDARY | FLAG_QCFAIL | FLAG_DUP | FLAG_SUPPLEMENTARY;

/// Active-window size at which settled reads are flushed into the counts array.
/// Large enough to amortise the settle scan over many reads, small enough that
/// live memory stays in the deep-coverage band rather than the whole contig.
const WINDOW_FLUSH_THRESHOLD: usize = 1024;

// BAM sequence encoding nibbles (SAMv1 §4.2 Table 1).
const NT16_A: u8 = 1;
const NT16_C: u8 = 2;
const NT16_G: u8 = 4;
const NT16_T: u8 = 8;

// CIGAR op codes (low nibble of packed BAM uint32).
const CIGAR_MATCH: u8 = 0;
const CIGAR_INS: u8 = 1;
const CIGAR_DEL: u8 = 2;
const CIGAR_REF_SKIP: u8 = 3;
const CIGAR_SOFT_CLIP: u8 = 4;
const CIGAR_HARD_CLIP: u8 = 5;
const CIGAR_EQUAL: u8 = 7;
const CIGAR_DIFF: u8 = 8;

/// Runtime options matching MethylDackel's `extract` defaults.
#[derive(Clone, Debug)]
pub struct ExtractOpts {
    /// Minimum mapping quality (-q, default 10).
    pub min_mapq: u8,
    /// Minimum base Phred quality (-p, default 5).
    pub min_phred: u8,
    /// FLAG bits that exclude a read if any are set (default 0xF00).
    pub ignore_flags: u16,
    /// FLAG bits that must all be set (default 0 = disabled).
    pub require_flags: u16,
    /// Minimum total depth to emit a position (default 1).
    pub min_depth: u32,
    /// Output filename prefix for bedGraph (suffix `_CpG.bedGraph` is appended).
    pub output_prefix: String,
}

impl Default for ExtractOpts {
    fn default() -> Self {
        Self {
            min_mapq: 10,
            min_phred: 5,
            ignore_flags: DEFAULT_IGNORE_FLAGS,
            require_flags: 0,
            min_depth: 1,
            output_prefix: "out".into(),
        }
    }
}

#[derive(Debug, Default, Serialize)]
pub struct ExtractStats {
    pub positions_examined: u64,
    pub positions_emitted: u64,
    pub reads_processed: u64,
    pub reads_filtered: u64,
}

/// Determine bisulfite strand (1=OT, 2=OB, 3=CTOT, 4=CTOB, 0=undetermined).
///
/// Exact port of `getStrand` from MethylDackel `common.c`.
fn get_strand(rec: &RawRecord) -> u8 {
    let flag = rec.flags();
    // XG aux tag: 'C' = CT-converted genome (OT/CTOT), 'G' = GA-converted (OB/CTOB).
    // aux_value returns the bytes after the 3-byte header (tag+type), so for Z:CT → b"CT\0".
    let xg: Option<u8> = rec.aux_value(*b"XG").and_then(|v| v.first().copied());
    if let Some(xg_base) = xg {
        if xg_base == b'C' {
            // XG=CT: CT-conversion genome (OT or CTOT strand).
            // flag & 0x51 = flag & (REVERSE|READ1).
            let f51 = flag & 0x51;
            let f91 = flag & 0x91;
            if f51 == 0x41 {
                return 1; // R1 forward = OT
            } else if f51 == 0x51 {
                return 3; // R1 reverse = CTOT
            } else if f91 == 0x81 {
                return 3; // R2 forward = CTOT
            } else if f91 == 0x91 {
                return 1; // R2 reverse = OT
            } else if flag & FLAG_REVERSE != 0 {
                return 3; // unpaired reverse = CTOT
            } else {
                return 1; // unpaired forward = OT
            }
        } else if xg_base == b'G' {
            // XG=GA: GA-conversion genome (OB or CTOB strand).
            let f51 = flag & 0x51;
            let f91 = flag & 0x91;
            if f51 == 0x41 {
                return 4; // R1 forward = CTOB
            } else if f51 == 0x51 {
                return 2; // R1 reverse = OB
            } else if f91 == 0x81 {
                return 2; // R2 forward = OB
            } else if f91 == 0x91 {
                return 4; // R2 reverse = CTOB
            } else if flag & FLAG_REVERSE != 0 {
                return 2; // unpaired reverse = OB
            } else {
                return 4; // unpaired forward = CTOB
            }
        }
    }

    // No XG tag: infer from FLAG orientation.
    if flag & FLAG_PAIRED != 0 {
        let f50 = flag & 0x50;
        let f90 = flag & 0x90;
        // R1 reverse (0x50) = OB; R2 forward (FLAG_READ2, not reverse) = OB.
        // R1 forward (FLAG_READ1, not reverse) = OT; R2 reverse (0x90) = OT.
        if f50 == 0x50 || (flag & FLAG_READ2 != 0 && flag & FLAG_REVERSE == 0) {
            2
        } else if flag & FLAG_READ1 != 0 || f90 == 0x90 {
            1
        } else {
            0
        }
    } else if flag & FLAG_REVERSE != 0 {
        2 // unpaired reverse = OB
    } else {
        1 // unpaired forward = OT
    }
}

/// Per-position methylation call for one pileup read.
///
/// Exact port of `updateMetrics` from MethylDackel `common.c`.
/// Returns Some(true)=methylated, Some(false)=unmethylated, None=skip.
fn update_metrics(qual: u8, min_phred: u8, nt16_base: u8, strand: u8) -> Option<bool> {
    if qual < min_phred {
        return None;
    }
    match strand {
        1 | 3 => {
            // OT/CTOT: C=methylated, T=unmethylated.
            if nt16_base == NT16_C {
                Some(true)
            } else if nt16_base == NT16_T {
                Some(false)
            } else {
                None
            }
        }
        2 | 4 => {
            // OB/CTOB: G=methylated, A=unmethylated.
            if nt16_base == NT16_G {
                Some(true)
            } else if nt16_base == NT16_A {
                Some(false)
            } else {
                None
            }
        }
        _ => None,
    }
}

/// Whether the reference base at `pos` is in a CpG context.
///
/// Port of `isCpG` from MethylDackel `common.c`.
/// Returns 1 if `seq[pos]` is C and `seq[pos+1]` is G (forward CpG),
/// -1 if `seq[pos]` is G and `seq[pos-1]` is C (reverse complement CpG), 0 otherwise.
fn is_cpg(seq: &[u8], pos: usize) -> i8 {
    if pos >= seq.len() {
        return 0;
    }
    let b = seq[pos];
    if b == b'C' || b == b'c' {
        if pos + 1 >= seq.len() {
            return 0;
        }
        let n = seq[pos + 1];
        if n == b'G' || n == b'g' { 1 } else { 0 }
    } else if b == b'G' || b == b'g' {
        if pos == 0 {
            return 0;
        }
        let p = seq[pos - 1];
        if p == b'C' || p == b'c' { -1 } else { 0 }
    } else {
        0
    }
}

// ---------------------------------------------------------------------------
// Per-read position mapping (MethylDackel `calculate_positions`, overlaps.c).
// Maps each query index to its reference position, or -1 for unmapped bases.
// ---------------------------------------------------------------------------

fn calculate_positions(rec: &RawRecord) -> Vec<i32> {
    let l_qseq = rec.sequence_len();
    let mut positions = vec![-1i32; l_qseq];
    let mut offset = 0usize;
    let mut ref_pos = rec.alignment_start();

    for (op, len) in rec.cigar_ops() {
        let len = len as usize;
        match op {
            CIGAR_MATCH | CIGAR_EQUAL | CIGAR_DIFF => {
                for _ in 0..len {
                    if offset < l_qseq {
                        positions[offset] = ref_pos;
                        ref_pos += 1;
                        offset += 1;
                    }
                }
            }
            CIGAR_INS | CIGAR_SOFT_CLIP => {
                for _ in 0..len {
                    if offset < l_qseq {
                        positions[offset] = -1;
                        offset += 1;
                    }
                }
            }
            CIGAR_DEL | CIGAR_REF_SKIP => {
                ref_pos += len as i32;
            }
            CIGAR_HARD_CLIP => {}
            _ => {}
        }
    }
    positions
}

/// Exclusive reference end (`bam_endpos`): alignment start plus the CIGAR's
/// reference span. Once the next read's start passes this, no later mate can
/// overlap, so the read is settled — safe to accumulate into the counts array
/// and drop from the active window.
fn ref_end(rec: &RawRecord) -> i32 {
    let mut ref_pos = rec.alignment_start();
    for (op, len) in rec.cigar_ops() {
        if matches!(
            op,
            CIGAR_MATCH | CIGAR_DEL | CIGAR_REF_SKIP | CIGAR_EQUAL | CIGAR_DIFF
        ) {
            ref_pos += len as i32;
        }
    }
    ref_pos
}

// ---------------------------------------------------------------------------
// Overlap quality adjustment (MethylDackel `cust_tweak_overlap_quality`, overlaps.c).
// ---------------------------------------------------------------------------

/// Apply MethylDackel's custom overlap quality adjustment to two overlapping mates.
///
/// `a_quals`/`b_quals` are modified in-place. Reads on opposite bisulfite strands
/// ((a_strand − b_strand) & 1 == 1) are not adjusted.
#[allow(clippy::too_many_arguments)]
fn tweak_overlap_quality(
    a_pos: &[i32],
    a_quals: &mut [u8],
    a_strand: u8,
    a_seq: &[u8],
    b_pos: &[i32],
    b_quals: &mut [u8],
    b_strand: u8,
    b_seq: &[u8],
) {
    // Skip if on opposite bisulfite strands (OT vs OB).
    if (a_strand.wrapping_sub(b_strand)) & 1 == 1 {
        return;
    }

    let na = a_pos.len();
    let nb = b_pos.len();

    let mut ia = 0usize;
    let mut ib = 0usize;

    // Skip to first mapped positions.
    while ia < na && a_pos[ia] < 0 {
        ia += 1;
    }
    while ib < nb && b_pos[ib] < 0 {
        ib += 1;
    }
    if ia == na || ib == nb {
        return;
    }

    // Advance to first overlapping position.
    if a_pos[ia] < b_pos[ib] {
        while ia < na && a_pos[ia] < b_pos[ib] {
            ia += 1;
        }
    } else {
        while ib < nb && b_pos[ib] < a_pos[ia] {
            ib += 1;
        }
    }
    if ia == na || ib == nb {
        return;
    }

    while ia < na && ib < nb {
        if a_pos[ia] < 0 || (ib < nb && a_pos[ia] < b_pos[ib]) {
            ia += 1;
            continue;
        }
        if b_pos[ib] < 0 || (ia < na && b_pos[ib] < a_pos[ia]) {
            ib += 1;
            continue;
        }
        if a_pos[ia] != b_pos[ib] {
            ia += 1;
            continue;
        }

        let a_base = a_seq[ia];
        let b_base = b_seq[ib];
        let a_q = a_quals[ia];
        let b_q = b_quals[ib];

        if a_base != b_base {
            // Mismatch.
            if a_q > b_q && a_base != 0xf0 {
                // a_base != N (nt16=15 in packed, but here we store unpacked nibbles)
                a_quals[ia] = a_q.saturating_sub(b_q);
                b_quals[ib] = 0;
            } else if b_q > a_q && b_base != 0xf0 {
                b_quals[ib] = b_q.saturating_sub(a_q);
                a_quals[ia] = 0;
            } else {
                a_quals[ia] = 0;
                b_quals[ib] = 0;
            }
        } else {
            // Match: boost the winner by 20%.
            if a_q > b_q {
                a_quals[ia] = ((a_q as u32 + (a_q as u32 * 20 / 100)).min(255)) as u8;
                b_quals[ib] = 0;
            } else {
                b_quals[ib] = ((b_q as u32 + (b_q as u32 * 20 / 100)).min(255)) as u8;
                a_quals[ia] = 0;
            }
        }

        ia += 1;
        ib += 1;
    }
}

/// Apply [`tweak_overlap_quality`] to two buffered mates in place, splitting their
/// quality borrows so neither read's `quals` is cloned.
fn tweak_overlap_quality_pair(a: &mut BufferedRead, b: &mut BufferedRead) {
    let (a_pos, a_strand, a_seq, a_quals) =
        (&a.ref_positions, a.strand, &a.seq_nibbles, &mut a.quals);
    let (b_pos, b_strand, b_seq, b_quals) =
        (&b.ref_positions, b.strand, &b.seq_nibbles, &mut b.quals);
    tweak_overlap_quality(
        a_pos, a_quals, a_strand, a_seq, b_pos, b_quals, b_strand, b_seq,
    );
}

// ---------------------------------------------------------------------------
// Buffered read for pileup.
// ---------------------------------------------------------------------------

struct BufferedRead {
    /// Reference positions for each query index, -1 = unmapped.
    ref_positions: Vec<i32>,
    /// Per-base quality scores, possibly modified by overlap tweak.
    quals: Vec<u8>,
    /// Per-base sequence nibbles (nt16 values from seq nibble extraction).
    seq_nibbles: Vec<u8>,
    /// Bisulfite strand (1=OT, 2=OB, 3=CTOT, 4=CTOB).
    strand: u8,
    /// Exclusive reference end ([`ref_end`]); drives the settle-and-flush window.
    end: i32,
}

impl BufferedRead {
    fn new(rec: &RawRecord) -> Self {
        let strand = get_strand(rec);
        let ref_positions = calculate_positions(rec);
        let l_qseq = rec.sequence_len();
        let quals: Vec<u8> = rec.quality_scores().to_vec();
        let seq_nibbles: Vec<u8> = (0..l_qseq).map(|i| rec.seq_nibble(i)).collect();
        let end = ref_end(rec);
        Self {
            ref_positions,
            quals,
            seq_nibbles,
            strand,
            end,
        }
    }
}

// ---------------------------------------------------------------------------
// Per-position pileup accumulator: O(total_bases) instead of O(N×M).
// ---------------------------------------------------------------------------

/// Accumulated methylation counts for a single reference position.
#[derive(Default, Clone, Copy)]
struct PosCounts {
    nmethyl: u32,
    nunmethyl: u32,
}

fn passes_filters(rec: &RawRecord, opts: &ExtractOpts) -> bool {
    let flag = rec.flags();
    if flag & FLAG_UNMAPPED != 0 || rec.reference_sequence_id() < 0 {
        return false;
    }
    if opts.ignore_flags != 0 && flag & opts.ignore_flags != 0 {
        return false;
    }
    if opts.require_flags != 0 && flag & opts.require_flags != opts.require_flags {
        return false;
    }
    if rec.mapping_quality() < opts.min_mapq {
        return false;
    }
    // keepSingleton=false: skip reads that are paired but mate is unmapped.
    if flag & FLAG_PAIRED != 0 && flag & FLAG_MATE_UNMAPPED != 0 {
        return false;
    }
    // keepDiscordant=false: skip paired reads that are not proper pairs.
    if flag & FLAG_PAIRED != 0 && flag & FLAG_PROPER_PAIR == 0 {
        return false;
    }
    true
}

/// Accumulate one read's methylation calls into `counts`, indexed by absolute
/// reference position (`counts[0]` is contig position 0).
fn accumulate_read(br: &BufferedRead, counts: &mut [PosCounts], ref_seq: &[u8], min_phred: u8) {
    let strand = br.strand;
    let is_ot = strand & 1 != 0; // strand 1 or 3 → OT/CTOT → counts on C positions

    for (qi, &rp) in br.ref_positions.iter().enumerate() {
        if rp < 0 {
            continue;
        }
        if rp as usize >= counts.len() {
            continue;
        }
        let rpos = rp as usize;

        // Check that this base in the reference is a CpG in the right direction.
        let base = ref_seq[rpos];
        let is_c_base = base == b'C' || base == b'c';
        if is_ot != is_c_base {
            continue; // OT reads on G positions or OB reads on C positions: skip
        }
        // Verify it's actually a CpG context.
        if is_cpg(ref_seq, rpos) == 0 {
            continue;
        }

        let nt16 = br.seq_nibbles[qi];
        let qual = br.quals[qi];

        match update_metrics(qual, min_phred, nt16, strand) {
            Some(true) => counts[rpos].nmethyl += 1,
            Some(false) => counts[rpos].nunmethyl += 1,
            None => {}
        }
    }
}

/// Streaming overlap window over the reads of one contig.
///
/// Records arrive in coordinate order. A read enters the active set; when its
/// proper-pair mate (same contig) arrives, [`tweak_overlap_quality_pair`] adjusts
/// both in place. Once the cursor (the start of an incoming read) passes a
/// buffered read's `end`, no later mate can overlap it, so the read is
/// **settled** and flushed into the counts array, bounding live memory to the
/// active window rather than the whole contig.
struct OverlapWindow {
    /// QNAME → index in `active` of the still-awaited first mate of a pair.
    pending: HashMap<Vec<u8>, usize>,
    /// Live reads whose mate could still arrive or that have not yet settled.
    active: Vec<BufferedRead>,
}

impl OverlapWindow {
    fn new() -> Self {
        Self {
            pending: HashMap::new(),
            active: Vec::new(),
        }
    }

    /// Buffer one record, pairing it with a pending mate if present.
    fn push(&mut self, rec: &RawRecord) {
        let flag = rec.flags();
        let track_overlap = flag & FLAG_PAIRED != 0
            && flag & FLAG_PROPER_PAIR != 0
            && rec.mate_reference_sequence_id() == rec.reference_sequence_id();

        let idx = self.active.len();
        self.active.push(BufferedRead::new(rec));

        if track_overlap {
            let name = rec.name().to_vec();
            if let Some(first_idx) = self.pending.remove(&name) {
                self.tweak(first_idx, idx);
            } else {
                self.pending.insert(name, idx);
            }
        }
    }

    /// Apply MethylDackel's overlap quality tweak to two active mates, borrowing
    /// both mutably via a split — no per-pair Vec clones.
    fn tweak(&mut self, a: usize, b: usize) {
        let (lo, hi) = (a.min(b), a.max(b));
        let (left, right) = self.active.split_at_mut(hi);
        let (ra, rb) = (&mut left[lo], &mut right[0]);
        let (first, second) = if a < b { (ra, rb) } else { (rb, ra) };
        tweak_overlap_quality_pair(first, second);
    }

    /// Flush every settled read (`end <= cursor`) into `counts`, dropping it from
    /// the active set and remapping surviving `pending` indices. A pending read
    /// that is itself settled had no mate arrive (else it would have been removed
    /// on pairing), so its entry is simply dropped.
    ///
    /// Compaction allocates only when a read is actually dropped: the common
    /// per-read call (no read settled this step) costs a single bounds-checked
    /// scan with no heap traffic.
    fn flush_settled(
        &mut self,
        cursor: i32,
        counts: &mut [PosCounts],
        ref_seq: &[u8],
        min_phred: u8,
    ) {
        if !self.active.iter().any(|r| r.end <= cursor) {
            return;
        }
        let mut new_index: Vec<Option<usize>> = Vec::with_capacity(self.active.len());
        let mut write = 0usize;
        for read in &self.active {
            if read.end <= cursor {
                accumulate_read(read, counts, ref_seq, min_phred);
                new_index.push(None);
            } else {
                new_index.push(Some(write));
                write += 1;
            }
        }
        let mut keep = new_index.iter();
        self.active.retain(|_| keep.next().unwrap().is_some());

        if !self.pending.is_empty() {
            self.pending.retain(|_, idx| match new_index[*idx] {
                Some(ni) => {
                    *idx = ni;
                    true
                }
                None => false,
            });
        }
    }

    /// Flush all remaining reads (end of contig) and reset for the next contig.
    fn drain_all(&mut self, counts: &mut [PosCounts], ref_seq: &[u8], min_phred: u8) {
        for read in &self.active {
            accumulate_read(read, counts, ref_seq, min_phred);
        }
        self.active.clear();
        self.pending.clear();
    }
}

// ---------------------------------------------------------------------------
// Reference sequence cache.
// ---------------------------------------------------------------------------

/// Load contigs from a FASTA file into a name → sequence map. Each contig is held
/// behind an `Arc` so the active [`ContigState`] can share it without borrowing the
/// whole reference map across the streaming loop.
fn load_reference(fasta_path: &Path) -> Result<HashMap<String, Arc<Vec<u8>>>> {
    use std::io::BufReader;

    let file = std::fs::File::open(fasta_path).map_err(RsomicsError::Io)?;
    let mut reader = noodles::fasta::io::Reader::new(BufReader::new(file));
    let mut contigs: HashMap<String, Arc<Vec<u8>>> = HashMap::new();

    for result in reader.records() {
        let record = result.map_err(RsomicsError::Io)?;
        let name = String::from_utf8_lossy(record.name()).into_owned();
        let seq = record.sequence().as_ref().to_vec();
        contigs.insert(name, Arc::new(seq));
    }
    Ok(contigs)
}

// ---------------------------------------------------------------------------
// Per-contig streaming state.
// ---------------------------------------------------------------------------

/// Accumulation state for the contig currently being streamed.
struct ContigState {
    tid: i32,
    ref_seq: Arc<Vec<u8>>,
    /// One [`PosCounts`] per reference position (contig length, clamped to the
    /// loaded sequence). Settled reads scatter their calls here as they flush.
    counts: Vec<PosCounts>,
    window: OverlapWindow,
}

/// Drain a contig's remaining window into its counts, then sweep CpG positions and
/// emit bedGraph lines for those at or above the depth threshold.
fn finalize_contig<W: Write>(
    mut state: ContigState,
    chrom_names: &[String],
    opts: &ExtractOpts,
    out: &mut W,
    stats: &mut ExtractStats,
) -> Result<()> {
    let ref_seq = Arc::clone(&state.ref_seq);
    state
        .window
        .drain_all(&mut state.counts, &ref_seq, opts.min_phred);

    let chrom_name = &chrom_names[state.tid as usize];
    for (pos, &c) in state.counts.iter().enumerate() {
        if is_cpg(&ref_seq, pos) != 0 {
            stats.positions_examined += 1;
            let depth = c.nmethyl + c.nunmethyl;
            if depth >= opts.min_depth {
                let pct = (100.0 * c.nmethyl as f64 / depth as f64) as u32;
                writeln!(
                    out,
                    "{}\t{}\t{}\t{}\t{}\t{}",
                    chrom_name,
                    pos,
                    pos + 1,
                    pct,
                    c.nmethyl,
                    c.nunmethyl
                )
                .map_err(RsomicsError::Io)?;
                stats.positions_emitted += 1;
            }
        }
    }
    Ok(())
}

// ---------------------------------------------------------------------------
// Main extraction entry point.
// ---------------------------------------------------------------------------

pub fn run(
    bam_path: &Path,
    fasta_path: &Path,
    opts: ExtractOpts,
    workers: NonZero<usize>,
) -> Result<ExtractStats> {
    let reference = load_reference(fasta_path)?;

    let output_path = format!("{}_CpG.bedGraph", opts.output_prefix);
    let out_file = std::fs::File::create(&output_path).map_err(RsomicsError::Io)?;
    let mut out = BufWriter::with_capacity(256 * 1024, out_file);

    // Write bedGraph header.
    writeln!(
        out,
        "track type=\"bedGraph\" description=\"{} CpG methylation levels\"",
        opts.output_prefix
    )
    .map_err(RsomicsError::Io)?;

    let mut reader = rsomics_bamio::open_with_workers(bam_path, workers)?;
    let header = reader.read_header().map_err(RsomicsError::Io)?;

    let chrom_names: Vec<String> = header
        .reference_sequences()
        .iter()
        .map(|(name, _)| name.to_string())
        .collect();

    let chrom_lens: Vec<u64> = header
        .reference_sequences()
        .iter()
        .map(|(_, seq)| usize::from(seq.length()) as u64)
        .collect();

    let mut stats = ExtractStats::default();
    let mut src = RawRecord::default();

    // Stream the coordinate-sorted BAM contig by contig. Reads for one contig
    // arrive contiguously and in start order, so a per-contig flat counts array is
    // accumulated incrementally and settled reads (those the cursor has passed)
    // are flushed out of the active window — live memory stays bounded to the
    // overlap window rather than the whole input.
    let mut state: Option<ContigState> = None;

    while read_record(reader.get_mut(), &mut src)? != 0 {
        stats.reads_processed += 1;
        let tid = src.reference_sequence_id();
        if tid < 0 {
            stats.reads_filtered += 1;
            continue;
        }
        if !passes_filters(&src, &opts) {
            stats.reads_filtered += 1;
            continue;
        }

        if state.as_ref().is_none_or(|s| s.tid != tid) {
            if let Some(prev) = state.take() {
                finalize_contig(prev, &chrom_names, &opts, &mut out, &mut stats)?;
            }
            let chrom_name = &chrom_names[tid as usize];
            match reference.get(chrom_name) {
                Some(ref_seq) => {
                    let seq_len = ref_seq.len().min(chrom_lens[tid as usize] as usize);
                    state = Some(ContigState {
                        tid,
                        ref_seq: Arc::clone(ref_seq),
                        counts: vec![PosCounts::default(); seq_len],
                        window: OverlapWindow::new(),
                    });
                }
                None => {
                    // Contig absent from the reference: drop its reads entirely.
                    state = None;
                    continue;
                }
            }
        }

        let cur = state.as_mut().unwrap();
        // Drain the window only once it has grown past the active-coverage band,
        // amortising the settle scan over many reads while keeping the live set
        // bounded (vs. buffering the whole contig).
        if cur.window.active.len() >= WINDOW_FLUSH_THRESHOLD {
            let cursor = src.alignment_start();
            cur.window
                .flush_settled(cursor, &mut cur.counts, &cur.ref_seq, opts.min_phred);
        }
        cur.window.push(&src);
    }

    if let Some(last) = state.take() {
        finalize_contig(last, &chrom_names, &opts, &mut out, &mut stats)?;
    }

    out.flush().map_err(RsomicsError::Io)?;
    Ok(stats)
}
