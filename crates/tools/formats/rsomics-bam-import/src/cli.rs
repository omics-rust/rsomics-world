use std::path::PathBuf;

use clap::Parser;
use rsomics_common::{CommonFlags, Result, RsomicsError, Tool, ToolMeta};
use rsomics_help::{Example, FlagSpec, HelpSpec, Origin, Section};

use rsomics_bam_import::{ImportMode, ImportOpts, import};

pub const META: ToolMeta = ToolMeta {
    name: env!("CARGO_PKG_NAME"),
    version: env!("CARGO_PKG_VERSION"),
};

#[derive(Parser, Debug)]
#[command(
    name = "rsomics-bam-import",
    version,
    about,
    long_about = None,
    disable_help_flag = true
)]
pub struct Cli {
    /// Single-end or interleaved FASTQ (or positional file(s)).
    #[arg(short = 's', long = "interleaved")]
    interleaved: Option<PathBuf>,

    /// Single-ended reads (no pairing flags set).
    #[arg(short = '0', long = "single")]
    single: Option<PathBuf>,

    /// Read-1 of a paired run.
    #[arg(short = '1', long = "read1")]
    read1: Option<PathBuf>,

    /// Read-2 of a paired run.
    #[arg(short = '2', long = "read2")]
    read2: Option<PathBuf>,

    /// Output BAM file (default stdout as SAM).
    #[arg(short = 'o', long = "output")]
    output: Option<PathBuf>,

    /// Write uncompressed BAM (or SAM when -o is absent).
    #[arg(short = 'u', long = "uncompressed", default_value_t = false)]
    uncompressed: bool,

    /// Add a complete @RG line (may be repeated; separate fields with tabs).
    #[arg(short = 'r', long = "rg-line", action = clap::ArgAction::Append)]
    rg_line: Vec<String>,

    /// Add a minimal @RG line with ID only.
    #[arg(short = 'R', long = "rg-id")]
    rg_id: Option<String>,

    /// Positional FASTQ inputs (one → SE or interleaved-detect; two → paired).
    #[arg(trailing_var_arg = true)]
    inputs: Vec<PathBuf>,

    #[command(flatten)]
    pub common: CommonFlags,
}

