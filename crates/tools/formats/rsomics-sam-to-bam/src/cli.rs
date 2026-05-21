use std::path::PathBuf;

use clap::Parser;
use rsomics_common::{CommonFlags, Result, RsomicsError, Tool, ToolMeta};
use rsomics_help::{Example, HelpSpec, Origin, Section};

use rsomics_sam_to_bam::convert;

pub const META: ToolMeta = ToolMeta {
    name: env!("CARGO_PKG_NAME"),
    version: env!("CARGO_PKG_VERSION"),
};

#[derive(Parser, Debug)]
#[command(
    name = "rsomics-sam-to-bam",
    version,
    about,
    long_about = None,
    disable_help_flag = true
)]
pub struct Cli {
    /// Input SAM file (- for stdin).
    #[arg(default_value = "-")]
    pub input: PathBuf,

    /// Output BAM file (default stdout).
    #[arg(short = 'o', long = "output", default_value = "-")]
    output: String,

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

        let count = convert(&self.input, &mut out)?;

        if !self.common.quiet {
            eprintln!("{count} records converted");
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
    tagline: "Convert SAM to BAM.",
    origin: Some(Origin {
        upstream: "samtools view -bS",
        upstream_license: "MIT",
        our_license: "MIT OR Apache-2.0",
        paper_doi: Some("10.1093/bioinformatics/btp352"),
    }),
    usage_lines: &["[input.sam] [-o output.bam]"],
    sections: &[Section {
        title: "OPTIONS",
        flags: &[],
    }],
    examples: &[
        Example {
            description: "Convert SAM to BAM",
            command: "rsomics-sam-to-bam input.sam -o output.bam",
        },
        Example {
            description: "Pipe from aligner",
            command: "aligner ... | rsomics-sam-to-bam -o output.bam",
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
