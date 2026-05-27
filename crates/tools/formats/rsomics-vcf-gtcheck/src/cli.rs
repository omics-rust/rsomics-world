use clap::Parser;
use rsomics_common::{CommonFlags, Result, Tool, ToolMeta};
use rsomics_help::{Example, FlagSpec, HelpSpec, Origin, Section};
use std::io;
use std::path::PathBuf;

use rsomics_vcf_gtcheck::{GtcheckArgs, GtcheckMode, run_gtcheck};

pub const META: ToolMeta = ToolMeta {
    name: env!("CARGO_PKG_NAME"),
    version: env!("CARGO_PKG_VERSION"),
};

#[derive(Parser, Debug)]
#[command(name = "rsomics-vcf-gtcheck", version, about, long_about = None, disable_help_flag = true)]
pub struct Cli {
    /// Input VCF/BCF (query); if -g is not given, performs cross-check across all samples.
    #[arg(value_name = "QUERY.vcf[.gz]")]
    input: PathBuf,

    /// Genotypes VCF to compare against (panel mode).
    #[arg(short = 'g', long = "genotypes", value_name = "FILE")]
    genotypes: Option<PathBuf>,

    /// Phred-scaled genotyping error probability (0 = raw GT mismatch count).
    #[arg(
        short = 'E',
        long = "error-probability",
        default_value_t = 40,
        value_name = "INT"
    )]
    error_probability: u32,

    /// Disable HWE probability calculation.
    #[arg(long = "no-HWE-prob")]
    no_hwe_prob: bool,

    /// Consider homozygous genotypes only (requires -g).
    #[arg(short = 'H', long = "homs-only")]
    homs_only: bool,

    /// Force use of GT field (default: PL if present, else GT).
    #[arg(long = "use-gt")]
    use_gt: bool,

    #[command(flatten)]
    pub common: CommonFlags,
}

impl Cli {
    pub fn execute(self) -> Result<()> {
        let args = GtcheckArgs {
            error_prob: self.error_probability,
            no_hwe_prob: self.no_hwe_prob,
            homs_only: self.homs_only,
            use_gt: self.use_gt,
        };

        let stdout = io::stdout();
        let mut out = io::BufWriter::new(stdout.lock());

        match &self.genotypes {
            Some(gt_path) => {
                let mode = GtcheckMode::QueryVsGt {
                    query: &self.input,
                    gt: gt_path,
                };
                run_gtcheck(&mode, &args, &mut out)
            }
            None => {
                let mode = GtcheckMode::CrossCheck(&self.input);
                run_gtcheck(&mode, &args, &mut out)
            }
        }
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
    tagline: "Sample concordance / discordance estimator (bcftools gtcheck port).",
    origin: Some(Origin {
        upstream: "bcftools gtcheck",
        upstream_license: "MIT",
        our_license: "MIT OR Apache-2.0",
        paper_doi: None,
    }),
    usage_lines: &["[-g <genotypes.vcf.gz>] <query.vcf[.gz]>"],
    sections: &[Section {
        title: "OPTIONS",
        flags: &[
            FlagSpec {
                short: None,
                long: "QUERY.vcf[.gz]",
                aliases: &[],
                value: Some("<path>"),
                type_hint: Some("Path"),
                required: true,
                default: None,
                description: "Query VCF (single or multi-sample).",
                why_default: None,
            },
            FlagSpec {
                short: Some('g'),
                long: "genotypes",
                aliases: &[],
                value: Some("<FILE>"),
                type_hint: Some("Path"),
                required: false,
                default: None,
                description: "Genotypes panel VCF to compare against. Omit for cross-check.",
                why_default: None,
            },
            FlagSpec {
                short: Some('E'),
                long: "error-probability",
                aliases: &[],
                value: Some("<INT>"),
                type_hint: Some("u32"),
                required: false,
                default: Some("40"),
                description: "Phred-scaled genotyping error. 0 = raw GT mismatch count.",
                why_default: None,
            },
            FlagSpec {
                short: None,
                long: "no-HWE-prob",
                aliases: &[],
                value: None,
                type_hint: None,
                required: false,
                default: None,
                description: "Disable HWE probability column.",
                why_default: None,
            },
            FlagSpec {
                short: Some('H'),
                long: "homs-only",
                aliases: &[],
                value: None,
                type_hint: None,
                required: false,
                default: None,
                description: "Compare homozygous genotypes only.",
                why_default: None,
            },
            FlagSpec {
                short: None,
                long: "use-gt",
                aliases: &[],
                value: None,
                type_hint: None,
                required: false,
                default: None,
                description: "Force GT field (default: PL if present, else GT).",
                why_default: None,
            },
        ],
    }],
    examples: &[
        Example {
            description: "Cross-check all samples in one VCF",
            command: "rsomics-vcf-gtcheck multi.vcf.gz",
        },
        Example {
            description: "Check query vs genotype panel",
            command: "rsomics-vcf-gtcheck -g panel.vcf.gz query.vcf.gz",
        },
        Example {
            description: "Raw GT mismatch count, no HWE",
            command: "rsomics-vcf-gtcheck --use-gt -E 0 --no-HWE-prob multi.vcf.gz",
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
