use std::path::PathBuf;

use clap::Parser;
use rsomics_common::{CommonFlags, Result, RsomicsError, ToolMeta};
use rsomics_help::{Example, FlagSpec, HelpSpec, Origin, Section};

use rsomics_vcf_annotate::annotate_vcf;

pub const META: ToolMeta = ToolMeta {
    name: env!("CARGO_PKG_NAME"),
    version: env!("CARGO_PKG_VERSION"),
};

#[derive(Parser, Debug)]
#[command(
    name = "rsomics-vcf-annotate",
    version,
    about,
    long_about = None,
    disable_help_flag = true
)]
pub struct Cli {
    pub input: PathBuf,
    #[arg(short = 'a', long = "annotations")]
    annotations: PathBuf,
    #[arg(short = 'o', long = "output", default_value = "-")]
    output: String,
    #[arg(long = "tag", default_value = "ANN")]
    tag: String,
    #[command(flatten)]
    pub common: CommonFlags,
}

impl Cli {
    pub fn execute(self) -> Result<()> {
        let mut out: Box<dyn std::io::Write> = if self.output == "-" {
            Box::new(std::io::stdout().lock())
        } else {
            Box::new(std::fs::File::create(&self.output).map_err(RsomicsError::Io)?)
        };
        let count = annotate_vcf(&self.input, &self.annotations, &mut out, &self.tag)?;
        if !self.common.quiet {
            eprintln!("{count} variants processed");
        }
        Ok(())
    }
}

pub static HELP: HelpSpec = HelpSpec {
    name: META.name,
    version: META.version,
    tagline: "Annotate VCF variants with labels from BED regions.",
    origin: Some(Origin {
        upstream: "bcftools annotate",
        upstream_license: "MIT",
        our_license: "MIT OR Apache-2.0",
        paper_doi: None,
    }),
    usage_lines: &["<input.vcf> -a <regions.bed> [--tag ANN] [-o out.vcf]"],
    sections: &[Section {
        title: "OPTIONS",
        flags: &[
            FlagSpec {
                short: Some('a'),
                long: "annotations",
                aliases: &[],
                value: Some("<path>"),
                type_hint: Some("Path"),
                required: true,
                default: None,
                description: "BED file with annotations (col4 = label).",
                why_default: None,
            },
            FlagSpec {
                short: None,
                long: "tag",
                aliases: &[],
                value: Some("<key>"),
                type_hint: Some("String"),
                required: false,
                default: Some("ANN"),
                description: "INFO tag name to add.",
                why_default: None,
            },
        ],
    }],
    examples: &[Example {
        description: "Annotate variants with gene names",
        command: "rsomics-vcf-annotate input.vcf -a genes.bed -o annotated.vcf",
    }],
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
