use std::num::NonZero;
use std::path::PathBuf;

use clap::Parser;
use rsomics_common::{CommonFlags, Result, Tool, ToolMeta};

use rsomics_bam_strandedness::infer_strandedness;

pub const META: ToolMeta = ToolMeta {
    name: env!("CARGO_PKG_NAME"),
    version: env!("CARGO_PKG_VERSION"),
};

#[derive(Parser, Debug)]
#[command(
    name = "rsomics-bam-strandedness",
    version,
    about = "Infer RNA-seq library strand protocol from BAM + BED12 gene model",
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

    /// Number of reads to sample (reads landing in a gene interval).
    #[arg(short = 's', long = "sample-size", default_value_t = 200_000)]
    pub sample_size: u64,

    /// Minimum MAPQ for a read to be considered.
    #[arg(long = "mapq", default_value_t = 30)]
    pub mapq: u8,

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

        let result = infer_strandedness(
            &self.input,
            &self.refgene,
            self.sample_size,
            self.mapq,
            workers,
        )?;

        if self.common.json {
            let j = serde_json::json!({
                "protocol": format!("{:?}", result.protocol),
                "spec1": result.spec1,
                "spec2": result.spec2,
                "other": result.other,
                "sampled": result.sampled,
            });
            println!("{}", serde_json::to_string_pretty(&j).unwrap());
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
