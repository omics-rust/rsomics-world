use std::path::PathBuf;

use clap::Parser;
use rsomics_common::{CommonFlags, Result, RsomicsError, ToolMeta};

use rsomics_rpkm_saturation::{
    SaturationOpts, compute_saturation, load_genes, load_reads, write_outputs,
};

pub const META: ToolMeta = ToolMeta {
    name: env!("CARGO_PKG_NAME"),
    version: env!("CARGO_PKG_VERSION"),
};

#[derive(Parser, Debug)]
#[command(
    name = "rsomics-rpkm-saturation",
    version,
    about = "Subsample-based RPKM saturation analysis (RSeQC RPKM_saturation.py port)",
    long_about = None
)]
pub struct Cli {
    /// Input BAM file (must be sorted and indexed).
    #[arg(short = 'i', long = "input-file")]
    pub input: PathBuf,

    /// Reference gene model in BED12 format.
    #[arg(short = 'r', long = "refgene")]
    pub refgene: PathBuf,

    /// Prefix for output files (<prefix>.eRPKM.xls, <prefix>.rawCount.xls).
    #[arg(short = 'o', long = "out-prefix")]
    pub out_prefix: String,

    /// Sampling lower bound percentile (0–100).
    #[arg(short = 'l', long = "percentile-floor", default_value_t = 5)]
    pub lower: u8,

    /// Sampling upper bound percentile (0–100).
    #[arg(short = 'u', long = "percentile-ceiling", default_value_t = 100)]
    pub upper: u8,

    /// Sampling step percentile (0–100).
    #[arg(short = 's', long = "percentile-step", default_value_t = 5)]
    pub step: u8,

    /// Minimum MAPQ for a read to be counted.
    #[arg(long = "mapq", default_value_t = 30)]
    pub mapq: u8,

    #[command(flatten)]
    pub common: CommonFlags,
}

impl Cli {
    pub fn execute(self) -> Result<()> {
        if self.lower > self.upper {
            return Err(RsomicsError::InvalidInput(
                "percentile-floor must be <= percentile-ceiling".to_string(),
            ));
        }
        if self.step == 0 {
            return Err(RsomicsError::InvalidInput(
                "percentile-step must be > 0".to_string(),
            ));
        }

        let workers = std::num::NonZero::new(self.common.thread_count())
            .unwrap_or(std::num::NonZero::<usize>::MIN);

        let genes = load_genes(&self.refgene)?;
        let reads = load_reads(&self.input, self.mapq, workers)?;

        if !self.common.quiet {
            eprintln!(
                "rsomics-rpkm-saturation: {} genes, {} mapped reads",
                genes.len(),
                reads.len()
            );
        }

        let opts = SaturationOpts {
            lower_pct: self.lower,
            upper_pct: self.upper,
            step_pct: self.step,
            min_mapq: self.mapq,
            seed: self.common.seed_rng(),
        };

        let results = compute_saturation(&reads, &genes, &opts)?;

        write_outputs(&self.out_prefix, &genes, &results)?;

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use clap::CommandFactory;

    #[test]
    fn cli_debug_assert() {
        Cli::command().debug_assert();
    }
}
