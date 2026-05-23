use std::path::PathBuf;

use clap::Parser;
use rsomics_common::{CommonFlags, Result, RsomicsError, Tool, ToolMeta};
use rsomics_help::{Example, FlagSpec, HelpSpec, Origin, Section};

use rsomics_bam_calmd::{CalmdOpts, calmd};

pub const META: ToolMeta = ToolMeta {
    name: env!("CARGO_PKG_NAME"),
    version: env!("CARGO_PKG_VERSION"),
};

#[derive(Parser, Debug)]
#[command(
    name = "rsomics-bam-calmd",
    version,
    about,
    long_about = None,
    disable_help_flag = true
)]
pub struct Cli {
    /// Input BAM file (coordinate-sorted for single-pass reference reuse).
    pub input: PathBuf,

    /// Reference FASTA. Must be indexed (`<ref>.fai` alongside it).
    pub reference: PathBuf,

    /// Output BAM file (default stdout).
    #[arg(short = 'o', long = "output", default_value = "-")]
    output: String,

    /// Rewrite reference-matching read bases as `=` in the output SEQ.
    #[arg(short = 'e', long = "convert-equal")]
    use_equal: bool,

    #[command(flatten)]
    pub common: CommonFlags,
}

impl Cli {
    pub fn execute(self) -> Result<()> {
        let opts = CalmdOpts {
            use_equal: self.use_equal,
        };

        let workers = std::num::NonZero::new(self.common.thread_count())
            .unwrap_or(std::num::NonZero::<usize>::MIN);

        let output_path = (self.output != "-").then(|| std::path::PathBuf::from(&self.output));
        let stats = calmd(
            &self.input,
            &self.reference,
            output_path.as_deref(),
            &opts,
            workers,
        )?;

        if self.common.json {
            eprintln!(
                "{}",
                serde_json::to_string(&stats)
                    .map_err(|e| RsomicsError::InvalidInput(format!("JSON: {e}")))?
            );
        } else {
            eprintln!(
                "{} records, {} recomputed, {} skipped (no seq), {} missing ref",
                stats.records, stats.computed, stats.no_sequence, stats.missing_ref
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
    tagline: "Recompute the MD and NM aux tags against a reference FASTA.",
    origin: Some(Origin {
        upstream: "samtools calmd",
        upstream_license: "MIT",
        our_license: "MIT OR Apache-2.0",
        paper_doi: None,
    }),
    usage_lines: &["<input.bam> <ref.fasta> [-o output.bam] [-e]"],
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
                default: Some("-"),
                description: "Output BAM file (default stdout).",
                why_default: None,
            },
            FlagSpec {
                short: Some('e'),
                long: "convert-equal",
                aliases: &[],
                value: None,
                type_hint: None,
                required: false,
                default: None,
                description: "Rewrite reference-matching read bases as '=' in the output SEQ.",
                why_default: None,
            },
        ],
    }],
    examples: &[
        Example {
            description: "Recompute MD/NM against a reference",
            command: "rsomics-bam-calmd aln.bam ref.fa -o calmd.bam",
        },
        Example {
            description: "Convert matching bases to '=' (compact storage)",
            command: "rsomics-bam-calmd aln.bam ref.fa -e -o calmd.bam",
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
