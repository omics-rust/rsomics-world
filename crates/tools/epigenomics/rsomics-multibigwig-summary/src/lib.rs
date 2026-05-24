//! Multi-bigWig mean-signal matrix, matching deeptools `multiBigwigSummary`
//! default semantics. The matrix (rows = genome bins or supplied BED regions,
//! columns = bigWig samples) is the input to `plotCorrelation` / `plotPCA` for
//! ChIP/ATAC/ATAC-seq sample-correlation QC.
//!
//! Two modes:
//!
//! - **bins** — tile every chromosome into fixed-width bins (deeptools default
//!   `--binSize 10000`) and compute the mean bigWig signal per bin per file.
//!   Only chromosomes present in ALL supplied bigWig files are processed
//!   (deeptools `getChromSizes` common-chromosome intersection). Chromosome
//!   order follows the first file's B-tree leaf order (pyBigWig `chroms()`
//!   iteration order). The last bin of each chromosome may be shorter than
//!   `bin_size` — it is included exactly as deeptools does.
//! - **BED-file** — compute the mean bigWig signal per supplied BED region per
//!   file. Regions are output in BED-declaration order (deeptools BED-file mode
//!   preserves declaration order for single-region BEDs; here we match that).
//!
//! ## Per-bin mean (deeptools `getScorePerBigWigBin` → `pyBigWig.stats`)
//!
//! deeptools calls `bwh.stats(chrom, start, end)` (default type `mean`) for
//! each bin. pyBigWig passes `exact=False` (the default), which delegates to
//! libBigWig's zoom-approximate path: `determineZoomLevel` selects the best
//! pre-computed zoom level, then `bwStatsFromZoom` reads the zoom CIR-tree and
//! computes `blockMean` with fractional-overlap scalars.
//!
//! Both modes call `BigWig::mean_stat_zoom`, which replicates that path exactly.
//! When no zoom level qualifies (query smaller than the finest zoom reduction ÷ 2),
//! `mean_stat_zoom` falls back to the full-resolution exact path, matching
//! libBigWig `bwStatsFromFull`.
//!
//! ## `--outRawCounts` format (the value oracle)
//!
//! deeptools writes a header `#'chr'\t'start'\t'end'\t'label1'\t'label2'…`
//! (single-quoted column names) then tab-separated data rows. Per-bin values
//! are formatted with Python's default float repr: `2.0`, `0.5`,
//! `0.6666666666666666`, `nan`. Labels default to each bigWig's basename.
//! The numpy `.npz` output is scoped out — the human-readable
//! `--outRawCounts` table is the value oracle we match.

use std::collections::HashMap;
use std::io::{BufWriter, Write};
use std::path::{Path, PathBuf};

use rsomics_bbi::{BigWig, mean_from_zoom_items, nanmean_from_full_items};
use rsomics_common::{Result, RsomicsError};
use rsomics_intervals::{Interval, bed};

/// deeptools `multiBigwigSummary --binSize` default.
pub const DEFAULT_BIN_SIZE: u32 = 10_000;

/// Knobs shared by both summary modes.
#[derive(Debug, Clone)]
pub struct SummaryOpts {
    /// Bin width in bases (bins mode only; deeptools default 10000).
    pub bin_size: u32,
}

impl Default for SummaryOpts {
    fn default() -> Self {
        Self {
            bin_size: DEFAULT_BIN_SIZE,
        }
    }
}

/// The assembled signal matrix: one row per bin/region, one column per bigWig.
pub struct SignalMatrix {
    /// Per-row region coordinates in output order.
    pub regions: Vec<(String, u32, u32)>,
    /// `values[row][col]` is the mean signal for `regions[row]` in bigWig `col`.
    /// `NaN` where the bigWig carries no data for that bin/region.
    pub values: Vec<Vec<f64>>,
    /// Per-bigWig column labels (basename of each bigWig path).
    pub labels: Vec<String>,
}

