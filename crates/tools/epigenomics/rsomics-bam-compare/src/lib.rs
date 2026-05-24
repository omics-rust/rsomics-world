//! Per-bin comparison of two BAMs as a bedGraph or bigWig track — deeptools
//! `bamCompare`.
//!
//! Both BAMs are binned by the shared [`rsomics_coverage_core`] primitive (same
//! tiling as `bamCoverage`), each scaled, then combined per bin by an
//! [`Operation`] (deeptools default `log2`). The genome binning lives in Layer A;
//! this crate is the bamCompare-specific layer: the readCount scale factors, the
//! per-bin two-value combination, and the bedGraph/bigWig emit.
//!
//! ## Scaling (deeptools `--scaleFactorsMethod readCount`, the default)
//!
//! For mapped read counts `m1`, `m2` (after the FLAG / MAPQ filter), each BAM's
//! scale factor is `min(m1, m2) / mi` — the larger library is scaled down to the
//! smaller (deeptools `bamCompare.get_scale_factors`). `--normalizeUsing` is
//! `None` by default and only valid with `--scaleFactorsMethod None`, so with
//! defaults the readCount factor is the only scaling applied.
//!
//! ## Combination (deeptools `getRatio` / `compute_ratio`)
//!
//! Per bin, with `v1 = scale[0] * cov1`, `v2 = scale[1] * cov2`:
//!
//! - `log2` — `log2((v1 + pc0) / (v2 + pc1))`
//! - `ratio` — `(v1 + pc0) / (v2 + pc1)`
//! - `reciprocal_ratio` — `r` if `r >= 1` else `-1 / r`, where
//!   `r = (v1 + pc0) / (v2 + pc1)`
//! - `subtract` — `v1 - v2`
//! - `add` — `v1 + v2`
//! - `mean` — `(v1 + v2) / 2`
//! - `first` — `v1`
//! - `second` — `v2`
//!
//! The pseudocount `[pc0, pc1]` (deeptools default `[1, 1]`) is added only for
//! the ratio-family operations (`log2` / `ratio` / `reciprocal_ratio`). bedGraph
//! values use Python's `{:g}` format.

#![allow(clippy::cast_precision_loss)]

use std::io::{BufWriter, Write};
use std::num::NonZero;
use std::path::Path;

use rsomics_bbi::{ChromInfo, Interval, write_bigwig};
use rsomics_common::{Result, RsomicsError};
use rsomics_coverage_core::{BinFilter, BinnedCoverage, ChromBins, compute_coverage};

/// Output format for the comparison track.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum OutputFormat {
    BedGraph,
    #[default]
    BigWig,
}

impl std::str::FromStr for OutputFormat {
    type Err = String;
    fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
        match s {
            "bedgraph" | "bedGraph" => Ok(Self::BedGraph),
            "bigwig" | "bigWig" | "BigWig" => Ok(Self::BigWig),
            _ => Err(format!(
                "unknown output format '{s}'; choose bedgraph or bigwig"
            )),
        }
    }
}

/// How two scaled per-bin coverage values are combined.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Operation {
    Log2,
    Ratio,
    ReciprocalRatio,
    Subtract,
    Add,
    Mean,
    First,
    Second,
}

impl std::str::FromStr for Operation {
    type Err = String;
    fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
        match s {
            "log2" => Ok(Self::Log2),
            "ratio" => Ok(Self::Ratio),
            "reciprocal_ratio" => Ok(Self::ReciprocalRatio),
            "subtract" => Ok(Self::Subtract),
            "add" => Ok(Self::Add),
            "mean" => Ok(Self::Mean),
            "first" => Ok(Self::First),
            "second" => Ok(Self::Second),
            _ => Err(format!(
                "unknown operation '{s}'; choose log2 ratio reciprocal_ratio \
                 subtract add mean first second"
            )),
        }
    }
}

#[derive(Debug, Clone)]
pub struct CompareOpts {
    /// Bin size in bases (deeptools default: 50).
    pub bin_size: u32,
    /// Skip reads whose FLAG has any of these bits set (deeptools default 0).
    pub skip_flags: u16,
    /// Minimum mapping quality (deeptools default 0 = no filter).
    pub min_mapq: u8,
    pub operation: Operation,
    /// Pseudocount `[numerator, denominator]` for ratio-family operations
    /// (deeptools default `[1, 1]`).
    pub pseudocount: [f64; 2],
}

