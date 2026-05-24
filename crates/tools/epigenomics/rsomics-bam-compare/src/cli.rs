use std::path::PathBuf;

use clap::Parser;
use rsomics_common::{CommonFlags, Result, RsomicsError, Tool, ToolMeta};
use rsomics_help::{Example, FlagSpec, HelpSpec, Origin, Section};

use rsomics_bam_compare::{CompareOpts, Operation, OutputFormat, bam_compare, bam_compare_bigwig};

pub const META: ToolMeta = ToolMeta {
    name: env!("CARGO_PKG_NAME"),
    version: env!("CARGO_PKG_VERSION"),
};

#[derive(Parser, Debug)]
#[command(
    name = "rsomics-bam-compare",
    version,
    about,
    long_about = None,
    disable_help_flag = true
)]
pub struct Cli {
    /// Treatment / numerator BAM (-b1).
    #[arg(short = '1', long = "bam1")]
    pub bam1: PathBuf,

    /// Control / denominator BAM (-b2).
    #[arg(short = '2', long = "bam2")]
    pub bam2: PathBuf,

    /// Output file (use `-` for stdout, only valid with --out-file-format bedgraph).
    #[arg(short = 'o', long, default_value = "-")]
    pub output: String,

    /// Output format: bedgraph or bigwig. deeptools default is bigwig.
    #[arg(long = "out-file-format", short = 'F', default_value = "bigwig")]
    pub out_file_format: OutputFormat,

    /// Bin size in bases.
    #[arg(long = "bin-size", short = 'b', default_value_t = 50)]
    pub bin_size: u32,

    /// Combine operation: `log2`, `ratio`, `reciprocal_ratio`, `subtract`,
    /// `add`, `mean`, `first`, `second`.
    #[arg(long = "operation", default_value = "log2")]
    pub operation: Operation,

    /// Pseudocount added before ratio-family division. One value applies to both
    /// numerator and denominator; two values set them separately.
    #[arg(long = "pseudocount", num_args = 1..=2, default_values_t = [1.0])]
    pub pseudocount: Vec<f64>,

    /// Skip reads with any of these FLAG bits set (hex or decimal). deeptools
    /// default 0. Use 0x400 to skip duplicates.
    #[arg(long = "skip-flags", default_value = "0")]
    pub skip_flags: String,

    /// Minimum mapping quality (deeptools default 0 = no filter).
    #[arg(long = "min-mapq", default_value_t = 0)]
    pub min_mapq: u8,

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
        let pseudocount = expand_pseudocount(&self.pseudocount)?;

        let opts = CompareOpts {
            bin_size: self.bin_size,
            skip_flags,
            min_mapq: self.min_mapq,
            operation: self.operation,
            pseudocount,
        };

        let workers = std::num::NonZero::new(self.common.thread_count())
            .unwrap_or(std::num::NonZero::<usize>::MIN);

        match self.out_file_format {
            OutputFormat::BigWig => {
                if self.output == "-" {
                    return Err(RsomicsError::InvalidInput(
                        "bigWig output requires a file path (-o <file.bw>); stdout is not supported".into(),
                    ));
                }
                bam_compare_bigwig(
                    &self.bam1,
                    &self.bam2,
                    std::path::Path::new(&self.output),
                    &opts,
                    workers,
                )?;
                if !self.common.quiet {
                    eprintln!("bigWig written to {}", self.output);
                }
            }
            OutputFormat::BedGraph => {
                let mut out: Box<dyn std::io::Write> = if self.output == "-" {
                    Box::new(std::io::stdout().lock())
                } else {
                    Box::new(std::fs::File::create(&self.output).map_err(RsomicsError::Io)?)
                };
                let lines = bam_compare(&self.bam1, &self.bam2, &mut out, &opts, workers)?;
                if !self.common.quiet {
                    eprintln!("{lines} bedGraph lines written");
                }
            }
        }
        Ok(())
    }
}

/// deeptools `--pseudocount`: one value broadcasts to both numerator and
/// denominator, two set them separately.
fn expand_pseudocount(values: &[f64]) -> Result<[f64; 2]> {
    match values {
        [a] => Ok([*a, *a]),
        [a, b] => Ok([*a, *b]),
        _ => Err(RsomicsError::InvalidInput(
            "--pseudocount takes one or two values".into(),
        )),
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
    tagline: "Per-bin comparison of two BAMs → bedGraph/bigWig (deeptools bamCompare port).",
    origin: Some(Origin {
        upstream: "deeptools bamCompare",
        upstream_license: "MIT",
        our_license: "MIT OR Apache-2.0",
        paper_doi: Some("10.1093/nar/gkw257"),
    }),
    usage_lines: &[
        "--bam1 treat.bam --bam2 ctrl.bam [-o out.bw] [-F bigwig] [--operation log2] [--bin-size 50]",
    ],
    sections: &[Section {
        title: "OPTIONS",
        flags: &[
            FlagSpec {
                short: Some('1'),
                long: "bam1",
                aliases: &[],
                value: Some("<bam>"),
                type_hint: Some("path"),
                required: true,
                default: None,
                description: "Treatment / numerator BAM.",
                why_default: None,
            },
            FlagSpec {
                short: Some('2'),
                long: "bam2",
                aliases: &[],
                value: Some("<bam>"),
                type_hint: Some("path"),
                required: true,
                default: None,
                description: "Control / denominator BAM.",
                why_default: None,
            },
            FlagSpec {
                short: Some('F'),
                long: "out-file-format",
                aliases: &[],
                value: Some("<format>"),
                type_hint: Some("str"),
                required: false,
                default: Some("bigwig"),
                description: "Output format: bedgraph or bigwig (deeptools default: bigwig).",
                why_default: None,
            },
            FlagSpec {
                short: Some('b'),
                long: "bin-size",
                aliases: &[],
                value: Some("<u32>"),
                type_hint: Some("u32"),
                required: false,
                default: Some("50"),
                description: "Bin size in bases.",
                why_default: None,
            },
            FlagSpec {
                short: None,
                long: "operation",
                aliases: &[],
                value: Some("<op>"),
                type_hint: Some("str"),
                required: false,
                default: Some("log2"),
                description: "log2, ratio, reciprocal_ratio, subtract, add, mean, first, second.",
                why_default: None,
            },
            FlagSpec {
                short: None,
                long: "pseudocount",
                aliases: &[],
                value: Some("<f64>"),
                type_hint: Some("f64"),
                required: false,
                default: Some("1"),
                description: "Pseudocount for ratio-family ops (one or two values).",
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
        ],
    }],
    examples: &[
        Example {
            description: "log2 ratio of two BAMs, 50 bp bins",
            command: "rsomics-bam-compare --bam1 treat.bam --bam2 ctrl.bam -o log2.bedgraph",
        },
        Example {
            description: "subtraction of scaled coverage, 100 bp bins",
            command: "rsomics-bam-compare -1 a.bam -2 b.bam --operation subtract --bin-size 100 -o diff.bedgraph",
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

    #[test]
    #[allow(clippy::float_cmp)] // exact literals broadcast verbatim — bit-exact
    fn expand_pseudocount_ok() {
        assert_eq!(expand_pseudocount(&[1.0]).unwrap(), [1.0, 1.0]);
        assert_eq!(expand_pseudocount(&[2.0, 3.0]).unwrap(), [2.0, 3.0]);
    }
}
