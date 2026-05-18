use std::path::PathBuf;

use clap::Parser;
use rsomics_common::{CommonFlags, Result, RsomicsError, ToolMeta};
use rsomics_help::{Example, FlagSpec, HelpSpec, Origin, Section};

use rsomics_bed_map::map_bed;

pub const META: ToolMeta = ToolMeta {
    name: env!("CARGO_PKG_NAME"),
    version: env!("CARGO_PKG_VERSION"),
};

#[derive(Parser, Debug)]
#[command(name = "rsomics-bed-map", version, about, long_about = None, disable_help_flag = true)]
pub struct Cli {
    #[arg(short = 'a')]
    pub a: PathBuf,
    #[arg(short = 'b')]
    pub b: PathBuf,
    #[arg(short = 'o', long = "output", default_value = "-")]
    output: String,
    #[arg(long = "op", default_value = "mean")]
    op: String,
    #[arg(short = 'c', long = "column", default_value_t = 5)]
    column: usize,
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
        let count = map_bed(&self.a, &self.b, &mut out, &self.op, self.column)?;
        if !self.common.quiet {
            eprintln!("{count} intervals mapped");
        }
        Ok(())
    }
}

pub static HELP: HelpSpec = HelpSpec {
    name: META.name,
    version: META.version,
    tagline: "Map/aggregate values from overlapping BED intervals.",
    origin: Some(Origin {
        upstream: "bedtools map",
        upstream_license: "MIT",
        our_license: "MIT OR Apache-2.0",
        paper_doi: None,
    }),
    usage_lines: &["-a <A.bed> -b <B.bed> [--op mean] [-c 5]"],
    sections: &[Section {
        title: "OPTIONS",
        flags: &[
            FlagSpec {
                short: None,
                long: "op",
                aliases: &[],
                value: Some("<op>"),
                type_hint: Some("String"),
                required: false,
                default: Some("mean"),
                description: "Aggregation: sum, mean, min, max, count.",
                why_default: None,
            },
            FlagSpec {
                short: Some('c'),
                long: "column",
                aliases: &[],
                value: Some("<N>"),
                type_hint: Some("usize"),
                required: false,
                default: Some("5"),
                description: "B column to aggregate (1-based).",
                why_default: Some("score column"),
            },
        ],
    }],
    examples: &[Example {
        description: "Mean score of overlapping features",
        command: "rsomics-bed-map -a regions.bed -b scores.bed --op mean",
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
