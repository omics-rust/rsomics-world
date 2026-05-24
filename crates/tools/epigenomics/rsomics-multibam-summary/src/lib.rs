//! Multi-BAM read-count matrix, matching deeptools `multiBamSummary` default
//! semantics. The matrix (rows = genome bins or supplied BED regions, columns =
//! BAM samples) is the input to `plotCorrelation` / `plotPCA` for ChIP/ATAC
//! sample-correlation QC.
//!
//! Two modes:
//!
//! - **bins** — tile every chromosome into fixed-width bins (deeptools default
//!   `--binSize 10000`) and count reads per bin per BAM. The per-BAM binning is
//!   delegated to [`rsomics_coverage_core::compute_coverage`] (Layer A, shared
//!   with `bamCoverage` / `bamCompare`): a read contributes +1 to every bin its
//!   reference span overlaps, and the partial last bin per chromosome is
//!   retained — `n_bins = ceil(chrom_len / binSize)`, exactly deeptools'
//!   `(end-start)//tile + (1 if remainder)`.
//! - **BED-file** — count reads per supplied BED region per BAM. deeptools tiles
//!   each region into a single tile and counts a read once per region it
//!   overlaps (its `last_eIdx` dedup collapses the per-block increments of one
//!   read to a single +1 per region). We reproduce that: each accepted read
//!   contributes +1 to every region its reference span overlaps. Output rows are
//!   sorted by chromosome (BAM-header order) then ascending position, as
//!   deeptools emits them — not BED declaration order.
//!
//! ## Read filter (deeptools `countReadsPerBin` defaults)
//!
//! deeptools defaults are `minMappingQuality=None`, `samFlag_include=None`,
//! `samFlag_exclude=None`, `ignoreDuplicates=False`: only **unmapped** reads are
//! skipped; secondary, supplementary and duplicate reads are kept. Both modes
//! share this filter via [`rsomics_coverage_core::BinFilter`] so `--min-mapq` /
//! `--sam-flag-exclude` behave identically across modes.
//!
//! ## `--outRawCounts` format (the value oracle)
//!
//! deeptools writes a header line `#'chr'\t'start'\t'end'\t'label1'\t'label2'…`
//! (column names single-quoted) followed by plain tab-separated data rows
//! `chrom\tstart\tend\tc1\tc2…`. The counts come from a float64 numpy array, so
//! integer counts print as `5.0`, zero as `0.0`. Labels default to each BAM's
//! basename. We reproduce all of this byte-for-byte.
//!
//! The numpy `.npz` matrix output deeptools also emits is **scoped out** — the
//! `.npz` is an opaque pickle/zip the downstream plot tools consume; the
//! human-readable, value-exact oracle is `--outRawCounts`, which we match.

// Genome coordinates and bin indices fit u64; every first-class target is
// 64-bit, so u64→usize never truncates. Counts are integral but printed as
// deeptools' float64 to match its output, hence the count→f64 casts.
#![allow(clippy::cast_precision_loss)]

use std::collections::HashMap;
use std::io::{BufReader, BufWriter, Write};
use std::num::NonZero;
use std::path::{Path, PathBuf};

use rsomics_bamio::raw::{self, RawRecord};
use rsomics_common::{Result, RsomicsError};
use rsomics_coverage_core::{BinFilter, compute_coverage};
use rsomics_intervals::{Interval, IntervalIndex, IntervalSet};

/// deeptools `multiBamSummary --binSize` default.
pub const DEFAULT_BIN_SIZE: u32 = 10_000;

// Reference-consuming CIGAR ops (BAM packed low-nibble codes), used to derive a
// read's reference span in BED-file mode. Matches `rsomics-coverage-core`.
const CIGAR_MATCH: u8 = 0;
const CIGAR_DELETION: u8 = 2;
const CIGAR_SKIP: u8 = 3;
const CIGAR_SEQ_MATCH: u8 = 7;
const CIGAR_SEQ_MISMATCH: u8 = 8;

