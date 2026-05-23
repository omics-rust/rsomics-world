use std::io::BufWriter;
use std::path::PathBuf;

use clap::Parser;
use rsomics_common::{CommonFlags, Result, RsomicsError, Tool, ToolMeta};
use rsomics_help::{Example, FlagSpec, HelpSpec, Origin, Section};

use rsomics_vcf_fill_tags::{FillTagsStats, Tags, fill_tags};

pub const META: ToolMeta = ToolMeta {
    name: env!("CARGO_PKG_NAME"),
    version: env!("CARGO_PKG_VERSION"),
};

#[derive(Parser, Debug)]
#[command(
    name = "rsomics-vcf-fill-tags",
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

    /// Comma-separated tag list to compute (default: all standard tags).
    /// Supported: `AN`,`AC`,`AF`,`MAF`,`NS`,`AC_Hom`,`AC_Het`,`AC_Hemi`,`HWE`,`ExcHet`
    #[arg(long = "tags")]
    tags: Option<String>,

    #[command(flatten)]
    pub common: CommonFlags,
}

impl Cli {
    pub fn execute(self) -> Result<()> {
        let tags = match self.tags {
            Some(ref list) => Tags::from_list(list)
                .map_err(|e| RsomicsError::InvalidInput(format!("--tags: {e}")))?,
            None => Tags::default_set(),
        };

        let mut out: Box<dyn std::io::Write> = if self.output == "-" {
            Box::new(BufWriter::new(std::io::stdout().lock()))
        } else {
            Box::new(BufWriter::new(
                std::fs::File::create(&self.output).map_err(RsomicsError::Io)?,
            ))
        };

        let FillTagsStats { total, processed } = fill_tags(&self.input, &mut out, tags)?;

        if !self.common.quiet {
            eprintln!("{processed}/{total} records annotated");
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
    tagline: "Recompute VCF INFO tags (AN/AC/AF/MAF/NS/AC_Hom/AC_Het/AC_Hemi/HWE/ExcHet) from FORMAT/GT.",
    origin: Some(Origin {
        upstream: "bcftools +fill-tags",
        upstream_license: "MIT",
        our_license: "MIT OR Apache-2.0",
        paper_doi: Some("10.1093/gigascience/giab008"),
    }),
    usage_lines: &["[OPTIONS] <INPUT.vcf>"],
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
                long: "tags",
                aliases: &[],
                value: Some("<LIST>"),
                type_hint: Some("String"),
                required: false,
                default: Some("all"),
                description: "Comma-separated tag names to compute.",
                why_default: None,
            },
        ],
    }],
    examples: &[
        Example {
            description: "Annotate with all standard tags",
            command: "rsomics-vcf-fill-tags input.vcf > annotated.vcf",
        },
        Example {
            description: "Annotate with only AN, AC, AF",
            command: "rsomics-vcf-fill-tags --tags AN,AC,AF input.vcf -o annotated.vcf",
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
