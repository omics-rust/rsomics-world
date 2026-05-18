use std::path::PathBuf;

use clap::Parser;
use rsomics_common::{CommonFlags, Result, RsomicsError, ToolMeta};
use rsomics_help::{Example, FlagSpec, HelpSpec, Origin, Section};

use rsomics_fasta_window::window_stats;

pub const META: ToolMeta = ToolMeta {
    name: env!("CARGO_PKG_NAME"),
    version: env!("CARGO_PKG_VERSION"),
};

#[derive(Parser, Debug)]
#[command(
    name = "rsomics-fasta-window",
    version,
    about,
    long_about = None,
    disable_help_flag = true
)]
pub struct Cli {
    /// Input FASTA file.
    pub input: PathBuf,

    /// Output file (default stdout).
    #[arg(short = 'o', long = "output", default_value = "-")]
    output: String,

    /// Window size in bases.
    #[arg(short = 'w', long = "window", default_value_t = 10000)]
    window: usize,

    /// Step size in bases.
    #[arg(short = 's', long = "step", default_value_t = 5000)]
    step: usize,

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

        let count = window_stats(&self.input, &mut out, self.window, self.step)?;

        if !self.common.quiet {
            eprintln!("{count} windows");
        }

        Ok(())
    }
}

pub static HELP: HelpSpec = HelpSpec {
    name: META.name,
    version: META.version,
    tagline: "Sliding-window GC% and length statistics over FASTA.",
    origin: Some(Origin {
        upstream: "bedtools nuc / seqkit sliding",
        upstream_license: "MIT",
        our_license: "MIT OR Apache-2.0",
        paper_doi: None,
    }),
    usage_lines: &["<input.fasta> [-w 10000] [-s 5000] [-o windows.tsv]"],
    sections: &[Section {
        title: "OPTIONS",
        flags: &[
            FlagSpec {
                short: Some('w'),
                long: "window",
                aliases: &[],
                value: Some("<bp>"),
                type_hint: Some("usize"),
                required: false,
                default: Some("10000"),
                description: "Window size in bases.",
                why_default: None,
            },
            FlagSpec {
                short: Some('s'),
                long: "step",
                aliases: &[],
                value: Some("<bp>"),
                type_hint: Some("usize"),
                required: false,
                default: Some("5000"),
                description: "Step size in bases.",
                why_default: None,
            },
        ],
    }],
    examples: &[Example {
        description: "10kb windows with 5kb step",
        command: "rsomics-fasta-window genome.fasta -w 10000 -s 5000",
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
