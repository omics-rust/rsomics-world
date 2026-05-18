use std::path::PathBuf;

use clap::Parser;
use rsomics_common::{CommonFlags, Result, RsomicsError, ToolMeta};
use rsomics_help::{Example, FlagSpec, HelpSpec, Origin, Section};

use rsomics_bed_sample::sample_bed;

pub const META: ToolMeta = ToolMeta {
    name: env!("CARGO_PKG_NAME"),
    version: env!("CARGO_PKG_VERSION"),
};

#[derive(Parser, Debug)]
#[command(
    name = "rsomics-bed-sample",
    version,
    about,
    long_about = None,
    disable_help_flag = true
)]
pub struct Cli {
    pub input: PathBuf,
    #[arg(short = 'n', long = "count", default_value_t = 1000)]
    count: usize,
    #[arg(short = 'o', long = "output", default_value = "-")]
    output: String,
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
        let sampled = sample_bed(&self.input, &mut out, self.count, self.common.seed_rng())?;
        if !self.common.quiet {
            eprintln!("{sampled} intervals sampled");
        }
        Ok(())
    }
}

pub static HELP: HelpSpec = HelpSpec {
    name: META.name,
    version: META.version,
    tagline: "Randomly sample intervals from a BED file.",
    origin: Some(Origin {
        upstream: "bedtools sample / shuf",
        upstream_license: "MIT",
        our_license: "MIT OR Apache-2.0",
        paper_doi: None,
    }),
    usage_lines: &["<input.bed> [-n 1000] [--seed 42] [-o sampled.bed]"],
    sections: &[Section {
        title: "OPTIONS",
        flags: &[FlagSpec {
            short: Some('n'),
            long: "count",
            aliases: &[],
            value: Some("<N>"),
            type_hint: Some("usize"),
            required: false,
            default: Some("1000"),
            description: "Number of intervals to sample.",
            why_default: None,
        }],
    }],
    examples: &[Example {
        description: "Sample 500 intervals",
        command: "rsomics-bed-sample peaks.bed -n 500",
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
