//! Heterozygote phasing from aligned reads — port of `samtools phase`.
//!
//! Algorithm (mirrors `phase.c`):
//!
//! 1. Pileup over the input BAM, accumulating per-position allele counts.
//! 2. Het-call via LOD score (`gl2cns` analog): a site is het if its phred-LOD
//!    exceeds `min_var_lod` (default 37) and both alleles meet a minor-allele
//!    rate threshold.
//! 3. Phase blocks are phased with a sliding-window DP (`dynaprog`) of window
//!    `k` (default 13) over the het variant positions.
//! 4. Each fragment is assigned a haplotype by matching its allele calls against
//!    the DP path (`fragphase`). Chimeric fragments are optionally detected and
//!    flipped.
//! 5. Text output (`CC`/`PS`/`FL`/`M[012]`/`//`) is written to stdout.
//! 6. If `-b PREFIX` is given, reads are split into `PREFIX.0.bam`,
//!    `PREFIX.1.bam`, and `PREFIX.chimera.bam`.
//!
//! # Numeric constants (from `phase.c`)
//!
//! | Name           | Value | Meaning |
//! |----------------|-------|---------|
//! | MAX_VARS       | 256   | maximum variant allele calls per fragment |
//! | FLIP_PENALTY   | 2     | chimera-flip boundary cost |
//! | FLIP_THRES     | 4     | minimum improvement to accept a chimera flip |
//! | MASK_THRES     | 3     | minimum per-haplotype phased count to keep a site unmasked |
//! | DEFAULT_LOD    | 37    | min het phred-LOD (`-q`) |
//! | DEFAULT_K      | 13    | DP window length (`-k`) |
//! | DEFAULT_MIN_BQ | 13    | min base quality (`-Q`) |
//! | DEFAULT_DEPTH  | 256   | max pileup depth (`-D`) |

use std::io::Write;
use std::num::NonZero;
use std::path::{Path, PathBuf};

use ahash::AHashMap;

use noodles::bam;
use noodles::bgzf;
use rsomics_bamio::raw::{self, RawRecord};
use rsomics_common::{Result, RsomicsError};
use serde::Serialize;

// ─── algorithm constants (phase.c) ───────────────────────────────────────────

/// Maximum het-variant allele calls stored per fragment (phase.c `MAX_VARS`).
const MAX_VARS: usize = 256;

/// Penalty per flip boundary when evaluating chimera candidates (phase.c
/// `FLIP_PENALTY`).
const FLIP_PENALTY: i32 = 2;

/// Minimum per-haplotype improvement to accept a chimera flip on both sides
/// (phase.c `FLIP_THRES`).
const FLIP_THRES: i32 = 4;

/// Minimum per-haplotype phased-read count to leave a marker unmasked
/// (phase.c `MASK_THRES`).
const MASK_THRES: i32 = 3;

// BAM FLAG bits (SAMv1 §1.4).
const FLAG_UNMAPPED: u16 = 0x4;
const FLAG_SECONDARY: u16 = 0x100;
const FLAG_QCFAIL: u16 = 0x200;
const FLAG_DUPLICATE: u16 = 0x400;
const FLAG_SUPPLEMENTARY: u16 = 0x800;

// CIGAR op codes (BAM packed encoding, low nibble).
const CIGAR_MATCH: u8 = 0;
const CIGAR_INS: u8 = 1;
const CIGAR_DEL: u8 = 2;
const CIGAR_SKIP: u8 = 3;
const CIGAR_SOFT_CLIP: u8 = 4;
const CIGAR_EQUAL: u8 = 7;
const CIGAR_DIFF: u8 = 8;

// ─── public API types ─────────────────────────────────────────────────────────

#[derive(Debug, Clone)]
pub struct PhaseOpts {
    /// DP window length (samtools `-k`, default 13).
    pub k: usize,
    /// BAM output prefix (samtools `-b`). When `None` only text output is written.
    pub bam_prefix: Option<PathBuf>,
    /// Minimum het phred-LOD (samtools `-q`, default 37).
    pub min_var_lod: u32,
    /// Minimum base quality (samtools `-Q`, default 13).
    pub min_base_q: u8,
    /// Maximum pileup depth (samtools `-D`, default 256).
    pub max_depth: usize,
    /// Enable chimera fixing (samtools default on; `-F` disables).
    pub fix_chimera: bool,
    /// Drop ambiguously phased reads into chimera output (samtools `-A`).
    pub drop_ambiguous: bool,
}

impl Default for PhaseOpts {
    fn default() -> Self {
        Self {
            k: 13,
            bam_prefix: None,
            min_var_lod: 37,
            min_base_q: 13,
            max_depth: 256,
            fix_chimera: true,
            drop_ambiguous: false,
        }
    }
}

#[derive(Debug, Default, Serialize)]
pub struct PhaseStats {
    pub records_in: u64,
    pub het_sites: u64,
    pub phase_sets: u64,
    pub masked_sites: u64,
    pub reads_hap0: u64,
    pub reads_hap1: u64,
    pub reads_chimera: u64,
}

