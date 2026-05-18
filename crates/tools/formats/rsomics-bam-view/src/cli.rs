use std::path::PathBuf;

use clap::Parser;
use rsomics_common::{CommonFlags, Result, ToolMeta};
use rsomics_help::{Example, FlagSpec, HelpSpec, Origin, Section};

use rsomics_bam_view::{view_bam, ViewFilter};

pub const META: ToolMeta = ToolMeta {
    name: env!("CARGO_PKG_NAME"),
    version: env!("CARGO_PKG_VERSION"),
};

#[derive(Parser, Debug)]
#[command(
    name = "rsomics-bam-view",
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

    /// Only include reads with all of these FLAG bits set.
    #[arg(short = 'f', long = "require-flags", default_value_t = 0)]
    require_flags: u16,

    /// Exclude reads with any of these FLAG bits set.
    #[arg(short = 'F', long = "exclude-flags", default_value_t = 0)]
    exclude_flags: u16,

    /// Minimum MAPQ.
    #[arg(long = "min-mapq", default_value_t = 0)]
    min_mapq: u8,

    /// Only print count of matching records.
    #[arg(short = 'c', long = "count")]
    count_only: bool,

    /// Omit header in SAM output.
    #[arg(long = "no-header")]
    no_header: bool,

    /// Output BAM instead of SAM.
    #[arg(short = 'b', long = "bam")]
    bam_output: bool,

    #[command(flatten)]
    pub common: CommonFlags,
}

impl Cli {
    pub fn execute(self) -> Result<()> {
        let filter = ViewFilter {
            require_flags: self.require_flags,
            exclude_flags: self.exclude_flags,
            min_mapq: self.min_mapq,
            count_only: self.count_only,
            with_header: !self.no_header,
            output_bam: self.bam_output,
        };

        let mut out: Box<dyn std::io::Write> = if self.output == "-" {
            Box::new(std::io::stdout().lock())
        } else {
            Box::new(
                std::fs::File::create(&self.output)
                    .map_err(rsomics_common::RsomicsError::Io)?,
            )
        };

        let count = view_bam(&self.input, &mut out, &filter)?;

        if self.count_only {
            writeln!(out, "{count}").map_err(rsomics_common::RsomicsError::Io)?;
        }

        if self.common.json {
            let j = serde_json::json!({ "count": count });
            eprintln!("{j}");
        }

        Ok(())
    }
}

use std::io::Write;

pub static HELP: HelpSpec = HelpSpec {
    name: META.name,
    version: META.version,
    tagline: "View, filter, and convert SAM/BAM/CRAM alignments.",
    origin: Some(Origin {
        upstream: "samtools view",
        upstream_license: "MIT",
        our_license: "MIT OR Apache-2.0",
        paper_doi: Some("10.1093/bioinformatics/btp352"),
    }),
    usage_lines: &["<input.bam> [-o out.sam] [-f FLAGS] [-F FLAGS] [-b] [-c]"],
    sections: &[Section {
        title: "OPTIONS",
        flags: &[
            FlagSpec {
                short: Some("-f"),
                long: "require-flags",
                aliases: &[],
                value: Some("<INT>"),
                type_hint: Some("u16"),
                required: false,
                default: Some("0"),
                description: "Only output reads with all FLAG bits set.",
                why_default: None,
            },
            FlagSpec {
                short: Some("-F"),
                long: "exclude-flags",
                aliases: &[],
                value: Some("<INT>"),
                type_hint: Some("u16"),
                required: false,
                default: Some("0"),
                description: "Exclude reads with any FLAG bits set.",
                why_default: None,
            },
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
                short: Some("-b"),
                long: "bam",
                aliases: &[],
                value: None,
                type_hint: None,
                required: false,
                default: None,
                description: "Output BAM instead of SAM.",
                why_default: None,
            },
            FlagSpec {
                short: Some("-c"),
                long: "count",
                aliases: &[],
                value: None,
                type_hint: None,
                required: false,
                default: None,
                description: "Only print count of matching records.",
                why_default: None,
            },
        ],
    }],
    examples: &[
        Example {
            description: "View BAM as SAM",
            command: "rsomics-bam-view input.bam",
        },
        Example {
            description: "Filter to properly paired reads",
            command: "rsomics-bam-view -f 2 input.bam -o filtered.sam",
        },
        Example {
            description: "Count unmapped reads",
            command: "rsomics-bam-view -f 4 -c input.bam",
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
