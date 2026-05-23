use std::io::BufWriter;
use std::path::PathBuf;

use clap::Parser;
use rsomics_common::{CommonFlags, Result, RsomicsError, Tool, ToolMeta};
use rsomics_help::{Example, FlagSpec, HelpSpec, Origin, Section};

use rsomics_vcf_setgt::{NewGt, SetGtStats, Target, set_gt};

pub const META: ToolMeta = ToolMeta {
    name: env!("CARGO_PKG_NAME"),
    version: env!("CARGO_PKG_VERSION"),
};

#[derive(Parser, Debug)]
#[command(
    name = "rsomics-vcf-setgt",
    version,
    about,
    long_about = None,
    disable_help_flag = true,
)]
pub struct Cli {
    /// Input VCF/BCF file.
    #[arg(value_name = "INPUT")]
    input: PathBuf,

    /// Output file (default stdout).
    #[arg(short = 'o', long = "output", default_value = "-")]
    output: String,

    /// Target genotype selector.
    /// Supported: `.` (any missing), `./x` (partially missing),
    /// `./.` (fully missing), `a` (all genotypes).
    #[arg(long = "target", value_name = "TARGET")]
    target: String,

    /// New genotype value.
    /// Supported: `.` (missing), `0` (ref), `p` (phase), `u` (unphase),
    /// `c:GT` (custom literal, e.g. `c:0/0`).
    #[arg(short = 'n', long = "new-gt", value_name = "NEWGT")]
    new_gt: String,

    #[command(flatten)]
    pub common: CommonFlags,
}

impl Cli {
    pub fn execute(self) -> Result<()> {
        let target = Target::parse(&self.target)?;
        let new_gt = NewGt::parse(&self.new_gt)?;

        let mut out: Box<dyn std::io::Write> = if self.output == "-" {
            Box::new(BufWriter::new(std::io::stdout().lock()))
        } else {
            Box::new(BufWriter::new(
                std::fs::File::create(&self.output)
                    .map_err(|e| RsomicsError::InvalidInput(format!("{}: {e}", self.output)))?,
            ))
        };

        let SetGtStats { total, changed } = set_gt(&self.input, &mut out, target, &new_gt)?;

        if !self.common.quiet {
            eprintln!("{changed}/{total} genotypes changed");
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
    tagline: "Conditionally rewrite VCF genotypes — port of bcftools +setGT.",
    origin: Some(Origin {
        upstream: "bcftools +setGT",
        upstream_license: "MIT",
        our_license: "MIT OR Apache-2.0",
        paper_doi: None,
    }),
    usage_lines: &["[OPTIONS] --target TARGET -n NEWGT <INPUT.vcf>"],
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
                description: "Input VCF/BCF file.",
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
                description: "Output file (stdout by default).",
                why_default: None,
            },
            FlagSpec {
                short: None,
                long: "target",
                aliases: &[],
                value: Some("<TARGET>"),
                type_hint: Some("String"),
                required: true,
                default: None,
                description: "Target selector: . (any missing), ./x (partially missing), ./. (fully missing), a (all).",
                why_default: None,
            },
            FlagSpec {
                short: Some('n'),
                long: "new-gt",
                aliases: &[],
                value: Some("<NEWGT>"),
                type_hint: Some("String"),
                required: true,
                default: None,
                description: "New GT value: . (missing), 0 (ref), p (phase), u (unphase), c:GT (custom).",
                why_default: None,
            },
        ],
    }],
    examples: &[
        Example {
            description: "Set all missing GTs to ref",
            command: "rsomics-vcf-setgt --target . -n 0 input.vcf",
        },
        Example {
            description: "Set fully-missing GTs to missing (no-op normalise)",
            command: "rsomics-vcf-setgt --target ./. -n . input.vcf -o out.vcf",
        },
        Example {
            description: "Set all GTs to custom 0/0",
            command: "rsomics-vcf-setgt --target a -n c:0/0 input.vcf -o out.vcf",
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
