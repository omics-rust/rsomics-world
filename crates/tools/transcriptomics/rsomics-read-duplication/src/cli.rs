use std::num::NonZero;
use std::path::PathBuf;

use clap::Parser;
use rsomics_common::{CommonFlags, Result, Tool, ToolMeta};

use rsomics_read_duplication::run_duplication;

pub const META: ToolMeta = ToolMeta {
    name: env!("CARGO_PKG_NAME"),
    version: env!("CARGO_PKG_VERSION"),
};

#[derive(Parser, Debug)]
#[command(
    name = "rsomics-read-duplication",
    version,
    about = "Sequence-based and position-based read duplication rate",
    long_about = None,
    disable_help_flag = true
)]
pub struct Cli {
    /// Input BAM file (must be sorted and indexed).
    #[arg(short = 'i', long = "input")]
    pub input: PathBuf,

    /// Prefix for output files (<prefix>.seq.DupRate.xls, <prefix>.pos.DupRate.xls).
    #[arg(short = 'o', long = "out-prefix")]
    pub out_prefix: PathBuf,

    /// Minimum MAPQ for an alignment to be considered.
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

        run_duplication(&self.input, &self.out_prefix, self.mapq, workers)?;
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
