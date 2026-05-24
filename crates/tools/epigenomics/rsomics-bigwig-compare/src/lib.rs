//! Per-bin comparison of two bigWig files as a bedGraph track — deeptools
//! `bigwigCompare`.
//!
//! The genome is tiled into `bin_size`-wide bins. For each chromosome common to
//! both files the per-base bigWig values are read, averaged per bin (deeptools
//! `getCoverageFromBigwig` → `numpy.mean` over each tile slice), then combined
//! per bin by an [`Operation`] (deeptools default `log2`). Output is bedGraph
//! with adjacent equal-value bins merged (unless `--fixedStep`).
//!
//! ## Chromosome set (deeptools `writeBedGraph`)
//!
//! Only chromosomes present in BOTH files are processed, in the first file's
//! B-tree leaf order (the order pyBigWig's `chroms()` dict yields, which
//! deeptools sorts its output by). Where a chromosome's declared length differs
//! between the files, the smaller length is used.
//!
//! ## Per-bin value (deeptools `getCoverageFromBigwig`)
//!
//! Per-base values are read for `[0, chrom_len)`. With `missing_data_as_zero`
//! (deeptools default — `not --skipNonCoveredRegions`) NaN bases become 0
//! first. Each bin's value is `numpy.mean` over its `bin_size`-base slice (the
//! final bin may be shorter). Without `missing_data_as_zero`, an all-NaN bin
//! averages to NaN.
//!
//! ## Combination (deeptools `getRatio` / `compute_ratio`)
//!
//! Per bin, with `v1 = scale[0] * cov1`, `v2 = scale[1] * cov2`. If either
//! scaled value is NaN the result is NaN. Otherwise:
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
//! the ratio-family operations. With `--skipZeroOverZero`, a bin whose two
//! coverage values sum to zero (before any pseudocount) is dropped. bedGraph
//! values use Python's `{:g}` format.

#![allow(clippy::cast_precision_loss)]

use std::collections::HashMap;
use std::io::{BufWriter, Write};
use std::path::Path;

use rsomics_bbi::BigWig;
use rsomics_common::{Result, RsomicsError};

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

/// Whether a ratio-family op (which alone consumes the pseudocount).
impl Operation {
    fn is_ratio(self) -> bool {
        matches!(self, Self::Log2 | Self::Ratio | Self::ReciprocalRatio)
    }
}

#[derive(Debug, Clone)]
pub struct CompareOpts {
    /// Bin size in bases (deeptools default: 50).
    pub bin_size: u32,
    pub operation: Operation,
    /// Per-file multiplicative scale (deeptools `--scaleFactors`, default
    /// `[1, 1]`).
    pub scale_factors: [f64; 2],
    /// Pseudocount `[numerator, denominator]` for ratio-family operations
    /// (deeptools default `[1, 1]`).
    pub pseudocount: [f64; 2],
    /// Treat bigWig-absent bases as 0 (deeptools default — the inverse of
    /// `--skipNonCoveredRegions`).
    pub missing_data_as_zero: bool,
    /// Drop bins whose two coverage values sum to zero, before any pseudocount
    /// (deeptools `--skipZeroOverZero`).
    pub skip_zero_over_zero: bool,
    /// Emit every bin instead of merging adjacent equal values (deeptools
    /// `--fixedStep`).
    pub fixed_step: bool,
}

impl Default for CompareOpts {
    fn default() -> Self {
        Self {
            bin_size: 50,
            operation: Operation::Log2,
            scale_factors: [1.0, 1.0],
            pseudocount: [1.0, 1.0],
            missing_data_as_zero: true,
            skip_zero_over_zero: false,
            fixed_step: false,
        }
    }
}

/// Open both bigWigs, combine per bin over the common chromosomes, emit
/// bedGraph to `output`. Returns the number of bedGraph lines written.
pub fn bigwig_compare(
    bw1: &Path,
    bw2: &Path,
    output: &mut dyn Write,
    opts: &CompareOpts,
) -> Result<u64> {
    let mut a = BigWig::open(bw1)?;
    let mut b = BigWig::open(bw2)?;

    let common = common_chroms(&a, &b);

    let mut out = BufWriter::with_capacity(256 * 1024, output);
    let mut lines: u64 = 0;
    for (chrom, len) in &common {
        lines += write_chrom(&mut out, &mut a, &mut b, chrom, *len, opts)?;
    }
    out.flush().map_err(RsomicsError::Io)?;
    Ok(lines)
}