fn label_of(path: &Path) -> String {
    path.file_name().map_or_else(
        || path.to_string_lossy().into_owned(),
        |n| n.to_string_lossy().into_owned(),
    )
}

/// Compute mean signal per fixed-width genome bin across every bigWig (deeptools
/// `multiBigwigSummary bins`). Only chromosomes common to all supplied bigWig
/// files are processed; chromosome order follows the first file's B-tree order.
pub fn summarize_bins(bws: &[PathBuf], opts: &SummaryOpts) -> Result<SignalMatrix> {
    if bws.is_empty() {
        return Err(RsomicsError::InvalidInput(
            "at least one bigWig is required".into(),
        ));
    }

    let mut handles: Vec<BigWig> = bws.iter().map(|p| BigWig::open(p)).collect::<Result<_>>()?;

    // Common chromosomes: intersection across all files, in first-file B-tree order.
    let common = common_chroms(&handles);

    let bin_size = opts.bin_size;
    let bs = bin_size as usize;

    // Pre-allocate the region list and a column-major values matrix.
    let total_bins: usize = common.iter().map(|(_, l)| (*l as usize).div_ceil(bs)).sum();
    let mut regions: Vec<(String, u32, u32)> = Vec::with_capacity(total_bins);
    // values_col[col] = column vector for one bigWig, accumulated row-by-row.
    let mut values_col: Vec<Vec<f64>> = vec![Vec::with_capacity(total_bins); bws.len()];

    for (chrom, chrom_len) in &common {
        let n_bins = (*chrom_len as usize).div_ceil(bs);

        // Build the region list for this chromosome.
        for bin_idx in 0..n_bins {
            // bin_idx < chrom_len / bin_size ≤ u32::MAX; safe cast.
            #[allow(clippy::cast_possible_truncation)]
            let b = bin_idx as u32;
            let start = b * bin_size;
            let end = (b + 1).saturating_mul(bin_size).min(*chrom_len);
            regions.push((chrom.clone(), start, end));
        }

        // For each bigWig, batch-load all zoom items for this chromosome once,
        // then sweep forward through bins — O(items + bins) per chrom instead
        // of O(bins × log(items)) R-tree walks.  Falls back to the
        // full-resolution batch path when no zoom level qualifies (bin_size too
        // small for any pre-computed zoom level).
        for (col, bw) in handles.iter_mut().enumerate() {
            if let Some((_level, items)) = bw.zoom_items_for_chrom(chrom, bin_size)? {
                let mut scan = 0usize;
                for bin_idx in 0..n_bins {
                    #[allow(clippy::cast_possible_truncation)]
                    let b = bin_idx as u32;
                    let start = b * bin_size;
                    let end = (b + 1).saturating_mul(bin_size).min(*chrom_len);
                    let (mean, next_scan) = mean_from_zoom_items(&items, start, end, scan);
                    scan = next_scan;
                    values_col[col].push(mean);
                }
            } else {
                // No qualifying zoom level: load full-resolution records for
                // the whole chromosome once, then sweep forward across bins.
                let items = bw.full_items_for_chrom(chrom)?.ok_or_else(|| {
                    RsomicsError::InvalidInput(format!(
                        "chromosome {chrom} missing from bigWig during data read"
                    ))
                })?;
                let mut scan = 0usize;
                for bin_idx in 0..n_bins {
                    #[allow(clippy::cast_possible_truncation)]
                    let b = bin_idx as u32;
                    let start = b * bin_size;
                    let end = (b + 1).saturating_mul(bin_size).min(*chrom_len);
                    let (mean, next_scan) = nanmean_from_full_items(&items, start, end, scan);
                    scan = next_scan;
                    values_col[col].push(mean);
                }
            }
        }
    }

    // Transpose from column-major to row-major.
    let values: Vec<Vec<f64>> = (0..total_bins)
        .map(|row| values_col.iter().map(|col| col[row]).collect())
        .collect();

    Ok(SignalMatrix {
        regions,
        values,
        labels: bws.iter().map(|p| label_of(p)).collect(),
    })
}