// ─── internal types ───────────────────────────────────────────────────────────

/// Allele encoding: 0 = ambiguous/low-quality, 1 = allele A, 2 = allele B.
type Allele = u8;

/// One heterozygous variant site.
struct HetSite {
    /// 0-based reference coordinate.
    pos: i64,
    /// ASCII base for allele-1.
    base_a: u8,
    /// ASCII base for allele-2.
    base_b: u8,
}

/// Per-fragment phasing data, keyed by QNAME hash.
#[derive(Default)]
struct Fragment {
    /// (variant_global_id, allele) pairs, in position order.
    alleles: Vec<(usize, Allele)>,
    /// Phase assignment (0 or 1); valid only when `phased`.
    phase: u8,
    /// True when in-phase and out-of-phase counts differ.
    phased: bool,
    /// True when identified as chimeric and tail alleles were flipped.
    flipped: bool,
    /// True when in-phase == out-of-phase (tied).
    ambiguous: bool,
    /// Allele calls agreeing with the DP path.
    in_phase: u32,
    /// Allele calls disagreeing with the DP path.
    out_phase: u32,
    /// Count of records overlapping het sites, for stats when BAM output is off.
    record_count: u64,
    /// The raw record payloads for BAM split output (populated only when `-b` is given).
    records: Vec<RawRecord>,
}

// ─── het-calling ─────────────────────────────────────────────────────────────

/// Call a het variant from allele counts, returning the LOD score if the site
/// passes (phase.c `gl2cns` analog).
///
/// Requirements: both alleles observed; minor-allele rate ≥ 10%; LOD ≥ threshold.
/// LOD ≈ 4 × minor_count (phred-like scaling matching phase.c's bit-shift formula).
fn call_het(count_a: u32, count_b: u32, min_lod: u32) -> Option<u32> {
    if count_a == 0 || count_b == 0 {
        return None;
    }
    let n = count_a + count_b;
    let minor = count_a.min(count_b);
    // Require ≥ 10% minor-allele rate to exclude sequencing noise.
    if minor * 10 < n {
        return None;
    }
    let lod = (4 * minor).min(u32::MAX / 2);
    if lod >= min_lod { Some(lod) } else { None }
}

// ─── dynamic-programming phasing ─────────────────────────────────────────────

/// Build the weight matrix for `dynaprog` from accumulated fragments.
///
/// For each variant window ending at position `i`, enumerates the 2^(k-1)
/// local haplotype patterns supported by each fragment and increments their
/// weights (phase.c `count_all` / `count1`).
fn build_weights(k: usize, n_vars: usize, fragments: &[Fragment]) -> Vec<Vec<i32>> {
    let states = 1usize << (k - 1);
    let mut w: Vec<Vec<i32>> = vec![vec![0i32; states]; n_vars];

    for frag in fragments {
        if frag.alleles.len() < 2 {
            continue;
        }
        let first = frag.alleles[0].0;
        let last = frag.alleles.last().unwrap().0;
        let span = last - first + 1;
        let mut local = vec![0u8; span];
        for &(vid, allele) in &frag.alleles {
            local[vid - first] = allele;
        }

        let win = k.min(span);
        for start in 0..=(span.saturating_sub(win)) {
            let global_i = first + start + win - 1;
            if global_i >= n_vars {
                break;
            }
            let n_ambi = local[start..start + win]
                .iter()
                .filter(|&&a| a == 0)
                .count();
            // phase.c skips windows with > 4 ambiguous bases.
            if n_ambi > 4 {
                continue;
            }
            let configs = 1usize << n_ambi;
            for cfg in 0..configs {
                let mut pattern = 0usize;
                let mut ambi_idx = 0;
                let mut valid = true;
                for j in 0..win {
                    let a = local[start + j];
                    let bit = if a == 0 {
                        let b = ((cfg >> ambi_idx) & 1) as u8 + 1;
                        ambi_idx += 1;
                        b
                    } else {
                        a
                    };
                    if bit == 0 || bit > 2 {
                        valid = false;
                        break;
                    }
                    pattern = (pattern << 1) | ((bit - 1) as usize);
                }
                if valid && pattern < states {
                    w[global_i][pattern] = w[global_i][pattern].saturating_add(1);
                }
            }
        }
    }
    w
}