/// Chromosomes present in BOTH files, in the first file's order. Length is the
/// smaller of the two when they disagree (deeptools' `min`).
fn common_chroms(a: &BigWig, b: &BigWig) -> Vec<(String, u32)> {
    let b_lens: HashMap<&str, u32> = b.chroms().collect();
    a.chroms()
        .filter_map(|(name, alen)| {
            b_lens
                .get(name)
                .map(|&blen| (name.to_string(), alen.min(blen)))
        })
        .collect()
}

/// Per-bin average bigWig values over `[0, len)`, deeptools
/// `getCoverageFromBigwig`: per-base values (NaN→0 when `missing_as_zero`)
/// averaged in `bin_size`-base tiles, the last tile possibly shorter.
fn binned_values(
    bw: &mut BigWig,
    chrom: &str,
    len: u32,
    bin_size: u32,
    missing_as_zero: bool,
) -> Result<Vec<f64>> {
    let per_base = bw.values(chrom, 0, len)?.ok_or_else(|| {
        RsomicsError::InvalidInput(format!("chromosome {chrom} vanished from bigWig"))
    })?;

    let n_bins = (len as usize).div_ceil(bin_size as usize);
    let bs = bin_size as usize;
    let mut bins = Vec::with_capacity(n_bins);
    let mut x = 0usize;
    while x < per_base.len() {
        let end = (x + bs).min(per_base.len());
        let slice = &per_base[x..end];
        // deeptools zero-fills NaN bases before the mean when missing_as_zero;
        // otherwise numpy.mean of a slice containing any NaN is itself NaN.
        let mut sum = 0.0f64;
        let mut any_nan = false;
        for &v in slice {
            if v.is_nan() {
                any_nan = true;
            } else {
                sum += f64::from(v);
            }
        }
        let mean = if slice.is_empty() || (any_nan && !missing_as_zero) {
            f64::NAN
        } else {
            sum / slice.len() as f64
        };
        bins.push(mean);
        x += bs;
    }
    Ok(bins)
}

/// deeptools `getRatio`: scale, NaN-propagate, then combine.
fn get_ratio(cov1: f64, cov2: f64, opts: &CompareOpts) -> f64 {
    let v1 = opts.scale_factors[0] * cov1;
    let v2 = opts.scale_factors[1] * cov2;
    if v1.is_nan() || v2.is_nan() {
        return f64::NAN;
    }
    if opts.operation.is_ratio() {
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
    } else {
        match opts.operation {
            Operation::Subtract => v1 - v2,
            Operation::Add => v1 + v2,
            // deeptools/numpy compute `(v1 + v2) / 2.0` (not f64::midpoint);
            // the bedGraph must match it bit-for-bit.
            #[allow(clippy::manual_midpoint)]
            Operation::Mean => (v1 + v2) / 2.0,
            Operation::First => v1,
            Operation::Second => v2,
            _ => unreachable!(),
        }
    }
}

