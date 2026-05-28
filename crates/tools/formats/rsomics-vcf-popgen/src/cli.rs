use std::path::PathBuf;

use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(
    name = "rsomics-vcf-popgen",
    about = "Population-genetics statistics from VCF (vcftools-compatible)",
    version
)]
pub struct Cli {
    #[command(subcommand)]
    pub cmd: Cmd,
}

#[derive(Subcommand)]
pub enum Cmd {
    /// Allele frequency per site (vcftools --freq)
    Freq {
        /// Input VCF file
        input: PathBuf,
    },
    /// Per-individual heterozygosity and inbreeding (vcftools --het)
    Het {
        /// Input VCF file
        input: PathBuf,
    },
    /// Hardy-Weinberg equilibrium test per biallelic site (vcftools --hardy)
    Hardy {
        /// Input VCF file
        input: PathBuf,
    },
    /// Missingness per site (vcftools --missing-site)
    MissingSite {
        /// Input VCF file
        input: PathBuf,
    },
    /// Missingness per individual (vcftools --missing-indv)
    MissingIndv {
        /// Input VCF file
        input: PathBuf,
    },
    /// Nucleotide diversity π in sliding windows (vcftools --window-pi)
    Pi {
        /// Input VCF file
        input: PathBuf,
        /// Window size in bp
        #[arg(long, default_value = "10000")]
        window: u64,
        /// Step size in bp (defaults to window size = non-overlapping windows)
        #[arg(long)]
        step: Option<u64>,
    },
    /// Singleton and private doubleton sites (vcftools --singletons)
    Singleton {
        /// Input VCF file
        input: PathBuf,
    },
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn cli_definition_is_valid() {
        <Cli as clap::CommandFactory>::command().debug_assert();
    }
}
