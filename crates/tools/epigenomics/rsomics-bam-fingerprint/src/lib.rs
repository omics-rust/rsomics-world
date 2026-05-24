//! ChIP-enrichment fingerprint, matching deeptools `plotFingerprint` default semantics.
//!
//! ## What the fingerprint is
//!
//! The genome is sampled at evenly spaced windows (bins). Each bin's value is
//! the per-base read coverage summed over the bin. Sorting those per-bin values
//! and plotting their normalised cumulative sum against the normalised bin rank
//! gives a Lorenz curve: a diagonal means coverage is uniform (no enrichment),
//! a sharp elbow near the right means a few bins hold most of the signal (strong
//! ChIP enrichment).
//!
//! ## Deterministic sampling (source parity with deeptools)
//!
//! deeptools does **not** pick bins at random. With the whole genome it sets
//! `stepSize = max(genomeSize / numberOfSamples, 1)` and walks each chromosome
//! at `stepSize` intervals, taking one bin `[i, i + binSize)` per step. The walk
//! happens inside genome "chunks" (`mapReduce` partitions) of length `chunkSize`,
//! and a bin that would cross a chunk boundary (`i + binSize > chunkEnd`) is
//! dropped — so chunk geometry, not just stepSize, decides which bins exist.
//! `chunkSize` itself is derived from the BAM's mapped-read count:
//!
//! ```text
//! reads_per_bp = max_mapped / genomeSize
//! chunkSize    = floor(stepSize * 1000 / (reads_per_bp * nBams))
//! chunkSize    = max(chunkSize, stepSize, binSize)
//! ```
//!
//! Reproducing all of this makes our sampled-bin set identical to deeptools'
//! when run with one BAM at `--numberOfProcessors 1` (no task shuffle).
//!
//! ## Per-bin coverage rule (`SumCoveragePerBin`)
//!
//! For each bin `[reg0, reg1)` (a single-tile region, `tileSize = binSize`):
//! every read with `FLAG & 0x4 == 0` is decomposed into aligned blocks
//! (`get_blocks()`: reference-consuming M/=/X runs, split by D and N). Each block
//! contributes `min(blockEnd, reg1) - max(blockStart, reg0)` bases (clamped to
//! `[0, binSize]`) to the bin. Default filters: skip unmapped only — no MAPQ
//! floor, no FLAG include/exclude, duplicates kept, reads not extended
//! (`extendReads=False` → `defaultFragmentLength == "read length"`).
//!
//! ## Output (`--out-raw-counts`)
//!
//! Mirrors `plotFingerprint --outRawCounts`: a `#plotFingerprint --outRawCounts`
//! header, a quoted label line, then one integer per sampled bin (the per-base
//! coverage, formatted `%d` exactly as deeptools casts the float column). Bins
//! appear in genome order (chromosome header order, ascending position).
//!
//! ## Fingerprint + summary (`--out-fingerprint`, `--out-quality-metrics`)
//!
//! The fingerprint is the sorted, cumulative, max-normalised coverage vs the
//! normalised rank — the data behind deeptools' PNG. The quality metrics
//! (AUC, X-intercept, elbow) use deeptools' exact formulas over the sorted
//! cumulative curve.
//!
//! ## Scope
//!
//! We emit the data tables (raw counts, fingerprint curve, summary metrics), not
//! the PNG plot — the same split as `rsomics-bam-signal`, which emits bedGraph
//! rather than bigWig. Synthetic/JSD/CHANCE columns (only produced with
//! `--JSDsample` against a reference BAM) are out of scope for this
//! single-input crate.

#![allow(clippy::cast_precision_loss)]

use std::io::{BufWriter, Write};
use std::num::NonZero;
use std::path::Path;

use rsomics_bamio::raw::{self, RawRecord};
use rsomics_common::{Result, RsomicsError};

const CIGAR_MATCH: u8 = 0;
const CIGAR_INSERTION: u8 = 1;
const CIGAR_DELETION: u8 = 2;
const CIGAR_SKIP: u8 = 3;
const CIGAR_SOFT_CLIP: u8 = 4;
const CIGAR_SEQ_MATCH: u8 = 7;
const CIGAR_SEQ_MISMATCH: u8 = 8;

