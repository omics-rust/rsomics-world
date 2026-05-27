use std::num::NonZero;
use std::path::PathBuf;

use clap::Parser;
use rsomics_common::{CommonFlags, Result, RsomicsError};

use rsomics_deletion_profile::{compute, write_r, write_txt};

#[derive(Parser, Debug)]
#[command(
    name = "rsomics-deletion-profile",
    version,
    about = "Per-base CIGAR-deletion rate along aligned reads.",
    long_about = None
)]
pub struct Cli {
    /// Input BAM file.
    #[arg(short = 'i', long = "input")]
    pub input: PathBuf,

    /// Alignment length of read (reads not matching this length are skipped).
    #[arg(short = 'l', long = "read-align-length")]
    pub read_length: usize,

    /// Prefix for output files (<prefix>.deletion_profile.txt and <prefix>.deletion_profile.r).
    #[arg(short = 'o', long = "out-prefix")]
    pub out_prefix: String,

    /// Maximum number of reads with deletions to use.
    #[arg(short = 'n', long = "read-num", default_value_t = 1_000_000)]
    pub read_num: u64,

    /// Minimum mapping quality.
    #[arg(long = "mapq", default_value_t = 30)]
    pub min_mapq: u8,

    #[command(flatten)]
    pub common: CommonFlags,
}

impl Cli {
    pub fn execute(self) -> Result<()> {
        let workers = NonZero::new(self.common.thread_count()).unwrap_or(NonZero::<usize>::MIN);

        let profile = compute(
            &self.input,
            self.read_length,
            self.min_mapq,
            self.read_num,
            workers,
        )?;

        if !self.common.quiet {
            eprintln!(
                "Process BAM file ...  Total reads used: {}",
                profile.reads_used
            );
        }

        let txt_path = format!("{}.deletion_profile.txt", self.out_prefix);
        let r_path = format!("{}.deletion_profile.r", self.out_prefix);
        let pdf_path = format!("{}.deletion_profile.pdf", self.out_prefix);

        let mut txt_file = std::fs::File::create(&txt_path)
            .map_err(|e| RsomicsError::InvalidInput(format!("creating {txt_path}: {e}")))?;
        write_txt(&profile, &mut txt_file)?;

        let mut r_file = std::fs::File::create(&r_path)
            .map_err(|e| RsomicsError::InvalidInput(format!("creating {r_path}: {e}")))?;
        write_r(&profile, &pdf_path, &mut r_file)?;

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
