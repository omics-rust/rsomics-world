//! Genome-binned BAM read-coverage primitive — the per-bin read-counting core
//! shared by `rsomics-bam-signal` (deeptools `bamCoverage`) and
//! `rsomics-bam-compare` (deeptools `bamCompare`).
//!
//! Both deeptools tools tile each chromosome into fixed-width bins and count,
//! per bin, the reads whose reference span overlaps it (deeptools
//! `countReadsPerBin`). bamCoverage then normalises one BAM's bins; bamCompare
//! combines two BAMs' bins per operation. The tiling + counting is identical
//! between them, so it lives here once (Layer A; B never depends on B). The
//! normalisation, bedGraph run-length emit, and two-BAM combination stay in the
//! respective tool crates — those differ.
//!
//! ## Counting semantics (deeptools source parity)
//!
//! A read is accepted into the count when it is mapped (FLAG 0x4 clear, refID
//! ≥ 0) and passes the [`BinFilter`] (`skip_flags` / `min_mapq`). With
//! deeptools' defaults (`samFlag_exclude=None`, `ignoreDuplicates=False`,
//! `minMappingQuality=None`) the filter is empty, so secondary and supplementary
//! reads are **not** excluded — pass `skip_flags = 0x900` to match samtools-style
//! filtering, `0x400` for duplicate-only.
//!
//! Fragment extent: with `extendReads=False` / `centerReads=False` (deeptools
//! defaults) the reference span is the alignment start plus every
//! reference-consuming CIGAR op (M/=/X/D/N), matching pysam `get_blocks()`.
//!
//! Bin counting: a read contributes +1 to every bin it overlaps —
//! `s_idx = floor(fragStart / binSize)`, `e_idx = ceil(fragEnd / binSize)`. The
//! partial last bin per chromosome is retained.

// Genome coordinates and bin indices fit u64; every first-class target is
// 64-bit (the workspace supports no 32-bit target), so u64→usize never
// truncates. refID/pos are validated non-negative before the i32→unsigned casts.
#![allow(clippy::cast_possible_truncation)]
#![allow(clippy::cast_sign_loss)]

use std::num::NonZero;
use std::path::Path;

use rsomics_bamio::raw::{self, RawRecord};
use rsomics_common::{Result, RsomicsError};

// CIGAR op codes (BAM packed encoding, low nibble): the reference-consuming ops.
const CIGAR_MATCH: u8 = 0;
const CIGAR_DELETION: u8 = 2;
const CIGAR_SKIP: u8 = 3;
const CIGAR_SEQ_MATCH: u8 = 7;
const CIGAR_SEQ_MISMATCH: u8 = 8;

/// Read-acceptance predicate. A read counts when none of `skip_flags` are set in
/// its FLAG and its MAPQ is at least `min_mapq`. Both default to "accept all
/// mapped reads", matching deeptools' empty default filter.
#[derive(Debug, Clone, Copy, Default)]
pub struct BinFilter {
    /// Skip reads whose FLAG has any of these bits set. deeptools default
    /// (`samFlag_exclude=None`, `ignoreDuplicates=False`) → 0 (no skip).
    pub skip_flags: u16,
    /// Minimum mapping quality. deeptools default (`minMappingQuality=None`) → 0
    /// (no filter).
    pub min_mapq: u8,
}

/// One chromosome's bin counts. `bins[i]` holds the read count for the half-open
/// reference window `[i*bin_size, min((i+1)*bin_size, chrom_len))`.
#[derive(Debug, Clone)]
pub struct ChromBins {
    pub name: String,
    pub chrom_len: u64,
    pub bins: Vec<u32>,
}

/// The result of binning one BAM: per-chromosome bin counts plus two read
/// totals that downstream normalisation needs.
#[derive(Debug, Clone)]
pub struct BinnedCoverage {
    /// Per-chromosome bin counts, in header reference order.
    pub chroms: Vec<ChromBins>,
    /// Reads that contributed at least one bin increment — the denominator
    /// deeptools `bamCoverage` uses for CPM/RPKM/RPGC (reads actually placed on
    /// the genome after the filter and the `ref_len > 0` / in-bounds checks).
    pub total_binned: u64,
    /// All mapped reads passing the filter, regardless of whether they landed in
    /// a bin — the count deeptools `bamCompare` uses for its readCount scale
    /// factor (`min(m1, m2) / mi`). With no filter this equals the BAM's mapped
    /// read count (`bam.mapped`).
    pub total_mapped: u64,
}

impl BinnedCoverage {
    /// Total signal across every bin — the BPM denominator
    /// (`sum of all bin counts`).
    #[must_use]
    pub fn total_signal(&self) -> u64 {
        self.chroms
            .iter()
            .map(|c| c.bins.iter().map(|&b| u64::from(b)).sum::<u64>())
            .sum()
    }
}

/// Scan `input`, tiling each reference at `bin_size` and counting reads per bin.
///
/// `bin_size` is in bases (deeptools default 50). `workers` controls BGZF
/// inflate parallelism (see [`rsomics_bamio::open_with_workers`]); it never
/// changes the result, only the read throughput.
pub fn compute_coverage(
    input: &Path,
    bin_size: u32,
    filter: BinFilter,
    workers: NonZero<usize>,
) -> Result<BinnedCoverage> {
    let mut reader = rsomics_bamio::open_with_workers(input, workers)?;
    let header = reader.read_header().map_err(RsomicsError::Io)?;

    let bin_size = u64::from(bin_size);
    let mut chroms: Vec<ChromBins> = header
        .reference_sequences()
        .iter()
        .map(|(name, seq)| {
            let len = usize::from(seq.length()) as u64;
            let n_bins = len.div_ceil(bin_size) as usize;
            ChromBins {
                name: name.to_string(),
                chrom_len: len,
                bins: vec![0u32; n_bins],
            }
        })
        .collect();

    let mut total_binned: u64 = 0;
    let mut total_mapped: u64 = 0;
    let mut record = RawRecord::default();

    while raw::read_record(reader.get_mut(), &mut record)? != 0 {
        let flags = record.flags();
        // Skip unmapped (FLAG 0x4).
        if flags & 0x4 != 0 {
            continue;
        }
        if record.reference_sequence_id() < 0 {
            continue;
        }
        if filter.skip_flags != 0 && (flags & filter.skip_flags) != 0 {
            continue;
        }
        if filter.min_mapq > 0 && record.mapping_quality() < filter.min_mapq {
            continue;
        }

        // Every mapped read that clears the filter is a "kept read" for the
        // bamCompare scale factor, even if its span falls outside all bins.
        total_mapped += 1;

        let tid = record.reference_sequence_id() as usize;
        let Some(chrom) = chroms.get_mut(tid) else {
            continue;
        };

        // 0-based alignment start (BAM raw pos field is 0-based).
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
        let frag_end = start0 + ref_len;

        let s_idx = (start0 / bin_size) as usize;
        let e_idx = (frag_end.div_ceil(bin_size) as usize).min(chrom.bins.len());
        if s_idx >= chrom.bins.len() {
            continue;
        }
        for b in &mut chrom.bins[s_idx..e_idx] {
            *b = b.saturating_add(1);
        }
        total_binned += 1;
    }

    Ok(BinnedCoverage {
        chroms,
        total_binned,
        total_mapped,
    })
}
