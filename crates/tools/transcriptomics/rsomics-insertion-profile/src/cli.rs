use std::num::NonZero;
use std::path::PathBuf;

use clap::Parser;
use rsomics_common::{CommonFlags, Result, Tool, ToolMeta};

use rsomics_insertion_profile::run_insertion_profile;

pub const META: ToolMeta = ToolMeta {
    name: env!("CARGO_PKG_NAME"),
    version: env!("CARGO_PKG_VERSION"),
};

#[derive(Parser, Debug)]
#[command(
    name = "rsomics-insertion-profile",
    version,
    about = "Per-position CIGAR-insertion rate along the read from a BAM file",
    long_about = None,
    disable_help_flag = true
)]
pub struct Cli {
    /// Input BAM or SAM file.
    #[arg(short = 'i', long = "input-file")]
    pub input_file: PathBuf,

    /// Prefix for output files (`<prefix>.insertion_profile.xls`, `<prefix>.insertion_profile.r`).
    #[arg(short = 'o', long = "out-prefix")]
    pub out_prefix: PathBuf,

    /// Sequencing layout: "SE" (single-end) or "PE" (paired-end).
    #[arg(short = 's', long = "sequencing", default_value = "SE")]
    pub sequencing: String,

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

        run_insertion_profile(
            &self.input_file,
            &self.out_prefix,
            &self.sequencing,
            self.mapq,
            workers,
        )
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