/// Sliding-window DP haplotype phaser (phase.c `dynaprog`).
///
/// `weights[i][x]` is the number of fragments supporting local haplotype pattern
/// `x` (a (k-1)-bit bitmask) at variant `i`. Returns a 0/1 path of length
/// `n_vars` assigning each variant to one of the two haplotypes.
fn dynaprog(k: usize, n_vars: usize, weights: &[Vec<i32>]) -> Vec<u8> {
    if n_vars == 0 {
        return Vec::new();
    }
    let states = 1usize << (k - 1);
    let mut prev = vec![0i32; states];
    let mut curr = vec![0i32; states];
    let mut bt: Vec<Vec<u8>> = vec![vec![0u8; states]; n_vars];

    for i in 0..n_vars {
        let w = &weights[i];
        for x in 0..states {
            // Complement within (k-1) bits.
            let xc = (states - 1) & !x;
            let y0 = x >> 1;
            let y1 = xc >> 1;
            let sw = if x < w.len() { w[x] } else { 0 } + if xc < w.len() { w[xc] } else { 0 };
            let c0 = prev[y0] + sw;
            let c1 = prev[y1] + sw;
            if c0 >= c1 {
                curr[x] = c0;
                bt[i][x] = 0;
            } else {
                curr[x] = c1;
                bt[i][x] = 1;
            }
        }
        std::mem::swap(&mut prev, &mut curr);
    }

    let best = (0..states).max_by_key(|&x| prev[x]).unwrap_or(0);
    let mut path = vec![0u8; n_vars];
    let mut x = best;
    for i in (0..n_vars).rev() {
        path[i] = (x & 1) as u8;
        let xc = (states - 1) & !x;
        x = if bt[i][x] == 0 { x >> 1 } else { xc >> 1 };
    }
    path
}

// ─── fragment phasing and chimera detection ───────────────────────────────────

/// Assign a haplotype phase to each fragment, counting allele agreement with
/// the DP path (phase.c `fragphase`). When `fix_chimera` is set, evaluates
/// per-fragment flip points to repair chimeric reads.
fn fragphase(fragments: &mut [Fragment], path: &[u8], fix_chimera: bool) {
    let n_vars = path.len();
    for frag in fragments.iter_mut() {
        let mut c = [0u32; 2];
        for &(vid, allele) in &frag.alleles {
            if vid >= n_vars || allele == 0 {
                continue;
            }
            // path[vid] == 0 → allele A is expected (allele value 1).
            let expected = path[vid] + 1;
            if allele == expected {
                c[0] += 1;
            } else {
                c[1] += 1;
            }
        }
        frag.in_phase = c[0];
        frag.out_phase = c[1];
        frag.phased = c[0] != c[1];
        frag.ambiguous = c[0] == c[1];
        frag.phase = if c[0] >= c[1] { 0 } else { 1 };
        frag.flipped = false;

        // Chimera detection (phase.c): only when both counts ≥ 3.
        if !fix_chimera || c[0] < 3 || c[1] < 3 {
            continue;
        }
        let base_score = c[frag.phase as usize] as i32;
        let n = frag.alleles.len();
        let mut best_gain = 0i32;
        let mut best_flip = 0usize;
        for flip_at in 1..n {
            let mut li = 0i32; // left-segment in-phase (head agrees with path)
            let mut ro = 0i32; // right-segment out-of-phase (tail disagrees with path → agrees after flip)
            for (j, &(vid, allele)) in frag.alleles.iter().enumerate() {
                if vid >= n_vars || allele == 0 {
                    continue;
                }
                let expected = path[vid] + 1;
                let m = allele == expected;
                if j < flip_at {
                    if m {
                        li += 1;
                    }
                } else if !m {
                    ro += 1;
                }
            }
            // Gain of flipping the tail at flip_at: head stays in-phase (li),
            // tail flips so out-of-phase becomes in-phase (ro). Minus FLIP_PENALTY.
            let flipped_score = li + ro;
            let gain = flipped_score - base_score - FLIP_PENALTY;
            if gain > best_gain && gain >= FLIP_THRES {
                best_gain = gain;
                best_flip = flip_at;
            }
        }
        if best_flip > 0 {
            // Flip allele encoding after best_flip (swap 1↔2).
            for (j, &mut (_, ref mut allele)) in frag.alleles.iter_mut().enumerate() {
                if j >= best_flip && *allele != 0 {
                    *allele = 3 - *allele; // 1↔2
                }
            }
            frag.flipped = true;
        }
    }
}

/// Identify low-confidence variant positions based on per-haplotype phased-read
/// support (phase.c `genmask`). Returns a per-variant masked flag.
fn genmask(n_vars: usize, path: &[u8], fragments: &[Fragment]) -> Vec<bool> {
    let mut phased_counts: Vec<[i32; 2]> = vec![[0, 0]; n_vars];
    for frag in fragments {
        if !frag.phased {
            continue;
        }
        for &(vid, allele) in &frag.alleles {
            if vid < n_vars && allele != 0 {
                phased_counts[vid][frag.phase as usize] += 1;
            }
        }
    }
    // path.len() == phased_counts.len() == n_vars by construction.
    debug_assert_eq!(path.len(), phased_counts.len());
    phased_counts
        .iter()
        .map(|&[c0, c1]| c0 < MASK_THRES || c1 < MASK_THRES)
        .collect()
}

// ─── text output ─────────────────────────────────────────────────────────────

