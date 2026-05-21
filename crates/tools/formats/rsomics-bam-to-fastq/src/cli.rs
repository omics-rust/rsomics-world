use std::path::PathBuf;

use clap::Parser;
use rsomics_common::{CommonFlags, Result, RsomicsError, Tool, ToolMeta};
use rsomics_help::{Example, FlagSpec, HelpSpec, Origin, Section};

use rsomics_bam_to_fastq::bam_to_fastq;

pub const META: ToolMeta = ToolMeta {
    name: env!("CARGO_PKG_NAME"),
    version: env!("CARGO_PKG_VERSION"),
};

#[derive(Parser, Debug)]
#[command(
    name = "rsomics-bam-to-fastq",
    version,
    about,
    long_about = None,
    disable_help_flag = true
)]
pub struct Cli {
    /// Input BAM file.
    pub input: PathBuf,

    /// Output FASTQ file (default stdout).
    #[arg(short = 'o', long = "output", default_value = "-")]
    output: String,

    /// Include secondary and supplementary alignments.
    #[arg(long = "include-secondary")]
    include_secondary: bool,

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

        let count = bam_to_fastq(&self.input, &mut out, self.include_secondary)?;

        if !self.common.quiet {
            eprintln!("{count} reads extracted");
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
    tagline: "Extract FASTQ reads from BAM.",
    origin: Some(Origin {
        upstream: "samtools fastq",
        upstream_license: "MIT",
        our_license: "MIT OR Apache-2.0",
        paper_doi: Some("10.1093/bioinformatics/btp352"),
    }),
    usage_lines: &["<input.bam> [-o output.fastq]"],
    sections: &[Section {
        title: "OPTIONS",
        flags: &[FlagSpec {
            short: None,
            long: "include-secondary",
            aliases: &[],
            value: None,
            type_hint: None,
            required: false,
            default: None,
            description: "Include secondary/supplementary alignments.",
            why_default: None,
        }],
    }],
    examples: &[Example {
        description: "Extract FASTQ from BAM",
        command: "rsomics-bam-to-fastq input.bam -o reads.fastq",
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
