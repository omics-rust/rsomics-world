use std::path::PathBuf;

use clap::Parser;
use rsomics_common::{CommonFlags, Result, RsomicsError, Tool, ToolMeta};
use rsomics_help::{Example, FlagSpec, HelpSpec, Origin, Section};

use rsomics_bam_mpileup::{MpileupOpts, mpileup};

pub const META: ToolMeta = ToolMeta {
    name: env!("CARGO_PKG_NAME"),
    version: env!("CARGO_PKG_VERSION"),
};

#[derive(Parser, Debug)]
#[command(
    name = "rsomics-bam-mpileup",
    version,
    about,
    long_about = None,
    disable_help_flag = true
)]
pub struct Cli {
    /// Input coordinate-sorted BAM file.
    pub input: PathBuf,

    /// Reference FASTA (enables `.`/`,` ref-match encoding). Requires `-B`
    /// (BAQ is not implemented; the samtools default applies BAQ when `-f` is
    /// given, so byte-exact reference-aware output needs `-B`).
    #[arg(short = 'f', long = "fasta", value_name = "REF")]
    fasta: Option<PathBuf>,

    /// Output file (default stdout).
    #[arg(short = 'o', long = "output", default_value = "-")]
    output: String,

    /// Minimum base quality for a base to be counted.
    #[arg(short = 'Q', long = "min-BQ", default_value_t = 13)]
    min_baseq: u8,

    /// Minimum mapping quality for a read to be used.
    #[arg(long = "min-MQ", default_value_t = 0)]
    min_mapq: u8,

    /// Max per-position depth.
    #[arg(short = 'd', long = "max-depth", default_value_t = 8000)]
    max_depth: u32,

    /// Disable BAQ (per-base alignment quality) computation.
    #[arg(short = 'B', long = "no-BAQ")]
    no_baq: bool,

    /// Do not discard anomalous read pairs (disable orphan filtering).
    #[arg(short = 'A', long = "count-orphans")]
    count_orphans: bool,

    /// Disable read-pair overlap detection.
    #[arg(short = 'x', long = "ignore-overlaps")]
    ignore_overlaps: bool,

    /// Skip reads with any of these FLAG bits set (default UNMAP,SECONDARY,QCFAIL,DUP).
    #[arg(long = "ff", default_value_t = 0x704)]
    rflag_filter: u16,

    /// Require all of these FLAG bits set (default none).
    #[arg(long = "rf", default_value_t = 0)]
    rflag_require: u16,

    /// Output all positions (including zero depth); use twice for all references.
    #[arg(short = 'a', action = clap::ArgAction::Count)]
    all: u8,

    #[command(flatten)]
    pub common: CommonFlags,
}

impl Cli {
    pub fn execute(self) -> Result<()> {
        let opts = MpileupOpts {
            min_baseq: self.min_baseq,
            min_mapq: self.min_mapq,
            max_depth: self.max_depth,
            no_overlaps: self.ignore_overlaps,
            no_orphan_filter: self.count_orphans,
            rflag_filter: self.rflag_filter,
            rflag_require: self.rflag_require,
            output_all: self.all,
            no_baq: self.no_baq,
        };

        let workers = std::num::NonZero::new(self.common.thread_count())
            .unwrap_or(std::num::NonZero::<usize>::MIN);

        let stats = if self.output == "-" {
            let mut out = std::io::stdout().lock();
            mpileup(&self.input, self.fasta.as_deref(), &mut out, &opts, workers)?
        } else {
            let file = std::fs::File::create(&self.output).map_err(|e| {
                RsomicsError::InvalidInput(format!("creating {}: {e}", self.output))
            })?;
            let mut out = std::io::BufWriter::new(file);
            mpileup(&self.input, self.fasta.as_deref(), &mut out, &opts, workers)?
        };

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

pub static HELP: HelpSpec = HelpSpec {
    name: META.name,
    version: META.version,
    tagline: "Per-position text pileup of read bases, qualities and map qualities.",
    origin: Some(Origin {
        upstream: "samtools mpileup",
        upstream_license: "MIT",
        our_license: "MIT OR Apache-2.0",
        paper_doi: None,
    }),
    usage_lines: &["<input.bam> [-f ref.fa -B] [-Q min-baseq] [-o out.pileup]"],
    sections: &[Section {
        title: "OPTIONS",
        flags: &[
            FlagSpec {
                short: Some('f'),
                long: "fasta",
                aliases: &[],
                value: Some("REF"),
                type_hint: None,
                required: false,
                default: None,
                description: "Reference FASTA for .,/= ref-match encoding (needs -B).",
                why_default: None,
            },
            FlagSpec {
                short: Some('Q'),
                long: "min-BQ",
                aliases: &[],
                value: Some("INT"),
                type_hint: None,
                required: false,
                default: Some("13"),
                description: "Minimum base quality for a base to be counted.",
                why_default: None,
            },
            FlagSpec {
                short: Some('B'),
                long: "no-BAQ",
                aliases: &[],
                value: None,
                type_hint: None,
                required: false,
                default: None,
                description: "Disable BAQ (required for reference-aware output).",
                why_default: None,
            },
            FlagSpec {
                short: Some('A'),
                long: "count-orphans",
                aliases: &[],
                value: None,
                type_hint: None,
                required: false,
                default: None,
                description: "Do not discard anomalous read pairs.",
                why_default: None,
            },
            FlagSpec {
                short: Some('x'),
                long: "ignore-overlaps",
                aliases: &[],
                value: None,
                type_hint: None,
                required: false,
                default: None,
                description: "Disable read-pair overlap detection.",
                why_default: None,
            },
            FlagSpec {
                short: Some('a'),
                long: "all",
                aliases: &[],
                value: None,
                type_hint: None,
                required: false,
                default: None,
                description: "Output all positions (twice: all references).",
                why_default: None,
            },
        ],
    }],
    examples: &[
        Example {
            description: "Default text pileup",
            command: "rsomics-bam-mpileup sorted.bam",
        },
        Example {
            description: "Reference-aware (BAQ disabled for samtools-exact output)",
            command: "rsomics-bam-mpileup -f ref.fa -B sorted.bam",
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
