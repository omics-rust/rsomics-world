//! FASTA consensus from a coordinate-sorted BAM — simple-mode port of
//! `samtools consensus -m simple`.
//!
//! Algorithm: `bam_consensus.c` `calculate_consensus_simple` + `basic_fasta`.
//! Per reference position the engine accumulates a weighted score for each of
//! A, C, G, T, and * (deletion); the weight per read is `qual` (or 1 when
//! `use_qual = false`) times a compatibility factor from the seqi2A/C/G/T
//! tables.  The highest-scoring allele is the call; if the second-highest
//! scores ≥ `het_fract × score1` and `ambig` is on, the two are OR-ed into an
//! IUPAC code.  If total depth < `min_depth` or the call's fraction of total
//! score < `call_fract`, the call is N (or * for a gap).
//!
//! Reference source: `samtools/bam_consensus.c` (MIT), tag 1.23.1.
//!
//! Pileup engine: custom lightweight walker over coordinate-sorted BAM records,
//! modelled on `samtools/bam_plbuf.c` / `consensus_pileup.c`.  Avoids the
//! per-column `Vec` allocations and HashMap overlap tracking of the generic
//! `rsomics-pileup` crate, which adds enough overhead to lose the perf gate
//! on a 150x WGS fixture.  Single-threaded decode + CIGAR walk; all I/O is
//! on the calling thread (readers are passed in from the caller).

use std::io::Write;
use std::num::NonZero;
use std::path::Path;

use rsomics_bamio::raw::RawRecord;
use rsomics_common::{Result, RsomicsError};
use serde::Serialize;

// seqi2{A,C,G,T} lookup tables: index is 4-bit seq_nt16 (A=1 C=2 G=4 T=8).
// Pure bases carry weight 8; 2-base ambiguity codes 4; 3-base 2; N 1.
// Source: bam_consensus.c static arrays, lines ~1908–1911 in 1.23.1.
//                            * A  C  M  G  R  S  V  T  W  Y  H  K  D  B  N
const SEQI2A: [u8; 16] = [0, 8, 0, 4, 0, 4, 0, 2, 0, 4, 0, 2, 0, 2, 0, 1];
const SEQI2C: [u8; 16] = [0, 0, 8, 4, 0, 0, 4, 2, 0, 0, 4, 2, 0, 0, 2, 1];
const SEQI2G: [u8; 16] = [0, 0, 0, 0, 8, 4, 4, 1, 0, 0, 0, 0, 4, 2, 2, 1];
const SEQI2T: [u8; 16] = [0, 0, 0, 0, 0, 0, 0, 0, 8, 4, 4, 2, 8, 2, 2, 1];

/// Score/depth slots: 0=A 1=C 2=G 3=T 4=* (gap).
const N_BASES: usize = 5;

// SAM FLAG bits (SAMv1 §1.4).
const FLAG_UNMAP: u16 = 0x4;
const FLAG_SECONDARY: u16 = 0x100;
const FLAG_QCFAIL: u16 = 0x200;
const FLAG_DUP: u16 = 0x400;

// CIGAR op codes (BAM packed encoding, low nibble).
const CIGAR_MATCH: u8 = 0;
const CIGAR_INS: u8 = 1;
const CIGAR_DEL: u8 = 2;
const CIGAR_REF_SKIP: u8 = 3;
const CIGAR_SOFT_CLIP: u8 = 4;
const CIGAR_HARD_CLIP: u8 = 5;
const CIGAR_PAD: u8 = 6;
const CIGAR_EQUAL: u8 = 7;
const CIGAR_DIFF: u8 = 8;

/// Default exclude-flags matching samtools consensus (UNMAP|SECONDARY|QCFAIL|DUP).
pub const DEFAULT_EXCL_FLAGS: u16 = FLAG_UNMAP | FLAG_SECONDARY | FLAG_QCFAIL | FLAG_DUP;

