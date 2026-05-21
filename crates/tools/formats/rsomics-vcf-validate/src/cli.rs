use std::path::PathBuf;

use clap::Parser;
use rsomics_common::{CommonFlags, Result, Tool, ToolMeta};
use rsomics_help::{Example, HelpSpec, Origin, Section};

use rsomics_vcf_validate::validate_vcf;

pub const META: ToolMeta = ToolMeta {
    name: env!("CARGO_PKG_NAME"),
    version: env!("CARGO_PKG_VERSION"),
};

#[derive(Parser, Debug)]
#[command(
    name = "rsomics-vcf-validate",
    version,
    about,
    long_about = None,
    disable_help_flag = true
)]
pub struct Cli {
    pub input: PathBuf,
    #[command(flatten)]
    pub common: CommonFlags,
}

impl Cli {
    pub fn execute(self) -> Result<()> {
        let result = validate_vcf(&self.input)?;
        if result.is_valid {
            eprintln!(
                "OK: {} variants, header={}, no errors",
                result.variants, result.has_header
            );
        } else {
            eprintln!(
                "INVALID: {} variants, {} errors:",
                result.variants,
                result.errors.len()
            );
            for err in &result.errors {
                eprintln!("  {err}");
            }
            return Err(rsomics_common::RsomicsError::InvalidInput(
                "validation failed".into(),
            ));
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
    tagline: "Validate VCF format integrity.",
    origin: Some(Origin {
        upstream: "bcftools / vcf-validator",
        upstream_license: "MIT",
        our_license: "MIT OR Apache-2.0",
        paper_doi: None,
    }),
    usage_lines: &["<input.vcf>"],
    sections: &[Section {
        title: "OPTIONS",
        flags: &[],
    }],
    examples: &[Example {
        description: "Validate a VCF file",
        command: "rsomics-vcf-validate variants.vcf",
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
