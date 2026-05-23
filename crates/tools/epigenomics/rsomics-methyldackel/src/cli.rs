use std::path::PathBuf;

use clap::Parser;
use rsomics_common::{CommonFlags, Result, Tool, ToolMeta};
use rsomics_help::{Example, FlagSpec, HelpSpec, Origin, Section};

use rsomics_methyldackel::{ExtractOpts, run};

pub const META: ToolMeta = ToolMeta {
    name: env!("CARGO_PKG_NAME"),
    version: env!("CARGO_PKG_VERSION"),
};

#[derive(Parser, Debug)]
#[command(
    name = "rsomics-methyldackel",
    version,
    about,
    long_about = None,
    disable_help_flag = true
)]
pub struct Cli {
    /// Reference FASTA file (must be indexed with samtools faidx).
    pub fasta: PathBuf,

    /// Bisulfite-aligned BAM file (coordinate-sorted, indexed).
    pub bam: PathBuf,

    /// Output prefix; bedGraph is written to <PREFIX>_CpG.bedGraph.
    #[arg(short = 'o', long, default_value = "out")]
    pub output: String,

    /// Minimum mapping quality (MethylDackel default 10; -q conflicts with --quiet so use long form).
    #[arg(long = "min-mapq", default_value_t = 10)]
    pub min_mapq: u8,

    /// Minimum base Phred quality (MethylDackel default 5).
    #[arg(long = "min-phred", default_value_t = 5)]
    pub min_phred: u8,

    /// Ignore reads with any of these FLAG bits set (hex or decimal, default 0xF00).
    #[arg(long = "ignore-flags", default_value = "0xF00")]
    pub ignore_flags: String,

    /// Require reads to have all of these FLAG bits set (hex or decimal, default 0).
    #[arg(long = "require-flags", default_value = "0")]
    pub require_flags: String,

    /// Minimum read depth to emit a position (default 1).
    #[arg(short = 'd', long = "min-depth", default_value_t = 1)]
    pub min_depth: u32,

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
        let ignore_flags = parse_flag_hex(&self.ignore_flags)?;
        let require_flags = parse_flag_hex(&self.require_flags)?;

        let opts = ExtractOpts {
            min_mapq: self.min_mapq,
            min_phred: self.min_phred,
            ignore_flags,
            require_flags,
            min_depth: self.min_depth,
            output_prefix: self.output.clone(),
        };

        let workers = std::num::NonZero::new(self.common.thread_count())
            .unwrap_or(std::num::NonZero::<usize>::MIN);

        let stats = run(&self.bam, &self.fasta, opts, workers)?;

        if !self.common.quiet {
            if self.common.json {
                println!(
                    "{}",
                    serde_json::to_string_pretty(&stats)
                        .map_err(|e| rsomics_common::RsomicsError::InvalidInput(e.to_string()))?
                );
            } else {
                eprintln!(
                    "{} CpG positions examined, {} emitted",
                    stats.positions_examined, stats.positions_emitted
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
    result.map_err(|e| {
        rsomics_common::RsomicsError::InvalidInput(format!("invalid flags '{s}': {e}"))
    })
}

pub static HELP: HelpSpec = HelpSpec {
    name: env!("CARGO_PKG_NAME"),
    version: env!("CARGO_PKG_VERSION"),
    tagline: "Per-CpG methylation extraction from bisulfite-aligned BAM (MethylDackel extract port).",
    origin: Some(Origin {
        upstream: "MethylDackel",
        upstream_license: "MIT",
        our_license: "MIT OR Apache-2.0",
        paper_doi: None,
    }),
    usage_lines: &["<ref.fa> <input.bam> -o <prefix> [-q 10] [-p 5]"],
    sections: &[Section {
        title: "OPTIONS",
        flags: &[
            FlagSpec {
                short: Some('o'),
                long: "output",
                aliases: &[],
                value: Some("<prefix>"),
                type_hint: Some("str"),
                required: false,
                default: Some("out"),
                description: "Output prefix; bedGraph written to <prefix>_CpG.bedGraph.",
                why_default: None,
            },
            FlagSpec {
                short: None,
                long: "min-mapq",
                aliases: &[],
                value: Some("<u8>"),
                type_hint: Some("u8"),
                required: false,
                default: Some("10"),
                description: "Minimum mapping quality (MethylDackel default).",
                why_default: None,
            },
            FlagSpec {
                short: None,
                long: "min-phred",
                aliases: &[],
                value: Some("<u8>"),
                type_hint: Some("u8"),
                required: false,
                default: Some("5"),
                description: "Minimum base Phred quality (MethylDackel default).",
                why_default: None,
            },
            FlagSpec {
                short: None,
                long: "ignore-flags",
                aliases: &[],
                value: Some("<hex|int>"),
                type_hint: Some("str"),
                required: false,
                default: Some("0xF00"),
                description: "Ignore reads with any of these FLAG bits (secondary|qcfail|dup|supplementary).",
                why_default: None,
            },
            FlagSpec {
                short: None,
                long: "require-flags",
                aliases: &[],
                value: Some("<hex|int>"),
                type_hint: Some("str"),
                required: false,
                default: Some("0"),
                description: "Require reads to have all these FLAG bits set.",
                why_default: None,
            },
            FlagSpec {
                short: Some('d'),
                long: "min-depth",
                aliases: &[],
                value: Some("<u32>"),
                type_hint: Some("u32"),
                required: false,
                default: Some("1"),
                description: "Minimum read depth to emit a CpG position.",
                why_default: None,
            },
        ],
    }],
    examples: &[
        Example {
            description: "Extract CpG methylation with default filters",
            command: "rsomics-methyldackel ref.fa input.bam -o methylation",
        },
        Example {
            description: "Lower MAPQ threshold (match MethylDackel test fixtures)",
            command: "rsomics-methyldackel ref.fa input.bam -q 2 -o methylation",
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
        assert_eq!(parse_flag_hex("0xF00").unwrap(), 0xF00);
        assert_eq!(parse_flag_hex("0xD00").unwrap(), 0xD00);
        assert_eq!(parse_flag_hex("0").unwrap(), 0);
        assert_eq!(parse_flag_hex("3840").unwrap(), 0xF00);
    }
}
