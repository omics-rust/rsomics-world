use std::path::PathBuf;

use clap::Parser;
use rsomics_common::{CommonFlags, Result, RsomicsError, ToolMeta};
use rsomics_help::{Example, FlagSpec, HelpSpec, Origin, Section};

use rsomics_bed_window::window_bed;

pub const META: ToolMeta = ToolMeta {
    name: env!("CARGO_PKG_NAME"),
    version: env!("CARGO_PKG_VERSION"),
};

#[derive(Parser, Debug)]
#[command(
    name = "rsomics-bed-window",
    version,
    about,
    long_about = None,
    disable_help_flag = true
)]
pub struct Cli {
    /// Input BED file A.
    #[arg(short = 'a')]
    pub a: PathBuf,

    /// Input BED file B.
    #[arg(short = 'b')]
    pub b: PathBuf,

    /// Output file (default stdout).
    #[arg(short = 'o', long = "output", default_value = "-")]
    output: String,

    /// Window size (bp) to extend each A interval.
    #[arg(short = 'w', long = "window", default_value_t = 1000)]
    window: u64,

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

        let count = window_bed(&self.a, &self.b, &mut out, self.window)?;

        if !self.common.quiet {
            eprintln!("{count} overlapping pairs");
        }

        Ok(())
    }
}

pub static HELP: HelpSpec = HelpSpec {
    name: META.name,
    version: META.version,
    tagline: "Find overlapping features within a window.",
    origin: Some(Origin {
        upstream: "bedtools window",
        upstream_license: "MIT",
        our_license: "MIT OR Apache-2.0",
        paper_doi: None,
    }),
    usage_lines: &["-a <A.bed> -b <B.bed> [-w 1000]"],
    sections: &[Section {
        title: "OPTIONS",
        flags: &[FlagSpec {
            short: Some('w'),
            long: "window",
            aliases: &[],
            value: Some("<bp>"),
            type_hint: Some("u64"),
            required: false,
            default: Some("1000"),
            description: "Window size in base pairs.",
            why_default: Some("bedtools default"),
        }],
    }],
    examples: &[Example {
        description: "Find B features within 5kb of A",
        command: "rsomics-bed-window -a peaks.bed -b genes.bed -w 5000",
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
