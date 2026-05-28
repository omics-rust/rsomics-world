use clap::{Parser, Subcommand};
use rsomics_pgen::Pgen;
use rsomics_plink_io::stats::{print_freq, print_hwe, print_missing};
use rsomics_plink_io::{allele_freq, hwe_stats, missingness, to_012, to_vcf};
use std::path::PathBuf;
use std::process::ExitCode;

#[derive(Parser)]
#[command(
    name = "rsomics-plink-io",
    about = "PLINK1 binary .bed/.bim/.fam statistics and format conversion",
    version
)]
struct Cli {
    #[command(subcommand)]
    cmd: Cmd,
}

#[derive(Subcommand)]
enum Cmd {
    /// Allele frequency per variant (plink --freq)
    Freq {
        /// Path prefix for .bed/.bim/.fam (without extension)
        bfile: PathBuf,
    },
    /// Missingness per variant (plink --missing --variant-only)
    Missing {
        /// Path prefix for .bed/.bim/.fam (without extension)
        bfile: PathBuf,
    },
    /// Hardy-Weinberg equilibrium per variant (plink --hardy)
    Hardy {
        /// Path prefix for .bed/.bim/.fam (without extension)
        bfile: PathBuf,
    },
    /// Export to VCF format
    ToVcf {
        /// Path prefix for .bed/.bim/.fam (without extension)
        bfile: PathBuf,
    },
    /// Export to 012 genotype matrix (0=hom-ref, 1=het, 2=hom-alt, -1=missing)
    #[command(name = "to-012")]
    To012 {
        /// Path prefix for .bed/.bim/.fam (without extension)
        bfile: PathBuf,
    },
}

fn main() -> ExitCode {
    let cli = Cli::parse();
    let result = run(cli);
    match result {
        Ok(()) => ExitCode::SUCCESS,
        Err(e) => {
            eprintln!("error: {e}");
            ExitCode::FAILURE
        }
    }
}

fn run(cli: Cli) -> anyhow::Result<()> {
    match cli.cmd {
        Cmd::Freq { bfile } => {
            let pgen = Pgen::load(&bfile)?;
            let records = allele_freq(&pgen);
            print_freq(&records);
        }
        Cmd::Missing { bfile } => {
            let pgen = Pgen::load(&bfile)?;
            let records = missingness(&pgen);
            print_missing(&records);
        }
        Cmd::Hardy { bfile } => {
            let pgen = Pgen::load(&bfile)?;
            let records = hwe_stats(&pgen);
            print_hwe(&records);
        }
        Cmd::ToVcf { bfile } => {
            let pgen = Pgen::load(&bfile)?;
            let stdout = std::io::stdout();
            to_vcf(&pgen, &mut stdout.lock())?;
        }
        Cmd::To012 { bfile } => {
            let pgen = Pgen::load(&bfile)?;
            let stdout = std::io::stdout();
            to_012(&pgen, &mut stdout.lock())?;
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn cli_definition_is_valid() {
        <Cli as clap::CommandFactory>::command().debug_assert();
    }
}
