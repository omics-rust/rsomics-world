use std::path::PathBuf;

use clap::Parser;
use rsomics_common::{CommonFlags, Result, RsomicsError, Tool, ToolMeta};

use rsomics_inner_distance::{InnerDistanceSummary, compute_inner_distance, write_output};

pub const META: ToolMeta = ToolMeta {
    name: env!("CARGO_PKG_NAME"),
    version: env!("CARGO_PKG_VERSION"),
};

#[derive(Parser, Debug)]
#[command(
    name = "rsomics-inner-distance",
    version,
    about = "Compute mRNA-aware inner-distance distribution for paired-end RNA-seq",
    long_about = "Compute the inner distance (insert size) of paired-end RNA-seq fragments \
using a BED12 gene model. For each properly-paired read pair, computes the distance \
between the two mates. When both fall in the same transcript, the mRNA (spliced) \
distance is used; otherwise falls back to the genomic distance.\n\n\
Outputs:\n  <prefix>.inner_distance.txt      per-pair distances with category\n  \
<prefix>.inner_distance_freq.txt histogram of inner distances",
    disable_help_flag = true,
    allow_negative_numbers = true
)]
pub struct Cli {
    /// Alignment file in BAM format (coordinate-sorted, indexed)
    #[arg(short = 'i', long = "input")]
    pub input_file: PathBuf,

    /// Prefix for output files
    #[arg(short = 'o', long = "out-prefix")]
    pub out_prefix: String,

    /// Reference gene model in BED12 format
    #[arg(short = 'r', long = "refgene")]
    pub refgene: PathBuf,

    /// Number of read-pairs used to estimate inner distance
    #[arg(short = 'k', long = "sample-size", default_value = "1000000")]
    pub sample_size: u64,

    /// Lower bound of inner distance (bp) for histogram
    #[arg(short = 'l', long = "lower-bound", default_value = "-250")]
    pub lower_bound: i32,

    /// Upper bound of inner distance (bp) for histogram
    #[arg(short = 'u', long = "upper-bound", default_value = "250")]
    pub upper_bound: i32,

    /// Step size (bp) of histogram bins
    #[arg(short = 's', long = "step", default_value = "5")]
    pub step: i32,

    /// Minimum mapping quality (phred scaled)
    #[arg(long = "mapq", default_value = "30")]
    pub mapq: u8,

    #[command(flatten)]
    pub common: CommonFlags,
}

impl Tool for Cli {
    fn meta() -> ToolMeta {
        META
    }

    fn common(&self) -> &CommonFlags {
        &self.common
    }

    fn execute(self) -> Result<()> {
        if self.step <= 0 {
            return Err(RsomicsError::InvalidInput(
                "step size must be a positive integer".to_string(),
            ));
        }

        eprintln!("Get exon regions from {} ...", self.refgene.display());

        let result = compute_inner_distance(
            &self.input_file,
            &self.refgene,
            self.sample_size,
            self.mapq,
            self.lower_bound,
            self.upper_bound,
            self.step,
        )?;

        eprintln!("Total read pairs  used {}", result.pair_num);

        write_output(&result, &self.out_prefix)?;

        if self.common.json {
            let summary = InnerDistanceSummary {
                pair_num: result.pair_num,
                output_prefix: self.out_prefix,
            };
            println!(
                "{}",
                serde_json::to_string_pretty(&summary)
                    .map_err(|e| RsomicsError::InvalidInput(e.to_string()))?
            );
        }

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
