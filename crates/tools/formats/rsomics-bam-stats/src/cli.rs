use std::path::PathBuf;

use clap::Parser;
use rsomics_common::{CommonFlags, Result, RsomicsError, Tool, ToolMeta};
use rsomics_help::{Example, HelpSpec, Origin, Section};

use rsomics_bam_stats::{compute_stats, write_stats};

pub const META: ToolMeta = ToolMeta {
    name: env!("CARGO_PKG_NAME"),
    version: env!("CARGO_PKG_VERSION"),
};

#[derive(Parser, Debug)]
#[command(
    name = "rsomics-bam-stats",
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
        let stats = compute_stats(&self.input)?;

        let mut out: Box<dyn std::io::Write> = if self.output == "-" {
            Box::new(std::io::stdout().lock())
        } else {
            Box::new(std::fs::File::create(&self.output).map_err(RsomicsError::Io)?)
        };

        if self.common.json {
            serde_json::to_writer_pretty(&mut out, &stats)
                .map_err(|e| RsomicsError::InvalidInput(format!("JSON: {e}")))?;
            writeln!(out).map_err(RsomicsError::Io)?;
        } else {
            write_stats(&stats, &mut out)?;
        }

        Ok(())
    }
}

use std::io::Write;

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
    tagline: "Comprehensive alignment statistics from BAM.",
    origin: Some(Origin {
        upstream: "samtools stats",
        upstream_license: "MIT",
        our_license: "MIT OR Apache-2.0",
        paper_doi: Some("10.1093/bioinformatics/btp352"),
    }),
    usage_lines: &["<input.bam> [-o stats.txt] [--json]"],
    sections: &[Section {
        title: "OPTIONS",
        flags: &[],
    }],
    examples: &[
        Example {
            description: "Compute alignment statistics",
            command: "rsomics-bam-stats input.bam",
        },
        Example {
            description: "JSON output",
            command: "rsomics-bam-stats input.bam --json",
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
