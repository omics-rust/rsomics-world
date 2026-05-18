use std::path::PathBuf;

use clap::Parser;
use rsomics_common::{CommonFlags, Result, RsomicsError, ToolMeta};
use rsomics_help::{Example, HelpSpec, Origin, Section};

use rsomics_fasta_concat::concat_fasta;

pub const META: ToolMeta = ToolMeta {
    name: env!("CARGO_PKG_NAME"),
    version: env!("CARGO_PKG_VERSION"),
};

#[derive(Parser, Debug)]
#[command(
    name = "rsomics-fasta-concat",
    version,
    about,
    long_about = None,
    disable_help_flag = true
)]
pub struct Cli {
    #[arg(required = true)]
    pub inputs: Vec<PathBuf>,
    #[arg(short = 'o', long = "output", default_value = "-")]
    output: String,
    #[command(flatten)]
    pub common: CommonFlags,
}

impl Cli {
    pub fn execute(self) -> Result<()> {
        let paths: Vec<&std::path::Path> = self.inputs.iter().map(PathBuf::as_path).collect();
        let mut out: Box<dyn std::io::Write> = if self.output == "-" {
            Box::new(std::io::stdout().lock())
        } else {
            Box::new(std::fs::File::create(&self.output).map_err(RsomicsError::Io)?)
        };
        let bytes = concat_fasta(&paths, &mut out)?;
        if !self.common.quiet {
            eprintln!("{} files, {} bytes", self.inputs.len(), bytes);
        }
        Ok(())
    }
}

pub static HELP: HelpSpec = HelpSpec {
    name: META.name,
    version: META.version,
    tagline: "Concatenate multiple FASTA files.",
    origin: Some(Origin {
        upstream: "cat",
        upstream_license: "N/A",
        our_license: "MIT OR Apache-2.0",
        paper_doi: None,
    }),
    usage_lines: &["<file1.fa> <file2.fa> [...] [-o merged.fa]"],
    sections: &[Section {
        title: "OPTIONS",
        flags: &[],
    }],
    examples: &[Example {
        description: "Concatenate chromosome files",
        command: "rsomics-fasta-concat chr1.fa chr2.fa chr3.fa -o genome.fa",
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
