use std::path::PathBuf;

use clap::Parser;
use rsomics_common::{CommonFlags, Result, RsomicsError, Tool, ToolMeta};
use rsomics_help::{Example, FlagSpec, HelpSpec, Origin, Section};

use rsomics_bam_reheader::{ReheaderOpts, reheader};

pub const META: ToolMeta = ToolMeta {
    name: env!("CARGO_PKG_NAME"),
    version: env!("CARGO_PKG_VERSION"),
};

#[derive(Parser, Debug)]
#[command(
    name = "rsomics-bam-reheader",
    version,
    about,
    long_about = None,
    disable_help_flag = true
)]
pub struct Cli {
    /// Replacement header in SAM text format.
    pub header: PathBuf,

    /// Input BAM file whose records are kept.
    pub input: PathBuf,

    /// Output BAM file (default stdout).
    #[arg(short = 'o', long = "output", default_value = "-")]
    output: String,

    /// Omit the @PG provenance line.
    #[arg(short = 'P', long = "no-PG")]
    no_pg: bool,

    #[command(flatten)]
    pub common: CommonFlags,
}

impl Cli {
    pub fn execute(self) -> Result<()> {
        let opts = ReheaderOpts {
            header_file: self.header,
            no_pg: self.no_pg,
        };

        let output_path = (self.output != "-").then(|| PathBuf::from(&self.output));
        let stats = reheader(&self.input, output_path.as_deref(), &opts)?;

        if self.common.json {
            eprintln!(
                "{}",
                serde_json::to_string(&stats)
                    .map_err(|e| RsomicsError::InvalidInput(format!("JSON: {e}")))?
            );
        } else if !self.common.quiet {
            eprintln!("header replaced ({} lines)", stats.header_lines);
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
    tagline: "Replace a BAM header, passing alignment blocks through verbatim.",
    origin: Some(Origin {
        upstream: "samtools reheader",
        upstream_license: "MIT",
        our_license: "MIT OR Apache-2.0",
        paper_doi: None,
    }),
    usage_lines: &["<new.header.sam> <in.bam> [-o out.bam] [-P]"],
    sections: &[Section {
        title: "OPTIONS",
        flags: &[
            FlagSpec {
                short: Some('o'),
                long: "output",
                aliases: &[],
                value: Some("FILE"),
                type_hint: None,
                required: false,
                default: Some("stdout"),
                description: "Output BAM file.",
                why_default: None,
            },
            FlagSpec {
                short: Some('P'),
                long: "no-PG",
                aliases: &[],
                value: None,
                type_hint: None,
                required: false,
                default: None,
                description: "Omit the @PG provenance line.",
                why_default: None,
            },
        ],
    }],
    examples: &[Example {
        description: "Replace a BAM's header",
        command: "rsomics-bam-reheader new_header.sam in.bam -o out.bam",
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
