use std::num::NonZero;
use std::path::PathBuf;

use clap::Parser;
use rsomics_common::{CommonFlags, Result, Tool, ToolMeta};

use rsomics_bam_read_dist::run_read_dist;

pub const META: ToolMeta = ToolMeta {
    name: env!("CARGO_PKG_NAME"),
    version: env!("CARGO_PKG_VERSION"),
};

#[derive(Parser, Debug)]
#[command(
    name = "rsomics-bam-read-dist",
    version,
    about = "Classify mapped reads into genomic regions from BAM + BED12 gene model",
    long_about = None,
    disable_help_flag = true
)]
pub struct Cli {
    /// Input BAM file.
    #[arg(short = 'i', long = "input")]
    pub input: PathBuf,

    /// Reference gene model in BED12 format.
    #[arg(short = 'r', long = "refgene")]
    pub refgene: PathBuf,

    #[command(flatten)]
    pub common: CommonFlags,
}

impl Cli {
    fn run_inner(self) -> Result<()> {
        let workers = self
            .common
            .threads
            .and_then(NonZero::new)
            .unwrap_or_else(|| {
                std::thread::available_parallelism().unwrap_or(NonZero::<usize>::MIN)
            });

        let result = run_read_dist(&self.input, &self.refgene, workers)?;

        if self.common.json {
            let j = serde_json::to_string_pretty(&result)
                .map_err(|e| rsomics_common::RsomicsError::Io(std::io::Error::other(e)))?;
            println!("{j}");
        } else {
            result
                .write_rseqc(std::io::stdout().lock())
                .map_err(rsomics_common::RsomicsError::Io)?;
        }

        Ok(())
    }
}

impl Tool for Cli {
    fn meta() -> ToolMeta {
        META
    }

    fn common(&self) -> &CommonFlags {
        &self.common
    }

    fn execute(self) -> Result<()> {
        self.run_inner()
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