#[derive(Debug, Clone)]
pub struct FingerprintOpts {
    /// Window size in bp (deeptools default: 500).
    pub bin_size: u32,
    /// Target number of sampled bins across the genome (deeptools default: 500000).
    pub number_of_samples: u64,
    /// Minimum mapping quality (deeptools default: 0 = no filter).
    pub min_mapq: u8,
    /// Skip reads whose FLAG has any of these bits set (deeptools `samFlagExclude`,
    /// default None → 0). Unmapped (0x4) reads are always skipped regardless.
    pub sam_flag_exclude: u16,
    /// Keep only reads whose FLAG has all of these bits set (deeptools
    /// `samFlagInclude`, default None → 0 = no requirement).
    pub sam_flag_include: u16,
}

impl Default for FingerprintOpts {
    fn default() -> Self {
        Self {
            bin_size: 500,
            number_of_samples: 500_000,
            min_mapq: 0,
            sam_flag_exclude: 0,
            sam_flag_include: 0,
        }
    }
}

struct ChromGeom {
    tid: usize,
    length: u64,
}

/// A coverage region as deeptools constructs it in `count_reads_in_region`.
///
/// When `stepSize != binSize` each region is one bin (`n_bins == 1`); when
/// `stepSize == binSize` a whole genome chunk becomes one tiled region of
/// `n_bins` contiguous `bin_size` tiles (the partial tail bin is dropped, as
/// deeptools floor-divides). `out_lo` is this region's offset into the global
/// genome-ordered counts vector.
struct Region {
    tid: usize,
    start: u64,
    /// Exclusive end of the region as deeptools' region tuple records it
    /// (`reg[1]`): the bin end for a single-bin region, the chunk end for a
    /// tiled region (which may exceed `start + n_bins * bin_size` when the
    /// partial tail bin was floor-dropped).
    end: u64,
    n_bins: u64,
    out_lo: usize,
}

/// The fingerprint result: per-bin per-base coverage in genome order.
pub struct Fingerprint {
    pub counts: Vec<u64>,
}

impl Fingerprint {
    /// Sorted, cumulative, max-normalised curve (the y-axis of the plot).
    /// Returned paired with the normalised rank (x-axis), both length N.
    pub fn cumulative_curve(&self) -> Vec<(f64, f64)> {
        let n = self.counts.len();
        let mut sorted = self.counts.clone();
        sorted.sort_unstable();
        let total: u64 = sorted.iter().sum();
        let mut acc: u64 = 0;
        sorted
            .iter()
            .enumerate()
            .map(|(i, &v)| {
                acc += v;
                let x = i as f64 / n as f64;
                let y = if total == 0 {
                    0.0
                } else {
                    acc as f64 / total as f64
                };
                (x, y)
            })
            .collect()
    }

    /// deeptools quality metrics over the sorted cumulative curve: (AUC,
    /// X-intercept, elbow). Formulas mirror `plotFingerprint.main`.
    pub fn quality_metrics(&self) -> QualityMetrics {
        let n = self.counts.len();
        let mut sorted = self.counts.clone();
        sorted.sort_unstable();
        let total: u64 = sorted.iter().sum();

        let mut counts = Vec::with_capacity(n);
        let mut acc: u64 = 0;
        for &v in &sorted {
            acc += v;
            counts.push(if total == 0 {
                0.0
            } else {
                acc as f64 / total as f64
            });
        }

        let auc = counts.iter().sum::<f64>() / n as f64;

        let x_int = counts
            .iter()
            .position(|&c| c > 0.0)
            .map_or(0.0, |k| (k + 1) as f64 / n as f64);

        let elbow = if n <= 1 {
            0.0
        } else {
            let mut best_k = 0usize;
            let mut best = f64::NEG_INFINITY;
            for (i, &c) in counts.iter().enumerate() {
                let line = i as f64 / (n as f64 - 1.0);
                let diff = line - c;
                if diff > best {
                    best = diff;
                    best_k = i;
                }
            }
            (best_k + 1) as f64 / n as f64
        };

        QualityMetrics { auc, x_int, elbow }
    }
}

