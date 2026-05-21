use std::path::PathBuf;

use clap::Parser;
use rsomics_common::{CommonFlags, Result, Tool, ToolMeta};
use rsomics_help::{Example, HelpSpec, Origin, Section};

use rsomics_fasta_validate::validate_fasta;

pub const META: ToolMeta = ToolMeta {
    name: env!("CARGO_PKG_NAME"),
    version: env!("CARGO_PKG_VERSION"),
};

#[derive(Parser, Debug)]
#[command(
    name = "rsomics-fasta-validate",
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
        let result = validate_fasta(&self.input)?;
        if result.is_valid {
            eprintln!("OK: {} sequences, no errors", result.sequences);
        } else {
            eprintln!(
                "INVALID: {} sequences, {} errors:",
                result.sequences,
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
    tagline: "Validate FASTA format integrity.",
    origin: Some(Origin {
        upstream: "biopython / seqkit seq --validate",
        upstream_license: "MIT",
        our_license: "MIT OR Apache-2.0",
        paper_doi: None,
    }),
    usage_lines: &["<input.fasta>"],
    sections: &[Section {
        title: "OPTIONS",
        flags: &[],
    }],
    examples: &[Example {
        description: "Validate a FASTA file",
        command: "rsomics-fasta-validate genome.fa",
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
