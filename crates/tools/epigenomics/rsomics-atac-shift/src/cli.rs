use std::path::PathBuf;

use clap::Parser;
use rsomics_common::{CommonFlags, Result, RsomicsError, Tool, ToolMeta};
use rsomics_help::{Example, FlagSpec, HelpSpec, Origin, Section};

use rsomics_atac_shift::{OutputMode, ShiftOpts, run};

pub const META: ToolMeta = ToolMeta {
    name: env!("CARGO_PKG_NAME"),
    version: env!("CARGO_PKG_VERSION"),
};

#[derive(Parser, Debug)]
#[command(
    name = "rsomics-atac-shift",
    version,
    about,
    long_about = None,
    disable_help_flag = true
)]
pub struct Cli {
    /// Input BAM file (coordinate-sorted, indexed).
    pub bam: PathBuf,

    /// Output file (BAM default; BED when --bed is set).
    #[arg(short = 'o', long)]
    pub output: PathBuf,

    /// Write insertion-site BED instead of shifted BAM.
    #[arg(long = "bed", default_value_t = false)]
    pub bed: bool,

    /// Minimum mapping quality.
    #[arg(long = "min-mapq", default_value_t = 0)]
    pub min_mapq: u8,

    /// Skip reads with any of these FLAG bits set (hex or decimal).
    #[arg(long = "skip-flags", default_value = "0")]
    pub skip_flags: String,

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
        let skip_flags = parse_flag_hex(&self.skip_flags)?;
        let opts = ShiftOpts {
            output_mode: if self.bed {
                OutputMode::Bed
            } else {
                OutputMode::Bam
            },
            min_mapq: self.min_mapq,
            skip_flags,
        };
        let workers = std::num::NonZero::new(self.common.thread_count())
            .unwrap_or(std::num::NonZero::<usize>::MIN);
        let stats = run(&self.bam, &self.output, &opts, workers)?;
        if !self.common.quiet {
            if self.common.json {
                println!(
                    "{}",
                    serde_json::to_string_pretty(&stats)
                        .map_err(|e| RsomicsError::InvalidInput(e.to_string()))?
                );
            } else {
                eprintln!(
                    "{} records read, {} written, {} skipped",
                    stats.records_read, stats.records_written, stats.records_skipped
                );
            }
        }
        Ok(())
    }
}

fn parse_flag_hex(s: &str) -> Result<u16> {
    let trimmed = s.trim();
    let result = if let Some(hex) = trimmed
        .strip_prefix("0x")
        .or_else(|| trimmed.strip_prefix("0X"))
    {
        u16::from_str_radix(hex, 16)
    } else {
        trimmed.parse::<u16>()
    };
    result.map_err(|e| RsomicsError::InvalidInput(format!("invalid --skip-flags '{s}': {e}")))
}

pub static HELP: HelpSpec = HelpSpec {
    name: env!("CARGO_PKG_NAME"),
    version: env!("CARGO_PKG_VERSION"),
    tagline: "ATAC-seq Tn5 insertion-bias correction (deeptools alignmentSieve --ATACshift port).",
    origin: Some(Origin {
        upstream: "deeptools alignmentSieve",
        upstream_license: "MIT",
        our_license: "MIT OR Apache-2.0",
        paper_doi: Some("10.1038/nmeth.2688"),
    }),
    usage_lines: &["<input.bam> -o <output.bam|output.bed> [--bed] [--min-mapq 20]"],
    sections: &[Section {
        title: "OPTIONS",
        flags: &[
            FlagSpec {
                short: Some('o'),
                long: "output",
                aliases: &[],
                value: Some("<path>"),
                type_hint: Some("path"),
                required: true,
                default: None,
                description: "Output BAM file (or BED when --bed is set).",
                why_default: None,
            },
            FlagSpec {
                short: None,
                long: "bed",
                aliases: &[],
                value: None,
                type_hint: Some("flag"),
                required: false,
                default: Some("false"),
                description: "Emit insertion-site BED instead of shifted BAM.",
                why_default: None,
            },
            FlagSpec {
                short: None,
                long: "min-mapq",
                aliases: &[],
                value: Some("<u8>"),
                type_hint: Some("u8"),
                required: false,
                default: Some("0"),
                description: "Minimum mapping quality.",
                why_default: None,
            },
            FlagSpec {
                short: None,
                long: "skip-flags",
                aliases: &[],
                value: Some("<hex|int>"),
                type_hint: Some("str"),
                required: false,
                default: Some("0"),
                description: "Skip reads with these FLAG bits. Use 0x400 for duplicates.",
                why_default: None,
            },
        ],
    }],
    examples: &[
        Example {
            description: "Shift ATAC reads and write BAM",
            command: "rsomics-atac-shift in.bam -o shifted.bam",
        },
        Example {
            description: "Write Tn5 insertion sites as BED",
            command: "rsomics-atac-shift in.bam -o insertions.bed --bed",
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

    #[test]
    fn parse_flag_hex_ok() {
        assert_eq!(parse_flag_hex("0x400").unwrap(), 0x400);
        assert_eq!(parse_flag_hex("1024").unwrap(), 1024);
        assert_eq!(parse_flag_hex("0").unwrap(), 0);
    }
}
