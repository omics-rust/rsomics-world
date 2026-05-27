use std::path::PathBuf;

use clap::Parser;
use rsomics_common::{CommonFlags, Result, RsomicsError};

use rsomics_fpkm_count::{FpkmOpts, StrandRules, count_bam, load_bed12, write_fpkm};

#[derive(Parser, Debug)]
#[command(
    name = "rsomics-fpkm-count",
    version,
    about = "Compute per-gene FPKM from a BAM + BED12 model (port of RSeQC FPKM_count)",
    long_about = None,
    disable_help_flag = true,
)]
pub struct Cli {
    /// Input BAM file (must be sorted and indexed).
    #[arg(short = 'i', long = "input")]
    pub input: PathBuf,

    /// Reference gene model in BED12 format.
    #[arg(short = 'r', long = "refgene")]
    pub refgene: PathBuf,

    /// Output file prefix; writes <prefix>.FPKM.xls.
    #[arg(short = 'o', long = "out-prefix")]
    pub out_prefix: String,

    /// Minimum mapping quality to count a read as uniquely mapped.
    #[arg(long = "mapq", default_value_t = 30)]
    pub mapq: u8,

    /// Skip multi-hit reads (those with MAPQ below --mapq).
    #[arg(short = 'u', long = "skip-multi-hits", default_value_t = false)]
    pub unique_only: bool,

    /// Strand specificity rule, e.g. "1++,1--,2+-,2-+".
    /// Absent means unstranded.
    #[arg(short = 's', long = "strand", default_value = "")]
    pub strand: String,

    #[command(flatten)]
    pub common: CommonFlags,
}

impl Cli {
    pub fn execute(self) -> Result<()> {
        let strand = StrandRules::parse(&self.strand)?;
        let opts = FpkmOpts {
            min_mapq: self.mapq,
            unique_only: self.unique_only,
            strand,
        };

        let genes = load_bed12(&self.refgene)?;
        let result = count_bam(&self.input, &genes, &opts)?;

        let out_path = format!("{}.FPKM.xls", self.out_prefix);
        let out_file = std::fs::File::create(&out_path).map_err(RsomicsError::Io)?;
        write_fpkm(&genes, &result, out_file)?;

        eprintln!(
            "total_fragments={} output={}",
            result.total_fragments, out_path
        );

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
