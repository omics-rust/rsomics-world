use std::path::PathBuf;

use clap::Parser;
use rsomics_common::{CommonFlags, Result, Tool, ToolMeta};
use rsomics_help::{Example, FlagSpec, HelpSpec, Origin, Section};

use rsomics_vcf_cnv::cnv::EmissionParams;
use rsomics_vcf_cnv::vcf::CnvArgs;

pub const META: ToolMeta = ToolMeta {
    name: env!("CARGO_PKG_NAME"),
    version: env!("CARGO_PKG_VERSION"),
};

#[derive(Parser, Debug)]
#[command(name = "rsomics-vcf-cnv", version, about, long_about = None, disable_help_flag = true)]
pub struct Cli {
    /// Input VCF (must have FORMAT/BAF and FORMAT/LRR per-sample fields).
    #[arg(value_name = "INPUT.vcf[.gz]")]
    input: PathBuf,

    /// Query sample name (required for multi-sample VCF).
    #[arg(short = 's', long = "sample", value_name = "STRING")]
    sample: Option<String>,

    /// Output directory for cn.<sample>.tab and summary.<sample>.tab.
    #[arg(
        short = 'o',
        long = "output-dir",
        default_value = ".",
        value_name = "PATH"
    )]
    output_dir: PathBuf,

    /// Read allele frequencies from file (CHR TAB POS TAB REF,ALT TAB AF).
    #[arg(short = 'f', long = "AF-file", value_name = "FILE")]
    af_file: Option<PathBuf>,

    /// Fraction of aberrant cells [1.0].
    #[arg(
        short = 'a',
        long = "aberrant",
        default_value_t = 1.0_f64,
        value_name = "FLOAT"
    )]
    aberrant: f64,

    /// Relative weight of BAF evidence [1.0].
    #[arg(
        short = 'b',
        long = "BAF-weight",
        default_value_t = 1.0_f64,
        value_name = "FLOAT"
    )]
    baf_weight: f64,

    /// Expected BAF standard deviation [0.04].
    #[arg(
        short = 'd',
        long = "BAF-dev",
        default_value_t = 0.04_f64,
        value_name = "FLOAT"
    )]
    baf_dev: f64,

    /// Uniform measurement error probability [1e-4].
    #[arg(
        short = 'e',
        long = "err-prob",
        default_value_t = 1e-4_f64,
        value_name = "FLOAT"
    )]
    err_prob: f64,

    /// Expected LRR standard deviation [0.2].
    #[arg(
        short = 'k',
        long = "LRR-dev",
        default_value_t = 0.2_f64,
        value_name = "FLOAT"
    )]
    lrr_dev: f64,

    /// Relative weight of LRR evidence [0.2].
    #[arg(
        short = 'l',
        long = "LRR-weight",
        default_value_t = 0.2_f64,
        value_name = "FLOAT"
    )]
    lrr_weight: f64,

    /// LRR moving-average smoothing window [10].
    #[arg(
        short = 'L',
        long = "LRR-smooth-win",
        default_value_t = 10_usize,
        value_name = "INT"
    )]
    lrr_smooth_win: usize,

    /// HMM off-diagonal transition probability P(x|y) [1e-9].
    #[arg(
        short = 'x',
        long = "xy-prob",
        default_value_t = 1e-9_f64,
        value_name = "FLOAT"
    )]
    xy_prob: f64,

    /// Write only summary.tab; skip per-site cn.tab output.
    #[arg(long = "summary-only")]
    summary_only: bool,

    #[command(flatten)]
    pub common: CommonFlags,
}

