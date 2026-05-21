use std::path::PathBuf;

use clap::Parser;
use rsomics_common::{CommonFlags, Result, RsomicsError, Tool, ToolMeta};
use rsomics_help::{Example, HelpSpec};

use rsomics_count_matrix::merge_counts;

pub const META: ToolMeta = ToolMeta {
    name: env!("CARGO_PKG_NAME"),
    version: env!("CARGO_PKG_VERSION"),
};

#[derive(Parser, Debug)]
#[command(name = "rsomics-count-matrix", version, about, long_about = None, disable_help_flag = true)]
pub struct Cli {
    #[arg(required = true, num_args = 1..)]
    inputs: Vec<PathBuf>,
    #[arg(short = 'o', long, default_value = "-")]
    output: String,
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
        let paths: Vec<&std::path::Path> = self.inputs.iter().map(PathBuf::as_path).collect();
        let mut out: Box<dyn std::io::Write> = if self.output == "-" {
            Box::new(std::io::stdout().lock())
        } else {
            Box::new(std::fs::File::create(&self.output).map_err(RsomicsError::Io)?)
        };
        let n = merge_counts(&paths, &mut out)?;
        if !self.common.quiet {
            eprintln!("{n} genes × {} samples", self.inputs.len());
        }
        Ok(())
    }
}

pub static HELP: HelpSpec = HelpSpec {
    name: env!("CARGO_PKG_NAME"),
    version: env!("CARGO_PKG_VERSION"),
    tagline: "Merge featureCounts/htseq-count outputs into a gene × sample count matrix.",
    origin: None,
    usage_lines: &["<sample1.txt> <sample2.txt> ... [-o matrix.tsv]"],
    sections: &[],
    examples: &[Example {
        description: "Merge three samples",
        command: "rsomics-count-matrix s1.txt s2.txt s3.txt -o counts.tsv",
    }],
    json_result_schema_doc: None,
};

#[cfg(test)]
mod tests {
    use super::*;
    use clap::CommandFactory;

    #[test]
    fn cli_debug_assert() {
        Cli::command().debug_assert();
    }
}