/// Write one chromosome's combined bins, mirroring deeptools
/// `writeBedGraph_worker`: skip-zero-over-zero, then either fixedStep (every
/// bin) or run-length merge (adjacent equal values; NaN bins never written; a
/// trailing run whose value is 0 or NaN is also not written).
fn write_chrom(
    out: &mut impl Write,
    a: &mut BigWig,
    b: &mut BigWig,
    chrom: &str,
    len: u32,
    opts: &CompareOpts,
) -> Result<u64> {
    let cov1 = binned_values(a, chrom, len, opts.bin_size, opts.missing_data_as_zero)?;
    let cov2 = binned_values(b, chrom, len, opts.bin_size, opts.missing_data_as_zero)?;

    let bin_size = u64::from(opts.bin_size);
    let chrom_len = u64::from(len);
    let n = cov1.len();
    let mut lines: u64 = 0;

    let mut prev: Option<f64> = None;
    let mut write_start: u64 = 0;
    let mut write_end: u64 = 0;

    for i in 0..n {
        let c1 = cov1[i];
        let c2 = cov2[i];

        // skipZeroOverZero: sum of the two coverage values == 0, before
        // pseudocount. NaN never equals 0 so a NaN bin is not skipped here.
        if opts.skip_zero_over_zero && (c1 + c2) == 0.0 {
            prev = None;
            continue;
        }

        let value = get_ratio(c1, c2, opts);

        if opts.fixed_step {
            let ws = i as u64 * bin_size;
            let we = (ws + bin_size).min(chrom_len);
            write_line(out, chrom, ws, we, value)?;
            lines += 1;
            continue;
        }

        match prev {
            None => {
                write_start = i as u64 * bin_size;
                write_end = (write_start + bin_size).min(chrom_len);
                prev = Some(value);
            }
            Some(pv) if bits_eq(pv, value) => {
                write_end = (write_end + bin_size).min(chrom_len);
            }
            Some(pv) => {
                if !pv.is_nan() {
                    write_line(out, chrom, write_start, write_end, pv)?;
                    lines += 1;
                }
                prev = Some(value);
                write_start = write_end;
                write_end = (write_start + bin_size).min(chrom_len);
            }
        }
    }

    if !opts.fixed_step
        && let Some(pv) = prev
        // deeptools: `if previousValue and writeStart != end and not isnan`.
        // `previousValue` is falsy for 0.0, so a trailing 0-run is not written.
        && pv != 0.0
        && !pv.is_nan()
        && write_start != chrom_len
    {
        write_line(out, chrom, write_start, chrom_len, pv)?;
        lines += 1;
    }

    Ok(lines)
}

/// deeptools merges on Python `==`, which is exact float equality (NaN never
/// equal). We mirror that: bit-exact equality, NaN != NaN.
fn bits_eq(a: f64, b: f64) -> bool {
    a == b
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

#[cfg(test)]
mod tests {
    use super::*;

    fn opts(op: Operation) -> CompareOpts {
        CompareOpts {
            operation: op,
            ..CompareOpts::default()
        }
    }

    #[test]
    fn ratio_doctest() {
        // deeptools getRatio doctest: ratio [9,19] pc [1,1] = 10/20 = 0.5
        let mut o = opts(Operation::Ratio);
        o.pseudocount = [1.0, 1.0];
        assert!((get_ratio(9.0, 19.0, &o) - 0.5).abs() < 1e-12);
    }

    #[test]
    fn ratio_zero_over_zero() {
        // [0,0] pc [1,1] = 1/1 = 1.0
        let o = opts(Operation::Ratio);
        assert!((get_ratio(0.0, 0.0, &o) - 1.0).abs() < 1e-12);
    }

    #[test]
    fn nan_propagates() {
        let o = opts(Operation::Ratio);
        assert!(get_ratio(f64::NAN, 1.0, &o).is_nan());
        assert!(get_ratio(1.0, f64::NAN, &o).is_nan());
    }

    #[test]
    fn subtract_no_pseudocount() {
        let o = opts(Operation::Subtract);
        // [20,10] = 10, pseudocount NOT applied to non-ratio ops
        assert!((get_ratio(20.0, 10.0, &o) - 10.0).abs() < 1e-12);
    }

    #[test]
    fn reciprocal_ratio_no_pc() {
        let mut o = opts(Operation::ReciprocalRatio);
        o.pseudocount = [0.0, 0.0];
        assert!((get_ratio(2.0, 1.0, &o) - 2.0).abs() < 1e-12); // 2/1=2 >=1
        assert!((get_ratio(1.0, 2.0, &o) - (-2.0)).abs() < 1e-12); // 1/2<1 → -1/(0.5)=-2
        assert!((get_ratio(1.0, 1.0, &o) - 1.0).abs() < 1e-12);
    }

    #[test]
    fn scale_factor_applied() {
        let mut o = opts(Operation::First);
        o.scale_factors = [0.5, 1.0];
        assert!((get_ratio(10.0, 20.0, &o) - 5.0).abs() < 1e-12);
    }

    #[test]
    fn g_format_basics() {
        assert_eq!(format_g(0.0), "0");
        assert_eq!(format_g(-0.0), "0");
        assert_eq!(format_g(1.0), "1");
        assert_eq!(format_g(0.5), "0.5");
        assert_eq!(format_g(f64::NAN), "nan");
        assert_eq!(format_g(1.5), "1.5");
        assert_eq!(format_g(-1.0), "-1");
    }
}