pub struct QualityMetrics {
    pub auc: f64,
    pub x_int: f64,
    pub elbow: f64,
}

/// Compute the per-bin per-base coverage fingerprint for `input`.
pub fn fingerprint(
    input: &Path,
    opts: &FingerprintOpts,
    workers: NonZero<usize>,
) -> Result<Fingerprint> {
    let chroms = read_chrom_geom(input, workers)?;
    let genome_size: u64 = chroms.iter().map(|c| c.length).sum();
    if genome_size == 0 {
        return Err(RsomicsError::InvalidInput(
            "BAM header declares zero genome length".into(),
        ));
    }

    let mapped = count_mapped(input, opts, workers)?;
    if mapped == 0 {
        return Err(RsomicsError::InvalidInput(
            "no mapped reads found — check MAPPING quality / flag filters".into(),
        ));
    }

    let step_size = (genome_size / opts.number_of_samples).max(1);

    let mean_chrom_len = genome_size as f64 / chroms.len() as f64;
    if mean_chrom_len < step_size as f64 {
        let min_samples = (genome_size as f64 / mean_chrom_len) as u64;
        return Err(RsomicsError::InvalidInput(format!(
            "--number-of-samples has to be bigger than {min_samples}"
        )));
    }

    let chunk_size = chunk_length(step_size, opts.bin_size, mapped, genome_size, 1);
    let bin_size = u64::from(opts.bin_size);

    let (regions, n_bins) = layout_regions(&chroms, step_size, bin_size, chunk_size);
    if n_bins == 0 {
        return Err(RsomicsError::InvalidInput(
            "no bins were sampled — decrease --bin-size or --number-of-samples".into(),
        ));
    }

    let counts = accumulate_coverage(
        input, opts, workers, &chroms, &regions, n_bins, step_size, bin_size,
    )?;
    Ok(Fingerprint { counts })
}

fn read_chrom_geom(input: &Path, workers: NonZero<usize>) -> Result<Vec<ChromGeom>> {
    let mut reader = rsomics_bamio::open_with_workers(input, workers)?;
    let header = reader.read_header().map_err(RsomicsError::Io)?;
    Ok(header
        .reference_sequences()
        .iter()
        .enumerate()
        .map(|(tid, (_, seq))| ChromGeom {
            tid,
            length: usize::from(seq.length()) as u64,
        })
        .collect())
}

/// Reads with `FLAG & 0x4 == 0`, matching pysam `.mapped` (the idxstats mapped
/// total deeptools uses to derive `chunkSize`). No MAPQ or include/exclude
/// filtering enters this count — only the unmapped bit.
fn count_mapped(input: &Path, _opts: &FingerprintOpts, workers: NonZero<usize>) -> Result<u64> {
    let mut reader = rsomics_bamio::open_with_workers(input, workers)?;
    reader.read_header().map_err(RsomicsError::Io)?;
    let mut record = RawRecord::default();
    let mut mapped: u64 = 0;
    while raw::read_record(reader.get_mut(), &mut record)? != 0 {
        if record.flags() & 0x4 == 0 && record.reference_sequence_id() >= 0 {
            mapped += 1;
        }
    }
    Ok(mapped)
}

/// deeptools `get_chunk_length` with `len(bamFilesHandles) == n_bams`.
fn chunk_length(
    step_size: u64,
    bin_size: u32,
    max_mapped: u64,
    genome_size: u64,
    n_bams: u64,
) -> u64 {
    let reads_per_bp = max_mapped as f64 / genome_size as f64;
    let mut chunk = (step_size as f64 * 1e3 / (reads_per_bp * n_bams as f64)) as u64;
    if chunk < step_size {
        chunk = step_size;
    }
    if u64::from(bin_size) > 0 && chunk < u64::from(bin_size) {
        chunk = u64::from(bin_size);
    }
    chunk
}