impl Default for CompareOpts {
    fn default() -> Self {
        Self {
            bin_size: 50,
            skip_flags: 0,
            min_mapq: 0,
            operation: Operation::Log2,
            pseudocount: [1.0, 1.0],
        }
    }
}

/// Bin both BAMs, combine per bin, emit bedGraph to `output`. Returns line count.
pub fn bam_compare(
    bam1: &Path,
    bam2: &Path,
    output: &mut dyn Write,
    opts: &CompareOpts,
    workers: NonZero<usize>,
) -> Result<u64> {
    let filter = BinFilter {
        skip_flags: opts.skip_flags,
        min_mapq: opts.min_mapq,
    };
    let cov1 = compute_coverage(bam1, opts.bin_size, filter, workers)?;
    let cov2 = compute_coverage(bam2, opts.bin_size, filter, workers)?;

    ensure_same_geometry(&cov1, &cov2)?;

    let scale = read_count_scale_factors(cov1.total_mapped, cov2.total_mapped);

    let bin_size = u64::from(opts.bin_size);
    let mut out = BufWriter::with_capacity(256 * 1024, output);
    let mut lines: u64 = 0;

    for (c1, c2) in cov1.chroms.iter().zip(&cov2.chroms) {
        if c1.bins.is_empty() {
            continue;
        }
        lines += write_chrom_bedgraph(&mut out, c1, c2, bin_size, scale, opts)?;
    }

    out.flush().map_err(RsomicsError::Io)?;
    Ok(lines)
}

/// Bin both BAMs, combine per bin, write a bigWig file to `output_path`.
///
/// Computes the same per-bin combined values as [`bam_compare`] (same readCount
/// scaling, same operation, same run-length merging), then converts to
/// `rsomics_bbi::Interval` sorted by `(chrom_id, start)` and calls
/// [`write_bigwig`].
pub fn bam_compare_bigwig(
    bam1: &Path,
    bam2: &Path,
    output_path: &Path,
    opts: &CompareOpts,
    workers: NonZero<usize>,
) -> Result<()> {
    let filter = BinFilter {
        skip_flags: opts.skip_flags,
        min_mapq: opts.min_mapq,
    };
    let cov1 = compute_coverage(bam1, opts.bin_size, filter, workers)?;
    let cov2 = compute_coverage(bam2, opts.bin_size, filter, workers)?;

    ensure_same_geometry(&cov1, &cov2)?;

    let scale = read_count_scale_factors(cov1.total_mapped, cov2.total_mapped);
    let bin_size = u64::from(opts.bin_size);

    let mut chroms_info: Vec<ChromInfo> = Vec::new();
    let mut intervals: Vec<Interval> = Vec::new();

    for (chrom_idx, (c1, c2)) in cov1.chroms.iter().zip(&cov2.chroms).enumerate() {
        if c1.bins.is_empty() {
            continue;
        }
        let chrom_id = u32::try_from(chrom_idx)
            .map_err(|_| RsomicsError::InvalidInput("too many chromosomes".into()))?;
        chroms_info.push(ChromInfo {
            name: c1.name.clone(),
            id: chrom_id,
            length: u32::try_from(c1.chrom_len).unwrap_or(u32::MAX),
        });
        collect_chrom_intervals(c1, c2, chrom_id, bin_size, scale, opts, &mut intervals);
    }

    let mut out = std::fs::File::create(output_path).map_err(RsomicsError::Io)?;
    write_bigwig(&mut out, &chroms_info, &intervals, opts.bin_size)?;
    Ok(())
}

