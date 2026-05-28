//! Population-genetics statistics from VCF.
//!
//! Each subcommand shares a single-pass VCF parser; output matches vcftools column
//! layout for drop-in compatibility.

#![allow(clippy::cast_precision_loss)]

pub mod freq;
pub mod hardy;
pub mod het;
pub mod missing;
pub mod pi;
pub mod singleton;
pub mod vcf;

pub use freq::{FreqRecord, freq};
pub use hardy::{HardyRecord, hardy};
pub use het::{HetRecord, het};
pub use missing::{MissingIndv, MissingSite, missing_indv, missing_site};
pub use pi::{PiWindow, pi_windows};
pub use singleton::{SingletonRecord, singletons};