/// Knobs shared by both counting modes.
#[derive(Debug, Clone)]
pub struct SummaryOpts {
    /// Bin width in bases (bins mode only; deeptools default 10000).
    pub bin_size: u32,
    /// Skip reads whose FLAG has any of these bits set. deeptools default
    /// (`samFlag_exclude=None`, `ignoreDuplicates=False`) → 0 (no skip).
    pub skip_flags: u16,
    /// Minimum mapping quality (deeptools default 0 = no filter).
    pub min_mapq: u8,
}

impl Default for SummaryOpts {
    fn default() -> Self {
        Self {
            bin_size: DEFAULT_BIN_SIZE,
            skip_flags: 0,
            min_mapq: 0,
        }
    }
}

/// The assembled count matrix: one row per bin/region, one column per BAM.
pub struct CountMatrix {
    /// Per-row region coordinates, in deeptools output order: chromosome
    /// (BAM-header order) then ascending position, for both bins and BED-file
    /// modes.
    pub regions: Vec<(String, u64, u64)>,
    /// `counts[row][col]` is the read count for `regions[row]` in BAM `col`.
    pub counts: Vec<Vec<u64>>,
    /// Per-BAM column labels (basename of each BAM path).
    pub labels: Vec<String>,
}

/// Default per-BAM label: the file's basename, matching deeptools
/// `[os.path.basename(x) for x in args.bamfiles]`.
fn label_of(path: &Path) -> String {
    path.file_name().map_or_else(
        || path.to_string_lossy().into_owned(),
        |n| n.to_string_lossy().into_owned(),
    )
}

/// Count reads per fixed-width genome bin across every BAM (deeptools
/// `multiBamSummary bins`). All BAMs must share the first BAM's reference
/// sequence set and lengths (the common-reference case multiBamSummary targets);
/// a mismatch fails loud rather than silently dropping rows.
pub fn summarize_bins(
    bams: &[PathBuf],
    opts: &SummaryOpts,
    workers: NonZero<usize>,
) -> Result<CountMatrix> {
    if bams.is_empty() {
        return Err(RsomicsError::InvalidInput(
            "at least one BAM is required".into(),
        ));
    }
    let filter = BinFilter {
        skip_flags: opts.skip_flags,
        min_mapq: opts.min_mapq,
    };

    let mut per_bam = Vec::with_capacity(bams.len());
    for bam in bams {
        per_bam.push(compute_coverage(bam, opts.bin_size, filter, workers)?);
    }

    // The first BAM defines the row layout; every other BAM must agree on the
    // reference set and chromosome lengths so columns line up bin-for-bin.
    let reference = &per_bam[0];
    for (bam, cov) in bams.iter().zip(&per_bam).skip(1) {
        if cov.chroms.len() != reference.chroms.len() {
            return Err(RsomicsError::InvalidInput(format!(
                "{} has {} reference sequences but {} has {}; multiBamSummary requires a shared reference",
                bam.display(),
                cov.chroms.len(),
                bams[0].display(),
                reference.chroms.len()
            )));
        }
        for (a, b) in reference.chroms.iter().zip(&cov.chroms) {
            if a.name != b.name || a.chrom_len != b.chrom_len {
                return Err(RsomicsError::InvalidInput(format!(
                    "{} reference {} (len {}) does not match {} reference {} (len {})",
                    bams[0].display(),
                    a.name,
                    a.chrom_len,
                    bam.display(),
                    b.name,
                    b.chrom_len
                )));
            }
        }
    }

    let bin_size = u64::from(opts.bin_size);
    let mut regions = Vec::new();
    let mut counts: Vec<Vec<u64>> = Vec::new();
    for (chrom_idx, chrom) in reference.chroms.iter().enumerate() {
        for (bin_idx, _) in chrom.bins.iter().enumerate() {
            let start = bin_idx as u64 * bin_size;
            let end = ((bin_idx as u64 + 1) * bin_size).min(chrom.chrom_len);
            regions.push((chrom.name.clone(), start, end));
            let row: Vec<u64> = per_bam
                .iter()
                .map(|cov| u64::from(cov.chroms[chrom_idx].bins[bin_idx]))
                .collect();
            counts.push(row);
        }
    }

    Ok(CountMatrix {
        regions,
        counts,
        labels: bams.iter().map(|p| label_of(p)).collect(),
    })
}

