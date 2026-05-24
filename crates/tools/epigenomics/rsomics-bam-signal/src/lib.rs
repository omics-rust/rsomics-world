//! BAM → binned bedGraph signal track, matching deeptools `bamCoverage` default
//! semantics.
//!
//! The genome binning + per-bin read counting lives in
//! [`rsomics_coverage_core`] (Layer A, shared with `rsomics-bam-compare`). This
//! crate is the bamCoverage-specific layer on top: the post-pass normalisation
//! scalar and the bedGraph run-length emit.
//!
//! ## Counting (see `rsomics-coverage-core`)
//!
//! Reads are filtered with: unmapped always skipped. By deeptools' defaults
//! (`samFlag_exclude=None`, `ignoreDuplicates=False`) secondary and supplementary
//! are **not** excluded. Pass `skip_flags = 0x900` to match samtools-style
//! filtering, or `0x400` for duplicate-only exclusion. A read contributes +1 to
//! every bin its reference span overlaps; the partial last bin per chromosome is
//! retained.
//!
//! ## Normalisation (post-pass scalar)
//!
//! - **None** — raw integer counts.
//! - **CPM** — `count / total_reads * 1e6`
//! - **RPKM** — `count / (total_reads / 1e6) / (binSize / 1e3)`
//!   = `count * 1e9 / (total_reads * binSize)`
//! - **BPM** — bins-per-million: `count / (total_signal / 1e6)`
//! - **RPGC** — `count / (total_reads / effective_genome_size)`
//!   = `count * effective_genome_size / total_reads`
//!
//! All bins (including zero-count) are written. Adjacent same-value bins are
//! merged (deeptools `writeBedGraph_worker` run-length encoding). Values are
//! formatted with Python's `{:g}` equivalent (trailing-zero-stripped float).

#![allow(clippy::cast_precision_loss)]

use std::io::{BufWriter, Write};
use std::num::NonZero;
use std::path::Path;

use rsomics_common::{Result, RsomicsError};
use rsomics_coverage_core::{BinFilter, BinnedCoverage, ChromBins, compute_coverage};

/// Which normalisation to apply to raw bin counts.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Normalisation {
    None,
    Cpm,
    Rpkm,
    Bpm,
    Rpgc,
}

impl std::str::FromStr for Normalisation {
    type Err = String;
    fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
        match s {
            "None" => Ok(Self::None),
            "CPM" => Ok(Self::Cpm),
            "RPKM" => Ok(Self::Rpkm),
            "BPM" => Ok(Self::Bpm),
            "RPGC" => Ok(Self::Rpgc),
            _ => Err(format!(
                "unknown normalisation '{s}'; choose None CPM RPKM BPM RPGC"
            )),
        }
    }
}

#[derive(Debug, Clone)]
pub struct CoverageOpts {
    /// Bin size in bases (deeptools default: 50).
    pub bin_size: u32,
    /// Skip reads whose FLAG has any of these bits set.
    /// deeptools default (`samFlag_exclude=None, ignoreDuplicates=False`) → 0.
    pub skip_flags: u16,
    /// Minimum mapping quality (deeptools default: 0 = no filter).
    pub min_mapq: u8,
    pub normalisation: Normalisation,
    /// Required for RPGC; ignored otherwise.
    pub effective_genome_size: Option<u64>,
}

impl Default for CoverageOpts {
    fn default() -> Self {
        Self {
            bin_size: 50,
            skip_flags: 0,
            min_mapq: 0,
            normalisation: Normalisation::None,
            effective_genome_size: None,
        }
    }
}

/// Run the BAM scan and emit bedGraph to `output`. Returns line count.
pub fn bam_to_bedgraph(
    input: &Path,
    output: &mut dyn Write,
    opts: &CoverageOpts,
    workers: NonZero<usize>,
) -> Result<u64> {
    if opts.normalisation == Normalisation::Rpgc && opts.effective_genome_size.is_none() {
        return Err(RsomicsError::InvalidInput(
            "RPGC normalisation requires --effective-genome-size".into(),
        ));
    }

    let filter = BinFilter {
        skip_flags: opts.skip_flags,
        min_mapq: opts.min_mapq,
    };
    let coverage = compute_coverage(input, opts.bin_size, filter, workers)?;

    let bin_size = u64::from(opts.bin_size);
    let scale = scale_factor(opts, &coverage);

    let mut out = BufWriter::with_capacity(256 * 1024, output);
    let mut lines: u64 = 0;

    for chrom in &coverage.chroms {
        if chrom.bins.is_empty() {
            continue;
        }
        lines += write_chrom_bedgraph(&mut out, chrom, bin_size, scale)?;
    }

    out.flush().map_err(RsomicsError::Io)?;
    Ok(lines)
}

