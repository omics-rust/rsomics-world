use std::path::PathBuf;

use clap::Parser;
use rsomics_common::{CommonFlags, Result, RsomicsError, ToolMeta};

use rsomics_mismatch_profile::{ProfileOpts, compute_profile, write_r_script, write_xls};

pub const META: ToolMeta = ToolMeta {
    name: env!("CARGO_PKG_NAME"),
    version: env!("CARGO_PKG_VERSION"),
};

#[derive(Parser, Debug)]
#[command(
    name = "rsomics-mismatch-profile",
    version,
    about = "Per-base mismatch-rate profile from BAM MD tags (RSeQC mismatch_profile.py port)",
    long_about = None,
    disable_help_flag = true
)]
pub struct Cli {
    /// Input BAM file (must contain MD tags).
    #[arg(short = 'i', long = "input")]
    pub input: PathBuf,

    /// Read alignment length (set to the original read length, e.g. 100 for 100M).
    #[arg(short = 'l', long = "read-align-length")]
    pub read_len: usize,

    /// Output prefix (produces <prefix>.mismatch_profile.xls and .r).
    #[arg(short = 'o', long = "out-prefix")]
    pub out_prefix: String,

    /// Maximum number of reads with mismatches to use.
    #[arg(short = 'n', long = "read-num", default_value_t = 1_000_000)]
    pub max_reads: u64,

    /// Minimum mapping quality (reads below this are skipped).
    #[arg(long = "mapq", default_value_t = 30)]
    pub min_mapq: u8,

    /// Print help.
    #[arg(short = 'h', long = "help", action = clap::ArgAction::Help)]
    help: Option<bool>,

    #[command(flatten)]
    pub common: CommonFlags,
}

impl Cli {
    pub fn execute(self) -> Result<()> {
        let opts = ProfileOpts {
            read_len: self.read_len,
            max_reads: self.max_reads,
            min_mapq: self.min_mapq,
            threads: self.common.thread_count(),
        };

        if !self.common.quiet {
            eprintln!("Process BAM file ...");
        }

        let profile = compute_profile(&self.input, &opts)?;

        if !self.common.quiet {
            eprintln!(" Total reads used: {}", profile.reads_used);
        }

        let xls_path = format!("{}.mismatch_profile.xls", self.out_prefix);
        let r_path = format!("{}.mismatch_profile.r", self.out_prefix);

        let mut xls_file = std::fs::File::create(&xls_path).map_err(RsomicsError::Io)?;
        write_xls(&profile, &mut xls_file)?;

        let mut r_file = std::fs::File::create(&r_path).map_err(RsomicsError::Io)?;
        write_r_script(&profile, &self.out_prefix, &mut r_file)?;

        Ok(())
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
