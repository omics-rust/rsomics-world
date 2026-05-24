use std::path::PathBuf;

use clap::Parser;
use rsomics_common::{CommonFlags, Result, RsomicsError, Tool, ToolMeta};
use rsomics_help::{Example, FlagSpec, HelpSpec, Origin, Section};

use rsomics_bigwig_compare::{CompareOpts, Operation, bigwig_compare};

pub const META: ToolMeta = ToolMeta {
    name: env!("CARGO_PKG_NAME"),
    version: env!("CARGO_PKG_VERSION"),
};

#[derive(Parser, Debug)]
#[command(
    name = "rsomics-bigwig-compare",
    version,
    about,
    long_about = None,
    disable_help_flag = true
)]
pub struct Cli {
    /// Treatment / numerator bigWig (-b1).
    #[arg(short = '1', long = "bigwig1")]
    pub bigwig1: PathBuf,

    /// Control / denominator bigWig (-b2).
    #[arg(short = '2', long = "bigwig2")]
    pub bigwig2: PathBuf,

    /// Output bedGraph file (use `-` for stdout).
    #[arg(short = 'o', long = "out-file-name", default_value = "-")]
    pub output: String,

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

    /// Per-file multiplicative scale as `s1:s2` (deeptools `--scaleFactors`).
    #[arg(long = "scale-factors")]
    pub scale_factors: Option<String>,

    /// Drop bins where both files lack coverage (sum is 0), before pseudocount.
    #[arg(long = "skip-zero-over-zero", default_value_t = false)]
    pub skip_zero_over_zero: bool,