#[derive(Debug, Clone)]
pub struct ConsensusOpts {
    /// Use base qualities as weights (`-q`/`--use-qual`). Default off.
    pub use_qual: bool,
    /// Minimum base quality to count a base (`--min-BQ`). Default 0.
    pub min_qual: u8,
    /// Minimum depth to emit a non-N call (`-d`/`--min-depth`). Default 1.
    pub min_depth: u32,
    /// Minimum fraction of total score that the called allele(s) must reach
    /// (`-c`/`--call-fract`). Default 0.75.
    pub call_fract: f64,
    /// Minimum ratio of second-best to best score to call a het
    /// (`-H`/`--het-fract`). Default 0.5.
    pub het_fract: f64,
    /// Enable IUPAC ambiguity codes (`-A`/`--ambig`). Default off.
    pub ambig: bool,
    /// Emit deletion (`*`) bases (`--show-del yes`). Default false.
    pub show_del: bool,
    /// FASTA/Q line length (`-l`/`--line-len`). Default 70.
    pub line_len: usize,
    /// Reads with any of these FLAG bits are excluded. Default `DEFAULT_EXCL_FLAGS`.
    pub excl_flags: u16,
    /// Only include reads with any of these FLAG bits set (0 = no filter).
    pub incl_flags: u16,
    /// Minimum mapping quality (`--min-MQ`). Default 0.
    pub min_mapq: u8,
}

impl Default for ConsensusOpts {
    fn default() -> Self {
        Self {
            use_qual: false,
            min_qual: 0,
            min_depth: 1,
            call_fract: 0.75,
            het_fract: 0.5,
            ambig: false,
            show_del: false,
            line_len: 70,
            excl_flags: DEFAULT_EXCL_FLAGS,
            incl_flags: 0,
            min_mapq: 0,
        }
    }
}

#[derive(Debug, Default, Clone, Serialize)]
pub struct ConsensusStats {
    pub sequences: u64,
    pub positions: u64,
}

/// `bam_consensus.c::calculate_consensus_simple` — returns ASCII base call and
/// quality (0–100 as a percentage of total score).
///
/// `reads`: `(base4, qual)` pairs, one per non-refskip read at this column.
/// `base4` is the 4-bit seq_nt16 value; gap/del is encoded as 16.
pub fn simple_call(reads: &[(u8, u8)], opts: &ConsensusOpts) -> (u8, u8) {
    // IUPAC het table: maps seqi-bit OR-value to ASCII.
    // Replicates `const char *het = "NACMGRSVTWYHKDBN*ac?g???t???????";` from
    // bam_consensus.c `calculate_consensus_simple`.
    const HET: &[u8; 32] = b"NACMGRSVTWYHKDBN*ac?g???t???????";

    let mut score = [0u64; N_BASES]; // A C G T *
    let mut tot_depth: u32 = 0;

    for &(b4, q) in reads {
        if q < opts.min_qual {
            continue;
        }
        let w: u64 = if opts.use_qual { q as u64 } else { 1 };
        if b4 < 16 {
            let b = b4 as usize;
            let qa = SEQI2A[b] as u64 * w;
            let qc = SEQI2C[b] as u64 * w;
            let qg = SEQI2G[b] as u64 * w;
            let qt = SEQI2T[b] as u64 * w;
            if qa > 0 {
                score[0] += qa;
            }
            if qc > 0 {
                score[1] += qc;
            }
            if qg > 0 {
                score[2] += qg;
            }
            if qt > 0 {
                score[3] += qt;
            }
        } else {
            score[4] += 8 * w;
        }
        tot_depth += 1;
    }

    let tscore: u64 = score.iter().sum();

    // Find best (call1) and second-best (call2).
    // Slot i → seqi bit = 1 << i  (0→A=1, 1→C=2, 2→G=4, 3→T=8, 4→*=16).
    let mut call1: usize = 15; // seqi 15 = N
    let mut call2: usize = 15;
    let mut score1: u64 = 0;
    let mut score2: u64 = 0;

    for i in 0..N_BASES {
        if score[i] > score1 {
            score2 = score1;
            call2 = call1;
            score1 = score[i];
            call1 = 1 << i;
        } else if score[i] > score2 {
            score2 = score[i];
            call2 = 1 << i;
        }
    }

    let mut used_base = call1;
    let mut used_score = score1;

    if opts.ambig && score1 > 0 && (score2 as f64) >= opts.het_fract * score1 as f64 {
        used_base |= call2;
        used_score += score2;
    }

    // Depth / fraction gate.
    if tot_depth < opts.min_depth
        || (tscore > 0 && (used_score as f64) < opts.call_fract * tscore as f64)
    {
        used_base = if call1 == 16 { 16 } else { 0 };
    }

    let cb = HET[used_base.min(31)];
    let cq: u8 = if used_base != 0 && tscore > 0 {
        (100 * used_score / tscore).min(100) as u8
    } else {
        0
    };

    (cb, cq)
}

