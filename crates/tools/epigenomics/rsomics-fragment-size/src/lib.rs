//! Paired-end insert-size distribution from a coordinate-sorted BAM.
//!
//! Reads proper-pair + first-in-pair reads only (avoids double-counting).
//! Fragment size = |TLEN| from the BAM record. Builds a u64 histogram over
//! sizes 0..=MAX_SIZE in a single pass. Sizes above the cap are binned at MAX_SIZE.
//!
//! ATAC nucleosome fractions follow the common ENCODE / ATAC-seq QC convention:
//! - NFR (nucleosome-free region): < 100 bp
//! - mono-nucleosome: 180–247 bp
//! - di-nucleosome: 315–473 bp
//!
//! Output: TSV histogram `size\tcount` + summary block.

#![allow(clippy::cast_precision_loss)]

use std::io::{BufWriter, Write};
use std::num::NonZero;
use std::path::Path;

use rsomics_bamio::raw::{RawRecord, read_record};
use rsomics_common::{Result, RsomicsError};
use serde::Serialize;

// SAM/BAM FLAG bits (SAMv1 §1.4).
const FLAG_PAIRED: u16 = 0x1;
const FLAG_PROPER_PAIR: u16 = 0x2;
const FLAG_UNMAPPED: u16 = 0x4;
const FLAG_READ1: u16 = 0x40;

/// Upper cap on the histogram array (inclusive). Sizes above this are counted
/// at MAX_SIZE. 2000 bp covers >99% of short-read paired-end library inserts.
pub const MAX_SIZE: usize = 2000;

/// ATAC-seq nucleosome-period size thresholds (bp, inclusive ranges).
pub const NFR_MAX: u64 = 100;
pub const MONO_MIN: u64 = 180;
pub const MONO_MAX: u64 = 247;
pub const DI_MIN: u64 = 315;
pub const DI_MAX: u64 = 473;

#[derive(Debug, Clone, Default)]
pub struct SizeOpts {
    pub min_mapq: u8,
    pub skip_flags: u16,
}

#[derive(Debug, Serialize)]
pub struct Summary {
    pub total_fragments: u64,
    pub mean: f64,
    pub median: u64,
    pub mode: u64,
    pub min: u64,
    pub max: u64,
    /// Fraction of fragments < 100 bp (NFR).
    pub nfr_fraction: f64,
    /// Fraction of fragments 180–247 bp (mono-nucleosome).
    pub mono_fraction: f64,
    /// Fraction of fragments 315–473 bp (di-nucleosome).
    pub di_fraction: f64,
}

/// One-pass histogram scan. Returns the u64 histogram array and the Summary.
pub fn compute(
    input: &Path,
    opts: &SizeOpts,
    workers: NonZero<usize>,
) -> Result<([u64; MAX_SIZE + 1], Summary)> {
    let mut reader = rsomics_bamio::open_with_workers(input, workers)?;
    reader.read_header().map_err(RsomicsError::Io)?;

    let mut hist = [0u64; MAX_SIZE + 1];
    let mut record = RawRecord::default();

    while read_record(reader.get_mut(), &mut record)? != 0 {
        let flags = record.flags();

        // Proper pair + first-in-pair only (avoids double-count).
        if flags & FLAG_PAIRED == 0
            || flags & FLAG_PROPER_PAIR == 0
            || flags & FLAG_UNMAPPED != 0
            || flags & FLAG_READ1 == 0
        {
            continue;
        }
        if opts.skip_flags != 0 && (flags & opts.skip_flags) != 0 {
            continue;
        }
        if opts.min_mapq > 0 && record.mapping_quality() < opts.min_mapq {
            continue;
        }

        let tlen = record.template_length();
        let size = tlen.unsigned_abs() as u64;
        let idx = (size as usize).min(MAX_SIZE);
        hist[idx] += 1;
    }

    let summary = summarise(&hist);
    Ok((hist, summary))
}

fn summarise(hist: &[u64; MAX_SIZE + 1]) -> Summary {
    let total: u64 = hist.iter().sum();
    if total == 0 {
        return Summary {
            total_fragments: 0,
            mean: 0.0,
            median: 0,
            mode: 0,
            min: 0,
            max: 0,
            nfr_fraction: 0.0,
            mono_fraction: 0.0,
            di_fraction: 0.0,
        };
    }

    let mut sum: u64 = 0;
    let mut min_size: u64 = u64::MAX;
    let mut max_size: u64 = 0;
    let mut mode_size: u64 = 0;
    let mut mode_count: u64 = 0;
    let mut nfr: u64 = 0;
    let mut mono: u64 = 0;
    let mut di: u64 = 0;

    for (i, &count) in hist.iter().enumerate() {
        if count == 0 {
            continue;
        }
        let size = i as u64;
        sum += size * count;
        if size < min_size {
            min_size = size;
        }
        if size > max_size {
            max_size = size;
        }
        if count > mode_count {
            mode_count = count;
            mode_size = size;
        }
        if size < NFR_MAX {
            nfr += count;
        }
        if (MONO_MIN..=MONO_MAX).contains(&size) {
            mono += count;
        }
        if (DI_MIN..=DI_MAX).contains(&size) {
            di += count;
        }
    }

    let mean = sum as f64 / total as f64;

    // Median: walk the histogram to find the 50th percentile.
    let half = total / 2;
    let mut cumulative: u64 = 0;
    let mut median: u64 = 0;
    for (i, &count) in hist.iter().enumerate() {
        cumulative += count;
        if cumulative > half {
            median = i as u64;
            break;
        }
    }

    let ft = total as f64;
    Summary {
        total_fragments: total,
        mean,
        median,
        mode: mode_size,
        min: min_size,
        max: max_size,
        nfr_fraction: nfr as f64 / ft,
        mono_fraction: mono as f64 / ft,
        di_fraction: di as f64 / ft,
    }
}

/// Write the histogram TSV to `output` (size\tcount), skipping zero-count sizes.
pub fn write_histogram(output: &mut dyn Write, hist: &[u64; MAX_SIZE + 1]) -> Result<()> {
    let mut out = BufWriter::with_capacity(256 * 1024, output);
    writeln!(out, "size\tcount").map_err(RsomicsError::Io)?;
    for (i, &count) in hist.iter().enumerate() {
        if count > 0 {
            writeln!(out, "{i}\t{count}").map_err(RsomicsError::Io)?;
        }
    }
    out.flush().map_err(RsomicsError::Io)?;
    Ok(())
}
