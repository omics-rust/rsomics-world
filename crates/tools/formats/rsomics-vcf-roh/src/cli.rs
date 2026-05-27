use clap::Parser;
use rsomics_common::{CommonFlags, Result, Tool, ToolMeta};
use rsomics_help::{Example, FlagSpec, HelpSpec, Origin, Section};
use std::io;
use std::path::PathBuf;

use rsomics_vcf_roh::{OutputMode, RohArgs, run_roh};

pub const META: ToolMeta = ToolMeta {
    name: env!("CARGO_PKG_NAME"),
    version: env!("CARGO_PKG_VERSION"),
};

#[derive(Parser, Debug)]
#[command(name = "rsomics-vcf-roh", version, about, long_about = None, disable_help_flag = true)]
pub struct Cli {
    /// Input VCF/BCF (plain, gzip, or bgzf).
    #[arg(value_name = "INPUT.vcf[.gz]")]
    input: PathBuf,

    /// Default allele frequency when AF cannot be determined (skip if not set).
    #[arg(long = "AF-dflt", value_name = "FLOAT")]
    af_dflt: Option<f64>,

    /// INFO tag to read allele frequency from (e.g. AF).
    #[arg(long = "AF-tag", value_name = "TAG")]
    af_tag: Option<String>,

    /// P(AZ|HW): transition probability from HW to autozygous state.
    #[arg(
        short = 'a',
        long = "hw-to-az",
        default_value_t = 6.7e-8_f64,
        value_name = "FLOAT"
    )]
    hw_to_az: f64,

    /// P(HW|AZ): transition probability from AZ to HW state.
    #[arg(
        short = 'H',
        long = "az-to-hw",
        default_value_t = 5e-9_f64,
        value_name = "FLOAT"
    )]
    az_to_hw: f64,

    /// Use GT field; FLOAT is the error probability for the two non-called genotypes (Phred-scaled).
    #[arg(short = 'G', long = "GTs-only", value_name = "FLOAT")]
    gts_only: Option<f64>,

    /// Skip indels (SNPs only).
    #[arg(short = 'I', long = "skip-indels")]
    skip_indels: bool,

    /// Skip hom-ref genotypes (0/0).
    #[arg(short = 'i', long = "ignore-homref")]
    ignore_homref: bool,

    /// Output type: s=per-site, r=regions (default: sr).
    #[arg(
        short = 'O',
        long = "output-type",
        default_value = "sr",
        value_name = "TYPE"
    )]
    output_type: String,

    /// Output file (default: stdout).
    #[arg(short = 'o', long = "output", value_name = "FILE")]
    output: Option<PathBuf>,

    /// Restrict to samples (comma-separated list).
    #[arg(short = 's', long = "samples", value_name = "LIST")]
    samples: Option<String>,

    #[command(flatten)]
    pub common: CommonFlags,
}

