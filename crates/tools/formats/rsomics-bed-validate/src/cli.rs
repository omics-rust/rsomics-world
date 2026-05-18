use std::path::PathBuf;

use clap::Parser;
use rsomics_common::{CommonFlags, Result, ToolMeta};
use rsomics_help::{Example, HelpSpec, Origin, Section};

use rsomics_bed_validate::validate_bed;

pub const META: ToolMeta = ToolMeta {
    name: env!("CARGO_PKG_NAME"),
    version: env!("CARGO_PKG_VERSION"),
};

#[derive(Parser, Debug)]
#[command(
    name = "rsomics-bed-validate",
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
        let result = validate_bed(&self.input)?;
        if result.is_valid {
            eprintln!("OK: {} intervals, no errors", result.records);
        } else {
            eprintln!(
                "INVALID: {} intervals, {} errors:",
                result.records,
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

pub static HELP: HelpSpec = HelpSpec {
    name: META.name,
    version: META.version,
    tagline: "Validate BED format integrity.",
    origin: Some(Origin {
        upstream: "bedtools / UCSC bedToBigBed validation",
        upstream_license: "MIT",
        our_license: "MIT OR Apache-2.0",
        paper_doi: None,
    }),
    usage_lines: &["<input.bed>"],
    sections: &[Section {
        title: "OPTIONS",
        flags: &[],
    }],
    examples: &[Example {
        description: "Validate a BED file",
        command: "rsomics-bed-validate intervals.bed",
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
