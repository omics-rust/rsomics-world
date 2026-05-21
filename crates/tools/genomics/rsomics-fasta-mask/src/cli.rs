use std::path::PathBuf;

use clap::Parser;
use rsomics_common::{CommonFlags, Result, RsomicsError, Tool, ToolMeta};
use rsomics_help::{Example, FlagSpec, HelpSpec, Origin, Section};

use rsomics_fasta_mask::{MaskMode, mask_fasta};

pub const META: ToolMeta = ToolMeta {
    name: env!("CARGO_PKG_NAME"),
    version: env!("CARGO_PKG_VERSION"),
};

#[derive(Parser, Debug)]
#[command(name = "rsomics-fasta-mask", version, about, long_about = None, disable_help_flag = true)]
pub struct Cli {
    pub fasta: PathBuf,
    #[arg(short = 'b', long)]
    bed: PathBuf,
    #[arg(long)]
    hard: bool,
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
        let mode = if self.hard {
            MaskMode::Hard
        } else {
            MaskMode::Soft
        };
        let mut out: Box<dyn std::io::Write> = if self.output == "-" {
            Box::new(std::io::stdout().lock())
        } else {
            Box::new(std::fs::File::create(&self.output).map_err(RsomicsError::Io)?)
        };
        let n = mask_fasta(&self.fasta, &self.bed, &mode, &mut out)?;
        if !self.common.quiet {
            eprintln!("{n} bases masked");
        }
        Ok(())
    }
}

pub static HELP: HelpSpec = HelpSpec {
    name: env!("CARGO_PKG_NAME"),
    version: env!("CARGO_PKG_VERSION"),
    tagline: "Mask FASTA sequences by BED regions — soft-mask (lowercase) or hard-mask (N).",
    origin: Some(Origin {
        upstream: "bedtools maskfasta",
        upstream_license: "MIT",
        our_license: "MIT OR Apache-2.0",
        paper_doi: None,
    }),
    usage_lines: &["<ref.fa> -b <regions.bed> [-o masked.fa] [--hard]"],
    sections: &[Section {
        title: "OPTIONS",
        flags: &[
            FlagSpec {
                short: Some('b'),
                long: "bed",
                aliases: &[],
                value: Some("<path>"),
                type_hint: Some("PathBuf"),
                required: true,
                default: None,
                description: "BED file with regions to mask.",
                why_default: None,
            },
            FlagSpec {
                short: None,
                long: "hard",
                aliases: &[],
                value: None,
                type_hint: Some("bool"),
                required: false,
                default: None,
                description: "Hard-mask with N instead of soft-mask (lowercase).",
                why_default: None,
            },
        ],
    }],
    examples: &[
        Example {
            description: "Soft-mask repeats",
            command: "rsomics-fasta-mask ref.fa -b repeats.bed -o masked.fa",
        },
        Example {
            description: "Hard-mask with N",
            command: "rsomics-fasta-mask ref.fa -b repeats.bed --hard -o masked.fa",
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
