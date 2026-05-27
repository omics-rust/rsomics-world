//! Subsample-based splice-junction saturation analysis.
//!
//! Algorithm (reimplemented from RSeQC junction_saturation.py):
//! 1. Parse BED12 gene models; extract all annotated splice sites (junction
//!    donor and acceptor positions) and annotated junctions (donor, acceptor
//!    pairs) into hash sets.
//! 2. Load all mapped, primary reads from the BAM.
//! 3. Shuffle all read indices once with a seeded ChaCha12 RNG.
//! 4. For each fraction F in [lower..upper] step S:
//!    a. Take the first ⌊F% × total_reads⌋ indices from the shuffled order.
//!    b. For each selected read, extract introns from the CIGAR `N` operations.
//!    c. Classify each observed junction as:
//!       - known: both donor and acceptor in the annotated junction set
//!       - partial novel: one of donor/acceptor is annotated
//!       - complete novel: neither is annotated
//! 5. Write one TSV file `<prefix>.junction_saturation.txt` with columns:
//!    `pct\tknown\tpartial_novel\tcomplete_novel`
//!
//! Using a single shuffle (prefix-based sampling) guarantees monotonicity:
//! the read set at fraction F1 is always a subset of the set at F2 > F1.
//! RSeQC's subsampling is non-deterministic (Python `random`). We use a
//! seedable ChaCha12 RNG so results are reproducible when `--seed` is given.

use std::collections::HashSet;
use std::io::{BufRead, Write};
use std::num::NonZero;
use std::path::Path;

use rand::SeedableRng;
use rand::seq::SliceRandom;
use rand_chacha::ChaCha12Rng;
#[allow(clippy::wildcard_imports)]
use rayon::prelude::*;
use rsomics_common::{Result, RsomicsError};

/// A splice junction: (chrom, donor_pos, acceptor_pos) in 0-based coordinates.
/// Donor = intron start (one past last exon base), acceptor = intron end (first
/// base of next exon). Stored as (intron_start, intron_end) 0-based half-open.
type Junction = (String, u64, u64);

/// Per-fraction junction counts.
#[derive(Debug, Clone, Default)]
pub struct FractionResult {
    pub pct: u8,
    pub known: usize,
    pub partial_novel: usize,
    pub complete_novel: usize,
}

/// Options for junction saturation analysis.
#[derive(Debug, Clone)]
pub struct JunctionSaturationOpts {
    /// Lower sampling fraction (percent), inclusive. Default 5.
    pub lower: u8,
    /// Upper sampling fraction (percent), inclusive. Default 100.
    pub upper: u8,
    /// Step between fractions. Default 5.
    pub step: u8,
    /// Minimum mapping quality. Default 0.
    pub min_mapq: u8,
    /// Minimum intron length to count a CIGAR N op as a real splice junction.
    /// Matches RSeQC default of 50 bp.
    pub min_intron: u64,
    /// RNG seed (None = random seed).
    pub seed: Option<u64>,
    /// BGZF inflate threads.
    pub threads: NonZero<usize>,
}

impl Default for JunctionSaturationOpts {
    fn default() -> Self {
        Self {
            lower: 5,
            upper: 100,
            step: 5,
            min_mapq: 0,
            min_intron: 50,
            seed: None,
            threads: NonZero::new(1).unwrap(),
        }
    }
}

// ── BED12 annotated junction extraction ──────────────────────────────────────

/// A splice site: (chrom, position) 0-based.
type SpliceSite = (String, u64);

