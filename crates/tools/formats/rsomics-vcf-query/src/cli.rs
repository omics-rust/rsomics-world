use std::path::PathBuf;

use clap::Parser;
use rsomics_common::{CommonFlags, Result, RsomicsError, Tool, ToolMeta};
use rsomics_help::{Example, FlagSpec, HelpSpec, Origin, Section};

use rsomics_vcf_query::query_vcf;

pub const META: ToolMeta = ToolMeta {
    name: env!("CARGO_PKG_NAME"),
    version: env!("CARGO_PKG_VERSION"),
};

#[derive(Parser, Debug)]
#[command(
    name = "rsomics-vcf-query",
    version,
    about,
    long_about = None,
    disable_help_flag = true
)]
pub struct Cli {
    /// Input VCF file.
    pub input: PathBuf,

    /// Output file (default stdout).
    #[arg(short = 'o', long = "output", default_value = "-")]
    output: String,

    /// Fields to extract (comma-separated: CHROM,POS,REF,ALT,QUAL,FILTER,ID).
    #[arg(short = 'f', long = "fields", default_value = "CHROM,POS,REF,ALT")]
    fields: String,

    #[command(flatten)]
    pub common: CommonFlags,
}

impl Cli {
    pub fn execute(self) -> Result<()> {
        let fields: Vec<String> = self.fields.split(',').map(|s| s.trim().to_string()).collect();

        let mut out: Box<dyn std::io::Write> = if self.output == "-" {
            Box::new(std::io::stdout().lock())
        } else {
            Box::new(std::fs::File::create(&self.output).map_err(RsomicsError::Io)?)
        };

        let count = query_vcf(&self.input, &mut out, &fields)?;

        if !self.common.quiet {
            eprintln!("{count} records");
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
    tagline: "Extract fields from VCF records.",
    origin: Some(Origin {
        upstream: "bcftools query",
        upstream_license: "MIT",
        our_license: "MIT OR Apache-2.0",
        paper_doi: Some("10.1093/bioinformatics/btp352"),
    }),
    usage_lines: &["<input.vcf> [-f CHROM,POS,REF,ALT] [-o output.tsv]"],
    sections: &[Section {
        title: "OPTIONS",
        flags: &[FlagSpec {
            short: Some('f'),
            long: "fields",
            aliases: &[],
            value: Some("<fields>"),
            type_hint: Some("String"),
            required: false,
            default: Some("CHROM,POS,REF,ALT"),
            description: "Comma-separated fields to extract.",
            why_default: None,
        }],
    }],
    examples: &[
        Example {
            description: "Extract CHROM, POS, REF, ALT",
            command: "rsomics-vcf-query input.vcf",
        },
        Example {
            description: "Extract specific fields",
            command: "rsomics-vcf-query input.vcf -f CHROM,POS,ID,QUAL",
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