/// Collect run-length-merged intervals for one chromosome into `out`.
#[allow(clippy::cast_possible_truncation)] // genomic coords fit u32; f64→f32 is the bigWig format
fn collect_chrom_intervals(
    c1: &ChromBins,
    c2: &ChromBins,
    chrom_id: u32,
    bin_size: u64,
    scale: [f64; 2],
    opts: &CompareOpts,
    out: &mut Vec<Interval>,
) {
    let n = c1.bins.len();
    let mut write_start: u64 = 0;
    let mut write_end: u64 = 0;
    let mut prev_val: Option<f64> = None;

    let flush = |start: u64, end: u64, val: f64, out: &mut Vec<Interval>| {
        if start == end {
            return;
        }
        out.push(Interval {
            chrom_id,
            start: start as u32,
            end: end as u32,
            value: val as f32,
        });
    };

    for i in 0..n {
        let bin_start = i as u64 * bin_size;
        let bin_end = ((i as u64 + 1) * bin_size).min(c1.chrom_len);
        let value = combine(c1.bins[i], c2.bins[i], scale, opts);

        match prev_val {
            None => {
                write_start = bin_start;
                write_end = bin_end;
                prev_val = Some(value);
            }
            Some(pv) if values_equal(pv, value) => {
                write_end = bin_end;
            }
            Some(pv) => {
                flush(write_start, write_end, pv, out);
                write_start = bin_start;
                write_end = bin_end;
                prev_val = Some(value);
            }
        }

        if i + 1 == n
            && let Some(pv) = prev_val
            && write_start != write_end
        {
            flush(write_start, write_end, pv, out);
        }
    }
}

/// The two BAMs must share an identical reference set (name + length + order):
/// deeptools requires matched geometry so bins line up index-for-index.
fn ensure_same_geometry(a: &BinnedCoverage, b: &BinnedCoverage) -> Result<()> {
    if a.chroms.len() != b.chroms.len() {
        return Err(RsomicsError::InvalidInput(format!(
            "BAMs differ in reference count: {} vs {}",
            a.chroms.len(),
            b.chroms.len()
        )));
    }
    for (x, y) in a.chroms.iter().zip(&b.chroms) {
        if x.name != y.name || x.chrom_len != y.chrom_len {
            return Err(RsomicsError::InvalidInput(format!(
                "BAM reference mismatch: {} (len {}) vs {} (len {})",
                x.name, x.chrom_len, y.name, y.chrom_len
            )));
        }
    }
    Ok(())
}

/// deeptools readCount scaling: the larger library is scaled down to the
/// smaller. `scale[i] = min(m1, m2) / mi`. A zero read count yields a 0 factor
/// for the empty side and 1 for the other (deeptools' `min/0` would be `inf`;
/// guarding to 0 keeps an empty BAM from poisoning every bin with NaN).
fn read_count_scale_factors(m1: u64, m2: u64) -> [f64; 2] {
    let min = m1.min(m2) as f64;
    let s1 = if m1 == 0 { 0.0 } else { min / m1 as f64 };
    let s2 = if m2 == 0 { 0.0 } else { min / m2 as f64 };
    [s1, s2]
}

/// Combine two scaled coverage values per the operation. Mirrors deeptools
/// `getRatio` + `compute_ratio`: ratio-family ops add the pseudocount and divide
/// (float `inf` / `nan` propagate as numpy would), others operate directly.
fn combine(cov1: u32, cov2: u32, scale: [f64; 2], opts: &CompareOpts) -> f64 {
    let v1 = scale[0] * f64::from(cov1);
    let v2 = scale[1] * f64::from(cov2);

    match opts.operation {
        Operation::Subtract => v1 - v2,
        Operation::Add => v1 + v2,
        // Not `f64::midpoint`: deeptools/numpy compute `(v1 + v2) / 2.0`, and
        // the bedGraph must match it bit-for-bit.
        #[allow(clippy::manual_midpoint)]
        Operation::Mean => (v1 + v2) / 2.0,
        Operation::First => v1,
        Operation::Second => v2,
        Operation::Log2 | Operation::Ratio | Operation::ReciprocalRatio => {
            let num = v1 + opts.pseudocount[0];
            let den = v2 + opts.pseudocount[1];
            let ratio = num / den;
            match opts.operation {
                Operation::Log2 => ratio.log2(),
                Operation::Ratio => ratio,
                Operation::ReciprocalRatio => {
                    if ratio >= 1.0 {
                        ratio
                    } else {
                        -1.0 / ratio
                    }
                }
                _ => unreachable!(),
            }
        }
    }
}

