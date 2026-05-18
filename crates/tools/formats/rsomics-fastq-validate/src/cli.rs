use std::path::PathBuf;

use clap::Parser;
use rsomics_common::{CommonFlags, Result, ToolMeta};
use rsomics_help::{Example, HelpSpec, Origin, Section};

use rsomics_fastq_validate::validate_fastq;

pub const META: ToolMeta = ToolMeta {
    name: env!("CARGO_PKG_NAME"),
    version: env!("CARGO_PKG_VERSION"),
};

#[derive(Parser, Debug)]
#[command(name = "rsomics-fastq-validate", version, about, long_about = None, disable_help_flag = true)]
pub struct Cli {
    pub input: PathBuf,
    #[command(flatten)]
    pub common: CommonFlags,
}

impl Cli {
    pub fn execute(self) -> Result<()> {
        let result = validate_fastq(&self.input)?;
        if result.is_valid {
            eprintln!("OK: {} records, no errors", result.records);
        } else {
            eprintln!(
                "INVALID: {} records, {} errors:",
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
    tagline: "Validate FASTQ format integrity.",
    origin: Some(Origin {
        upstream: "fastq_info / fqtools validate",
        upstream_license: "MIT",
        our_license: "MIT OR Apache-2.0",
        paper_doi: None,
    }),
    usage_lines: &["<input.fastq>"],
    sections: &[Section {
        title: "OPTIONS",
        flags: &[],
    }],
    examples: &[Example {
        description: "Validate a FASTQ file",
        command: "rsomics-fastq-validate reads.fq",
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
