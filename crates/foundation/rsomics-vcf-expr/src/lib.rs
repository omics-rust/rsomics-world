//! bcftools-style VCF filter-expression parser and per-sample evaluator.
//!
//! Implements the subset of the bcftools filter language needed for
//! `rsomics-vcf-setgt -t q` and `rsomics-vcf-filter -i/-e`:
//!
//! **Supported field references**
//! - `FMT/<TAG>` / `FORMAT/<TAG>` — per-sample FORMAT field (numeric)
//! - `INFO/<TAG>` — site-level INFO field (numeric or string)
//! - `QUAL` — site-level QUAL column (numeric)
//! - `FILTER` — site-level FILTER string
//! - `GT` — per-sample genotype with special string values:
//!   `"."`, `"hom"`, `"het"`, `"ref"`, `"alt"`, `"miss"` / `"missing"`
//!
//! **Operators**
//! - Comparison: `<` `<=` `>` `>=` `==` `!=`
//! - Logical: `&&` `||`
//! - Parentheses for grouping
//!
//! **Scoped out** (exits with error if used):
//! - Regex operators `~` `!~`
//! - Array-index syntax `TAG[0]`
//! - Aggregation functions `N_PASS()`, `F_PASS()`, `SMPL_MAX()`, etc.
//! - Arithmetic `+` `-` `*` `/` `%`
//! - Special fields `N_MISSING`, `F_MISSING`, `N_ALT`, `AC`, `AN`, `AF`
//!
//! ## Origin
//!
//! Independent Rust implementation based on the MIT-licensed bcftools source
//! (filter.c, develop branch) and the VCF 4.3 format specification.
//! Consulted: <https://github.com/samtools/bcftools>
//! License: MIT OR Apache-2.0
//! Upstream credit: bcftools <https://github.com/samtools/bcftools> (MIT)

pub mod eval;
pub mod parse;

pub use eval::{EvalContext, EvalError, SampleResult, eval_expr};
pub use parse::{Expr, ParseError, parse_expr};
