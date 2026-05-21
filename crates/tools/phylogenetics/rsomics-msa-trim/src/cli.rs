use clap::Parser;
use rsomics_common::{CommonFlags, Result, RsomicsError, Tool, ToolMeta};
use rsomics_help::{Example, FlagSpec, HelpSpec, Origin, Section};
use rsomics_msa_trim::trim_msa;
use std::path::PathBuf;

pub const META: ToolMeta = ToolMeta {
    name: env!("CARGO_PKG_NAME"),
    version: env!("CARGO_PKG_VERSION"),
};

#[derive(Parser, Debug)]
#[command(name = "rsomics-msa-trim", version, about, long_about = None, disable_help_flag = true)]
pub struct Cli {
    pub input: PathBuf,
    #[arg(short = 'g', long, default_value_t = 0.5)]
    max_gap: f64,
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
        let (orig, kept) = trim_msa(&self.input, self.max_gap, &mut out)?;
        if !self.common.quiet {
            eprintln!("{orig} → {kept} columns ({} removed)", orig - kept);
        }
        Ok(())
    }
}

pub static HELP: HelpSpec = HelpSpec {
    name: env!("CARGO_PKG_NAME"),
    version: env!("CARGO_PKG_VERSION"),
    tagline: "Trim MSA columns by gap fraction — Rust replacement for trimAl -gt.",
    origin: Some(Origin {
        upstream: "trimAl",
        upstream_license: "GPL-3",
        our_license: "MIT OR Apache-2.0",
        paper_doi: Some("10.1093/bioinformatics/btp348"),
    }),
    usage_lines: &["<alignment.fa> [-g 0.5] [-o trimmed.fa]"],
    sections: &[Section {
        title: "OPTIONS",
        flags: &[FlagSpec {
            short: Some('g'),
            long: "max-gap",
            aliases: &[],
            value: Some("<fraction>"),
            type_hint: Some("f64"),
            required: false,
            default: Some("0.5"),
            description: "Maximum gap fraction per column (0.0–1.0).",
            why_default: None,
        }],
    }],
    examples: &[Example {
        description: "Remove columns with >50% gaps",
        command: "rsomics-msa-trim alignment.fa -g 0.5 -o trimmed.fa",
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