/// Compute mean signal per supplied BED region across every bigWig (deeptools
/// `multiBigwigSummary BED-file --BED`). Output rows are sorted by chromosome
/// (first bigWig's B-tree order) then ascending `(start, end)`, matching
/// deeptools' BED-file mode row ordering regardless of input BED order.
pub fn summarize_bed(bws: &[PathBuf], bed_path: &Path, opts: &SummaryOpts) -> Result<SignalMatrix> {
    let _ = opts; // bin_size unused in BED mode
    if bws.is_empty() {
        return Err(RsomicsError::InvalidInput(
            "at least one bigWig is required".into(),
        ));
    }

    let regions_bed = load_bed(bed_path)?;
    let mut handles: Vec<BigWig> = bws.iter().map(|p| BigWig::open(p)).collect::<Result<_>>()?;

    // Chrom rank from the first bigWig's B-tree order.
    let chrom_rank: HashMap<String, usize> = handles[0]
        .chroms()
        .enumerate()
        .map(|(i, (n, _))| (n.to_owned(), i))
        .collect();

    let mut rows: Vec<(usize, u32, u32, String, Vec<f64>)> = Vec::with_capacity(regions_bed.len());
    for interval in &regions_bed {
        // BED coordinates fit u32 (genome positions); genomic tools reject >4 Gbp chromosomes.
        let start = u32::try_from(interval.start).map_err(|_| {
            RsomicsError::InvalidInput(format!("BED start {} exceeds u32", interval.start))
        })?;
        let end = u32::try_from(interval.end).map_err(|_| {
            RsomicsError::InvalidInput(format!("BED end {} exceeds u32", interval.end))
        })?;
        let chrom_idx = chrom_rank
            .get(&interval.chrom)
            .copied()
            .unwrap_or(usize::MAX);
        let row: Vec<f64> = handles
            .iter_mut()
            .map(|bw| bin_mean(bw, &interval.chrom, start, end))
            .collect::<Result<_>>()?;
        rows.push((chrom_idx, start, end, interval.chrom.clone(), row));
    }

    rows.sort_by_key(|(ci, s, e, _, _)| (*ci, *s, *e));

    let mut regions: Vec<(String, u32, u32)> = Vec::with_capacity(rows.len());
    let mut values: Vec<Vec<f64>> = Vec::with_capacity(rows.len());
    for (_, start, end, chrom, row) in rows {
        regions.push((chrom, start, end));
        values.push(row);
    }

    Ok(SignalMatrix {
        regions,
        values,
        labels: bws.iter().map(|p| label_of(p)).collect(),
    })
}

/// Chromosomes present in ALL bigWig files, in the first file's B-tree leaf
/// order. Length is the minimum across files (deeptools `getChromSizes`).
fn common_chroms(handles: &[BigWig]) -> Vec<(String, u32)> {
    let first_order: Vec<(String, u32)> = handles[0]
        .chroms()
        .map(|(n, l)| (n.to_owned(), l))
        .collect();
    if handles.len() == 1 {
        return first_order;
    }

    // Build len maps for the remaining files.
    let rest_maps: Vec<HashMap<&str, u32>> = handles[1..]
        .iter()
        .map(|bw| bw.chroms().collect())
        .collect();

    first_order
        .into_iter()
        .filter_map(|(name, len)| {
            let min_len = rest_maps
                .iter()
                .try_fold(len, |acc, m| m.get(name.as_str()).map(|&l| acc.min(l)))?;
            Some((name, min_len))
        })
        .collect()
}

