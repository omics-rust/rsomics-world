use std::path::PathBuf;

use clap::Parser;
use rsomics_common::{CommonFlags, Result, RsomicsError, Tool, ToolMeta};
use rsomics_help::{Example, FlagSpec, HelpSpec, Section};

use rsomics_tpm::normalize_tpm;

pub const META: ToolMeta = ToolMeta {
    name: env!("CARGO_PKG_NAME"),
    version: env!("CARGO_PKG_VERSION"),
};

#[derive(Parser, Debug)]
#[command(name = "rsomics-tpm", version, about, long_about = None, disable_help_flag = true)]
pub struct Cli {
    pub counts: PathBuf,
    #[arg(short = 'l', long)]
    lengths: PathBuf,
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
        let n = normalize_tpm(&self.counts, &self.lengths, &mut out)?;
        if !self.common.quiet {
            eprintln!("{n} genes normalized to TPM");
        }
        Ok(())
    }
}

pub static HELP: HelpSpec = HelpSpec {
    name: env!("CARGO_PKG_NAME"),
    version: env!("CARGO_PKG_VERSION"),
    tagline: "TPM normalization of a gene count matrix given gene lengths.",
    origin: None,
    usage_lines: &["<counts.tsv> -l <gene_lengths.tsv> [-o tpm.tsv]"],
    sections: &[Section {
        title: "OPTIONS",
        flags: &[FlagSpec {
            short: Some('l'),
            long: "lengths",
            aliases: &[],
            value: Some("<path>"),
            type_hint: Some("PathBuf"),
            required: true,
            default: None,
            description: "Gene lengths TSV (gene<TAB>length_bp).",
            why_default: None,
        }],
    }],
    examples: &[Example {
        description: "Normalize counts to TPM",
        command: "rsomics-tpm counts.tsv -l gene_lengths.tsv -o tpm.tsv",
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
