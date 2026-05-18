use std::path::PathBuf;

use clap::Parser;
use rsomics_common::{CommonFlags, Result, RsomicsError, ToolMeta};
use rsomics_help::{Example, HelpSpec, Origin, Section};

use rsomics_bam_coverage::{compute_coverage, write_coverage};

pub const META: ToolMeta = ToolMeta {
    name: env!("CARGO_PKG_NAME"),
    version: env!("CARGO_PKG_VERSION"),
};

#[derive(Parser, Debug)]
#[command(
    name = "rsomics-bam-coverage",
    version,
    about,
    long_about = None,
    disable_help_flag = true
)]
pub struct Cli {
    /// Input BAM file.
    pub input: PathBuf,

    /// Output file (default stdout).
    #[arg(short = 'o', long = "output", default_value = "-")]
    output: String,

    #[command(flatten)]
    pub common: CommonFlags,
}

impl Cli {
    pub fn execute(self) -> Result<()> {
        let cov = compute_coverage(&self.input)?;

        let mut out: Box<dyn std::io::Write> = if self.output == "-" {
            Box::new(std::io::stdout().lock())
        } else {
            Box::new(std::fs::File::create(&self.output).map_err(RsomicsError::Io)?)
        };

        if self.common.json {
            serde_json::to_writer_pretty(&mut out, &cov)
                .map_err(|e| RsomicsError::InvalidInput(format!("JSON: {e}")))?;
            writeln!(out).map_err(RsomicsError::Io)?;
        } else {
            write_coverage(&cov, &mut out)?;
        }

        Ok(())
    }
}

use std::io::Write;

pub static HELP: HelpSpec = HelpSpec {
    name: META.name,
    version: META.version,
    tagline: "Per-reference coverage histogram from BAM.",
    origin: Some(Origin {
        upstream: "samtools coverage",
        upstream_license: "MIT",
        our_license: "MIT OR Apache-2.0",
        paper_doi: Some("10.1093/bioinformatics/btp352"),
    }),
    usage_lines: &["<input.bam> [-o coverage.tsv] [--json]"],
    sections: &[Section {
        title: "OPTIONS",
        flags: &[],
    }],
    examples: &[Example {
        description: "Per-reference coverage",
        command: "rsomics-bam-coverage input.bam -o coverage.tsv",
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
