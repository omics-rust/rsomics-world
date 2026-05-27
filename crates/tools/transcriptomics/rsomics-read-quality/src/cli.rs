use std::num::NonZero;
use std::path::PathBuf;

use clap::Parser;
use rsomics_common::{CommonFlags, Result, ToolMeta};
use rsomics_help::{Example, FlagSpec, HelpSpec, Origin, Section};

use rsomics_read_quality::{ReadQualityOpts, run_read_quality};

pub const META: ToolMeta = ToolMeta {
    name: env!("CARGO_PKG_NAME"),
    version: env!("CARGO_PKG_VERSION"),
};

#[derive(Parser, Debug)]
#[command(
    name = "rsomics-read-quality",
    version,
    about,
    long_about = None,
    disable_help_flag = true
)]
pub struct Cli {
    /// Input BAM file.
    #[arg(short = 'i', long = "input-file", value_name = "BAM")]
    pub input: PathBuf,

    /// Prefix for output files (`<PREFIX>.qual.r`).
    #[arg(short = 'o', long = "out-prefix", value_name = "PREFIX")]
    pub output_prefix: String,

    /// Reduce factor: boxplot `times` values are divided by this.
    /// Set to 1 for maximum precision. Higher values reduce R memory use.
    #[arg(short = 'r', long = "reduce", default_value_t = 1, value_name = "INT")]
    pub reduce: u64,

    /// Minimum mapping quality (Phred scaled) for a read to be included.
    #[arg(long = "mapq", default_value_t = 30, value_name = "INT")]
    pub min_mapq: u8,

    #[command(flatten)]
    pub common: CommonFlags,
}

impl Cli {
    pub fn execute(self) -> Result<()> {
        let workers = NonZero::new(self.common.thread_count()).unwrap_or(NonZero::<usize>::MIN);
        let opts = ReadQualityOpts {
            min_mapq: self.min_mapq,
            reduce: self.reduce,
            workers,
        };
        run_read_quality(&self.input, &self.output_prefix, &opts)
    }
}

pub static HELP: HelpSpec = HelpSpec {
    name: META.name,
    version: META.version,
    tagline: "Per-base read quality heatmap + boxplot from BAM (RSeQC read_quality.py port).",
    origin: Some(Origin {
        upstream: "RSeQC read_quality.py",
        upstream_license: "GPL-2",
        our_license: "MIT OR Apache-2.0",
        paper_doi: Some("10.1093/bioinformatics/bts526"),
    }),
    usage_lines: &["-i <BAM> -o <PREFIX>"],
    sections: &[Section {
        title: "OPTIONS",
        flags: &[
            FlagSpec {
                short: Some('i'),
                long: "input-file",
                aliases: &[],
                value: Some("<path>"),
                type_hint: Some("Path"),
                required: true,
                default: None,
                description: "Input BAM file.",
                why_default: None,
            },
            FlagSpec {
                short: Some('o'),
                long: "out-prefix",
                aliases: &[],
                value: Some("<prefix>"),
                type_hint: Some("String"),
                required: true,
                default: None,
                description: "Output prefix. Writes <prefix>.qual.r.",
                why_default: None,
            },
            FlagSpec {
                short: Some('r'),
                long: "reduce",
                aliases: &[],
                value: Some("<int>"),
                type_hint: Some("u64"),
                required: false,
                default: Some("1"),
                description: "Boxplot reduce divisor. Increase to reduce R memory at cost of precision.",
                why_default: None,
            },
            FlagSpec {
                short: None,
                long: "mapq",
                aliases: &[],
                value: Some("<int>"),
                type_hint: Some("u8"),
                required: false,
                default: Some("30"),
                description: "Minimum MAPQ to include a read.",
                why_default: None,
            },
        ],
    }],
    examples: &[
        Example {
            description: "Basic quality profile",
            command: "rsomics-read-quality -i aligned.bam -o out/prefix",
        },
        Example {
            description: "Lower MAPQ threshold",
            command: "rsomics-read-quality -i aligned.bam -o out/prefix --mapq 20",
        },
        Example {
            description: "Reduce R memory",
            command: "rsomics-read-quality -i aligned.bam -o out/prefix -r 100",
        },
    ],
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