/// Count reads per supplied BED region across every BAM (deeptools
/// `multiBamSummary BED-file --BED regions.bed`). A read contributes +1 to every
/// region its reference span overlaps (deeptools collapses one read's per-block
/// increments to a single +1 per region via its `last_eIdx` dedup).
///
/// Output rows are sorted by chromosome (first BAM's header order) then ascending
/// `(start, end)` — deeptools processes the genome left-to-right per chromosome
/// and emits in that order regardless of BED declaration order.
pub fn summarize_bed(
    bams: &[PathBuf],
    bed: &Path,
    opts: &SummaryOpts,
    workers: NonZero<usize>,
) -> Result<CountMatrix> {
    if bams.is_empty() {
        return Err(RsomicsError::InvalidInput(
            "at least one BAM is required".into(),
        ));
    }
    let chrom_rank = header_chrom_rank(&bams[0])?;
    let regions = load_bed(bed)?;
    let set: IntervalSet = regions.iter().cloned().collect();
    let index = IntervalIndex::build(&set);

    let filter = BinFilter {
        skip_flags: opts.skip_flags,
        min_mapq: opts.min_mapq,
    };

    let mut counts: Vec<Vec<u64>> = vec![vec![0u64; bams.len()]; regions.len()];
    // Map a region's (chrom,start,end) identity back to its row(s). Duplicate
    // BED lines share an identity; each gets the same per-region count, matching
    // deeptools (which counts each region independently but identically).
    let mut row_of: HashMap<(String, u64, u64), Vec<usize>> = HashMap::new();
    for (i, r) in regions.iter().enumerate() {
        row_of
            .entry((r.chrom.clone(), r.start, r.end))
            .or_default()
            .push(i);
    }

    for (col, bam) in bams.iter().enumerate() {
        count_bed_one_bam(bam, &index, &row_of, &filter, col, workers, &mut counts)?;
    }

    // Reorder into deeptools' (chrom-header-order, start, end) row order. A
    // region on a chromosome absent from the BAM header sorts last (rank
    // `usize::MAX`), keeping output deterministic.
    let mut order: Vec<usize> = (0..regions.len()).collect();
    order.sort_by_key(|&i| {
        let r = &regions[i];
        (
            chrom_rank.get(&r.chrom).copied().unwrap_or(usize::MAX),
            r.start,
            r.end,
        )
    });

    let sorted_regions = order
        .iter()
        .map(|&i| {
            let r = &regions[i];
            (r.chrom.clone(), r.start, r.end)
        })
        .collect();
    let sorted_counts = order.iter().map(|&i| counts[i].clone()).collect();

    Ok(CountMatrix {
        regions: sorted_regions,
        counts: sorted_counts,
        labels: bams.iter().map(|p| label_of(p)).collect(),
    })
}

/// Map each reference name to its position in the BAM header (0-based), the rank
/// deeptools uses to order chromosomes in its output.
fn header_chrom_rank(bam: &Path) -> Result<HashMap<String, usize>> {
    let mut reader = rsomics_bamio::open_with_workers(bam, NonZero::<usize>::MIN)?;
    let header = reader.read_header().map_err(RsomicsError::Io)?;
    Ok(header
        .reference_sequences()
        .keys()
        .enumerate()
        .map(|(i, name)| (name.to_string(), i))
        .collect())
}

