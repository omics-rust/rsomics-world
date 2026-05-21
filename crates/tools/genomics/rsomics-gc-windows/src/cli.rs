use std::path::PathBuf;

use clap::Parser;
use rsomics_common::{CommonFlags, Result, RsomicsError, Tool, ToolMeta};
use rsomics_help::{Example, FlagSpec, HelpSpec, Origin, Section};

use rsomics_gc_windows::gc_windows;

pub const META: ToolMeta = ToolMeta {
    name: env!("CARGO_PKG_NAME"),
    version: env!("CARGO_PKG_VERSION"),
};

#[derive(Parser, Debug)]
#[command(
    name = "rsomics-gc-windows",
    version,
    about,
    long_about = None,
    disable_help_flag = true
)]
pub struct Cli {
    pub input: PathBuf,
    #[arg(short = 'w', long, default_value_t = 100)]
    window: usize,
    #[arg(short = 's', long, default_value_t = 0)]
    step: usize,
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
        let n = gc_windows(&self.input, self.window, self.step, &mut out)?;
        if !self.common.quiet {
            eprintln!("{n} windows");
        }
        Ok(())
    }
}

pub static HELP: HelpSpec = HelpSpec {
    name: env!("CARGO_PKG_NAME"),
    version: env!("CARGO_PKG_VERSION"),
    tagline: "Compute per-window GC content across a FASTA reference.",
    origin: Some(Origin {
        upstream: "bedtools nuc / picard CollectGcBiasMetrics",
        upstream_license: "MIT / MIT",
        our_license: "MIT OR Apache-2.0",
        paper_doi: None,
    }),
    usage_lines: &["<ref.fa> [-w 100] [-s 50] [-o gc.bed]"],
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
                default: Some("100"),
                description: "Window size in base pairs.",
                why_default: None,
            },
            FlagSpec {
                short: Some('s'),
                long: "step",
                aliases: &[],
                value: Some("<bp>"),
                type_hint: Some("usize"),
                required: false,
                default: Some("window"),
                description: "Step size (0 = non-overlapping = window).",
                why_default: None,
            },
        ],
    }],
    examples: &[Example {
        description: "100bp non-overlapping windows",
        command: "rsomics-gc-windows ref.fa -w 100 -o gc.bed",
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
