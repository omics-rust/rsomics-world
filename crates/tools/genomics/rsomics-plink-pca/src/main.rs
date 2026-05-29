use std::process::ExitCode;

use anyhow::Context;
use clap::Parser;

use rsomics_pgen::Pgen;
use rsomics_plink_pca::run_pca;

#[derive(Parser, Debug)]
#[command(
    name = "rsomics-plink-pca",
    version,
    about = "PCA from PLINK1 binary filesets via GRM eigendecomposition",
    long_about = None
)]
struct Cli {
    /// Path to PLINK1 binary fileset prefix (reads .bed/.bim/.fam).
    #[arg(short = 'p', long = "plink")]
    plink: std::path::PathBuf,

    /// Output prefix; writes <prefix>.eigenvec and <prefix>.eigenval.
    #[arg(short = 'o', long = "out", default_value = "pca")]
    output: String,

    /// Number of principal components to compute.
    #[arg(short = 'k', long = "pcs", default_value = "10")]
    pcs: usize,
}

fn run(args: &Cli) -> anyhow::Result<()> {
    let pgen = Pgen::load(&args.plink)
        .with_context(|| format!("loading PLINK fileset {}", args.plink.display()))?;

    let evec_path = format!("{}.eigenvec", args.output);
    let eval_path = format!("{}.eigenval", args.output);

    let mut evec_file = std::io::BufWriter::new(
        std::fs::File::create(&evec_path).with_context(|| format!("creating {evec_path:?}"))?,
    );
    let mut eval_file = std::io::BufWriter::new(
        std::fs::File::create(&eval_path).with_context(|| format!("creating {eval_path:?}"))?,
    );

    run_pca(&pgen, args.pcs, &mut evec_file, &mut eval_file)?;
    Ok(())
}

fn main() -> ExitCode {
    let args = Cli::parse();
    match run(&args) {
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