/// Builds deeptools' coverage regions in genome order. Mirrors the `mapReduce`
/// chunk walk (`range(0, size, chunkSize)`) crossed with `count_reads_in_region`:
///
/// - `stepSize == binSize` → each chunk is one tiled region spanning
///   `nBins = (chunkEnd - chunkStart) // binSize` contiguous tiles (partial tail
///   dropped, deeptools floor-divides).
/// - `stepSize != binSize` → one single-bin region per `range(chunkStart,
///   chunkEnd, stepSize)` step, skipping a bin that crosses the chunk end.
///
/// Returns the regions plus the total bin count (the length of the output).
fn layout_regions(
    chroms: &[ChromGeom],
    step_size: u64,
    bin_size: u64,
    chunk_size: u64,
) -> (Vec<Region>, usize) {
    let tiled = step_size == bin_size;
    let mut regions = Vec::new();
    let mut out_lo = 0usize;
    for chrom in chroms {
        let mut chunk_start = 0u64;
        while chunk_start < chrom.length {
            let chunk_end = (chunk_start + chunk_size).min(chrom.length);
            if tiled {
                let n_bins = (chunk_end - chunk_start) / bin_size;
                if n_bins > 0 {
                    regions.push(Region {
                        tid: chrom.tid,
                        start: chunk_start,
                        end: chunk_end,
                        n_bins,
                        out_lo,
                    });
                    out_lo += n_bins as usize;
                }
            } else {
                let mut i = chunk_start;
                while i < chunk_end {
                    if i + bin_size > chunk_end {
                        break;
                    }
                    regions.push(Region {
                        tid: chrom.tid,
                        start: i,
                        end: i + bin_size,
                        n_bins: 1,
                        out_lo,
                    });
                    out_lo += 1;
                    i += step_size;
                }
            }
            chunk_start += chunk_size;
        }
    }
    (regions, out_lo)
}

/// Single streaming pass: every kept read is routed to its chromosome's regions
/// and its aligned blocks are spread across the bins via deeptools'
/// `SumCoveragePerBin.get_coverage_of_region` arithmetic.
#[allow(clippy::too_many_arguments)]
fn accumulate_coverage(
    input: &Path,
    opts: &FingerprintOpts,
    workers: NonZero<usize>,
    chroms: &[ChromGeom],
    regions: &[Region],
    n_bins: usize,
    step_size: u64,
    bin_size: u64,
) -> Result<Vec<u64>> {
    // Per-chromosome slice [lo, hi) into `regions`, indexed by tid. regions are
    // grouped by chromosome in header order, ascending start within each.
    let mut chrom_span: Vec<(usize, usize)> = vec![(0, 0); chroms.len()];
    {
        let mut idx = 0usize;
        while idx < regions.len() {
            let tid = regions[idx].tid;
            let lo = idx;
            while idx < regions.len() && regions[idx].tid == tid {
                idx += 1;
            }
            chrom_span[tid] = (lo, idx);
        }
    }

    let mut counts = vec![0i64; n_bins];

    let mut reader = rsomics_bamio::open_with_workers(input, workers)?;
    reader.read_header().map_err(RsomicsError::Io)?;
    let mut record = RawRecord::default();
    let mut blocks: Vec<(u64, u64)> = Vec::new();

    while raw::read_record(reader.get_mut(), &mut record)? != 0 {
        let flags = record.flags();
        if flags & 0x4 != 0 {
            continue;
        }
        let tid = record.reference_sequence_id();
        if tid < 0 {
            continue;
        }
        if opts.sam_flag_include != 0 && (flags & opts.sam_flag_include) != opts.sam_flag_include {
            continue;
        }
        if opts.sam_flag_exclude != 0 && (flags & opts.sam_flag_exclude) != 0 {
            continue;
        }
        if opts.min_mapq > 0 && record.mapping_quality() < opts.min_mapq {
            continue;
        }

        let tid = tid as usize;
        let (lo, hi) = match chrom_span.get(tid) {
            Some(&span) if span.0 < span.1 => span,
            _ => continue,
        };
        let chrom_regions = &regions[lo..hi];

        let start0 = record.alignment_start() as u64;
        aligned_blocks(start0, &record, &mut blocks);
        add_read(&mut counts, chrom_regions, &blocks, step_size, bin_size);
    }

    Ok(counts.into_iter().map(|c| c.max(0) as u64).collect())
}

