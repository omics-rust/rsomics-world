use std::path::PathBuf;

use clap::Parser;
use rsomics_common::{CommonFlags, Result, RsomicsError, ToolMeta};
use rsomics_help::{Example, FlagSpec, HelpSpec, Origin, Section};

use rsomics_vcf_view::{ViewOpts, view_vcf};

pub const META: ToolMeta = ToolMeta {
    name: env!("CARGO_PKG_NAME"),
    version: env!("CARGO_PKG_VERSION"),
};

#[derive(Parser, Debug)]
#[command(name = "rsomics-vcf-view", version, about, long_about = None, disable_help_flag = true)]
pub struct Cli {
    pub input: PathBuf,
    #[arg(short = 'o', long = "output", default_value = "-")]
    output: String,
    #[arg(short = 'H', long = "header-only")]
    header_only: bool,
    #[arg(long = "no-header")]
    no_header: bool,
    #[arg(short = 'c', long = "count")]
    count_only: bool,
    #[arg(short = 'r', long = "region")]
    region: Option<String>,
    #[command(flatten)]
    pub common: CommonFlags,
}

impl Cli {
    pub fn execute(self) -> Result<()> {
        let opts = ViewOpts {
            header_only: self.header_only,
            no_header: self.no_header,
            count_only: self.count_only,
            region: self.region,
        };
        let mut out: Box<dyn std::io::Write> = if self.output == "-" {
            Box::new(std::io::stdout().lock())
        } else {
            Box::new(std::fs::File::create(&self.output).map_err(RsomicsError::Io)?)
        };
        view_vcf(&self.input, &mut out, &opts)?;
        Ok(())
    }
}

pub static HELP: HelpSpec = HelpSpec {
    name: META.name,
    version: META.version,
    tagline: "View VCF header, count variants, or filter by region.",
    origin: Some(Origin {
        upstream: "bcftools view",
        upstream_license: "MIT",
        our_license: "MIT OR Apache-2.0",
        paper_doi: None,
    }),
    usage_lines: &["<input.vcf> [-H] [--no-header] [-c] [-r chr1]"],
    sections: &[Section {
        title: "OPTIONS",
        flags: &[
            FlagSpec {
                short: Some('H'),
                long: "header-only",
                aliases: &[],
                value: None,
                type_hint: None,
                required: false,
                default: None,
                description: "Print only the VCF header.",
                why_default: None,
            },
            FlagSpec {
                short: Some('c'),
                long: "count",
                aliases: &[],
                value: None,
                type_hint: None,
                required: false,
                default: None,
                description: "Only count variant records.",
                why_default: None,
            },
            FlagSpec {
                short: Some('r'),
                long: "region",
                aliases: &[],
                value: Some("<chrom>"),
                type_hint: Some("String"),
                required: false,
                default: None,
                description: "Filter to this chromosome.",
                why_default: None,
            },
        ],
    }],
    examples: &[
        Example {
            description: "View header only",
            command: "rsomics-vcf-view input.vcf -H",
        },
        Example {
            description: "Count variants on chr1",
            command: "rsomics-vcf-view input.vcf -c -r chr1",
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
