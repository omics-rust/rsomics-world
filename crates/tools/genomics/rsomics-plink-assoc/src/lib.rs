//! PLINK1 case/control association test and quantitative trait linear regression.
//!
//! Implements plink --assoc (chi-squared allelic test) and plink --linear
//! (per-variant linear regression for quantitative phenotypes).

#![allow(clippy::cast_precision_loss)]

pub mod assoc;
pub mod linear;

pub use assoc::{AssocRecord, assoc_test, print_assoc};
pub use linear::{LinearRecord, linear_test, print_linear};
