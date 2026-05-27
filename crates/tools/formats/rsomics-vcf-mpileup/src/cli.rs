use std::num::NonZeroUsize;
use std::path::PathBuf;

use clap::Parser;
use rsomics_common::{CommonFlags, Result, ToolMeta};

use rsomics_vcf_mpileup::{
    DEFAULT_MAX_DEPTH, DEFAULT_MIN_BASEQ, DEFAULT_MIN_MAPQ, VcfMpileupOpts, run,
};

pub const META: ToolMeta = ToolMeta {
    name: env!("CARGO_PKG_NAME"),
    version: env!("CARGO_PKG_VERSION"),
};

#[derive(Parser, Debug)]
#[command(
    name = "rsomics-vcf-mpileup",
    version,
    about = "VCF-emitting pileup with genotype likelihoods — single-sample SNP mode (bcftools mpileup port)",
    long_about = None,
    disable_help_flag = true
)]
pub struct Cli {
    /// Input BAM (coordinate-sorted; index required only with -r).
    #[arg(value_name = "INPUT.bam")]
    pub input: PathBuf,

    /// Faidx-indexed reference FASTA (required).
    #[arg(short = 'f', long = "fasta-ref", value_name = "FILE")]
    pub fasta_ref: PathBuf,

    /// Restrict pileup to a region ("chrom" or "chrom:start-end", 1-based).
    #[arg(short = 'r', long = "region", value_name = "REG")]
    pub region: Option<String>,

    /// Optional extra FORMAT/INFO tags to enable (e.g. "DP,AD"). Currently a
    /// no-op placeholder; AD and DP are always emitted. Kept for CLI compat.
    #[arg(short = 'a', long = "annotate", value_name = "LIST")]
    pub annotate: Option<String>,

    /// Skip alignments with mapQ < INT.
    #[arg(long = "min-MQ", value_name = "INT", default_value_t = DEFAULT_MIN_MAPQ)]
    pub min_mapq: u8,

    /// Skip bases with baseQ < INT.
    #[arg(short = 'Q', long = "min-BQ", value_name = "INT", default_value_t = DEFAULT_MIN_BASEQ)]
    pub min_baseq: u8,

    #[command(flatten)]
    pub common: CommonFlags,
}

impl Cli {
    pub fn execute(self) -> Result<()> {
        let opts = VcfMpileupOpts {
            min_mapq: self.min_mapq,
            min_baseq: self.min_baseq,
            max_depth: DEFAULT_MAX_DEPTH,
            region: self.region,
            threads: NonZeroUsize::new(self.common.thread_count()).unwrap_or(NonZeroUsize::MIN),
        };
        let stdout = std::io::stdout().lock();
        run(&self.input, &self.fasta_ref, stdout, &opts)
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
