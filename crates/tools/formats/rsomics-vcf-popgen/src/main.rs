mod cli;

use std::process::ExitCode;

use clap::Parser;

use rsomics_vcf_popgen::{
    freq::{freq, print_freq},
    hardy::{hardy, print_hardy},
    het::{het, print_het},
    missing::{missing_indv, missing_site, print_missing_indv, print_missing_site},
    pi::{pi_windows, print_pi},
    singleton::{print_singletons, singletons},
};

fn main() -> ExitCode {
    let cli = cli::Cli::parse();
    match run(cli) {
        Ok(()) => ExitCode::SUCCESS,
        Err(e) => {
            eprintln!("error: {e:#}");
            ExitCode::FAILURE
        }
    }
}

fn run(cli: cli::Cli) -> anyhow::Result<()> {
    match cli.cmd {
        cli::Cmd::Freq { input } => {
            let records = freq(&input)?;
            print_freq(&records);
        }
        cli::Cmd::Het { input } => {
            let records = het(&input)?;
            print_het(&records);
        }
        cli::Cmd::Hardy { input } => {
            let records = hardy(&input)?;
            print_hardy(&records);
        }
        cli::Cmd::MissingSite { input } => {
            let records = missing_site(&input)?;
            print_missing_site(&records);
        }
        cli::Cmd::MissingIndv { input } => {
            let records = missing_indv(&input)?;
            print_missing_indv(&records);
        }
        cli::Cmd::Pi {
            input,
            window,
            step,
        } => {
            let step = step.unwrap_or(window);
            let records = pi_windows(&input, window, step)?;
            print_pi(&records);
        }
        cli::Cmd::Singleton { input } => {
            let records = singletons(&input)?;
            print_singletons(&records);
        }
    }
    Ok(())
}
