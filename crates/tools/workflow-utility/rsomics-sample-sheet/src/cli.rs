use std::path::PathBuf;

use clap::Parser;
use rsomics_common::{CommonFlags, Result, RsomicsError, Tool, ToolMeta};
use rsomics_help::{Example, HelpSpec};

use rsomics_sample_sheet::validate_sample_sheet;

pub const META: ToolMeta = ToolMeta {
    name: env!("CARGO_PKG_NAME"),
    version: env!("CARGO_PKG_VERSION"),
};

#[derive(Parser, Debug)]
#[command(name = "rsomics-sample-sheet", version, about, long_about = None, disable_help_flag = true)]
pub struct Cli {
    pub input: PathBuf,
    #[arg(short = 'o', long, default_value = "-")]
    output: String,
    #[command(flatten)]
    pub common: CommonFlags,
}

impl Tool for Cli {
    fn meta() -> ToolMeta {
        META
    }
    fn common(&self) -> &CommonFlags {
        &self.common
    }

    fn execute(self) -> Result<()> {
        let mut out: Box<dyn std::io::Write> = if self.output == "-" {
            Box::new(std::io::stdout().lock())
        } else {
            Box::new(std::fs::File::create(&self.output).map_err(RsomicsError::Io)?)
        };
        let entries = validate_sample_sheet(&self.input, &mut out)?;
        let valid = entries.iter().filter(|e| e.valid).count();
        let invalid = entries.len() - valid;
        if !self.common.quiet {
            eprintln!(
                "{} samples: {valid} valid, {invalid} invalid",
                entries.len()
            );
        }
        if invalid > 0 {
            return Err(RsomicsError::InvalidInput(format!(
                "{invalid} samples failed validation"
            )));
        }
        Ok(())
    }
}

pub static HELP: HelpSpec = HelpSpec {
    name: env!("CARGO_PKG_NAME"),
    version: env!("CARGO_PKG_VERSION"),
    tagline: "Validate sample sheets — check paths, detect PE, report errors.",
    origin: None,
    usage_lines: &["<samples.tsv> [-o report.tsv]"],
    sections: &[],
    examples: &[Example {
        description: "Validate a sample sheet",
        command: "rsomics-sample-sheet samples.tsv",
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