impl Cli {
    pub fn execute(self) -> Result<()> {
        let emission = EmissionParams {
            baf_dev2: self.baf_dev * self.baf_dev,
            lrr_dev2: self.lrr_dev * self.lrr_dev,
            baf_bias: self.baf_weight,
            lrr_bias: self.lrr_weight,
            err_prob: self.err_prob,
            cell_frac: self.aberrant,
        };
        let args = CnvArgs {
            sample: self.sample,
            output_dir: self.output_dir,
            af_file: self.af_file,
            emission,
            xy_prob: self.xy_prob,
            lrr_smooth_win: self.lrr_smooth_win,
            summary_only: self.summary_only,
        };
        rsomics_vcf_cnv::run_cnv(&self.input, &args)
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
    tagline: "HMM-based CNV caller from BAF + LRR in a single-sample VCF (bcftools cnv port).",
    origin: Some(Origin {
        upstream: "bcftools cnv",
        upstream_license: "MIT",
        our_license: "MIT OR Apache-2.0",
        paper_doi: None,
    }),
    usage_lines: &["[OPTIONS] <INPUT.vcf[.gz]>"],
    sections: &[Section {
        title: "OPTIONS",
        flags: &[
            FlagSpec {
                short: Some('s'),
                long: "sample",
                aliases: &[],
                value: Some("<STRING>"),
                type_hint: Some("String"),
                required: false,
                default: None,
                description: "Query sample name (required for multi-sample VCF).",
                why_default: None,
            },
            FlagSpec {
                short: Some('o'),
                long: "output-dir",
                aliases: &[],
                value: Some("<PATH>"),
                type_hint: Some("Path"),
                required: false,
                default: Some("."),
                description: "Output directory; writes cn.<sample>.tab and summary.<sample>.tab.",
                why_default: None,
            },
            FlagSpec {
                short: Some('f'),
                long: "AF-file",
                aliases: &[],
                value: Some("<FILE>"),
                type_hint: Some("Path"),
                required: false,
                default: None,
                description: "Allele frequency file (CHR TAB POS TAB REF,ALT TAB AF).",
                why_default: None,
            },
            FlagSpec {
                short: Some('a'),
                long: "aberrant",
                aliases: &[],
                value: Some("<FLOAT>"),
                type_hint: Some("f64"),
                required: false,
                default: Some("1.0"),
                description: "Fraction of aberrant cells.",
                why_default: None,
            },
            FlagSpec {
                short: Some('b'),
                long: "BAF-weight",
                aliases: &[],
                value: Some("<FLOAT>"),
                type_hint: Some("f64"),
                required: false,
                default: Some("1.0"),
                description: "Relative contribution of BAF evidence to HMM emissions.",
                why_default: None,
            },
            FlagSpec {
                short: Some('d'),
                long: "BAF-dev",
                aliases: &[],
                value: Some("<FLOAT>"),
                type_hint: Some("f64"),
                required: false,
                default: Some("0.04"),
                description: "Expected BAF standard deviation.",
                why_default: None,
            },
            FlagSpec {
                short: Some('e'),
                long: "err-prob",
                aliases: &[],
                value: Some("<FLOAT>"),
                type_hint: Some("f64"),
                required: false,
                default: Some("1e-4"),
                description: "Uniform measurement error floor for emission probabilities.",
                why_default: None,
            },
            FlagSpec {
                short: Some('k'),
                long: "LRR-dev",
                aliases: &[],
                value: Some("<FLOAT>"),
                type_hint: Some("f64"),
                required: false,
                default: Some("0.2"),
                description: "Expected LRR standard deviation.",
                why_default: None,
            },
            FlagSpec {
                short: Some('l'),
                long: "LRR-weight",
                aliases: &[],
                value: Some("<FLOAT>"),
                type_hint: Some("f64"),
                required: false,
                default: Some("0.2"),
                description: "Relative contribution of LRR evidence to HMM emissions.",
                why_default: None,
            },
            FlagSpec {
                short: Some('L'),
                long: "LRR-smooth-win",
                aliases: &[],
                value: Some("<INT>"),
                type_hint: Some("usize"),
                required: false,
                default: Some("10"),
                description: "Window size for LRR moving-average smoothing.",
                why_default: None,
            },
            FlagSpec {
                short: Some('x'),
                long: "xy-prob",
                aliases: &[],
                value: Some("<FLOAT>"),
                type_hint: Some("f64"),
                required: false,
                default: Some("1e-9"),
                description: "HMM off-diagonal transition probability P(state_i | state_j).",
                why_default: None,
            },
            FlagSpec {
                short: None,
                long: "summary-only",
                aliases: &[],
                value: None,
                type_hint: None,
                required: false,
                default: None,
                description: "Skip per-site cn.tab; write only summary.tab regions.",
                why_default: None,
            },
        ],
    }],
    examples: &[
        Example {
            description: "Run CNV calling on a single-sample array VCF",
            command: "rsomics-vcf-cnv -o results/ sample.vcf.gz",
        },
        Example {
            description: "Specify sample and disable LRR component",
            command: "rsomics-vcf-cnv -s SAMPLE -l0 -o results/ multi.vcf.gz",
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