impl Cli {
    pub fn execute(self) -> Result<()> {
        // Resolve the read group line: -r fields are tab-joined, -R is a simple ID.
        let rg_line: Option<String> = if self.rg_line.is_empty() {
            self.rg_id.as_ref().map(|id| format!("@RG\tID:{id}"))
        } else {
            let joined = self.rg_line.join("\t");
            // samtools prepends @RG if the user omitted it
            if joined.starts_with("@RG") {
                Some(joined)
            } else {
                Some(format!("@RG\t{joined}"))
            }
        };

        let mode = resolve_mode(
            self.interleaved.as_deref(),
            self.single.as_deref(),
            self.read1.as_deref(),
            self.read2.as_deref(),
            &self.inputs,
        )?;

        let workers = self
            .common
            .threads
            .and_then(std::num::NonZero::new)
            .unwrap_or_else(|| {
                std::thread::available_parallelism().unwrap_or(std::num::NonZero::<usize>::MIN)
            });

        let opts = ImportOpts {
            mode,
            output: self.output,
            uncompressed: self.uncompressed,
            rg_line,
            workers,
        };

        let stats = import(&opts)?;

        if self.common.json {
            eprintln!(
                "{}",
                serde_json::to_string(&stats)
                    .map_err(|e| RsomicsError::InvalidInput(format!("JSON: {e}")))?
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

/// Resolve mode from explicit flags and positional args, mirroring samtools
/// import's auto-detect: one positional → SE (or interleaved by read-name
/// sniff at parse time, handled in the library); two positionals → paired.
fn resolve_mode(
    interleaved: Option<&std::path::Path>,
    single: Option<&std::path::Path>,
    read1: Option<&std::path::Path>,
    read2: Option<&std::path::Path>,
    inputs: &[PathBuf],
) -> Result<ImportMode> {
    if let (Some(r1), Some(r2)) = (read1, read2) {
        return Ok(ImportMode::Paired(r1.to_path_buf(), r2.to_path_buf()));
    }
    if let Some(p) = interleaved {
        return Ok(ImportMode::Interleaved(p.to_path_buf()));
    }
    if let Some(p) = single {
        return Ok(ImportMode::Single(p.to_path_buf()));
    }
    match inputs.len() {
        0 => Err(RsomicsError::InvalidInput(
            "no input files; supply -0, -s, -1/-2, or positional FASTQ paths".into(),
        )),
        1 => Ok(ImportMode::Single(inputs[0].clone())),
        2 => Ok(ImportMode::Paired(inputs[0].clone(), inputs[1].clone())),
        n => Err(RsomicsError::InvalidInput(format!(
            "expected 1 or 2 positional inputs, got {n}"
        ))),
    }
}

pub static HELP: HelpSpec = HelpSpec {
    name: META.name,
    version: META.version,
    tagline: "Convert FASTQ reads to unaligned BAM.",
    origin: Some(Origin {
        upstream: "samtools import",
        upstream_license: "MIT",
        our_license: "MIT OR Apache-2.0",
        paper_doi: None,
    }),
    usage_lines: &["[-0|-s|-1 R1 -2 R2] [-o out.bam] [-r RG_FIELDS] [file.fastq ...]"],
    sections: &[Section {
        title: "OPTIONS",
        flags: &[
            FlagSpec {
                short: Some('s'),
                long: "interleaved",
                aliases: &[],
                value: Some("FILE"),
                type_hint: None,
                required: false,
                default: None,
                description: "Interleaved paired FASTQ (R1 and R2 alternating).",
                why_default: None,
            },
            FlagSpec {
                short: Some('0'),
                long: "single",
                aliases: &[],
                value: Some("FILE"),
                type_hint: None,
                required: false,
                default: None,
                description: "Single-ended reads (no pairing flags).",
                why_default: None,
            },
            FlagSpec {
                short: Some('1'),
                long: "read1",
                aliases: &[],
                value: Some("FILE"),
                type_hint: None,
                required: false,
                default: None,
                description: "Read-1 of a paired run.",
                why_default: None,
            },
            FlagSpec {
                short: Some('2'),
                long: "read2",
                aliases: &[],
                value: Some("FILE"),
                type_hint: None,
                required: false,
                default: None,
                description: "Read-2 of a paired run.",
                why_default: None,
            },
            FlagSpec {
                short: Some('o'),
                long: "output",
                aliases: &[],
                value: Some("FILE"),
                type_hint: None,
                required: false,
                default: Some("stdout (SAM)"),
                description: "Output BAM file.",
                why_default: None,
            },
            FlagSpec {
                short: Some('r'),
                long: "rg-line",
                aliases: &[],
                value: Some("STR"),
                type_hint: None,
                required: false,
                default: None,
                description: "RG fields (e.g. 'ID:lib1\\tSM:sample'). Repeatable.",
                why_default: None,
            },
            FlagSpec {
                short: Some('R'),
                long: "rg-id",
                aliases: &[],
                value: Some("STR"),
                type_hint: None,
                required: false,
                default: None,
                description: "Add a minimal @RG line with just ID:STR.",
                why_default: None,
            },
            FlagSpec {
                short: Some('u'),
                long: "uncompressed",
                aliases: &[],
                value: None,
                type_hint: None,
                required: false,
                default: Some("false"),
                description: "Write uncompressed BAM.",
                why_default: None,
            },
        ],
    }],
    examples: &[
        Example {
            description: "Single-end FASTQ to BAM",
            command: "rsomics-bam-import -o out.bam reads.fastq",
        },
        Example {
            description: "Paired-end FASTQ to BAM with read group",
            command: "rsomics-bam-import -1 r1.fastq -2 r2.fastq -r 'ID:lib1\tSM:s1' -o out.bam",
        },
        Example {
            description: "Interleaved FASTQ to BAM",
            command: "rsomics-bam-import -s interleaved.fastq -o out.bam",
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
