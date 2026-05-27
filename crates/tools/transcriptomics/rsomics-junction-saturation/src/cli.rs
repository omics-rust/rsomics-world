use std::num::NonZero;
use std::path::PathBuf;

use clap::Parser;
use rsomics_common::{CommonFlags, Result, ToolMeta};

use rsomics_junction_saturation::{JunctionSaturationOpts, run};

pub const META: ToolMeta = ToolMeta {
    name: env!("CARGO_PKG_NAME"),
    version: env!("CARGO_PKG_VERSION"),
};

#[derive(Parser, Debug)]
#[command(
    name = "rsomics-junction-saturation",
    version,
    about = "Subsample-based splice-junction saturation analysis (RSeQC junction_saturation.py port)",
    long_about = None
)]
pub struct Cli {
    /// Input coordinate-sorted BAM (index not required).
    #[arg(short = 'i', long = "input", value_name = "FILE")]
    pub input: PathBuf,

    /// BED12 gene annotation.
    #[arg(short = 'r', long = "refgene", value_name = "FILE")]
    pub refgene: PathBuf,

    /// Output prefix (creates <prefix>.junction_saturation.txt).
    #[arg(short = 'o', long = "output-prefix", value_name = "STR")]
    pub prefix: String,

    /// Lower bound sampling fraction (percent).
    #[arg(
        short = 'l',
        long = "low-bound",
        value_name = "INT",
        default_value_t = 5
    )]
    pub lower: u8,

    /// Upper bound sampling fraction (percent).
    #[arg(
        short = 'u',
        long = "upper-bound",
        value_name = "INT",
        default_value_t = 100
    )]
    pub upper: u8,

    /// Step between fractions (percent).
    #[arg(short = 's', long = "step", value_name = "INT", default_value_t = 5)]
    pub step: u8,

    /// Minimum mapping quality for reads.
    #[arg(long = "mapq", value_name = "INT", default_value_t = 0)]
    pub mapq: u8,

    /// Minimum intron length to count as a splice junction.
    #[arg(long = "min-intron", value_name = "INT", default_value_t = 50)]
    pub min_intron: u64,

    #[command(flatten)]
    pub common: CommonFlags,
}

impl Cli {
    pub fn execute(self) -> Result<()> {
        let opts = JunctionSaturationOpts {
            lower: self.lower,
            upper: self.upper,
            step: self.step,
            min_mapq: self.mapq,
            min_intron: self.min_intron,
            seed: self.common.seed,
            threads: NonZero::new(self.common.thread_count()).unwrap_or(NonZero::<usize>::MIN),
        };
        run(&self.input, &self.refgene, &self.prefix, &opts)
    }
}

#[cfg(test)]
mod tests {
    use clap::CommandFactory;

    use super::*;

    #[test]
    fn cli_debug_assert() {
        Cli::command().debug_assert();
    }
}