impl Cli {
    pub fn execute(self) -> Result<()> {
        let want_sites = self.output_type.contains('s') || self.output_type.contains('S');
        let want_regions = self.output_type.contains('r') || self.output_type.contains('R');
        let output_mode = OutputMode {
            sites: want_sites,
            regions: want_regions,
        };

        let fake_pl_phred = self.gts_only.map(|phred| 10f64.powf(-phred / 10.0));

        let args = RohArgs {
            hw_to_az: self.hw_to_az,
            az_to_hw: self.az_to_hw,
            af_dflt: self.af_dflt,
            af_tag: self.af_tag,
            fake_pl_error: fake_pl_phred,
            skip_indels: self.skip_indels,
            ignore_homref: self.ignore_homref,
            samples: self.samples,
            output_mode,
            output: self.output,
        };

        let stdout = io::stdout();
        let mut out: Box<dyn io::Write> = match &args.output {
            Some(path) => {
                Box::new(std::fs::File::create(path).map_err(rsomics_common::RsomicsError::Io)?)
            }
            None => Box::new(io::BufWriter::new(stdout.lock())),
        };

        run_roh(&self.input, &args, &mut out)
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
    tagline: "Detect runs of homozygosity (ROH) via a 2-state HMM (bcftools roh port).",
    origin: Some(Origin {
        upstream: "bcftools roh",
        upstream_license: "MIT",
        our_license: "MIT OR Apache-2.0",
        paper_doi: None,
    }),
    usage_lines: &["[OPTIONS] <INPUT.vcf[.gz]>"],
    sections: &[Section {
        title: "OPTIONS",
        flags: &[
            FlagSpec {
                short: None,
                long: "AF-dflt",
                aliases: &[],
                value: Some("<FLOAT>"),
                type_hint: Some("f64"),
                required: false,
                default: None,
                description: "Default allele frequency when AF is unknown (skip site if not set).",
                why_default: None,
            },
            FlagSpec {
                short: None,
                long: "AF-tag",
                aliases: &[],
                value: Some("<TAG>"),
                type_hint: Some("String"),
                required: false,
                default: None,
                description: "INFO tag to read ALT allele frequency from (e.g. AF).",
                why_default: None,
            },
            FlagSpec {
                short: Some('a'),
                long: "hw-to-az",
                aliases: &[],
                value: Some("<FLOAT>"),
                type_hint: Some("f64"),
                required: false,
                default: Some("6.7e-8"),
                description: "P(AZ|HW) transition probability from Hardy-Weinberg to autozygous state.",
                why_default: None,
            },
            FlagSpec {
                short: Some('H'),
                long: "az-to-hw",
                aliases: &[],
                value: Some("<FLOAT>"),
                type_hint: Some("f64"),
                required: false,
                default: Some("5e-9"),
                description: "P(HW|AZ) transition probability from autozygous to Hardy-Weinberg state.",
                why_default: None,
            },
            FlagSpec {
                short: Some('G'),
                long: "GTs-only",
                aliases: &[],
                value: Some("<FLOAT>"),
                type_hint: Some("f64"),
                required: false,
                default: None,
                description: "Use GT field; FLOAT is Phred-scaled error for non-called genotypes.",
                why_default: None,
            },
            FlagSpec {
                short: Some('I'),
                long: "skip-indels",
                aliases: &[],
                value: None,
                type_hint: None,
                required: false,
                default: None,
                description: "Skip indels (SNPs only).",
                why_default: None,
            },
            FlagSpec {
                short: Some('i'),
                long: "ignore-homref",
                aliases: &[],
                value: None,
                type_hint: None,
                required: false,
                default: None,
                description: "Skip hom-ref genotypes (0/0).",
                why_default: None,
            },
            FlagSpec {
                short: Some('O'),
                long: "output-type",
                aliases: &[],
                value: Some("<TYPE>"),
                type_hint: Some("str"),
                required: false,
                default: Some("sr"),
                description: "Output: s=per-site ST lines, r=ROH region RG lines.",
                why_default: None,
            },
            FlagSpec {
                short: Some('o'),
                long: "output",
                aliases: &[],
                value: Some("<FILE>"),
                type_hint: Some("Path"),
                required: false,
                default: None,
                description: "Output file (default: stdout).",
                why_default: None,
            },
            FlagSpec {
                short: Some('s'),
                long: "samples",
                aliases: &[],
                value: Some("<LIST>"),
                type_hint: Some("String"),
                required: false,
                default: None,
                description: "Comma-separated list of samples to analyze (default: all).",
                why_default: None,
            },
        ],
    }],
    examples: &[
        Example {
            description: "Detect ROH regions using default AF 0.4 and GT-only mode",
            command: "rsomics-vcf-roh -G30 --AF-dflt 0.4 -Or input.vcf.gz",
        },
        Example {
            description: "Per-site state output using AF from INFO/AF tag",
            command: "rsomics-vcf-roh --AF-tag AF -Os input.vcf.gz",
        },
        Example {
            description: "Both sites and regions, specific samples",
            command: "rsomics-vcf-roh -G30 --AF-dflt 0.4 -s NA12878,NA12877 input.vcf.gz",
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
