use std::path::PathBuf;

use clap::Parser;
use rsomics_common::{CommonFlags, Result, Tool, ToolMeta};
use rsomics_help::{Example, FlagSpec, HelpSpec, Origin, Section};

use rsomics_bam_depth::{DepthOpts, compute_depth};

pub const META: ToolMeta = ToolMeta {
    name: env!("CARGO_PKG_NAME"),
    version: env!("CARGO_PKG_VERSION"),
};

#[derive(Parser, Debug)]
#[command(
    name = "rsomics-bam-depth",
    version,
    about,
    long_about = None,
    disable_help_flag = true
)]
pub struct Cli {
    /// Input BAM file.
    pub input: PathBuf,

    /// Output file (default stdout).
    #[arg(short = 'o', long = "output", default_value = "-")]
    output: String,

    /// Minimum mapping quality.
    #[arg(long = "min-mapq", default_value_t = 0)]
    min_mapq: u8,

    /// Maximum depth to report per position.
    #[arg(long = "max-depth", default_value_t = 8000)]
    max_depth: u32,

    #[command(flatten)]
    pub common: CommonFlags,
}

impl Cli {
    pub fn execute(self) -> Result<()> {
        let opts = DepthOpts {
            min_mapq: self.min_mapq,
            max_depth: self.max_depth,
            ..Default::default()
        };

        let mut out: Box<dyn std::io::Write> = if self.output == "-" {
            Box::new(std::io::stdout().lock())
        } else {
            Box::new(std::fs::File::create(&self.output).map_err(rsomics_common::RsomicsError::Io)?)
        };

        let lines = compute_depth(&self.input, &mut out, &opts)?;

        if self.common.json {
            let j = serde_json::json!({ "lines": lines });
            eprintln!("{j}");
        }

        Ok(())
    }
}

impl Tool for Cli {
    fn meta() -> ToolMeta {
        META
    }

    fn common(&self) -> &CommonFlags {
        &self.common
    }

    fn execute(self) -> Result<()> {
        self.execute()
    }
}

pub static HELP: HelpSpec = HelpSpec {
    name: META.name,
    version: META.version,
    tagline: "Per-base depth from BAM alignments.",
    origin: Some(Origin {
        upstream: "samtools depth",
        upstream_license: "MIT",
        our_license: "MIT OR Apache-2.0",
        paper_doi: Some("10.1093/bioinformatics/btp352"),
    }),
    usage_lines: &["<input.bam> [-o out.tsv] [--min-mapq N]"],
    sections: &[Section {
        title: "OPTIONS",
        flags: &[
            FlagSpec {
                short: None,
                long: "min-mapq",
                aliases: &[],
                value: Some("<INT>"),
                type_hint: Some("u8"),
                required: false,
                default: Some("0"),
                description: "Minimum mapping quality.",
                why_default: None,
            },
            FlagSpec {
                short: None,
                long: "max-depth",
                aliases: &[],
                value: Some("<INT>"),
                type_hint: Some("u32"),
                required: false,
                default: Some("8000"),
                description: "Maximum depth to report per position.",
                why_default: Some("samtools default"),
            },
        ],
    }],
    examples: &[Example {
        description: "Compute per-base depth",
        command: "rsomics-bam-depth input.bam -o depth.tsv",
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