    /// Skip non-covered regions instead of treating them as zero.
    #[arg(
        long = "skip-non-covered-regions",
        alias = "skip-nas",
        default_value_t = false
    )]
    pub skip_non_covered_regions: bool,

    /// Emit every bin instead of merging adjacent equal values.
    #[arg(long = "fixed-step", default_value_t = false)]
    pub fixed_step: bool,

    /// Output format. Only `bedgraph` is supported (bigWig output needs a BBI
    /// writer this crate does not ship).
    #[arg(long = "out-file-format", short = 'O', default_value = "bedgraph")]
    pub out_file_format: String,

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
        if self.out_file_format != "bedgraph" {
            return Err(RsomicsError::InvalidInput(format!(
                "--out-file-format '{}' unsupported; only bedgraph is implemented",
                self.out_file_format
            )));
        }

        let pseudocount = expand_pseudocount(&self.pseudocount)?;
        let scale_factors = parse_scale_factors(self.scale_factors.as_deref())?;

        let opts = CompareOpts {
            bin_size: self.bin_size,
            operation: self.operation,
            scale_factors,
            pseudocount,
            missing_data_as_zero: !self.skip_non_covered_regions,
            skip_zero_over_zero: self.skip_zero_over_zero,
            fixed_step: self.fixed_step,
        };

        let mut out: Box<dyn std::io::Write> = if self.output == "-" {
            Box::new(std::io::stdout().lock())
        } else {
            Box::new(std::fs::File::create(&self.output).map_err(RsomicsError::Io)?)
        };

        let lines = bigwig_compare(&self.bigwig1, &self.bigwig2, &mut out, &opts)?;

        if !self.common.quiet {
            eprintln!("{lines} bedGraph lines written");
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

/// deeptools `--scaleFactors s1:s2`. Absent → `[1, 1]`.
fn parse_scale_factors(s: Option<&str>) -> Result<[f64; 2]> {
    let Some(s) = s else {
        return Ok([1.0, 1.0]);
    };
    let parts: Vec<&str> = s.split(':').collect();
    let err = || RsomicsError::InvalidInput(format!("invalid --scale-factors '{s}' (want s1:s2)"));
    match parts.as_slice() {
        [a, b] => {
            let a: f64 = a.parse().map_err(|_| err())?;
            let b: f64 = b.parse().map_err(|_| err())?;
            Ok([a, b])
        }
        _ => Err(err()),
    }
}

pub static HELP: HelpSpec = HelpSpec {
    name: env!("CARGO_PKG_NAME"),
    version: env!("CARGO_PKG_VERSION"),
    tagline: "Per-bin comparison of two bigWigs → bedGraph (deeptools bigwigCompare port).",
    origin: Some(Origin {
        upstream: "deeptools bigwigCompare",
        upstream_license: "MIT",
        our_license: "MIT OR Apache-2.0",
        paper_doi: Some("10.1093/nar/gkw257"),
    }),
    usage_lines: &[
        "-1 a.bw -2 b.bw [-o out.bg] [--operation log2] [--bin-size 50] [--out-file-format bedgraph]",
    ],
    sections: &[Section {
        title: "OPTIONS",
        flags: &[
            FlagSpec {
                short: Some('1'),
                long: "bigwig1",
                aliases: &[],
                value: Some("<bw>"),
                type_hint: Some("path"),
                required: true,
                default: None,
                description: "Treatment / numerator bigWig.",
                why_default: None,
            },
            FlagSpec {
                short: Some('2'),
                long: "bigwig2",
                aliases: &[],
                value: Some("<bw>"),
                type_hint: Some("path"),
                required: true,
                default: None,
                description: "Control / denominator bigWig.",
                why_default: None,
            },
            FlagSpec {
                short: Some('o'),
                long: "out-file-name",
                aliases: &[],
                value: Some("<file>"),
                type_hint: Some("path"),
                required: false,
                default: Some("-"),
                description: "Output bedGraph (`-` = stdout).",
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
                long: "scale-factors",
                aliases: &[],
                value: Some("<s1:s2>"),
                type_hint: Some("str"),
                required: false,
                default: Some("1:1"),
                description: "Per-file multiplicative scale.",
                why_default: None,
            },
            FlagSpec {
                short: None,
                long: "skip-zero-over-zero",
                aliases: &[],
                value: None,
                type_hint: Some("flag"),
                required: false,
                default: Some("false"),
                description: "Drop bins where both files lack coverage.",
                why_default: None,
            },
            FlagSpec {
                short: None,
                long: "skip-non-covered-regions",
                aliases: &["skip-nas"],
                value: None,
                type_hint: Some("flag"),
                required: false,
                default: Some("false"),
                description: "Skip non-covered regions instead of treating as 0.",
                why_default: None,
            },
            FlagSpec {
                short: None,
                long: "fixed-step",
                aliases: &[],
                value: None,
                type_hint: Some("flag"),
                required: false,
                default: Some("false"),
                description: "Emit every bin (no run-length merge).",
                why_default: None,
            },
            FlagSpec {
                short: Some('O'),
                long: "out-file-format",
                aliases: &[],
                value: Some("<bedgraph>"),
                type_hint: Some("str"),
                required: false,
                default: Some("bedgraph"),
                description: "Only bedgraph is supported.",
                why_default: None,
            },
        ],
    }],
    examples: &[
        Example {
            description: "log2 ratio of two bigWigs, 50 bp bins",
            command: "rsomics-bigwig-compare -b1 a.bw -b2 b.bw -o log2.bg",
        },
        Example {
            description: "subtraction, 100 bp bins, every bin emitted",
            command: "rsomics-bigwig-compare -b1 a.bw -b2 b.bw --operation subtract --bin-size 100 --fixed-step -o diff.bg",
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
    #[allow(clippy::float_cmp)] // exact literals broadcast verbatim — bit-exact
    fn expand_pseudocount_ok() {
        assert_eq!(expand_pseudocount(&[1.0]).unwrap(), [1.0, 1.0]);
        assert_eq!(expand_pseudocount(&[2.0, 3.0]).unwrap(), [2.0, 3.0]);
    }

    #[test]
    #[allow(clippy::float_cmp)]
    fn scale_factors_parse() {
        assert_eq!(parse_scale_factors(None).unwrap(), [1.0, 1.0]);
        assert_eq!(parse_scale_factors(Some("0.7:1")).unwrap(), [0.7, 1.0]);
        assert!(
            parse_scale_factors(Some("bad"))
                .unwrap_err()
                .to_string()
                .contains("scale-factors")
        );
    }
}