/// Load all annotated splice junctions and individual splice sites from a
/// BED12 gene model file.
///
/// Returns `(junctions, splice_sites)` where:
/// - `junctions`: set of `(chrom, intron_start, intron_end)` 0-based half-open
/// - `splice_sites`: set of `(chrom, pos)` for every donor and acceptor
pub(crate) fn load_annotated_junctions(
    path: &Path,
) -> Result<(HashSet<Junction>, HashSet<SpliceSite>)> {
    let f = std::fs::File::open(path)
        .map_err(|e| RsomicsError::InvalidInput(format!("{}: {e}", path.display())))?;
    let reader = std::io::BufReader::new(f);
    let mut junctions: HashSet<Junction> = HashSet::new();
    let mut splice_sites: HashSet<SpliceSite> = HashSet::new();

    for (lineno, line) in reader.lines().enumerate() {
        let line = line.map_err(RsomicsError::Io)?;
        let line = line.trim();
        if line.is_empty() || line.starts_with('#') || line.starts_with("track") {
            continue;
        }
        let cols: Vec<&str> = line.split('\t').collect();
        if cols.len() < 12 {
            return Err(RsomicsError::InvalidInput(format!(
                "BED12 requires 12 columns at line {}; got {}",
                lineno + 1,
                cols.len()
            )));
        }

        let chrom = cols[0].to_string();
        let tx_start: u64 = cols[1]
            .parse()
            .map_err(|_| RsomicsError::InvalidInput(format!("bad start at line {}", lineno + 1)))?;
        let block_count: usize = cols[9].parse().map_err(|_| {
            RsomicsError::InvalidInput(format!("bad blockCount at line {}", lineno + 1))
        })?;
        if block_count < 2 {
            continue; // single-exon gene — no junctions
        }
        let block_sizes: Vec<u64> = cols[10]
            .trim_end_matches(',')
            .split(',')
            .map(|s| {
                s.trim().parse::<u64>().map_err(|_| {
                    RsomicsError::InvalidInput(format!("bad blockSize at line {}", lineno + 1))
                })
            })
            .collect::<Result<_>>()?;
        let block_starts: Vec<u64> = cols[11]
            .trim_end_matches(',')
            .split(',')
            .map(|s| {
                s.trim().parse::<u64>().map_err(|_| {
                    RsomicsError::InvalidInput(format!("bad blockStart at line {}", lineno + 1))
                })
            })
            .collect::<Result<_>>()?;

        if block_sizes.len() < block_count || block_starts.len() < block_count {
            return Err(RsomicsError::InvalidInput(format!(
                "blockSizes/blockStarts length mismatch at line {}",
                lineno + 1
            )));
        }

        // Build exon absolute coordinates.
        let exons: Vec<(u64, u64)> = (0..block_count)
            .map(|i| {
                let start = tx_start + block_starts[i];
                let end = start + block_sizes[i];
                (start, end)
            })
            .collect();

        // Each consecutive exon pair gives one junction:
        // donor = exon[i].end, acceptor = exon[i+1].start.
        for i in 0..(block_count - 1) {
            let donor = exons[i].1; // end of exon i (0-based, exclusive)
            let acceptor = exons[i + 1].0; // start of exon i+1 (0-based, inclusive)
            if donor >= acceptor {
                continue; // degenerate
            }
            junctions.insert((chrom.clone(), donor, acceptor));
            splice_sites.insert((chrom.clone(), donor));
            splice_sites.insert((chrom.clone(), acceptor));
        }
    }

    Ok((junctions, splice_sites))
}

// ── CIGAR parsing: extract introns from BAM records ──────────────────────────

/// A raw BAM record's CIGAR-derived junction: (chrom_idx, intron_start,
/// intron_end) where intron_start is 0-based.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
struct RawJunction {
    ref_id: i32,
    start: u64,
    end: u64,
}

/// Extract splice junctions from CIGAR operations.
/// Only CIGAR N operations with length >= `min_intron` are returned.
fn extract_junctions(
    cigar: &[(u8, u32)],
    ref_start: u64,
    min_intron: u64,
    ref_id: i32,
) -> Vec<RawJunction> {
    let mut junctions = Vec::new();
    let mut ref_pos = ref_start;

    for &(op_code, op_len) in cigar {
        let op_len = op_len as u64;
        match op_code {
            // M, X, = : consume reference
            0 | 7 | 8 => ref_pos += op_len,
            // D: consume reference (deletion)
            2 => ref_pos += op_len,
            // N: intron (skip reference)
            3 => {
                let intron_start = ref_pos;
                let intron_end = ref_pos + op_len;
                ref_pos = intron_end;
                if op_len >= min_intron {
                    junctions.push(RawJunction {
                        ref_id,
                        start: intron_start,
                        end: intron_end,
                    });
                }
            }
            // I, S, H, P: don't consume reference
            _ => {}
        }
    }
    junctions
}

// ── BAM loading ───────────────────────────────────────────────────────────────

struct RawRead {
    ref_id: i32,
    pos: u64,
    cigar: Vec<(u8, u32)>,
}

/// Load all usable reads from a BAM file into memory.
/// Filters: mapped, primary (not SECONDARY/SUPPLEMENTARY), not QCFAIL/DUP,
/// MAPQ >= `min_mapq`.
pub(crate) fn load_reads(
    bam: &Path,
    min_mapq: u8,
    threads: NonZero<usize>,
) -> Result<(Vec<RawRead>, Vec<String>)> {
    let mut reader = rsomics_bamio::open_with_workers(bam, threads)?;
    let header = reader.read_header().map_err(RsomicsError::Io)?;

    let ref_names: Vec<String> = header
        .reference_sequences()
        .keys()
        .map(|n| n.to_string())
        .collect();

    let inner = reader.get_mut();
    let mut reads = Vec::new();
    let mut raw = rsomics_bamio::raw::RawRecord::default();

    loop {
        let n = rsomics_bamio::raw::read_record(inner, &mut raw)?;
        if n == 0 {
            break;
        }

        let flags = raw.flags();
        // Skip unmapped, secondary, supplementary, QC-fail, duplicate.
        const SKIP: u16 = 0x4 | 0x100 | 0x800 | 0x200 | 0x400;
        if flags & SKIP != 0 {
            continue;
        }
        if raw.mapping_quality() < min_mapq {
            continue;
        }
        let ref_id = raw.reference_sequence_id();
        if ref_id < 0 {
            continue;
        }
        let pos = raw.alignment_start() as u64;
        // Collect CIGAR ops eagerly so the record can be overwritten.
        let cigar: Vec<(u8, u32)> = raw.cigar_ops().collect();

        reads.push(RawRead { ref_id, pos, cigar });
    }

    Ok((reads, ref_names))
}

