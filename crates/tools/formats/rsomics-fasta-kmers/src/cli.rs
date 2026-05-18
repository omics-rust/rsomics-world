use std::path::PathBuf;

use clap::Parser;
use rsomics_common::{CommonFlags, Result, RsomicsError, ToolMeta};
use rsomics_help::{Example, FlagSpec, HelpSpec, Origin, Section};

use rsomics_fasta_kmers::count_kmers;

pub const META: ToolMeta = ToolMeta {
    name: env!("CARGO_PKG_NAME"),
    version: env!("CARGO_PKG_VERSION"),
};

#[derive(Parser, Debug)]
#[command(name = "rsomics-fasta-kmers", version, about, long_about = None, disable_help_flag = true)]
pub struct Cli {
    pub input: PathBuf,
    #[arg(short = 'o', long = "output", default_value = "-")]
    output: String,
    #[arg(short = 'k', default_value_t = 21)]
    k: usize,
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
        let total = count_kmers(&self.input, &mut out, self.k)?;
        if !self.common.quiet {
            eprintln!("{total} k-mers counted (k={})", self.k);
        }
        Ok(())
    }
}

pub static HELP: HelpSpec = HelpSpec {
    name: META.name,
    version: META.version,
    tagline: "Count k-mer frequencies in FASTA sequences.",
    origin: Some(Origin {
        upstream: "jellyfish count / KMC",
        upstream_license: "MIT",
        our_license: "MIT OR Apache-2.0",
        paper_doi: None,
    }),
    usage_lines: &["<input.fasta> [-k 21] [-o kmers.tsv]"],
    sections: &[Section {
        title: "OPTIONS",
        flags: &[FlagSpec {
            short: Some('k'),
            long: "k",
            aliases: &[],
            value: Some("<N>"),
            type_hint: Some("usize"),
            required: false,
            default: Some("21"),
            description: "K-mer size.",
            why_default: Some("standard for genomics"),
        }],
    }],
    examples: &[Example {
        description: "Count 21-mers",
        command: "rsomics-fasta-kmers genome.fasta -k 21",
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