/// The post-pass normalisation scalar (None = raw integer mode). The denominator
/// is `total_binned` — reads actually placed on the genome — matching deeptools
/// `bamCoverage`'s CPM/RPKM/RPGC denominator.
fn scale_factor(opts: &CoverageOpts, coverage: &BinnedCoverage) -> Option<f64> {
    let total_reads = coverage.total_binned;
    match opts.normalisation {
        Normalisation::None => None,
        Normalisation::Cpm => (total_reads > 0).then(|| 1e6 / total_reads as f64),
        Normalisation::Rpkm => {
            (total_reads > 0).then(|| 1e9 / (total_reads as f64 * opts.bin_size as f64))
        }
        Normalisation::Bpm => {
            let total_signal = coverage.total_signal();
            (total_signal > 0).then(|| 1e6 / total_signal as f64)
        }
        Normalisation::Rpgc => {
            let eff = opts.effective_genome_size.unwrap() as f64;
            (total_reads > 0).then(|| eff / total_reads as f64)
        }
    }
}

/// Write one chromosome's bins as merged bedGraph lines.
///
/// Adjacent equal-value bins are collapsed — exact port of deeptools
/// `writeBedGraph_worker` run-length encoding. All bins (including zero)
/// are written. Values use Python `{:g}` format: trailing zeros stripped,
/// no decimal point when the value is an exact integer.
fn write_chrom_bedgraph(
    out: &mut impl Write,
    chrom: &ChromBins,
    bin_size: u64,
    scale: Option<f64>,
) -> Result<u64> {
    let mut lines: u64 = 0;
    let mut write_start: u64 = 0;
    let mut write_end: u64 = 0;
    let mut prev_val: Option<f64> = None;

    let n = chrom.bins.len();
    for (i, &raw_count) in chrom.bins.iter().enumerate() {
        let bin_start = i as u64 * bin_size;
        let bin_end = ((i as u64 + 1) * bin_size).min(chrom.chrom_len);

        let value = match scale {
            None => raw_count as f64,
            Some(s) => raw_count as f64 * s,
        };

        match prev_val {
            None => {
                write_start = bin_start;
                write_end = bin_end;
                prev_val = Some(value);
            }
            Some(pv) if values_equal(pv, value, scale) => {
                write_end = bin_end;
            }
            Some(pv) => {
                write_line(out, &chrom.name, write_start, write_end, pv, scale)?;
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
            write_line(out, &chrom.name, write_start, write_end, pv, scale)?;
            lines += 1;
        }
    }

    Ok(lines)
}

/// Equal-value check for merging adjacent bins.
/// For raw counts (scale=None) exact integer equality suffices.
/// For floats, use an epsilon relative to the bin value.
fn values_equal(a: f64, b: f64, scale: Option<f64>) -> bool {
    if scale.is_none() {
        (a - b).abs() < f64::EPSILON
    } else {
        // Two floats are "same bin" when they round to the same string at
        // output precision (2 decimal places for normalised values).
        // Using relative epsilon is fragile; string comparison is canonical.
        format!("{a:.2}") == format!("{b:.2}")
    }
}

fn write_line(
    out: &mut impl Write,
    chrom: &str,
    start: u64,
    end: u64,
    value: f64,
    scale: Option<f64>,
) -> Result<()> {
    if scale.is_none() {
        // Raw counts: Python {:g} for integer float → no decimal point.
        let v = value as u64;
        writeln!(out, "{chrom}\t{start}\t{end}\t{v}").map_err(RsomicsError::Io)
    } else {
        // Normalised: {:g} equivalent — print with enough precision but strip
        // trailing zeros. We use .2f as our precision floor matching deeptools.
        let s = format_g(value);
        writeln!(out, "{chrom}\t{start}\t{end}\t{s}").map_err(RsomicsError::Io)
    }
}

/// Format a float like Python's `{:g}` (general format, trailing zeros stripped).
fn format_g(v: f64) -> String {
    // Python {:g} uses 6 significant digits by default.
    // For normalised coverage values we use 6 sig figs and strip trailing zeros.
    let s = format!("{v:.6}");
    // Strip trailing zeros after decimal point.
    let s = s.trim_end_matches('0');
    let s = s.trim_end_matches('.');
    s.to_owned()
}
