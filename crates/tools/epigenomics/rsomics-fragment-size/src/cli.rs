use std::path::PathBuf;

use clap::Parser;
use rsomics_common::{CommonFlags, Result, RsomicsError, Tool, ToolMeta};
use rsomics_help::{Example, FlagSpec, HelpSpec, Origin, Section};

use rsomics_fragment_size::{SizeOpts, compute, write_histogram};

pub const META: ToolMeta = ToolMeta {
    name: env!("CARGO_PKG_NAME"),
    version: env!("CARGO_PKG_VERSION"),
};

#[derive(Parser, Debug)]
#[command(
    name = "rsomics-fragment-size",
    version,
    about,
    long_about = None,
    disable_help_flag = true
)]
pub struct Cli {
    /// Input BAM file (coordinate-sorted; must be paired-end).
    pub bam: PathBuf,

    /// Output histogram TSV file (use `-` for stdout).
    #[arg(short = 'o', long, default_value = "-")]
    pub output: String,

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
        let opts = SizeOpts {
            min_mapq: self.min_mapq,
            skip_flags,
        };
        let workers = std::num::NonZero::new(self.common.thread_count())
            .unwrap_or(std::num::NonZero::<usize>::MIN);

        let (hist, summary) = compute(&self.bam, &opts, workers)?;

        if self.common.json {
            println!(
                "{}",
                serde_json::to_string_pretty(&summary)
                    .map_err(|e| RsomicsError::InvalidInput(e.to_string()))?
            );
            return Ok(());
        }

        let mut out: Box<dyn std::io::Write> = if self.output == "-" {
            Box::new(std::io::stdout().lock())
        } else {
            Box::new(std::fs::File::create(&self.output).map_err(RsomicsError::Io)?)
        };

        write_histogram(&mut out, &hist)?;

        if !self.common.quiet {
            eprintln!(
                "fragments: {}  mean: {:.1}  median: {}  mode: {}",
                summary.total_fragments, summary.mean, summary.median, summary.mode
            );
            eprintln!(
                "NFR (<100 bp): {:.3}  mono (180-247): {:.3}  di (315-473): {:.3}",
                summary.nfr_fraction, summary.mono_fraction, summary.di_fraction
            );
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
    tagline: "Paired-end insert-size histogram from a BAM with ATAC nucleosome fractions.",
    origin: Some(Origin {
        upstream: "samtools stats (IS) / picard CollectInsertSizeMetrics",
        upstream_license: "MIT",
        our_license: "MIT OR Apache-2.0",
        paper_doi: None,
    }),
    usage_lines: &["<input.bam> [-o histogram.tsv] [--min-mapq 20]"],
    sections: &[Section {
        title: "OPTIONS",
        flags: &[
            FlagSpec {
                short: Some('o'),
                long: "output",
                aliases: &[],
                value: Some("<path|->"),
                type_hint: Some("path"),
                required: false,
                default: Some("-"),
                description: "Output histogram TSV file (- = stdout).",
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
            description: "Print histogram to stdout",
            command: "rsomics-fragment-size in.bam",
        },
        Example {
            description: "Write histogram to file and print summary",
            command: "rsomics-fragment-size in.bam -o frag_hist.tsv",
        },
        Example {
            description: "JSON summary output",
            command: "rsomics-fragment-size in.bam --json",
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
