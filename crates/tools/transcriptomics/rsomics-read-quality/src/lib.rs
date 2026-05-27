/// Per-base read quality heatmap and boxplot from a BAM file.
///
/// ## Origin
///
/// This crate is an independent Rust reimplementation of `RSeQC read_quality.py`
/// based on:
/// - The RSeQC documentation: <https://rseqc.sourceforge.net/#read-quality-py>
/// - Black-box behaviour testing against the upstream binary (RSeQC 2.6.2)
///
/// No source code from the GPL-2 upstream was used as reference during
/// implementation. Test fixtures are independently generated.
///
/// License: MIT OR Apache-2.0.
/// Upstream credit: RSeQC <http://rseqc.sourceforge.net/> (GPL-2).
pub mod qual;
pub use qual::{QualMatrix, ReadQualityOpts, run_read_quality};