// ---------------------------------------------------------------------------
// Lightweight pileup walker — consensus-specific, no HashMap overhead.
// Mirrors samtools consensus_pileup.c: a sorted buffer of active reads,
// a reference cursor, and a CIGAR walk per read.
// ---------------------------------------------------------------------------

/// Compute the reference end position of a read given its start and cigar.
/// Matches htslib `bam_endpos`: only CIGAR_MATCH/DEL/REF_SKIP/EQUAL/DIFF
/// consume reference bases.
fn ref_end(beg: i64, cigar: &[(u8, i32)]) -> i64 {
    let mut pos = beg;
    for &(op, len) in cigar {
        match op {
            CIGAR_MATCH | CIGAR_DEL | CIGAR_REF_SKIP | CIGAR_EQUAL | CIGAR_DIFF => {
                pos += i64::from(len);
            }
            _ => {}
        }
    }
    pos
}

/// One active read in the pileup buffer: raw record + cached CIGAR + walk state.
struct ActiveRead {
    rec: RawRecord,
    cigar: Vec<(u8, i32)>,
    /// Reference end (exclusive).
    end: i64,
    /// Current CIGAR op index.
    k: usize,
    /// Reference position at the start of cigar[k].
    x: i64,
    /// Query position at the start of cigar[k] (counts consumed query bases).
    y: i32,
}

impl ActiveRead {
    fn new(rec: RawRecord) -> Self {
        let beg = i64::from(rec.alignment_start());
        let cigar: Vec<(u8, i32)> = rec.cigar_ops().map(|(o, l)| (o, l as i32)).collect();
        // Advance past leading soft/hard clips and non-ref ops to find first ref op.
        let mut k = 0usize;
        let x = beg;
        let mut y = 0i32;
        while k < cigar.len() {
            let (op, l) = cigar[k];
            match op {
                CIGAR_MATCH | CIGAR_DEL | CIGAR_REF_SKIP | CIGAR_EQUAL | CIGAR_DIFF => break,
                CIGAR_INS | CIGAR_SOFT_CLIP => {
                    y += l;
                    k += 1;
                }
                CIGAR_HARD_CLIP | CIGAR_PAD => {
                    k += 1;
                }
                _ => {
                    k += 1;
                }
            }
        }
        let end = ref_end(beg, &cigar);
        ActiveRead {
            rec,
            cigar,
            end,
            k,
            x,
            y,
        }
    }

    /// Advance CIGAR walk to reference position `pos`.  Returns `(base4, qual, is_del, is_refskip)`.
    /// `base4 == 16` for a deletion; `is_refskip` marks intron skips to exclude.
    fn at(&mut self, pos: i64) -> (u8, u8, bool, bool) {
        // Advance op if cursor has moved past current op's span.
        loop {
            if self.k >= self.cigar.len() {
                return (0, 0, true, false); // past end — treat as del
            }
            let (op, l) = self.cigar[self.k];
            let op_end = self.x + i64::from(l);
            if pos < op_end {
                break;
            }
            // Move to next op, skipping non-ref-consuming ops.
            match op {
                CIGAR_MATCH | CIGAR_EQUAL | CIGAR_DIFF => {
                    self.y += l;
                    self.x = op_end;
                }
                CIGAR_DEL | CIGAR_REF_SKIP => {
                    self.x = op_end;
                }
                _ => {}
            }
            self.k += 1;
            // Skip leading non-ref-consuming ops of the next run.
            while self.k < self.cigar.len() {
                let (nop, nl) = self.cigar[self.k];
                match nop {
                    CIGAR_MATCH | CIGAR_DEL | CIGAR_REF_SKIP | CIGAR_EQUAL | CIGAR_DIFF => break,
                    CIGAR_INS | CIGAR_SOFT_CLIP => {
                        self.y += nl;
                        self.k += 1;
                    }
                    _ => {
                        self.k += 1;
                    }
                }
            }
        }

        let (op, _) = self.cigar[self.k];
        match op {
            CIGAR_MATCH | CIGAR_EQUAL | CIGAR_DIFF => {
                let qpos = (self.y + (pos - self.x) as i32) as usize;
                let b4 = self.rec.seq_nibble(qpos);
                let qs = self.rec.quality_scores();
                let q = if qpos < qs.len() { qs[qpos] } else { 0 };
                (b4, q, false, false)
            }
            CIGAR_DEL => (16, 0, true, false),
            CIGAR_REF_SKIP => (16, 0, true, true),
            _ => (0, 0, true, false),
        }
    }
}

