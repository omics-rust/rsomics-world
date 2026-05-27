use clap::Parser;
use rsomics_common::{CommonFlags, Result, Tool, ToolMeta};
use rsomics_help::{Example, FlagSpec, HelpSpec, Origin, Section};
use std::path::PathBuf;

use rsomics_vcf_polysomy::{PolysomyArgs, run_polysomy};

pub const META: ToolMeta = ToolMeta {
    name: env!("CARGO_PKG_NAME"),
    version: env!("CARGO_PKG_VERSION"),
};

const VERSION_STR: &str = concat!(env!("CARGO_PKG_VERSION"), "+htslib-rsomics");

#[derive(Parser, Debug)]
#[command(
    name = "rsomics-vcf-polysomy",
    version,
    about,
    long_about = None,
    disable_help_flag = true
)]
pub struct Cli {
    /// Input VCF/BCF file (must contain FORMAT/BAF field).
    #[arg(value_name = "INPUT.vcf[.gz]")]
    input: PathBuf,

    /// Output directory for dist.dat (created if absent).
    #[arg(short = 'o', long = "output-dir", value_name = "DIR")]
    output_dir: PathBuf,

    /// Sample name to analyse (required for multi-sample VCF).
    #[arg(short = 's', long = "sample", value_name = "NAME")]
    sample: Option<String>,

    /// Goodness-of-fit threshold; higher = more permissive.
    #[arg(
        short = 'f',
        long = "fit-th",
        default_value_t = 3.3,
        value_name = "FLOAT"
    )]
    fit_th: f64,

    /// Penalty for increasing copy number (0–1, larger = stricter).
    #[arg(
        short = 'c',
        long = "cn-penalty",
        default_value_t = 0.7,
        value_name = "FLOAT"
    )]
    cn_penalty: f64,

    /// Peak symmetry threshold (0–1, larger = stricter).
    #[arg(long = "peak-symmetry", default_value_t = 0.5, value_name = "FLOAT")]
    peak_symmetry: f64,

    /// Minimum peak size (0–1).
    #[arg(
        short = 'b',
        long = "peak-size",
        default_value_t = 0.1,
        value_name = "FLOAT"
    )]
    min_peak_size: f64,

    /// Minimum detectable aberrant-cell fraction.
    #[arg(
        short = 'm',
        long = "min-fraction",
        default_value_t = 0.1,
        value_name = "FLOAT"
    )]
    min_fraction: f64,

    /// Include AA peak in CN2/CN3 evaluation.
    #[arg(short = 'i', long = "include-aa")]
    include_aa: bool,

    #[command(flatten)]
    pub common: CommonFlags,
}

impl Cli {
    pub fn execute(self) -> Result<()> {
        let args = PolysomyArgs {
            sample: self.sample,
            fit_th: self.fit_th,
            cn_penalty: self.cn_penalty,
            peak_symmetry: self.peak_symmetry,
            min_peak_size: self.min_peak_size,
            min_fraction: self.min_fraction,
            include_aa: self.include_aa,
            smooth: 3,
        };

        let cmd_str = format!(
            "polysomy -o {} {}",
            self.output_dir.display(),
            self.input.display()
        );

        run_polysomy(&self.input, &self.output_dir, &args, VERSION_STR, &cmd_str)
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
    tagline: "Estimate per-chromosome copy number from BAF distributions (bcftools polysomy port).",
    origin: Some(Origin {
        upstream: "bcftools polysomy",
        upstream_license: "MIT",
        our_license: "MIT OR Apache-2.0",
        paper_doi: None,
    }),
    usage_lines: &["-o <output-dir> [OPTIONS] <INPUT.vcf[.gz]>"],
    sections: &[Section {
        title: "OPTIONS",
        flags: &[
            FlagSpec {
                short: None,
                long: "INPUT.vcf[.gz]",
                aliases: &[],
                value: Some("<path>"),
                type_hint: Some("Path"),
                required: true,
                default: None,
                description: "Input VCF/BCF containing FORMAT/BAF field.",
                why_default: None,
            },
            FlagSpec {
                short: Some('o'),
                long: "output-dir",
                aliases: &[],
                value: Some("<DIR>"),
                type_hint: Some("Path"),
                required: true,
                default: None,
                description: "Directory for dist.dat output.",
                why_default: None,
            },
            FlagSpec {
                short: Some('s'),
                long: "sample",
                aliases: &[],
                value: Some("<NAME>"),
                type_hint: Some("String"),
                required: false,
                default: None,
                description: "Sample to analyse (required for multi-sample VCF).",
                why_default: None,
            },
            FlagSpec {
                short: Some('f'),
                long: "fit-th",
                aliases: &[],
                value: Some("<FLOAT>"),
                type_hint: Some("f64"),
                required: false,
                default: Some("3.3"),
                description: "Goodness-of-fit threshold.",
                why_default: None,
            },
            FlagSpec {
                short: Some('c'),
                long: "cn-penalty",
                aliases: &[],
                value: Some("<FLOAT>"),
                type_hint: Some("f64"),
                required: false,
                default: Some("0.7"),
                description: "Penalty for higher copy number (0–1).",
                why_default: None,
            },
            FlagSpec {
                short: None,
                long: "peak-symmetry",
                aliases: &[],
                value: Some("<FLOAT>"),
                type_hint: Some("f64"),
                required: false,
                default: Some("0.5"),
                description: "Peak symmetry threshold (0–1).",
                why_default: None,
            },
            FlagSpec {
                short: Some('b'),
                long: "peak-size",
                aliases: &[],
                value: Some("<FLOAT>"),
                type_hint: Some("f64"),
                required: false,
                default: Some("0.1"),
                description: "Minimum peak size fraction.",
                why_default: None,
            },
            FlagSpec {
                short: Some('m'),
                long: "min-fraction",
                aliases: &[],
                value: Some("<FLOAT>"),
                type_hint: Some("f64"),
                required: false,
                default: Some("0.1"),
                description: "Minimum detectable aberrant-cell fraction.",
                why_default: None,
            },
            FlagSpec {
                short: Some('i'),
                long: "include-aa",
                aliases: &[],
                value: None,
                type_hint: None,
                required: false,
                default: None,
                description: "Include AA peak in CN2/CN3 evaluation.",
                why_default: None,
            },
        ],
    }],
    examples: &[
        Example {
            description: "Estimate CN for single-sample VCF",
            command: "rsomics-vcf-polysomy -o outdir sample.vcf.gz",
        },
        Example {
            description: "Analyse one sample from multi-sample VCF",
            command: "rsomics-vcf-polysomy -s NA12878 -o outdir cohort.vcf.gz",
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
