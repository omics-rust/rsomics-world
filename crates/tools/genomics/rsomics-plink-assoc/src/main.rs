use clap::{Parser, Subcommand};
use rsomics_pgen::Pgen;
use rsomics_plink_assoc::{assoc_test, linear_test, print_assoc, print_linear};
use std::path::PathBuf;
use std::process::ExitCode;

#[derive(Parser)]
#[command(
    name = "rsomics-plink-assoc",
    about = "PLINK1 case/control chi-squared and linear regression association tests",
    version
)]
struct Cli {
    #[command(subcommand)]
    cmd: Cmd,
}

#[derive(Subcommand)]
enum Cmd {
    /// Chi-squared allelic association test (plink --assoc)
    Assoc {
        /// Path prefix for .bed/.bim/.fam (without extension)
        bfile: PathBuf,
    },
    /// Linear regression association test for quantitative phenotypes (plink --linear)
    Linear {
        /// Path prefix for .bed/.bim/.fam (without extension)
        bfile: PathBuf,
    },
}

fn main() -> ExitCode {
    let cli = Cli::parse();
    match run(cli) {
        Ok(()) => ExitCode::SUCCESS,
        Err(e) => {
            eprintln!("error: {e:#}");
            ExitCode::FAILURE
        }
    }
}

fn run(cli: Cli) -> anyhow::Result<()> {
    match cli.cmd {
        Cmd::Assoc { bfile } => {
            let pgen = Pgen::load(&bfile)?;
            let records = assoc_test(&pgen);
            let stdout = std::io::stdout();
            print_assoc(&records, &mut stdout.lock());
        }
        Cmd::Linear { bfile } => {
            let pgen = Pgen::load(&bfile)?;
            let records = linear_test(&pgen);
            let stdout = std::io::stdout();
            print_linear(&records, &mut stdout.lock());
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
