use std::path::PathBuf;

use clap::Parser;
use rsomics_common::{CommonFlags, Result, RsomicsError, ToolMeta};
use rsomics_help::{Example, FlagSpec, HelpSpec, Origin, Section};

use rsomics_fasta_extract::extract_fasta;

pub const META: ToolMeta = ToolMeta {
    name: env!("CARGO_PKG_NAME"),
    version: env!("CARGO_PKG_VERSION"),
};

#[derive(Parser, Debug)]
#[command(name = "rsomics-fasta-extract", version, about, long_about = None, disable_help_flag = true)]
pub struct Cli {
    pub input: PathBuf,
    #[arg(short = 'l', long = "list")]
    names: PathBuf,
    #[arg(short = 'o', long = "output", default_value = "-")]
    output: String,
    #[arg(short = 'v', long = "exclude")]
    exclude: bool,
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
        let count = extract_fasta(&self.input, &self.names, &mut out, self.exclude)?;
        if !self.common.quiet {
            eprintln!("{count} sequences extracted");
        }
        Ok(())
    }
}

pub static HELP: HelpSpec = HelpSpec {
    name: META.name,
    version: META.version,
    tagline: "Extract sequences by name from FASTA.",
    origin: Some(Origin {
        upstream: "seqtk subseq / samtools faidx",
        upstream_license: "MIT",
        our_license: "MIT OR Apache-2.0",
        paper_doi: None,
    }),
    usage_lines: &["<input.fasta> -l <names.txt> [-v] [-o output.fasta]"],
    sections: &[Section {
        title: "OPTIONS",
        flags: &[
            FlagSpec {
                short: Some('l'),
                long: "list",
                aliases: &[],
                value: Some("<path>"),
                type_hint: Some("Path"),
                required: true,
                default: None,
                description: "File with sequence names (one per line).",
                why_default: None,
            },
            FlagSpec {
                short: Some('v'),
                long: "exclude",
                aliases: &[],
                value: None,
                type_hint: None,
                required: false,
                default: None,
                description: "Exclude listed names instead of including.",
                why_default: None,
            },
        ],
    }],
    examples: &[Example {
        description: "Extract specific chromosomes",
        command: "rsomics-fasta-extract genome.fa -l chroms.txt",
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