/// pysam `get_blocks()`: aligned blocks of reference-consuming CIGAR. M/=/X
/// extend the current block; D, N **and I** all break it (pysam emits a fresh
/// block at every insertion even though I consumes no reference, so the two
/// blocks abut). Soft/hard clips and padding are ignored. Fills `out` (cleared
/// first) so the caller can reuse one allocation across records.
fn aligned_blocks(start0: u64, record: &RawRecord, out: &mut Vec<(u64, u64)>) {
    blocks_from_cigar(start0, record.cigar_ops(), out);
}

fn blocks_from_cigar(
    start0: u64,
    cigar: impl Iterator<Item = (u8, u32)>,
    out: &mut Vec<(u64, u64)>,
) {
    out.clear();
    let mut pos = start0;
    let mut block_start = start0;
    let mut in_block = false;
    for (kind, len) in cigar {
        let len = u64::from(len);
        match kind {
            CIGAR_MATCH | CIGAR_SEQ_MATCH | CIGAR_SEQ_MISMATCH => {
                if !in_block {
                    block_start = pos;
                    in_block = true;
                }
                pos += len;
            }
            // I, D and N all break the current block (pysam emits a fresh block
            // at every insertion too, even though I consumes no reference).
            CIGAR_INSERTION | CIGAR_DELETION | CIGAR_SKIP => {
                if in_block {
                    out.push((block_start, pos));
                    in_block = false;
                }
                if kind != CIGAR_INSERTION {
                    pos += len;
                }
            }
            CIGAR_SOFT_CLIP => {}
            _ => {}
        }
    }
    if in_block && pos > block_start {
        out.push((block_start, pos));
    }
}

/// Routes one read's aligned blocks into its chromosome's regions, applying
/// deeptools' `SumCoveragePerBin.get_coverage_of_region` arithmetic per region.
/// `chrom_regions` is ascending by start; the read can touch the (single) tiled
/// region of its chunk, or — in the sparse single-bin layout — at most a handful
/// of consecutive single-bin regions.
fn add_read(
    counts: &mut [i64],
    chrom_regions: &[Region],
    blocks: &[(u64, u64)],
    step_size: u64,
    bin_size: u64,
) {
    let Some(&(read_start, _)) = blocks.first() else {
        return;
    };
    let read_end = blocks.last().unwrap().1;

    let _ = step_size;
    // First region whose end is past read_start (deeptools fetches reads in
    // [reg.start, reg.end); a read overlapping that span is processed).
    let mut r = chrom_regions.partition_point(|reg| reg.end <= read_start);
    while r < chrom_regions.len() {
        let reg = &chrom_regions[r];
        if reg.start >= read_end {
            break;
        }
        cover_region(
            &mut counts[reg.out_lo..reg.out_lo + reg.n_bins as usize],
            reg,
            blocks,
            bin_size,
        );
        r += 1;
    }
}