// ── Saturation computation ────────────────────────────────────────────────────

/// Run junction saturation analysis.
///
/// Writes one output file: `<prefix>.junction_saturation.txt` with columns
/// `pct\tknown\tpartial_novel\tcomplete_novel`.
pub fn run(bam: &Path, bed: &Path, prefix: &str, opts: &JunctionSaturationOpts) -> Result<()> {
    let (anno_junctions, splice_sites) = load_annotated_junctions(bed)?;
    let (reads, ref_names) = load_reads(bam, opts.min_mapq, opts.threads)?;

    let n_reads = reads.len();

    // Pre-extract all junctions from all reads.
    let all_read_junctions: Vec<Vec<RawJunction>> = reads
        .par_iter()
        .map(|r| extract_junctions(&r.cigar, r.pos, opts.min_intron, r.ref_id))
        .collect();

    // Build the fraction list.
    let fractions: Vec<u8> = {
        let mut f = Vec::new();
        let mut pct = opts.lower;
        while pct <= opts.upper {
            f.push(pct);
            pct = pct.saturating_add(opts.step);
            if pct > opts.upper && f.last().copied() != Some(opts.upper) {
                f.push(opts.upper);
                break;
            }
        }
        f.dedup();
        f
    };

    let seed = opts.seed.unwrap_or_else(rand::random);

    // Shuffle all read indices once with the global seed.  Each fraction then
    // takes the first ⌊F% × N⌋ indices from this single permutation, so
    // sample(F1) ⊆ sample(F2) for F1 ≤ F2 — monotonicity is guaranteed by
    // construction.
    let shuffled_indices: Vec<usize> = {
        let mut rng = ChaCha12Rng::seed_from_u64(seed);
        let mut idx: Vec<usize> = (0..n_reads).collect();
        idx.shuffle(&mut rng);
        idx
    };

    // For each fraction, scan the first N shuffled reads and count junctions.
    let results: Vec<FractionResult> = fractions
        .par_iter()
        .map(|&pct| {
            let n_sample = ((n_reads as f64 * pct as f64 / 100.0).floor() as usize).min(n_reads);

            // Collect unique junctions from sampled reads.
            let mut obs: HashSet<(i32, u64, u64)> = HashSet::new();
            for &idx in &shuffled_indices[..n_sample] {
                for junc in &all_read_junctions[idx] {
                    obs.insert((junc.ref_id, junc.start, junc.end));
                }
            }

            // Classify junctions.
            let mut known = 0usize;
            let mut partial_novel = 0usize;
            let mut complete_novel = 0usize;

            for (ref_id, start, end) in &obs {
                let chrom = match ref_names.get(*ref_id as usize) {
                    Some(n) => n,
                    None => continue,
                };
                let donor_key = (chrom.clone(), *start);
                let acceptor_key = (chrom.clone(), *end);
                let has_donor = splice_sites.contains(&donor_key);
                let has_acceptor = splice_sites.contains(&acceptor_key);

                let junc_key = (chrom.clone(), *start, *end);
                if anno_junctions.contains(&junc_key) {
                    known += 1;
                } else if has_donor || has_acceptor {
                    partial_novel += 1;
                } else {
                    complete_novel += 1;
                }
            }

            FractionResult {
                pct,
                known,
                partial_novel,
                complete_novel,
            }
        })
        .collect();

    // Write output.
    let out_path = format!("{prefix}.junction_saturation.txt");
    let mut out = std::io::BufWriter::new(
        std::fs::File::create(&out_path)
            .map_err(|e| RsomicsError::InvalidInput(format!("{out_path}: {e}")))?,
    );

    writeln!(out, "pct\tknown\tpartial_novel\tcomplete_novel").map_err(RsomicsError::Io)?;
    for r in &results {
        writeln!(
            out,
            "{}\t{}\t{}\t{}",
            r.pct, r.known, r.partial_novel, r.complete_novel
        )
        .map_err(RsomicsError::Io)?;
    }

    Ok(())
}