/// Write the CC format-legend lines (phase.c `output_cc`).
fn write_cc_header<W: Write>(w: &mut W) -> std::io::Result<()> {
    writeln!(w, "CC\tPS\tchr\tphaseSetStart\tphaseSetEnd")?;
    writeln!(w, "CC\tFL\tchr\tfilterStart\tfilterEnd")?;
    writeln!(
        w,
        "CC\tM0\tchr\tpsStart\tpos\tallele0\tallele1\thetIdx\t#supp0\t#err0\t#supp1\t#err1"
    )?;
    writeln!(w, "CC\tM1 = phased (unmasked)")?;
    writeln!(w, "CC\tM2 = masked")?;
    Ok(())
}

// ─── BAM aux tag helpers ──────────────────────────────────────────────────────

/// Append or replace a 4-byte signed-int aux field (BAM type `i`).
fn set_aux_i32(rec: &mut RawRecord, tag: [u8; 2], value: i32) {
    rec.set_aux(tag, b'i', &value.to_le_bytes());
}

// ─── phase block driver ───────────────────────────────────────────────────────

/// Phase one contiguous het-site block and write its text output.
#[allow(clippy::too_many_arguments)]
fn phase_block<W: Write>(
    chrom: &str,
    sites: &[HetSite],
    fragments: &mut [Fragment],
    opts: &PhaseOpts,
    stdout_w: &mut W,
    stats: &mut PhaseStats,
    bam_writers: &mut Option<
        [bam::io::Writer<bgzf::io::Writer<std::io::BufWriter<std::fs::File>>>; 3],
    >,
    _header: &noodles::sam::Header,
) -> Result<()> {
    let n_vars = sites.len();
    if n_vars == 0 || fragments.is_empty() {
        return Ok(());
    }

    stats.phase_sets += 1;

    let weights = build_weights(opts.k, n_vars, fragments);
    let path = dynaprog(opts.k, n_vars, &weights);

    // First pass without chimera fixing to compute masking.
    fragphase(fragments, &path, false);
    let masked = genmask(n_vars, &path, fragments);
    stats.masked_sites += masked.iter().filter(|&&m| m).count() as u64;

    // Second pass with chimera fixing if enabled.
    if opts.fix_chimera {
        fragphase(fragments, &path, true);
    }

    let ps_start = sites[0].pos + 1; // 1-based
    let ps_end = sites[n_vars - 1].pos + 1;

    writeln!(stdout_w, "PS\t{chrom}\t{ps_start}\t{ps_end}").map_err(RsomicsError::Io)?;

    // FL lines for masked runs.
    let mut fl_start: Option<i64> = None;
    for (i, &is_masked) in masked.iter().enumerate() {
        let pos1 = sites[i].pos + 1;
        match (is_masked, fl_start) {
            (true, None) => fl_start = Some(pos1),
            (false, Some(start)) => {
                writeln!(stdout_w, "FL\t{chrom}\t{start}\t{}", sites[i - 1].pos + 1)
                    .map_err(RsomicsError::Io)?;
                fl_start = None;
            }
            _ => {}
        }
    }
    if let Some(start) = fl_start {
        writeln!(stdout_w, "FL\t{chrom}\t{start}\t{ps_end}").map_err(RsomicsError::Io)?;
    }

    // M lines per variant.
    for (i, site) in sites.iter().enumerate() {
        let pos1 = site.pos + 1;
        let marker_type = if masked[i] { 2 } else { 1 };
        let mut supp = [[0u32; 2]; 2]; // supp[hap][0=support, 1=error]
        for frag in fragments.iter() {
            if !frag.phased {
                continue;
            }
            for &(vid, allele) in &frag.alleles {
                if vid != i || allele == 0 {
                    continue;
                }
                let expected = path[i] + 1;
                let is_err = (allele != expected) as usize;
                supp[frag.phase as usize][is_err] += 1;
            }
        }
        writeln!(
            stdout_w,
            "M{marker_type}\t{chrom}\t{ps_start}\t{pos1}\t{}\t{}\t{i}\t{}\t{}\t{}\t{}",
            site.base_a as char,
            site.base_b as char,
            supp[0][0],
            supp[0][1],
            supp[1][0],
            supp[1][1],
        )
        .map_err(RsomicsError::Io)?;
    }

    // Distribute reads to BAM writers and accumulate stats.
    for frag in fragments.iter_mut() {
        let which = hap_bucket(frag, opts);
        // When BAM output is off, records is empty; use record_count for stats.
        let n = if frag.records.is_empty() {
            frag.record_count
        } else {
            frag.records.len() as u64
        };
        match which {
            0 => stats.reads_hap0 += n,
            1 => stats.reads_hap1 += n,
            _ => stats.reads_chimera += n,
        }

        // Tag and write records (only populated when `-b` is given).
        for rec in frag.records.iter_mut() {
            set_aux_i32(rec, *b"YP", frag.phase as i32);
            set_aux_i32(rec, *b"YF", frag.flipped as i32);
            set_aux_i32(rec, *b"YI", frag.in_phase as i32);
            set_aux_i32(rec, *b"YO", frag.out_phase as i32);
            if let Some(first_site) = sites.first() {
                set_aux_i32(rec, *b"YS", (first_site.pos + 1) as i32);
            }

            if let Some(writers) = bam_writers.as_mut() {
                raw::write_record(writers[which].get_mut(), rec)?;
            }
        }
    }

    // Phase-set terminator.
    writeln!(stdout_w, "//").map_err(RsomicsError::Io)?;

    Ok(())
}

