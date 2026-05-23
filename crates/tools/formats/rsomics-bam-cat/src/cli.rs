use std::path::PathBuf;

use clap::Parser;
use rsomics_common::{CommonFlags, Result, RsomicsError, Tool, ToolMeta};
use rsomics_help::{Example, FlagSpec, HelpSpec, Origin, Section};

use rsomics_bam_cat::{CatOpts, cat};

pub const META: ToolMeta = ToolMeta {
    name: env!("CARGO_PKG_NAME"),
    version: env!("CARGO_PKG_VERSION"),
};

#[derive(Parser, Debug)]
#[command(
    name = "rsomics-bam-cat",
    version,
    about,
    long_about = None,
    disable_help_flag = true
)]
pub struct Cli {
    /// Input BAM files, concatenated in order.
    #[arg(required = true)]
    pub inputs: Vec<PathBuf>,

    /// Output BAM file (default stdout).
    #[arg(short = 'o', long = "output", default_value = "-")]
    output: String,

    /// Use the header from this BAM file instead of the first input's.
    /// (samtools cat spells this `-h`; `-h` is reserved for help here, so the
    /// option is long-only.)
    #[arg(long = "header")]
    header: Option<PathBuf>,

    /// Omit the @PG provenance line.
    #[arg(short = 'P', long = "no-PG")]
    no_pg: bool,

    #[command(flatten)]
    pub common: CommonFlags,
}

impl Cli {
    pub fn execute(self) -> Result<()> {
        let opts = CatOpts {
            header_file: self.header,
            no_pg: self.no_pg,
        };

        let output_path = (self.output != "-").then(|| PathBuf::from(&self.output));
        let stats = cat(&self.inputs, output_path.as_deref(), &opts)?;

        if self.common.json {
            eprintln!(
                "{}",
                serde_json::to_string(&stats)
                    .map_err(|e| RsomicsError::InvalidInput(format!("JSON: {e}")))?
            );
        } else if !self.common.quiet {
            eprintln!("{} BAM files concatenated", stats.inputs);
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
    tagline: "Concatenate BAM files by copying compressed BGZF blocks verbatim.",
    origin: Some(Origin {
        upstream: "samtools cat",
        upstream_license: "MIT",
        our_license: "MIT OR Apache-2.0",
        paper_doi: None,
    }),
    usage_lines: &["<in1.bam> <in2.bam> ... [-o out.bam] [--header hdr.bam] [-P]"],
    sections: &[Section {
        title: "OPTIONS",
        flags: &[
            FlagSpec {
                short: Some('o'),
                long: "output",
                aliases: &[],
                value: Some("FILE"),
                type_hint: None,
                required: false,
                default: Some("stdout"),
                description: "Output BAM file.",
                why_default: None,
            },
            FlagSpec {
                short: None,
                long: "header",
                aliases: &[],
                value: Some("FILE"),
                type_hint: None,
                required: false,
                default: None,
                description: "Use this file's header instead of the first input's.",
                why_default: None,
            },
            FlagSpec {
                short: Some('P'),
                long: "no-PG",
                aliases: &[],
                value: None,
                type_hint: None,
                required: false,
                default: None,
                description: "Omit the @PG provenance line.",
                why_default: None,
            },
        ],
    }],
    examples: &[
        Example {
            description: "Concatenate two BAM shards",
            command: "rsomics-bam-cat part1.bam part2.bam -o all.bam",
        },
        Example {
            description: "Concatenate using an external header",
            command: "rsomics-bam-cat -h hdr.bam part1.bam part2.bam -o all.bam",
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
