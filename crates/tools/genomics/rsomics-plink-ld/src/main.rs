use std::process::ExitCode;

use anyhow::Context;
use clap::Parser;

use rsomics_pgen::Pgen;
use rsomics_plink_ld::compute_ld;

#[derive(Parser, Debug)]
#[command(
    name = "rsomics-plink-ld",
    version,
    about = "Pairwise LD (r²) computation from PLINK1 binary filesets",
    long_about = None
)]
struct Cli {
    /// Path to PLINK1 binary fileset prefix (e.g. data → reads data.bed/.bim/.fam).
    #[arg(short = 'p', long = "plink")]
    plink: std::path::PathBuf,

    /// Output file path ('-' for stdout).
    #[arg(short = 'o', long = "out", default_value = "-")]
    output: String,

    /// Sliding window size in variants (0 = all pairs on same chromosome).
    #[arg(short = 'w', long = "window", default_value = "50")]
    window: usize,

    /// Minimum r² to include in output (0.0 = all pairs).
    #[arg(long = "min-r2", default_value = "0.0")]
    min_r2: f64,

    /// Number of threads (reserved; currently single-threaded).
    #[arg(short = 't', long, default_value = "1")]
    threads: usize,
}

fn run(args: Cli) -> anyhow::Result<()> {
    let pgen = Pgen::load(&args.plink)
        .with_context(|| format!("loading PLINK fileset {:?}", args.plink))?;

    let mut out: Box<dyn std::io::Write> = if args.output == "-" {
        Box::new(std::io::BufWriter::new(std::io::stdout()))
    } else {
        Box::new(std::io::BufWriter::new(
            std::fs::File::create(&args.output)
                .with_context(|| format!("creating output file {:?}", args.output))?,
        ))
    };

    // Header line matching PLINK1 .ld format.
    writeln!(out, "CHR_A\tBP_A\tSNP_A\tCHR_B\tBP_B\tSNP_B\tR2")?;

    compute_ld(&pgen, args.window, args.min_r2, &mut out)?;

    Ok(())
}

fn main() -> ExitCode {
    let args = Cli::parse();
    match run(args) {
        Ok(()) => ExitCode::SUCCESS,
        Err(e) => {
            eprintln!("error: {e:#}");
            ExitCode::FAILURE
        }
    }
}

#[cfg(test)]
mod tests {
    use super::Cli;
    use clap::CommandFactory;

    #[test]
    fn cli_debug_assert() {
        Cli::command().debug_assert();
    }
}