/// Determine which BAM bucket (0=hap0, 1=hap1, 2=chimera) a fragment belongs to
/// (phase.c `dump_aln` routing logic).
fn hap_bucket(frag: &Fragment, opts: &PhaseOpts) -> usize {
    if frag.ambiguous && opts.drop_ambiguous {
        return 2;
    }
    if frag.phased && frag.flipped {
        return 2;
    }
    if !frag.phased {
        // Random routing for unphased reads (phase.c uses rand(); we use a
        // deterministic hash of the first allele position to keep output stable).
        return frag.alleles.first().map_or(0, |&(v, _)| v) & 1;
    }
    frag.phase as usize
}

// ─── nibble → ASCII base ──────────────────────────────────────────────────────

/// Decode a 4-bit nibble to an unambiguous ACGT byte, or `b'N'` for anything else.
fn nibble_to_acgt(n: u8) -> u8 {
    match n & 0xf {
        1 => b'A',
        2 => b'C',
        4 => b'G',
        8 => b'T',
        _ => b'N',
    }
}

// ─── main entry point ─────────────────────────────────────────────────────────

/// Phase heterozygous SNPs from `input` BAM. Text output to `stdout_w`; BAM
/// split files created if `opts.bam_prefix` is set.
///
/// Algorithm: two-pass over the BAM file.
///
/// Pass 1 — pileup only, no record storage:
///   Stream every primary alignment; walk its CIGAR to fill a per-position allele
///   accumulator. No records are cloned or buffered. After reading the whole file,
///   convert the pileup into a per-contig het-site list using `call_het`.
///
/// Pass 2 — phasing:
///   Re-open the BAM from the start. For each contig:
///   - If it has no het sites: route records directly to split BAM output with a
///     deterministic hash (no record storage, no clone).
///   - If it has het sites: assign allele calls per fragment, store only records that
///     have ≥1 allele call, then run the DP phaser and write output.
///
/// Two file reads avoids buffering the entire contig in memory, cutting peak RSS
/// from O(contig records × record size) to O(het-site count + phased fragments).
pub fn phase<W: Write>(
    input: &Path,
    stdout_w: &mut W,
    opts: &PhaseOpts,
    workers: NonZero<usize>,
) -> Result<PhaseStats> {
    let mut stats = PhaseStats::default();

    // raw::read_record bypasses the noodles record layer and reads directly from
    // the inner BufRead via reader.get_mut(). bgzf::io::MultithreadedReader's
    // BufRead impl interacts poorly with header-then-raw-read sequences across
    // platforms, so we always use a single-threaded reader here.
    let _ = workers;
    let st = NonZero::<usize>::new(1).unwrap();

    // ── Pass 1: pileup — read every primary record, walk CIGAR, fill accumulator ──
    let header = {
        let mut r = rsomics_bamio::open_with_workers(input, st)?;
        let h = r.read_header().map_err(RsomicsError::Io)?;
        let n_refs = h.reference_sequences().len();
        // Per-contig pileup: two-tier structure to minimise memory.
        //
        // Tier 1 (mono_pileups): position → (count, first_base).  Positions where
        //   only one distinct base has been observed — the vast majority in a
        //   diploid sample.  These can never be het sites.
        //
        // Tier 2 (pileups): position → (count_a, count_b, base_a, base_b).  Only
        //   positions where a SECOND distinct base has been seen are promoted here.
        //   This keeps the full-entry map small (proportional to variant density,
        //   not coverage).
        let mut mono_pileups: Vec<AHashMap<i64, (u32, u8)>> = vec![AHashMap::new(); n_refs];
        let mut pileups: Vec<AHashMap<i64, (u32, u32, u8, u8)>> = vec![AHashMap::new(); n_refs];
        let mut rec = RawRecord::default();
        loop {
            let nbytes = raw::read_record(r.get_mut(), &mut rec)?;
            if nbytes == 0 {
                break;
            }
            stats.records_in += 1;
            let flags = rec.flags();
            if flags
                & (FLAG_UNMAPPED
                    | FLAG_SECONDARY
                    | FLAG_QCFAIL
                    | FLAG_DUPLICATE
                    | FLAG_SUPPLEMENTARY)
                != 0
            {
                continue;
            }
            let tid = rec.reference_sequence_id();
            if tid < 0 || tid as usize >= n_refs {
                continue;
            }
            accumulate_pileup(
                &rec,
                opts.min_base_q,
                &mut mono_pileups[tid as usize],
                &mut pileups[tid as usize],
            );
        }
        (h, pileups)
    };
    let (header, pileups) = header;
    let ref_seqs = header.reference_sequences();
    let n_refs = ref_seqs.len();

    // Identify het sites per contig from the pileup.
    // sites_by_tid[i] = vec of HetSite (sorted by pos) for contig i.
    // Site lookup uses binary search on the sorted position field — faster than
    // a HashMap for the typical 10^2–10^4 site counts per contig.
    let mut sites_by_tid: Vec<Vec<HetSite>> = Vec::with_capacity(n_refs);
    for (tid, pileup) in pileups.into_iter().enumerate() {
        let mut sorted_pos: Vec<i64> = pileup.keys().copied().collect();
        sorted_pos.sort_unstable();
        let mut sites: Vec<HetSite> = Vec::new();
        for pos in sorted_pos {
            let (ca, cb, base_a, base_b) = pileup[&pos];
            let total = ca + cb;
            if total as usize > opts.max_depth {
                continue;
            }
            if base_a == b'N' || base_b == b'N' || base_a == base_b {
                continue;
            }
            if let Some(_lod) = call_het(ca, cb, opts.min_var_lod) {
                sites.push(HetSite {
                    pos,
                    base_a,
                    base_b,
                });
                stats.het_sites += 1;
            }
        }
        let _ = tid; // tid == sites_by_tid.len() at this point
        sites_by_tid.push(sites);
    }

    // Build BAM split writers if -b is specified.
    // bam::io::Writer::new(W) wraps W with a BGZF compressor internally,
    // yielding Writer<bgzf::io::Writer<W>>.
    let mut bam_writers: Option<
        [bam::io::Writer<bgzf::io::Writer<std::io::BufWriter<std::fs::File>>>; 3],
    > = opts
        .bam_prefix
        .as_ref()
        .map(|prefix| -> Result<_> {
            let open = |suffix: &str| -> Result<
                bam::io::Writer<bgzf::io::Writer<std::io::BufWriter<std::fs::File>>>,
            > {
                let path = PathBuf::from(format!("{}.{suffix}.bam", prefix.display()));
                let f = std::fs::File::create(&path).map_err(RsomicsError::Io)?;
                Ok(bam::io::Writer::new(std::io::BufWriter::new(f)))
            };
            let mut w0 = open("0")?;
            let mut w1 = open("1")?;
            let mut wc = open("chimera")?;
            w0.write_header(&header).map_err(RsomicsError::Io)?;
            w1.write_header(&header).map_err(RsomicsError::Io)?;
            wc.write_header(&header).map_err(RsomicsError::Io)?;
            Ok([w0, w1, wc])
        })
        .transpose()?;

    write_cc_header(stdout_w).map_err(RsomicsError::Io)?;

    if n_refs == 0 {
        return Ok(stats);
    }

    // When no split BAM output is requested and there are no het sites anywhere,
    // Pass 2 has nothing to do (no records to route, no text lines to emit).
    // Skip the second file read entirely.
    let any_het = sites_by_tid.iter().any(|s| !s.is_empty());
    if bam_writers.is_none() && !any_het {
        return Ok(stats);
    }

    // ── Pass 2: phasing — re-read BAM, process contig by contig ─────────────────
    //
    // For contigs with no het sites: route each primary record directly to the
    // split BAM bucket (deterministic hash of QNAME) without cloning or buffering.
    //
    // For contigs with het sites: assign allele calls per fragment; store only
    // records that have ≥1 allele call (so we can tag them with YP/YF/YI/YO/YS
    // after the DP). Records with no allele calls are routed immediately.
    let mut reader2 = rsomics_bamio::open_with_workers(input, st)?;
    reader2.read_header().map_err(RsomicsError::Io)?;

    let mut cur_tid: i32 = -1;
    let mut frags: AHashMap<u64, Fragment> = AHashMap::new();
    let mut rec = RawRecord::default();

    // Flush the accumulated fragment map for the contig that just ended.
    macro_rules! flush_phased_contig {
        ($tid:expr) => {{
            let tid: usize = $tid;
            let sites = &sites_by_tid[tid];
            let chrom = ref_seqs
                .get_index(tid)
                .map(|(name, _)| name.to_string())
                .unwrap_or_else(|| format!("ref{tid}"));

            if sites.is_empty() {
                // No het sites: per-record fast path already routed all records
                // directly; nothing buffered here to flush.
            } else {
                let n_sites = sites.len();
                let mut frag_vec: Vec<Fragment> = frags.drain().map(|(_, f)| f).collect();

                // Phase-block boundary detection.
                let mut block_starts: Vec<usize> = vec![0];
                if n_sites > 1 {
                    let mut reach = vec![0usize; n_sites];
                    for f in &frag_vec {
                        let min_v = f.alleles.iter().map(|&(v, _)| v).min().unwrap_or(0);
                        let max_v = f.alleles.iter().map(|&(v, _)| v).max().unwrap_or(0);
                        if min_v < n_sites {
                            reach[min_v] = reach[min_v].max(max_v);
                        }
                    }
                    for i in 1..n_sites {
                        reach[i] = reach[i].max(reach[i - 1]);
                    }
                    for i in 1..n_sites {
                        if reach[i - 1] < i {
                            block_starts.push(i);
                        }
                    }
                }
                block_starts.push(n_sites);

                for w in block_starts.windows(2) {
                    let (blk_lo, blk_hi) = (w[0], w[1]);
                    let blk_sites = &sites[blk_lo..blk_hi];
                    let mut blk_frags: Vec<Fragment> = frag_vec
                        .iter_mut()
                        .filter_map(|f| {
                            let blk_alleles: Vec<(usize, Allele)> = f
                                .alleles
                                .iter()
                                .filter(|&&(v, _)| v >= blk_lo && v < blk_hi)
                                .map(|&(v, a)| (v - blk_lo, a))
                                .collect();
                            if blk_alleles.len() < 2 {
                                return None;
                            }
                            let records = std::mem::take(&mut f.records);
                            let record_count = f.record_count;
                            Some(Fragment {
                                alleles: blk_alleles,
                                records,
                                record_count,
                                ..Default::default()
                            })
                        })
                        .collect();
                    phase_block(
                        &chrom,
                        blk_sites,
                        &mut blk_frags,
                        opts,
                        stdout_w,
                        &mut stats,
                        &mut bam_writers,
                        &header,
                    )?;
                }

                // Fragments with <2 allele calls in every block → route unphased.
                if let Some(writers) = bam_writers.as_mut() {
                    for frag in &mut frag_vec {
                        if frag.records.is_empty() {
                            continue;
                        }
                        let bucket = frag.alleles.first().map_or(0, |&(v, _)| v & 1);
                        for r in frag.records.drain(..) {
                            raw::write_record(writers[bucket].get_mut(), &r)?;
                            match bucket {
                                0 => stats.reads_hap0 += 1,
                                1 => stats.reads_hap1 += 1,
                                _ => stats.reads_chimera += 1,
                            }
                        }
                    }
                }
            }
            frags.clear();
        }};
    }

    loop {
        let nbytes = raw::read_record(reader2.get_mut(), &mut rec)?;
        if nbytes == 0 {
            break;
        }

        let flags = rec.flags();
        if flags
            & (FLAG_UNMAPPED | FLAG_SECONDARY | FLAG_QCFAIL | FLAG_DUPLICATE | FLAG_SUPPLEMENTARY)
            != 0
        {
            continue;
        }

        let tid = rec.reference_sequence_id();
        if tid < 0 || tid as usize >= n_refs {
            continue;
        }
        let tid_u = tid as usize;

        if tid != cur_tid {
            if cur_tid >= 0 {
                flush_phased_contig!(cur_tid as usize);
            }
            cur_tid = tid;
        }

        let sites = &sites_by_tid[tid_u];
        if sites.is_empty() {
            // No het sites on this contig: route directly without buffering.
            if let Some(writers) = bam_writers.as_mut() {
                let bucket = (fnv64(rec.name()) & 1) as usize;
                raw::write_record(writers[bucket].get_mut(), &rec)?;
                match bucket {
                    0 => stats.reads_hap0 += 1,
                    _ => stats.reads_hap1 += 1,
                }
            }
        } else {
            let qhash = fnv64(rec.name());
            let frag = frags.entry(qhash).or_default();
            assign_alleles(&rec, opts.min_base_q, sites, frag);
            if frag.alleles.is_empty() {
                // No allele calls at any het site: route immediately without clone.
                if let Some(writers) = bam_writers.as_mut() {
                    let bucket = (fnv64(rec.name()) & 1) as usize;
                    raw::write_record(writers[bucket].get_mut(), &rec)?;
                    match bucket {
                        0 => stats.reads_hap0 += 1,
                        _ => stats.reads_hap1 += 1,
                    }
                }
            } else {
                // Has allele calls at het sites.
                frag.record_count += 1;
                if bam_writers.is_some() {
                    // Buffer record bytes only when BAM split output is requested.
                    frag.records.push(rec.clone());
                }
            }
        }
    }

    if cur_tid >= 0 {
        flush_phased_contig!(cur_tid as usize);
    }

    // bam_writers drops here; bgzf::io::Writer::drop calls try_finish, writing the
    // BGZF EOF block and flushing any buffered data.

    Ok(stats)
}