/// Zoom-approximate mean bigWig signal over `[start, end)` on `chrom`,
/// matching `pyBigWig.stats(chrom, start, end)` with `exact=False`.
/// Returns `NaN` when the chromosome is absent or all positions in the range
/// lack data.
fn bin_mean(bw: &mut BigWig, chrom: &str, start: u32, end: u32) -> Result<f64> {
    Ok(bw.mean_stat_zoom(chrom, start, end)?.unwrap_or(f64::NAN))
}

/// Write the matrix in deeptools `--outRawCounts` format: a `#`-prefixed header
/// with single-quoted column names, then tab-separated data rows. Values are
/// formatted with Python's default float repr (`2.0`, `0.5`,
/// `0.6666666666666666`, `nan`).
pub fn write_raw_counts(out: &mut dyn Write, matrix: &SignalMatrix) -> Result<()> {
    let mut w = BufWriter::with_capacity(256 * 1024, out);

    w.write_all(b"#'chr'\t'start'\t'end'\t")
        .map_err(RsomicsError::Io)?;
    let quoted: Vec<String> = matrix.labels.iter().map(|l| format!("'{l}'")).collect();
    writeln!(w, "{}", quoted.join("\t")).map_err(RsomicsError::Io)?;

    for (i, (chrom, start, end)) in matrix.regions.iter().enumerate() {
        write!(w, "{chrom}\t{start}\t{end}").map_err(RsomicsError::Io)?;
        for &v in &matrix.values[i] {
            write!(w, "\t{}", py_float(v)).map_err(RsomicsError::Io)?;
        }
        w.write_all(b"\n").map_err(RsomicsError::Io)?;
    }

    w.flush().map_err(RsomicsError::Io)?;
    Ok(())
}

/// Format a `f64` the way Python's `str(float)` does:
///   - `nan` for NaN.
///   - Always includes a decimal point: `2.0`, `0.5`, `0.6666666666666666`.
///
/// Rust's `{}` for `f64` omits the `.0` suffix for whole numbers, so we add it
/// only when the formatted string contains no `.` or `e`.
fn py_float(v: f64) -> String {
    if v.is_nan() {
        return "nan".to_owned();
    }
    let s = format!("{v}");
    if s.contains('.') || s.contains('e') || s.contains('E') {
        s
    } else {
        format!("{s}.0")
    }
}

fn load_bed(path: &Path) -> Result<Vec<Interval>> {
    let file = std::fs::File::open(path)
        .map_err(|e| RsomicsError::InvalidInput(format!("{}: {e}", path.display())))?;
    bed::read(std::io::BufReader::new(file))
        .map_err(|e| RsomicsError::InvalidInput(format!("reading BED {}: {e}", path.display())))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn label_is_basename() {
        assert_eq!(label_of(Path::new("/a/b/sample.bw")), "sample.bw");
        assert_eq!(label_of(Path::new("sample.bw")), "sample.bw");
    }

    #[test]
    fn py_float_format() {
        assert_eq!(py_float(f64::NAN), "nan");
        assert_eq!(py_float(2.0f64), "2.0");
        assert_eq!(py_float(0.5f64), "0.5");
        assert_eq!(py_float(1.0 / 3.0), "0.3333333333333333");
    }

    #[test]
    fn write_raw_counts_format() {
        let matrix = SignalMatrix {
            regions: vec![("chr1".into(), 0, 100), ("chr1".into(), 100, 200)],
            values: vec![vec![2.0, 0.5], vec![f64::NAN, 1.0]],
            labels: vec!["a.bw".into(), "b.bw".into()],
        };
        let mut buf = Vec::new();
        write_raw_counts(&mut buf, &matrix).unwrap();
        let s = String::from_utf8(buf).unwrap();
        let mut lines = s.lines();
        assert_eq!(
            lines.next().unwrap(),
            "#'chr'\t'start'\t'end'\t'a.bw'\t'b.bw'"
        );
        assert_eq!(lines.next().unwrap(), "chr1\t0\t100\t2.0\t0.5");
        assert_eq!(lines.next().unwrap(), "chr1\t100\t200\tnan\t1.0");
    }
}