/// Write a FASTA record to `out`, wrapping at `line_len`.
fn write_fasta(
    out: &mut dyn Write,
    name: &str,
    seq: &[u8],
    line_len: usize,
) -> std::io::Result<()> {
    if seq.is_empty() {
        return Ok(());
    }
    writeln!(out, ">{name}")?;
    for chunk in seq.chunks(line_len) {
        out.write_all(chunk)?;
        out.write_all(b"\n")?;
    }
    Ok(())
}

/// Compute simple-mode FASTA consensus for every covered contig.
pub fn consensus(
    bam_path: &Path,
    out: &mut dyn Write,
    opts: &ConsensusOpts,
    workers: NonZero<usize>,
) -> Result<ConsensusStats> {
    let mut stats = ConsensusStats::default();

    // raw::read_record bypasses the noodles record layer and reads directly from
    // the inner BufRead via reader.get_mut(). bgzf::io::MultithreadedReader's
    // BufRead impl interacts poorly with header-then-raw-read sequences across
    // platforms, so we always use a single-threaded reader here.
    let _ = workers;
    let st = NonZero::<usize>::new(1).unwrap();
    let mut reader = rsomics_bamio::open_with_workers(bam_path, st)?;
    let header = reader.read_header().map_err(RsomicsError::Io)?;

    let ref_names: Vec<String> = header
        .reference_sequences()
        .keys()
        .map(ToString::to_string)
        .collect();

    let reader_mut = reader.get_mut();
    let mut record = RawRecord::default();

    // Active read buffer: reads whose alignment span covers the current cursor.
    let mut active: Vec<ActiveRead> = Vec::with_capacity(512);
    // Per-column base buffer: (base4, qual) pairs for simple_call.
    let mut col_buf: Vec<(u8, u8)> = Vec::with_capacity(512);

    // Current contig state.
    let mut cur_tid: i32 = -1;
    let mut cur_seq: Vec<u8> = Vec::new();
    let mut last_pos: i64 = -1;

    // Pileup cursor: (tid, pos) of next column to emit.
    let mut cursor_tid: i32 = -1;
    let mut cursor_pos: i64 = 0;

    // Next record (lookahead — we buffer one record to know when to flush).
    let mut next_rec: Option<RawRecord> = None;

    // Read the first record.
    {
        let n = rsomics_bamio::raw::read_record(reader_mut, &mut record)?;
        if n > 0 {
            next_rec = Some(record.clone());
        }
    }

    loop {
        // Feed records that start at or before cursor_pos (or prime the cursor).
        loop {
            let Some(ref nr) = next_rec else { break };
            let flag = nr.flags();
            if nr.reference_sequence_id() < 0 || flag & FLAG_UNMAP != 0 {
                // Filtered — drop and advance.
                let n = rsomics_bamio::raw::read_record(reader_mut, &mut record)?;
                next_rec = if n > 0 { Some(record.clone()) } else { None };
                continue;
            }
            if opts.excl_flags != 0 && flag & opts.excl_flags != 0 {
                let n = rsomics_bamio::raw::read_record(reader_mut, &mut record)?;
                next_rec = if n > 0 { Some(record.clone()) } else { None };
                continue;
            }
            if opts.incl_flags != 0 && flag & opts.incl_flags == 0 {
                let n = rsomics_bamio::raw::read_record(reader_mut, &mut record)?;
                next_rec = if n > 0 { Some(record.clone()) } else { None };
                continue;
            }
            if nr.mapping_quality() < opts.min_mapq {
                let n = rsomics_bamio::raw::read_record(reader_mut, &mut record)?;
                next_rec = if n > 0 { Some(record.clone()) } else { None };
                continue;
            }

            let rtid = nr.reference_sequence_id();
            let rbeg = i64::from(nr.alignment_start());

            // If cursor not yet set, initialise to this read's start.
            if cursor_tid < 0 {
                cursor_tid = rtid;
                cursor_pos = rbeg;
            }

            // Read is ahead of cursor — stop feeding.
            if rtid > cursor_tid || (rtid == cursor_tid && rbeg > cursor_pos) {
                break;
            }

            // Consume the lookahead record.
            let cur_nr = next_rec.take().unwrap();
            active.push(ActiveRead::new(cur_nr));

            // Read the next record.
            let n = rsomics_bamio::raw::read_record(reader_mut, &mut record)?;
            next_rec = if n > 0 { Some(record.clone()) } else { None };
        }

        // Prune reads that ended before cursor_pos.
        active.retain(|r| r.rec.reference_sequence_id() == cursor_tid && r.end > cursor_pos);

        if active.is_empty() {
            // No active reads: either done or jump to next read's position.
            let Some(ref nr) = next_rec else {
                // All input consumed — emit final contig.
                break;
            };
            // Jump cursor to next read.
            let rtid = nr.reference_sequence_id();
            let rbeg = i64::from(nr.alignment_start());
            if rtid != cursor_tid {
                // Contig switch with no active reads — flush current contig.
                if cur_tid >= 0 && !cur_seq.is_empty() {
                    let name = ref_names.get(cur_tid as usize).ok_or_else(|| {
                        RsomicsError::InvalidInput(format!("tid {cur_tid} not in header"))
                    })?;
                    write_fasta(out, name, &cur_seq, opts.line_len).map_err(RsomicsError::Io)?;
                    stats.sequences += 1;
                    cur_seq.clear();
                    cur_tid = -1;
                    last_pos = -1;
                }
            }
            cursor_tid = rtid;
            cursor_pos = rbeg;
            continue;
        }

        // Emit column at (cursor_tid, cursor_pos).
        let emit_tid = cursor_tid;
        let emit_pos = cursor_pos;

        // Handle contig switch.
        if emit_tid != cur_tid {
            if cur_tid >= 0 && !cur_seq.is_empty() {
                let name = ref_names.get(cur_tid as usize).ok_or_else(|| {
                    RsomicsError::InvalidInput(format!("tid {cur_tid} not in header"))
                })?;
                write_fasta(out, name, &cur_seq, opts.line_len).map_err(RsomicsError::Io)?;
                stats.sequences += 1;
                cur_seq.clear();
            }
            cur_tid = emit_tid;
            last_pos = -1;
        }

        // Collect bases from active reads.
        col_buf.clear();
        for ar in &mut active {
            let (b4, q, is_del, is_refskip) = ar.at(emit_pos);
            if is_refskip {
                continue;
            }
            col_buf.push((if is_del { 16 } else { b4 }, q));
        }

        let (cb, _cq) = simple_call(&col_buf, opts);

        if cb == b'*' && !opts.show_del {
            last_pos = emit_pos;
            cursor_pos += 1;
            continue;
        }

        // Fill gap between last covered position and this one with N.
        if !cur_seq.is_empty() && emit_pos > last_pos + 1 {
            let gap = (emit_pos - last_pos - 1) as usize;
            cur_seq.extend(std::iter::repeat_n(b'N', gap));
            stats.positions += gap as u64;
        }

        cur_seq.push(cb);
        stats.positions += 1;
        last_pos = emit_pos;
        cursor_pos += 1;
    }

    // Emit final contig.
    if cur_tid >= 0 && !cur_seq.is_empty() {
        let name = ref_names
            .get(cur_tid as usize)
            .ok_or_else(|| RsomicsError::InvalidInput(format!("tid {cur_tid} not in header")))?;
        write_fasta(out, name, &cur_seq, opts.line_len).map_err(RsomicsError::Io)?;
        stats.sequences += 1;
    }

    Ok(stats)
}
