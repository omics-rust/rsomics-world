use std::num::NonZero;
use std::path::PathBuf;

use clap::Parser;
use rsomics_common::{CommonFlags, Result, Tool, ToolMeta};

use rsomics_bam_junctions::annotate_junctions;

pub const META: ToolMeta = ToolMeta {
    name: env!("CARGO_PKG_NAME"),
    version: env!("CARGO_PKG_VERSION"),
};

#[derive(Parser, Debug)]
#[command(
    name = "rsomics-bam-junctions",
    version,
    about = "Annotate splice junctions from BAM reads vs BED12 gene model",
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

    /// Minimum intron length (bp).
    #[arg(short = 'm', long = "min-intron", default_value_t = 50)]
    pub min_intron: i64,

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

        let counts = annotate_junctions(
            &self.input,
            &self.refgene,
            self.min_intron,
            self.mapq,
            workers,
        )?;

        if self.common.json {
            let j = serde_json::to_string_pretty(&counts).unwrap();
            println!("{j}");
        } else {
            counts
                .write_rseqc_stdout(std::io::stdout().lock())
                .map_err(rsomics_common::RsomicsError::Io)?;
            counts
                .write_rseqc_stderr(std::io::stderr().lock())
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
