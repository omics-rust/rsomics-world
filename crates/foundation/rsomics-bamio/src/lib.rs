//! Shared BAM input for the `rsomics-bam-*` tools.
//!
//! samtools inflates BGZF on a single thread by default, and a single-threaded
//! pure-Rust reader (zlib-rs) loses to its libdeflate inner loop. Inflating
//! BGZF blocks across a worker pool is the lever that puts our reader ahead of
//! `samtools` default invocations on multi-core hosts, so every BAM tool reads
//! through this one primitive rather than constructing a plain reader.

use std::fs::File;
use std::num::NonZero;
use std::path::Path;

use noodles::{bam, bgzf};
use rsomics_common::{Result, RsomicsError};

/// A BAM reader whose BGZF blocks are inflated across a worker pool.
pub type ParallelBamReader = bam::io::Reader<bgzf::io::MultithreadedReader<File>>;

/// Open `input` with one inflate worker per available core.
pub fn open_parallel(input: &Path) -> Result<ParallelBamReader> {
    let workers = std::thread::available_parallelism().unwrap_or(NonZero::<usize>::MIN);
    open_with_workers(input, workers)
}

/// Open `input` with an explicit worker count (1 = effectively single-threaded).
pub fn open_with_workers(input: &Path, workers: NonZero<usize>) -> Result<ParallelBamReader> {
    let file = File::open(input)
        .map_err(|e| RsomicsError::InvalidInput(format!("{}: {e}", input.display())))?;
    Ok(bam::io::Reader::from(
        bgzf::io::MultithreadedReader::with_worker_count(workers, file),
    ))
}
