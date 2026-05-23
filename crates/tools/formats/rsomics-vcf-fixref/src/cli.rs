use std::io::BufWriter;
use std::path::PathBuf;

use clap::Parser;
use rsomics_common::{CommonFlags, Result, RsomicsError, Tool, ToolMeta};
use rsomics_help::{Example, FlagSpec, HelpSpec, Origin, Section};

use rsomics_vcf_fixref::{FixMode, fixref};

pub const META: ToolMeta = ToolMeta {
    name: env!("CARGO_PKG_NAME"),
    version: env!("CARGO_PKG_VERSION"),
};

#[derive(Parser, Debug)]
#[command(
    name = "rsomics-vcf-fixref",
    version,
    about,
    long_about = None,
    disable_help_flag = true,
)]
pub struct Cli {
    /// Input VCF file.
    #[arg(value_name = "INPUT")]
    input: PathBuf,

    /// Reference FASTA file (must have an adjacent .fa.fai or .fasta.fai index).
    #[arg(short = 'f', long = "fasta", value_name = "FASTA")]
    fasta: PathBuf,

    /// Fix/check mode: `check` (stats only, pass-through), `flip` (fix unambiguous sites),
    /// `flip-all` (fix all sites including A/T and C/G ambiguous pairs).
    /// The `top`, `id`, and `stats` modes require a dbSNP VCF and are not implemented.
    #[arg(
        short = 'm',
        long = "mode",
        default_value = "check",
        value_name = "MODE"
    )]
    mode: String,

    /// Output file (default stdout).
    #[arg(short = 'o', long = "output", default_value = "-")]
    output: String,

    #[command(flatten)]
    pub common: CommonFlags,
}

impl Cli {
    pub fn execute(self) -> Result<()> {
        let mode = FixMode::parse(&self.mode).ok_or_else(|| {
            RsomicsError::InvalidInput(format!(
                "unknown mode '{}' — valid: check, flip, flip-all",
                self.mode
            ))
        })?;

        let mut out: Box<dyn std::io::Write> = if self.output == "-" {
            Box::new(BufWriter::new(std::io::stdout().lock()))
        } else {
            Box::new(BufWriter::new(
                std::fs::File::create(&self.output).map_err(RsomicsError::Io)?,
            ))
        };

        let stats = fixref(&self.input, &self.fasta, &mut out, mode)?;

        if !self.common.quiet {
            eprintln!(
                "{} sites: {} ok, {} swapped, {} flipped, {} flip+swap, {} unresolved, {} errors",
                stats.total,
                stats.nok,
                stats.nswap,
                stats.nflip,
                stats.nflip_swap,
                stats.nunresolved,
                stats.nerr,
            );
        }

        Ok(())
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
    tagline: "Check/fix VCF REF allele and strand against a reference FASTA (bcftools +fixref port).",
    origin: Some(Origin {
        upstream: "bcftools +fixref",
        upstream_license: "MIT",
        our_license: "MIT OR Apache-2.0",
        paper_doi: None,
    }),
    usage_lines: &["[OPTIONS] -f <REF.fa> <INPUT.vcf>"],
    sections: &[Section {
        title: "OPTIONS",
        flags: &[
            FlagSpec {
                short: None,
                long: "INPUT",
                aliases: &[],
                value: Some("<path>"),
                type_hint: Some("Path"),
                required: true,
                default: None,
                description: "Input VCF file.",
                why_default: None,
            },
            FlagSpec {
                short: Some('f'),
                long: "fasta",
                aliases: &[],
                value: Some("<path>"),
                type_hint: Some("Path"),
                required: true,
                default: None,
                description: "Reference FASTA (must have a .fai sidecar index).",
                why_default: None,
            },
            FlagSpec {
                short: Some('m'),
                long: "mode",
                aliases: &[],
                value: Some("<mode>"),
                type_hint: Some("String"),
                required: false,
                default: Some("check"),
                description: "check | flip | flip-all",
                why_default: None,
            },
            FlagSpec {
                short: Some('o'),
                long: "output",
                aliases: &[],
                value: Some("<path>"),
                type_hint: Some("String"),
                required: false,
                default: Some("-"),
                description: "Output VCF file (stdout by default).",
                why_default: None,
            },
        ],
    }],
    examples: &[
        Example {
            description: "Check REF alleles (pass-through, stats to stderr)",
            command: "rsomics-vcf-fixref -f hg38.fa input.vcf > checked.vcf",
        },
        Example {
            description: "Fix unambiguous mismatches in-place",
            command: "rsomics-vcf-fixref -f hg38.fa -m flip input.vcf -o fixed.vcf",
        },
        Example {
            description: "Fix all mismatches including A/T and C/G ambiguous pairs",
            command: "rsomics-vcf-fixref -f hg38.fa -m flip-all input.vcf -o fixed.vcf",
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