/// Scan one BAM, incrementing `counts[row][col]` for every region each accepted
/// read overlaps. A read hits a region at most once even if multiple of its
/// CIGAR blocks fall inside (the index reports each region once per query).
fn count_bed_one_bam(
    bam: &Path,
    index: &IntervalIndex,
    row_of: &HashMap<(String, u64, u64), Vec<usize>>,
    filter: &BinFilter,
    col: usize,
    workers: NonZero<usize>,
    counts: &mut [Vec<u64>],
) -> Result<()> {
    let mut reader = rsomics_bamio::open_with_workers(bam, workers)?;
    let header = reader.read_header().map_err(RsomicsError::Io)?;
    let refs: Vec<String> = header
        .reference_sequences()
        .keys()
        .map(ToString::to_string)
        .collect();

    let mut record = RawRecord::default();
    while raw::read_record(reader.get_mut(), &mut record)? != 0 {
        let flags = record.flags();
        if flags & 0x4 != 0 {
            continue;
        }
        let tid = record.reference_sequence_id();
        if tid < 0 {
            continue;
        }
        if filter.skip_flags != 0 && (flags & filter.skip_flags) != 0 {
            continue;
        }
        if filter.min_mapq > 0 && record.mapping_quality() < filter.min_mapq {
            continue;
        }
        let Some(chrom) = refs.get(tid as usize) else {
            continue;
        };

        let start0 = record.alignment_start() as u64;
        let ref_len: u64 = record
            .cigar_ops()
            .filter_map(|(kind, len)| match kind {
                CIGAR_MATCH | CIGAR_DELETION | CIGAR_SKIP | CIGAR_SEQ_MATCH
                | CIGAR_SEQ_MISMATCH => Some(u64::from(len)),
                _ => None,
            })
            .sum();
        if ref_len == 0 {
            continue;
        }
        let end = start0 + ref_len;

        index.for_each_overlap(chrom, start0, end, |region| {
            if let Some(rows) = row_of.get(&(region.chrom.clone(), region.start, region.end)) {
                for &row in rows {
                    counts[row][col] += 1;
                }
            }
        });
    }
    Ok(())
}

fn load_bed(path: &Path) -> Result<Vec<Interval>> {
    let file = std::fs::File::open(path)
        .map_err(|e| RsomicsError::InvalidInput(format!("{}: {e}", path.display())))?;
    rsomics_intervals::bed::read(BufReader::new(file))
        .map_err(|e| RsomicsError::InvalidInput(format!("reading BED {}: {e}", path.display())))
}

/// Write the matrix in deeptools `--outRawCounts` format: a `#`-prefixed header
/// with single-quoted column names, then plain tab-separated data rows. Counts
/// print as deeptools' float64 (`5.0`, `0.0`).
pub fn write_raw_counts(out: &mut dyn Write, matrix: &CountMatrix) -> Result<()> {
    let mut w = BufWriter::with_capacity(256 * 1024, out);

    w.write_all(b"#'chr'\t'start'\t'end'\t")
        .map_err(RsomicsError::Io)?;
    let quoted: Vec<String> = matrix.labels.iter().map(|l| format!("'{l}'")).collect();
    writeln!(w, "{}", quoted.join("\t")).map_err(RsomicsError::Io)?;

    for (i, (chrom, start, end)) in matrix.regions.iter().enumerate() {
        write!(w, "{chrom}\t{start}\t{end}").map_err(RsomicsError::Io)?;
        for &c in &matrix.counts[i] {
            // deeptools' numpy float64 prints an integral count as "N.0".
            write!(w, "\t{:.1}", c as f64).map_err(RsomicsError::Io)?;
        }
        w.write_all(b"\n").map_err(RsomicsError::Io)?;
    }

    w.flush().map_err(RsomicsError::Io)?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn label_is_basename() {
        assert_eq!(label_of(Path::new("/a/b/sample.bam")), "sample.bam");
        assert_eq!(label_of(Path::new("sample.bam")), "sample.bam");
    }

    #[test]
    fn float_count_format() {
        let matrix = CountMatrix {
            regions: vec![("chr1".into(), 0, 10), ("chr1".into(), 10, 20)],
            counts: vec![vec![5, 0], vec![3, 12]],
            labels: vec!["a.bam".into(), "b.bam".into()],
        };
        let mut buf = Vec::new();
        write_raw_counts(&mut buf, &matrix).unwrap();
        let s = String::from_utf8(buf).unwrap();
        let mut lines = s.lines();
        assert_eq!(
            lines.next().unwrap(),
            "#'chr'\t'start'\t'end'\t'a.bam'\t'b.bam'"
        );
        assert_eq!(lines.next().unwrap(), "chr1\t0\t10\t5.0\t0.0");
        assert_eq!(lines.next().unwrap(), "chr1\t10\t20\t3.0\t12.0");
    }
}
