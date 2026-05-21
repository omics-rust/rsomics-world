use std::path::PathBuf;

use clap::Parser;
use rsomics_common::{CommonFlags, Result, RsomicsError, Tool, ToolMeta};
use rsomics_help::{Example, FlagSpec, HelpSpec, Origin, Section};

use rsomics_bam_markdup::{MarkdupOpts, markdup};

pub const META: ToolMeta = ToolMeta {
    name: env!("CARGO_PKG_NAME"),
    version: env!("CARGO_PKG_VERSION"),
};

#[derive(Parser, Debug)]
#[command(
    name = "rsomics-bam-markdup",
    version,
    about,
    long_about = None,
    disable_help_flag = true
)]
pub struct Cli {
    /// Input sorted BAM file.
    pub input: PathBuf,

    /// Output BAM file (default stdout).
    #[arg(short = 'o', long = "output", default_value = "-")]
    output: String,

    /// Remove duplicates instead of marking them.
    #[arg(short = 'r', long = "remove")]
    remove: bool,

    #[command(flatten)]
    pub common: CommonFlags,
}

impl Cli {
    pub fn execute(self) -> Result<()> {
        let opts = MarkdupOpts {
            remove: self.remove,
        };

        let mut out: Box<dyn std::io::Write> = if self.output == "-" {
            Box::new(std::io::stdout().lock())
        } else {
            Box::new(std::fs::File::create(&self.output).map_err(RsomicsError::Io)?)
        };

        let stats = markdup(&self.input, &mut out, &opts)?;

        if self.common.json {
            eprintln!(
                "{}",
                serde_json::to_string(&stats)
                    .map_err(|e| RsomicsError::InvalidInput(format!("JSON: {e}")))?
            );
        } else {
            eprintln!(
                "{} total, {} marked, {} removed",
                stats.total, stats.duplicates_marked, stats.duplicates_removed
            );
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
    tagline: "Mark or remove PCR/optical duplicates in sorted BAM.",
    origin: Some(Origin {
        upstream: "samtools markdup / picard MarkDuplicates",
        upstream_license: "MIT",
        our_license: "MIT OR Apache-2.0",
        paper_doi: Some("10.1093/bioinformatics/btp352"),
    }),
    usage_lines: &["<input.bam> [-o output.bam] [-r]"],
    sections: &[Section {
        title: "OPTIONS",
        flags: &[FlagSpec {
            short: Some('r'),
            long: "remove",
            aliases: &[],
            value: None,
            type_hint: None,
            required: false,
            default: None,
            description: "Remove duplicates instead of marking.",
            why_default: None,
        }],
    }],
    examples: &[
        Example {
            description: "Mark duplicates",
            command: "rsomics-bam-markdup sorted.bam -o marked.bam",
        },
        Example {
            description: "Remove duplicates",
            command: "rsomics-bam-markdup sorted.bam -r -o deduped.bam",
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
