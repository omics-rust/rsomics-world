use std::path::{Path, PathBuf};

use clap::Parser;
use rsomics_common::{CommonFlags, Result, RsomicsError, Tool, ToolMeta};
use rsomics_help::{Example, FlagSpec, HelpSpec, Origin, Section};

use rsomics_bam_collate::{CollateOpts, collate};

pub const META: ToolMeta = ToolMeta {
    name: env!("CARGO_PKG_NAME"),
    version: env!("CARGO_PKG_VERSION"),
};

#[derive(Parser, Debug)]
#[command(
    name = "rsomics-bam-collate",
    version,
    about,
    long_about = None,
    disable_help_flag = true
)]
pub struct Cli {
    /// Input BAM file.
    pub input: PathBuf,

    /// Output BAM file (default stdout).
    #[arg(short = 'o', long = "output", default_value = "-")]
    output: String,

    /// Write uncompressed BAM output.
    #[arg(short = 'u', long = "uncompressed")]
    uncompressed: bool,

    #[command(flatten)]
    pub common: CommonFlags,
}

impl Cli {
    pub fn execute(self) -> Result<()> {
        let opts = CollateOpts {
            uncompressed: self.uncompressed,
        };

        let output_path: Option<&Path> = if self.output == "-" {
            None
        } else {
            Some(Path::new(&self.output))
        };

        let workers = std::num::NonZero::new(self.common.thread_count())
            .unwrap_or(std::num::NonZero::<usize>::MIN);
        let total = collate(&self.input, output_path, &opts, workers)?;

        if self.common.json {
            eprintln!(
                "{}",
                serde_json::to_string(&serde_json::json!({ "total": total }))
                    .map_err(|e| RsomicsError::InvalidInput(format!("JSON: {e}")))?
            );
        } else {
            eprintln!("{total} records collated");
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
    tagline: "Group BAM reads by QNAME so mates are adjacent (not coordinate-sorted).",
    origin: Some(Origin {
        upstream: "samtools collate",
        upstream_license: "MIT",
        our_license: "MIT OR Apache-2.0",
        paper_doi: Some("10.1093/bioinformatics/btp352"),
    }),
    usage_lines: &["<input.bam> [-o output.bam] [-u]"],
    sections: &[Section {
        title: "OPTIONS",
        flags: &[
            FlagSpec {
                short: Some('o'),
                long: "output",
                aliases: &[],
                value: Some("<path>"),
                type_hint: Some("Path"),
                required: false,
                default: Some("-"),
                description: "Output BAM file (default stdout).",
                why_default: None,
            },
            FlagSpec {
                short: Some('u'),
                long: "uncompressed",
                aliases: &[],
                value: None,
                type_hint: Some("bool"),
                required: false,
                default: Some("false"),
                description: "Write uncompressed BAM output.",
                why_default: None,
            },
        ],
    }],
    examples: &[
        Example {
            description: "Collate by QNAME into a file",
            command: "rsomics-bam-collate input.bam -o collated.bam",
        },
        Example {
            description: "Collate, uncompressed, piped into fixmate",
            command: "rsomics-bam-collate input.bam -u -o - | samtools fixmate - out.bam",
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