// ─── pileup accumulation helpers ─────────────────────────────────────────────

/// Walk one record's CIGAR and accumulate per-position base counts.
///
/// Two-tier pileup: `mono` tracks positions with exactly one distinct base
/// (the common case); `multi` tracks only positions where a second distinct
/// base has appeared (the candidate het sites).  Keeping only variant-density
/// entries in the full-detail `multi` map cuts peak RSS and hash-table
/// overhead proportional to coverage.
fn accumulate_pileup(
    rec: &RawRecord,
    min_bq: u8,
    mono: &mut AHashMap<i64, (u32, u8)>,
    multi: &mut AHashMap<i64, (u32, u32, u8, u8)>,
) {
    let start0 = rec.alignment_start() as i64;
    if start0 < 0 {
        return;
    }
    let qual = rec.quality_scores();
    let seq_len = rec.sequence_len();

    let mut ref_pos = start0;
    let mut qpos = 0usize;

    for (op, len) in rec.cigar_ops() {
        let len = len as usize;
        match op {
            CIGAR_MATCH | CIGAR_EQUAL | CIGAR_DIFF => {
                for i in 0..len {
                    if qpos + i >= seq_len {
                        break;
                    }
                    let bq = if qual.is_empty() {
                        0
                    } else {
                        qual.get(qpos + i).copied().unwrap_or(0)
                    };
                    if bq >= min_bq {
                        let nib = rec.seq_nibble(qpos + i);
                        let base = nibble_to_acgt(nib);
                        if base != b'N' {
                            let rp = ref_pos + i as i64;
                            if let Some(entry) = multi.get_mut(&rp) {
                                // Already a multi-allele site.
                                if entry.2 == base {
                                    entry.0 += 1;
                                } else if entry.3 == base {
                                    entry.1 += 1;
                                }
                                // Third+ allele: ignore (multi-allelic).
                            } else {
                                use ahash::HashMap as AHashMapAlias;
                                let _ = AHashMapAlias::<i64, u8>::new();
                                match mono.entry(rp) {
                                    ahash::hash_map::Entry::Occupied(mut e) => {
                                        let (cnt, b0) = *e.get();
                                        if b0 == base {
                                            e.get_mut().0 += 1;
                                        } else {
                                            // Second distinct base: promote to multi.
                                            e.remove();
                                            multi.insert(rp, (cnt, 1, b0, base));
                                        }
                                    }
                                    ahash::hash_map::Entry::Vacant(e) => {
                                        e.insert((1, base));
                                    }
                                }
                            }
                        }
                    }
                }
                ref_pos += len as i64;
                qpos += len;
            }
            CIGAR_INS | CIGAR_SOFT_CLIP => {
                qpos += len;
            }
            CIGAR_DEL | CIGAR_SKIP => {
                ref_pos += len as i64;
            }
            _ => {} // HARD_CLIP, PAD: no movement.
        }
    }
}

