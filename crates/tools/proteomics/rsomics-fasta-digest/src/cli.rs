use clap::Parser;
use rsomics_common::{CommonFlags, Result, RsomicsError, Tool, ToolMeta};
use rsomics_fasta_digest::{Enzyme, digest};
use rsomics_help::{Example, FlagSpec, HelpSpec, Section};
use std::path::PathBuf;

pub const META: ToolMeta = ToolMeta {
    name: env!("CARGO_PKG_NAME"),
    version: env!("CARGO_PKG_VERSION"),
};

#[derive(Parser, Debug)]
#[command(name = "rsomics-fasta-digest", version, about, long_about = None, disable_help_flag = true)]
pub struct Cli {
    pub input: PathBuf,
    #[arg(short = 'e', long, default_value = "trypsin")]
    enzyme: String,
    #[arg(short = 'm', long, default_value_t = 0)]
    missed_cleavages: usize,
    #[arg(long, default_value_t = 6)]
    min_len: usize,
    #[arg(long, default_value_t = 50)]
    max_len: usize,
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
        let enzyme = match self.enzyme.as_str() {
            "trypsin" => Enzyme::Trypsin,
            "lysc" => Enzyme::LysC,
            "chymotrypsin" => Enzyme::Chymotrypsin,
            other => {
                return Err(RsomicsError::InvalidInput(format!(
                    "unknown enzyme '{other}': use trypsin/lysc/chymotrypsin"
                )));
            }
        };
        let mut out: Box<dyn std::io::Write> = if self.output == "-" {
            Box::new(std::io::stdout().lock())
        } else {
            Box::new(std::fs::File::create(&self.output).map_err(RsomicsError::Io)?)
        };
        let n = digest(
            &self.input,
            &enzyme,
            self.missed_cleavages,
            self.min_len,
            self.max_len,
            &mut out,
        )?;
        if !self.common.quiet {
            eprintln!("{n} peptides");
        }
        Ok(())
    }
}

pub static HELP: HelpSpec = HelpSpec {
    name: env!("CARGO_PKG_NAME"),
    version: env!("CARGO_PKG_VERSION"),
    tagline: "In-silico protein digestion — enzyme cleavage + peptide filtering.",
    origin: None,
    usage_lines: &["<proteins.fa> [-e trypsin] [-m 1] [--min-len 6] [--max-len 50]"],
    sections: &[Section {
        title: "OPTIONS",
        flags: &[
            FlagSpec {
                short: Some('e'),
                long: "enzyme",
                aliases: &[],
                value: Some("<name>"),
                type_hint: Some("String"),
                required: false,
                default: Some("trypsin"),
                description: "Enzyme: trypsin, lysc, chymotrypsin.",
                why_default: None,
            },
            FlagSpec {
                short: Some('m'),
                long: "missed-cleavages",
                aliases: &[],
                value: Some("<int>"),
                type_hint: Some("usize"),
                required: false,
                default: Some("0"),
                description: "Max missed cleavages.",
                why_default: None,
            },
        ],
    }],
    examples: &[Example {
        description: "Tryptic digest with 1 missed cleavage",
        command: "rsomics-fasta-digest proteins.fa -e trypsin -m 1 -o peptides.tsv",
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
