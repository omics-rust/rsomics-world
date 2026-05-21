use clap::Parser;
use rsomics_common::{CommonFlags, Result, RsomicsError, Tool, ToolMeta};
use rsomics_fm_search::search_fasta;
use rsomics_help::{Example, FlagSpec, HelpSpec, Section};
use std::path::PathBuf;

pub const META: ToolMeta = ToolMeta {
    name: env!("CARGO_PKG_NAME"),
    version: env!("CARGO_PKG_VERSION"),
};

#[derive(Parser, Debug)]
#[command(name = "rsomics-fm-search", version, about, long_about = None, disable_help_flag = true)]
pub struct Cli {
    pub fasta: PathBuf,
    #[arg(short = 'p', long)]
    pattern: String,
    #[arg(short = 'c', long)]
    count: bool,
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
        let mut out: Box<dyn std::io::Write> = if self.output == "-" {
            Box::new(std::io::stdout().lock())
        } else {
            Box::new(std::fs::File::create(&self.output).map_err(RsomicsError::Io)?)
        };
        let pattern = self.pattern.to_ascii_uppercase().into_bytes();
        let n = search_fasta(&self.fasta, &pattern, self.count, &mut out)?;
        if !self.common.quiet {
            eprintln!("{n} hits");
        }
        Ok(())
    }
}

pub static HELP: HelpSpec = HelpSpec {
    name: env!("CARGO_PKG_NAME"),
    version: env!("CARGO_PKG_VERSION"),
    tagline: "Exact substring search in FASTA using FM-index.",
    origin: None,
    usage_lines: &["<ref.fa> -p <pattern> [-c] [-o matches.bed]"],
    sections: &[Section {
        title: "OPTIONS",
        flags: &[
            FlagSpec {
                short: Some('p'),
                long: "pattern",
                aliases: &[],
                value: Some("<seq>"),
                type_hint: Some("String"),
                required: true,
                default: None,
                description: "DNA/protein pattern to search.",
                why_default: None,
            },
            FlagSpec {
                short: Some('c'),
                long: "count",
                aliases: &[],
                value: None,
                type_hint: Some("bool"),
                required: false,
                default: None,
                description: "Print count per sequence instead of positions.",
                why_default: None,
            },
        ],
    }],
    examples: &[
        Example {
            description: "Find all ATCG occurrences",
            command: "rsomics-fm-search genome.fa -p ATCGATCG",
        },
        Example {
            description: "Count per sequence",
            command: "rsomics-fm-search genome.fa -p ATCG -c",
        },
    ],
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