/// Walk one record's CIGAR and fill in allele calls at het sites into `frag`.
///
/// Het-site lookup uses binary search on the sorted `sites` slice (by `pos`).
/// For typical per-contig site counts (10^2–10^4), binary search beats
/// HashMap on this narrow key range.
fn assign_alleles(rec: &RawRecord, min_bq: u8, sites: &[HetSite], frag: &mut Fragment) {
    let start0 = rec.alignment_start() as i64;
    if start0 < 0 {
        return;
    }
    let qual = rec.quality_scores();
    let seq_len = rec.sequence_len();

    let mut ref_pos = start0;
    let mut qpos = 0usize;

    for (op, len) in rec.cigar_ops() {
        let len = len as usize;
        match op {
            CIGAR_MATCH | CIGAR_EQUAL | CIGAR_DIFF => {
                for i in 0..len {
                    if qpos + i >= seq_len {
                        break;
                    }
                    let rp = ref_pos + i as i64;
                    if let Ok(var_id) = sites.binary_search_by_key(&rp, |s| s.pos) {
                        let bq = if qual.is_empty() {
                            0
                        } else {
                            qual.get(qpos + i).copied().unwrap_or(0)
                        };
                        let site = &sites[var_id];
                        let allele = if bq >= min_bq {
                            let nib = rec.seq_nibble(qpos + i);
                            let base = nibble_to_acgt(nib);
                            if base == site.base_a {
                                1
                            } else if base == site.base_b {
                                2
                            } else {
                                0
                            }
                        } else {
                            0
                        };
                        if frag.alleles.len() < MAX_VARS {
                            frag.alleles.push((var_id, allele));
                        }
                    }
                }
                ref_pos += len as i64;
                qpos += len;
            }
            CIGAR_INS | CIGAR_SOFT_CLIP => {
                qpos += len;
            }
            CIGAR_DEL | CIGAR_SKIP => {
                ref_pos += len as i64;
            }
            _ => {}
        }
    }
}

// ─── QNAME hashing ───────────────────────────────────────────────────────────

/// FNV-1a 64-bit hash for QNAME bytes (phase.c uses `X31_hash_string`; we use
/// FNV-1a for similar avalanche properties without the modular arithmetic).
fn fnv64(data: &[u8]) -> u64 {
    const OFFSET: u64 = 0xcbf2_9ce4_8422_2325;
    const PRIME: u64 = 0x0000_0100_0000_01b3;
    let mut h = OFFSET;
    for &b in data {
        h ^= u64::from(b);
        h = h.wrapping_mul(PRIME);
    }
    h
}
