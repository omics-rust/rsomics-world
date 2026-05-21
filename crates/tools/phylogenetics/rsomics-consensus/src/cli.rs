use clap::Parser;
use rsomics_common::{CommonFlags, Result, RsomicsError, Tool, ToolMeta};
use rsomics_consensus::consensus;
use rsomics_help::{Example, FlagSpec, HelpSpec, Section};
use std::path::PathBuf;

pub const META: ToolMeta = ToolMeta {
    name: env!("CARGO_PKG_NAME"),
    version: env!("CARGO_PKG_VERSION"),
};

#[derive(Parser, Debug)]
#[command(name = "rsomics-consensus", version, about, long_about = None, disable_help_flag = true)]
pub struct Cli {
    pub input: PathBuf,
    #[arg(long, default_value_t = 0.5)]
    threshold: f64,
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
        let len = consensus(&self.input, self.threshold, &mut out)?;
        if !self.common.quiet {
            eprintln!("{len} bp consensus");
        }
        Ok(())
    }
}

pub static HELP: HelpSpec = HelpSpec {
    name: env!("CARGO_PKG_NAME"),
    version: env!("CARGO_PKG_VERSION"),
    tagline: "Consensus sequence from a multiple sequence alignment.",
    origin: None,
    usage_lines: &["<alignment.fa> [-t 0.5] [-o consensus.fa]"],
    sections: &[Section {
        title: "OPTIONS",
        flags: &[FlagSpec {
            short: Some('t'),
            long: "threshold",
            aliases: &[],
            value: Some("<fraction>"),
            type_hint: Some("f64"),
            required: false,
            default: Some("0.5"),
            description: "Minimum base frequency to call consensus (else N).",
            why_default: None,
        }],
    }],
    examples: &[Example {
        description: "Majority-rule consensus",
        command: "rsomics-consensus alignment.fa -t 0.5 -o cons.fa",
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