/// deeptools `SumCoveragePerBin.get_coverage_of_region` for one region. `cov` is
/// the bin slice for this region (`tileSize == bin_size`, `nRegBins == n_bins`).
/// Per-base coverage is spread across tiles with deeptools' exact integer
/// arithmetic — including the `ceil`/`eIdx` clamp and `last_eIdx` block guard
/// that the upstream tiled path uses (and that make wide reads over-count a tile
/// rather than clipping to true per-base overlap; matched for byte parity).
fn cover_region(cov: &mut [i64], reg: &Region, blocks: &[(u64, u64)], bin_size: u64) {
    let reg0 = reg.start as i64;
    let reg1 = reg.end as i64;
    let tile = bin_size as i64;
    let n_reg_bins = reg.n_bins as i64;
    let len_cov = cov.len() as i64;
    let cov_end = reg0 + len_cov * tile;

    let mut last_eidx: Option<i64> = None;
    for &(bs, be) in blocks {
        let mut frag_start = bs as i64;
        let mut frag_end = be as i64;
        if frag_end - frag_start == 0 {
            continue;
        }
        if frag_end <= reg0 || frag_start >= reg1 {
            continue;
        }
        if frag_start < reg0 {
            frag_start = reg0;
        }
        if frag_end > cov_end {
            frag_end = cov_end;
        }

        let mut s_idx = ((frag_start - reg0) / tile).max(0);
        let mut e_idx = (div_ceil_i64(frag_end - reg0, tile)).min(n_reg_bins);
        if e_idx >= len_cov {
            e_idx = len_cov - 1;
        }
        if let Some(last) = last_eidx {
            s_idx = last.max(s_idx);
            if s_idx >= e_idx {
                continue;
            }
        }

        // First bin: partial overlap from frag_start to the bin's end.
        let first = if frag_end < reg0 + (s_idx + 1) * tile {
            frag_end - frag_start
        } else {
            reg0 + (s_idx + 1) * tile - frag_start
        };
        cov[s_idx as usize] += first.min(tile);

        let mut k = s_idx + 1;
        while k < e_idx {
            cov[k as usize] += tile;
            k += 1;
        }
        while e_idx - s_idx >= n_reg_bins {
            e_idx -= 1;
        }
        if e_idx > s_idx {
            let mut last = frag_end - (reg0 + e_idx * tile);
            if last > tile {
                last = tile;
            } else if last < 0 {
                last = 0;
            }
            cov[e_idx as usize] += last;
        }
        last_eidx = Some(e_idx);
    }
}

fn div_ceil_i64(a: i64, b: i64) -> i64 {
    (a + b - 1) / b
}

/// Write the `--outRawCounts` table: deeptools header, quoted label, one `%d`
/// per sampled bin in genome order.
pub fn write_raw_counts(out: &mut dyn Write, label: &str, fp: &Fingerprint) -> Result<()> {
    let mut w = BufWriter::new(out);
    writeln!(w, "#plotFingerprint --outRawCounts").map_err(RsomicsError::Io)?;
    writeln!(w, "'{label}'").map_err(RsomicsError::Io)?;
    for &c in &fp.counts {
        writeln!(w, "{c}").map_err(RsomicsError::Io)?;
    }
    w.flush().map_err(RsomicsError::Io)
}

/// Write the fingerprint curve: `rank<TAB>fraction`, one row per sampled bin.
pub fn write_fingerprint(out: &mut dyn Write, fp: &Fingerprint) -> Result<()> {
    let mut w = BufWriter::new(out);
    writeln!(w, "#rank\tfraction").map_err(RsomicsError::Io)?;
    for (x, y) in fp.cumulative_curve() {
        writeln!(w, "{x}\t{y}").map_err(RsomicsError::Io)?;
    }
    w.flush().map_err(RsomicsError::Io)
}

