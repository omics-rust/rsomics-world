use clap::Parser;
use rsomics_common::{CommonFlags, Result, RsomicsError, ToolMeta};
use rsomics_fasta_unique::unique_fasta;
use rsomics_help::{Example, HelpSpec, Origin, Section};
use std::path::PathBuf;

pub const META: ToolMeta = ToolMeta {
    name: env!("CARGO_PKG_NAME"),
    version: env!("CARGO_PKG_VERSION"),
};

#[derive(Parser, Debug)]
#[command(name = "rsomics-fasta-unique", version, about, long_about = None, disable_help_flag = true)]
pub struct Cli {
    pub input: PathBuf,
    #[arg(short = 'o', long = "output", default_value = "-")]
    output: String,
    #[command(flatten)]
    pub common: CommonFlags,
}

impl Cli {
    pub fn execute(self) -> Result<()> {
        let mut out: Box<dyn std::io::Write> = if self.output == "-" {
            Box::new(std::io::stdout().lock())
        } else {
            Box::new(std::fs::File::create(&self.output).map_err(RsomicsError::Io)?)
        };
        let (total, kept) = unique_fasta(&self.input, &mut out)?;
        if !self.common.quiet {
            eprintln!("{kept}/{total} unique");
        }
        Ok(())
    }
}

pub static HELP: HelpSpec = HelpSpec {
    name: META.name,
    version: META.version,
    tagline: "Output unique FASTA sequences (hash-based dedup).",
    origin: Some(Origin {
        upstream: "seqkit rmdup -s",
        upstream_license: "MIT",
        our_license: "MIT OR Apache-2.0",
        paper_doi: None,
    }),
    usage_lines: &["<input.fa> [-o unique.fa]"],
    sections: &[Section {
        title: "OPTIONS",
        flags: &[],
    }],
    examples: &[Example {
        description: "Keep unique sequences",
        command: "rsomics-fasta-unique input.fa -o unique.fa",
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
