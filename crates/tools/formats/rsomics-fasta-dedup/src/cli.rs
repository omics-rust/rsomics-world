use std::path::PathBuf;

use clap::Parser;
use rsomics_common::{CommonFlags, Result, RsomicsError, ToolMeta};
use rsomics_help::{Example, FlagSpec, HelpSpec, Origin, Section};

use rsomics_fasta_dedup::dedup_fasta;

pub const META: ToolMeta = ToolMeta {
    name: env!("CARGO_PKG_NAME"),
    version: env!("CARGO_PKG_VERSION"),
};

#[derive(Parser, Debug)]
#[command(name = "rsomics-fasta-dedup", version, about, long_about = None, disable_help_flag = true)]
pub struct Cli {
    pub input: PathBuf,
    #[arg(short = 'o', long = "output", default_value = "-")]
    output: String,
    #[arg(short = 'n', long = "by-name")]
    by_name: bool,
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
        let (total, kept) = dedup_fasta(&self.input, &mut out, self.by_name)?;
        if !self.common.quiet {
            eprintln!("{kept}/{total} unique sequences");
        }
        Ok(())
    }
}

pub static HELP: HelpSpec = HelpSpec {
    name: META.name,
    version: META.version,
    tagline: "Remove duplicate FASTA sequences.",
    origin: Some(Origin {
        upstream: "seqkit rmdup / cd-hit",
        upstream_license: "MIT",
        our_license: "MIT OR Apache-2.0",
        paper_doi: None,
    }),
    usage_lines: &["<input.fasta> [-n] [-o deduped.fasta]"],
    sections: &[Section {
        title: "OPTIONS",
        flags: &[FlagSpec {
            short: Some('n'),
            long: "by-name",
            aliases: &[],
            value: None,
            type_hint: None,
            required: false,
            default: None,
            description: "Dedup by name instead of sequence content.",
            why_default: None,
        }],
    }],
    examples: &[Example {
        description: "Remove duplicate sequences",
        command: "rsomics-fasta-dedup input.fa -o unique.fa",
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
