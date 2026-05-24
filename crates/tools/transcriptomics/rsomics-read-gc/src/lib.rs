//! Per-read GC% distribution from a BAM file.
//!
//! Mirrors the algorithm of `RSeQC` `read_GC.py` (LGPL-2.1+):
//! - Iterates all mapped reads; skips unmapped and QC-fail.
//! - MAPQ threshold filter (default 30); reads with MAPQ < threshold are skipped.
//! - No secondary/supplementary filter — `readGC` does not apply those.
//! - GC% per read = `"%4.2f" % ((G+C) / len * 100)`.  N bases remain in the
//!   denominator (len is total sequence length, matching `RSeQC`'s
//!   `(len(RNA_read)+0.0)` denominator).
//! - Writes `<prefix>.GC.xls`: header `GC%\tread_count`, one row per observed
//!   GC% value.  `RSeQC` emits rows in Python dict insertion order; we emit rows
//!   sorted by GC% ascending (compat test sorts both sides before comparison).
//!
//! ## Origin
//!
//! This crate is an independent Rust reimplementation of `RSeQC`
//! `read_GC.py` based on:
//! - The published method: Wang et al. 2012 <https://doi.org/10.1093/bioinformatics/bts356>
//! - The public SAM/BAM format specification
//! - Reading the LGPL-2.1+ `RSeQC` 5.0.4 source (`SAM.py::readGC`)
//!   to derive exact GC% formatting, filter logic, and output format
//!   (LGPL allows reading; implementation is independent Rust)
//! - Black-box behaviour testing against `RSeQC` 5.0.4
//!
//! License: MIT OR Apache-2.0.
//! Upstream credit: `RSeQC` <https://rseqc.sourceforge.net/> (LGPL-2.1+).

use std::collections::HashMap;
use std::fs::File;
use std::io::{BufWriter, Write};
use std::num::NonZero;
use std::path::Path;

use rsomics_bamio::raw::{self, RawRecord};
use rsomics_common::{Result, RsomicsError};

// SAM flag bits used by readGC.
const FLAG_UNMAPPED: u16 = 0x0004;
const FLAG_QCFAIL: u16 = 0x0200;

/// BAM 4-bit nibble indices for G and C.
const NIBBLE_G: u8 = 4;
const NIBBLE_C: u8 = 2;

/// Count G+C bases in a BAM nibble-packed sequence.
///
/// N bases and ambiguity codes are NOT counted as GC; they remain in the
/// denominator (total length), matching `RSeQC`'s `len(RNA_read)` denominator.
fn count_gc(rec: &RawRecord) -> (usize, usize) {
    let n = rec.sequence_len();
    let mut gc = 0usize;
    for i in 0..n {
        let nib = rec.seq_nibble(i);
        if nib == NIBBLE_G || nib == NIBBLE_C {
            gc += 1;
        }
    }
    (gc, n)
}

/// Format GC% matching Python `"%4.2f" % (gc / total * 100.0)`.
///
/// Uses IEEE 754 f64 arithmetic, which matches `CPython`'s `%` operator
/// (both delegate to the platform C `sprintf("%4.2f", ...)`).
/// Result is 2 decimal places; no leading-space padding in the key string.
fn format_gc(gc: usize, total: usize) -> String {
    // Read lengths and GC counts fit comfortably in u32 (max ~2^31 bp); f64
    // mantissa precision is sufficient for the 2-decimal-place output.
    #[allow(clippy::cast_precision_loss)]
    let pct = (gc as f64) / (total as f64) * 100.0;
    format!("{pct:.2}")
}

/// GC% histogram: maps GC% string (e.g. `"42.86"`) to read count.
pub type GcHistogram = HashMap<String, u64>;

/// Scan `bam_path` and build a GC% histogram.
///
/// Filters applied (matching `RSeQC` `readGC`):
/// - Skip unmapped reads (FLAG 0x0004).
/// - Skip QC-fail reads (FLAG 0x0200).
/// - Skip reads with MAPQ < `mapq_cut`.
///
/// Secondary and supplementary reads are NOT filtered — `readGC` does not apply those flags.
pub fn compute_gc(bam_path: &Path, mapq_cut: u8, workers: NonZero<usize>) -> Result<GcHistogram> {
    let mut reader = rsomics_bamio::open_with_workers(bam_path, workers)?;
    reader.read_header().map_err(RsomicsError::Io)?;

    let inner = reader.get_mut();
    let mut rec = RawRecord::default();
    let mut hist: GcHistogram = HashMap::new();

    loop {
        let n = raw::read_record(inner, &mut rec)?;
        if n == 0 {
            break;
        }

        let flags = rec.flags();
        if flags & (FLAG_UNMAPPED | FLAG_QCFAIL) != 0 {
            continue;
        }
        if rec.mapping_quality() < mapq_cut {
            continue;
        }

        let (gc, total) = count_gc(&rec);
        if total == 0 {
            continue;
        }
        let key = format_gc(gc, total);
        *hist.entry(key).or_insert(0) += 1;
    }

    Ok(hist)
}

/// Write GC% histogram as `<prefix>.GC.xls` matching `RSeQC` output format.
///
/// Header: `GC%\tread_count`.  Rows are sorted by GC% value ascending.
/// (`RSeQC` emits in Python dict insertion order; compat tests sort both sides
/// before field comparison.)
pub fn write_gc_xls(hist: &GcHistogram, out_prefix: &Path) -> Result<()> {
    let prefix_str = out_prefix
        .file_name()
        .unwrap_or_default()
        .to_string_lossy()
        .into_owned();
    let dir = out_prefix.parent().unwrap_or(Path::new("."));
    let xls_path = dir.join(format!("{prefix_str}.GC.xls"));

    let f = File::create(&xls_path).map_err(RsomicsError::Io)?;
    let mut w = BufWriter::new(f);
    writeln!(w, "GC%\tread_count").map_err(RsomicsError::Io)?;

    let mut rows: Vec<(&str, u64)> = hist.iter().map(|(k, &v)| (k.as_str(), v)).collect();
    rows.sort_unstable_by(|(a, _), (b, _)| {
        let fa: f64 = a.parse().unwrap_or(0.0);
        let fb: f64 = b.parse().unwrap_or(0.0);
        fa.partial_cmp(&fb).unwrap_or(std::cmp::Ordering::Equal)
    });

    for (gc_str, count) in rows {
        writeln!(w, "{gc_str}\t{count}").map_err(RsomicsError::Io)?;
    }
    Ok(())
}

/// Run the full GC analysis and write the `.GC.xls` output file.
pub fn run_gc(
    bam_path: &Path,
    out_prefix: &Path,
    mapq_cut: u8,
    workers: NonZero<usize>,
) -> Result<GcHistogram> {
    eprintln!("Read BAM file ...");
    let hist = compute_gc(bam_path, mapq_cut, workers)?;
    eprintln!("Done");
    eprintln!("writing GC content ...");
    write_gc_xls(&hist, out_prefix)?;
    Ok(hist)
}
