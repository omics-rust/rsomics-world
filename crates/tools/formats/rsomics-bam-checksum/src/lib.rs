/// Order-independent BAM checksum algorithm, porting samtools checksum (MIT).
///
/// ## Origin
///
/// This crate is an independent Rust reimplementation of `samtools checksum`
/// based on:
/// - The samtools source: `bam_checksum.c` (MIT, Copyright 2024-2025 Genome
///   Research Ltd., Author: James Bonfield)
/// - The SAM/BAM format specification (SAMv1)
/// - Black-box behaviour testing against the upstream binary
///
/// The upstream source is MIT-licensed; reading and citing it is permitted.
/// Source URL: <https://github.com/samtools/samtools/blob/1.23.1/bam_checksum.c>
///
/// License: MIT OR Apache-2.0.
/// Upstream credit: samtools <https://github.com/samtools/samtools> (MIT).
pub mod checksum;
pub use checksum::{ChecksumOpts, ChecksumResult, run_checksum};