/// Write the quality-metrics summary: deeptools' AUC / X-intercept / elbow.
pub fn write_quality_metrics(out: &mut dyn Write, label: &str, fp: &Fingerprint) -> Result<()> {
    let m = fp.quality_metrics();
    let mut w = BufWriter::new(out);
    writeln!(w, "Sample\tAUC\tX-intercept\tElbow Point").map_err(RsomicsError::Io)?;
    writeln!(w, "{}\t{}\t{}\t{}", label, m.auc, m.x_int, m.elbow).map_err(RsomicsError::Io)?;
    w.flush().map_err(RsomicsError::Io)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn blocks(record_blocks: &[(u64, u64)]) -> Vec<(u64, u64)> {
        record_blocks.to_vec()
    }

    #[test]
    fn chunk_length_floors_to_step_and_bin() {
        // reads_per_bp = 1.0; chunk = step*1000 / (1*1) = 500000, dominates.
        assert_eq!(chunk_length(500, 500, 1_000_000, 1_000_000, 1), 500_000);
        // dense: reads_per_bp large → chunk floored to step.
        let c = chunk_length(50, 50, 1_000_000, 1_000, 1);
        assert!(c >= 50);
    }

    #[test]
    fn div_ceil_matches_python_ceil() {
        assert_eq!(div_ceil_i64(5, 200), 1);
        assert_eq!(div_ceil_i64(200, 200), 1);
        assert_eq!(div_ceil_i64(201, 200), 2);
        assert_eq!(div_ceil_i64(1046, 200), 6);
    }

    #[test]
    fn single_bin_region_is_clamped_per_base_overlap() {
        // One bin [1000,1200), one read block [1096,1146): contributes 50 bp.
        let reg = Region {
            tid: 0,
            start: 1000,
            end: 1200,
            n_bins: 1,
            out_lo: 0,
        };
        let mut cov = vec![0i64];
        cover_region(&mut cov, &reg, &blocks(&[(1096, 1146)]), 200);
        assert_eq!(cov[0], 50);
    }

    #[test]
    fn tiled_region_spreads_full_middle_tile() {
        // deeptools tiled quirk: a read [959,1009) with sIdx=4 eIdx=6 dumps a
        // full 200 into the middle tile (bin 5) even though it only reaches 1009.
        let reg = Region {
            tid: 0,
            start: 0,
            end: 10000,
            n_bins: 50,
            out_lo: 0,
        };
        let mut cov = vec![0i64; 50];
        cover_region(&mut cov, &reg, &blocks(&[(959, 1009)]), 200);
        assert_eq!(cov[5], 200);
        assert_eq!(cov[4], 41); // 1000 - 959
    }

    #[test]
    fn blocks_split_on_insertion_deletion_skip() {
        let mut out = Vec::new();
        // 30M5I25M from 1999 → pysam [(1999,2029),(2029,2054)] (split on I).
        blocks_from_cigar(
            1999,
            [(CIGAR_MATCH, 30), (CIGAR_INSERTION, 5), (CIGAR_MATCH, 25)].into_iter(),
            &mut out,
        );
        assert_eq!(out, vec![(1999, 2029), (2029, 2054)]);

        // 40M5D20M from 1199 → [(1199,1239),(1244,1264)] (split on D, gap 5).
        blocks_from_cigar(
            1199,
            [(CIGAR_MATCH, 40), (CIGAR_DELETION, 5), (CIGAR_MATCH, 20)].into_iter(),
            &mut out,
        );
        assert_eq!(out, vec![(1199, 1239), (1244, 1264)]);

        // 30M100N30M from 499 → [(499,529),(629,659)] (split on N, gap 100).
        blocks_from_cigar(
            499,
            [(CIGAR_MATCH, 30), (CIGAR_SKIP, 100), (CIGAR_MATCH, 30)].into_iter(),
            &mut out,
        );
        assert_eq!(out, vec![(499, 529), (629, 659)]);

        // 5S55M from 1599 → soft-clip ignored, [(1599,1654)].
        blocks_from_cigar(
            1599,
            [(CIGAR_SOFT_CLIP, 5), (CIGAR_MATCH, 55)].into_iter(),
            &mut out,
        );
        assert_eq!(out, vec![(1599, 1654)]);
    }

    #[test]
    fn quality_metrics_uniform_curve() {
        // Uniform counts → AUC ≈ 0.5+, elbow near 1, x-int = 1/n.
        let fp = Fingerprint {
            counts: vec![10; 100],
        };
        let m = fp.quality_metrics();
        assert!((m.x_int - 0.01).abs() < 1e-12);
        assert!(m.auc > 0.0 && m.auc < 1.0);
    }

    #[test]
    fn quality_metrics_all_zero_is_safe() {
        let fp = Fingerprint {
            counts: vec![0; 10],
        };
        let m = fp.quality_metrics();
        assert_eq!(m.auc, 0.0);
        assert_eq!(m.x_int, 0.0);
    }
}
