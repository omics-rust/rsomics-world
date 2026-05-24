use std::io::Write;
use std::path::PathBuf;

use clap::Parser;
use rsomics_common::{CommonFlags, Result, Tool, ToolMeta};

use rsomics_genebody_coverage::{compute_coverage, load_transcripts, sample_name_from_path};

pub const META: ToolMeta = ToolMeta {
    name: env!("CARGO_PKG_NAME"),
    version: env!("CARGO_PKG_VERSION"),
};

#[derive(Parser, Debug)]
#[command(
    name = "rsomics-genebody-coverage",
    version,
    about = "Gene-body coverage profile (5'→3') for RNA-seq 3'-bias / degradation QC",
    long_about = None,
    disable_help_flag = true
)]
pub struct Cli {
    /// Input BAM file(s), sorted and indexed.
    #[arg(short = 'i', long = "input", num_args = 1.., value_delimiter = ',')]
    pub input: Vec<PathBuf>,

    /// Reference gene model in BED12 format.
    #[arg(short = 'r', long = "refgene")]
    pub refgene: PathBuf,

    /// Minimum mRNA length (bp); transcripts shorter than this are skipped.
    #[arg(short = 'l', long = "minimum-length", default_value_t = 100)]
    pub min_length: usize,

    /// Prefix for output files.
    #[arg(short = 'o', long = "out-prefix")]
    pub out_prefix: PathBuf,

    #[command(flatten)]
    pub common: CommonFlags,
}

impl Cli {
    fn run_inner(self) -> Result<()> {
        if self.min_length < 100 {
            return Err(rsomics_common::RsomicsError::InvalidInput(
                "The number specified to \"-l\" cannot be smaller than 100.".into(),
            ));
        }

        eprintln!("Read BED file (reference gene model) ...");
        let transcripts = load_transcripts(&self.refgene, self.min_length)?;
        eprintln!("Total {} transcripts loaded", transcripts.len());

        let txt_path = self.out_prefix.with_extension("").with_file_name({
            let stem = self
                .out_prefix
                .file_name()
                .and_then(|n| n.to_str())
                .unwrap_or("out");
            format!("{stem}.geneBodyCoverage.txt")
        });

        let mut out_file = std::fs::File::create(&txt_path).map_err(|e| {
            rsomics_common::RsomicsError::Io(std::io::Error::other(format!(
                "creating {}: {e}",
                txt_path.display()
            )))
        })?;

        // Write header row once.
        write!(out_file, "Percentile")?;
        for i in 1u32..=100 {
            write!(out_file, "\t{i}")?;
        }
        writeln!(out_file)?;

        for bam_path in &self.input {
            eprintln!("Processing {} ...", bam_path.display());
            let coverage = compute_coverage(bam_path, &transcripts)?;
            let name = sample_name_from_path(bam_path);
            write!(out_file, "{name}")?;
            for v in &coverage {
                write!(out_file, "\t{v}")?;
            }
            writeln!(out_file)?;
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