/// Write one chromosome's combined bins as merged bedGraph lines. Adjacent
/// equal-value bins are collapsed (deeptools `writeBedGraph` run-length
/// encoding). All bins (including those that combine to zero) are written.
fn write_chrom_bedgraph(
    out: &mut impl Write,
    c1: &rsomics_coverage_core::ChromBins,
    c2: &rsomics_coverage_core::ChromBins,
    bin_size: u64,
    scale: [f64; 2],
    opts: &CompareOpts,
) -> Result<u64> {
    let mut lines: u64 = 0;
    let mut write_start: u64 = 0;
    let mut write_end: u64 = 0;
    let mut prev_val: Option<f64> = None;

    let n = c1.bins.len();
    for i in 0..n {
        let bin_start = i as u64 * bin_size;
        let bin_end = ((i as u64 + 1) * bin_size).min(c1.chrom_len);

        let value = combine(c1.bins[i], c2.bins[i], scale, opts);

        match prev_val {
            None => {
                write_start = bin_start;
                write_end = bin_end;
                prev_val = Some(value);
            }
            Some(pv) if values_equal(pv, value) => {
                write_end = bin_end;
            }
            Some(pv) => {
                write_line(out, &c1.name, write_start, write_end, pv)?;
                lines += 1;
                write_start = bin_start;
                write_end = bin_end;
                prev_val = Some(value);
            }
        }

        if i + 1 == n
            && let Some(pv) = prev_val
            && write_start != write_end
        {
            write_line(out, &c1.name, write_start, write_end, pv)?;
            lines += 1;
        }
    }

    Ok(lines)
}

/// Two combined values are "the same bin" when they format identically at output
/// precision — deeptools merges on the written value, so string equality is the
/// canonical test (a relative epsilon mis-splits near the format boundary).
fn values_equal(a: f64, b: f64) -> bool {
    format_g(a) == format_g(b)
}

fn write_line(out: &mut impl Write, chrom: &str, start: u64, end: u64, value: f64) -> Result<()> {
    let s = format_g(value);
    writeln!(out, "{chrom}\t{start}\t{end}\t{s}").map_err(RsomicsError::Io)
}

/// Format a float like Python's `{:g}` (6 significant digits, trailing zeros
/// stripped). deeptools writes bedGraph values with `{:g}`.
fn format_g(v: f64) -> String {
    if v == 0.0 {
        // {:g} prints negative-zero as "-0"; numpy/deeptools emit "0".
        return "0".to_owned();
    }
    if v.is_nan() {
        return "nan".to_owned();
    }
    if v.is_infinite() {
        return if v > 0.0 { "inf" } else { "-inf" }.to_owned();
    }
    python_g(v)
}

/// Python `{:g}`: 6 significant digits, switching to exponent form outside
/// `1e-4..1e16`, with trailing zeros (and a bare trailing `.`) stripped.
///
/// The decimal exponent is taken from a 6-sig-fig scientific render
/// (`{:.5e}`) rather than `log10().floor()` — that render already rounds to
/// the precision Python uses, so its exponent is the post-rounding one (a
/// value like `999999.6` rounds up to `1e6`, which `log10().floor()` would
/// mis-bucket as exponent 5).
fn python_g(v: f64) -> String {
    let sci = format!("{v:.5e}");
    let (_, exp_str) = sci.split_once('e').unwrap();
    let exp: i32 = exp_str.parse().unwrap();

    if !(-4..16).contains(&exp) {
        return normalise_exponential(&sci);
    }
    let decimals = usize::try_from((5 - exp).max(0)).unwrap();
    let s = format!("{v:.decimals$}");
    let s = if s.contains('.') {
        s.trim_end_matches('0').trim_end_matches('.')
    } else {
        &s
    };
    s.to_owned()
}

/// Rust's `{:e}` gives e.g. `1.5e2` / `1e-5`; Python `{:g}` wants `1.5e+02` /
/// `1e-05` (sign always present, exponent ≥ 2 digits) with mantissa zeros
/// stripped.
fn normalise_exponential(s: &str) -> String {
    let (mantissa, exp) = s.split_once('e').unwrap();
    let mantissa = if mantissa.contains('.') {
        mantissa.trim_end_matches('0').trim_end_matches('.')
    } else {
        mantissa
    };
    let (sign, digits) = match exp.strip_prefix('-') {
        Some(rest) => ('-', rest),
        None => ('+', exp.strip_prefix('+').unwrap_or(exp)),
    };
    format!("{mantissa}e{sign}{digits:0>2}")
}
