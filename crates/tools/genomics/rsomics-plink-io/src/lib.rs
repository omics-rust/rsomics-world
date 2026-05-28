//! PLINK1 binary format I/O: stats computation and format export.

#![allow(clippy::cast_precision_loss)]

pub mod export;
pub mod stats;

pub use export::{to_012, to_vcf};
pub use stats::{FreqRecord, HweRecord, MissingRecord, allele_freq, hwe_stats, missingness};
