use std::path::PathBuf;

use clap::Parser;
use rsomics_common::{CommonFlags, Result, RsomicsError, Tool, ToolMeta};
use rsomics_help::{Example, FlagSpec, HelpSpec, Origin, Section};

use rsomics_bam_ampliconclip::{ClipOpts, Clipping, ampliconclip};

pub const META: ToolMeta = ToolMeta {
    name: env!("CARGO_PKG_NAME"),
    version: env!("CARGO_PKG_VERSION"),
};

#[derive(Parser, Debug)]
#[command(
    name = "rsomics-bam-ampliconclip",
    version,
    about,
    long_about = None,
    disable_help_flag = true
)]
pub struct Cli {
    /// Input coordinate-sorted BAM file.
    pub input: PathBuf,

    /// BED file of primer regions to clip.
    #[arg(short = 'b', long = "bed")]
    bed: PathBuf,

    /// Output BAM file (default stdout).
    #[arg(short = 'o', long = "output", default_value = "-")]
    output: String,

    /// Hard clip primers (remove SEQ/QUAL) instead of soft clip (default).
    #[arg(long = "hard-clip")]
    hard_clip: bool,

    /// Clip on both 5' and 3' ends.
    #[arg(long = "both-ends")]
    both_ends: bool,

    /// Use strand data from the BED to match read direction.
    #[arg(long = "strand")]
    strand: bool,

    /// Match a region within this many bases (default 5).
    #[arg(long = "tolerance", default_value_t = 5)]
    tolerance: i64,

    /// Mark unclipped, mapped reads as QCFAIL.
    #[arg(long = "fail")]
    fail: bool,

    /// Only output clipped reads.
    #[arg(long = "clipped")]
    clipped: bool,

    /// Do not write excluded (unmapped or QCFAIL) reads.
    #[arg(long = "no-excluded")]
    no_excluded: bool,

    /// Do not output reads this size or shorter (active query length).
    #[arg(long = "filter-len", default_value_t = -1)]
    filter_len: i64,

    /// Mark as QCFAIL reads this size or shorter (active query length).
    #[arg(long = "fail-len", default_value_t = -1)]
    fail_len: i64,

    /// Unmap reads this size or shorter (active query length, default 0).
    #[arg(long = "unmap-len", default_value_t = 0)]
    unmap_len: i64,

    /// Keep the NM and MD tags on clipped reads (default deletes them).
    #[arg(long = "keep-tag")]
    keep_tag: bool,

    /// Do not add a @PG line to the header.
    #[arg(long = "no-PG")]
    no_pg: bool,

    #[command(flatten)]
    pub common: CommonFlags,
}

impl Cli {
    pub fn execute(self) -> Result<()> {
        if self.tolerance < 0 {
            return Err(RsomicsError::InvalidInput(format!(
                "invalid tolerance of {}, must be >= 0",
                self.tolerance
            )));
        }

        let opts = ClipOpts {
            clipping: if self.hard_clip {
                Clipping::Hard
            } else {
                Clipping::Soft
            },
            both_ends: self.both_ends,
            use_strand: self.strand,
            tolerance: self.tolerance,
            mark_fail: self.fail,
            write_clipped: self.clipped,
            no_excluded: self.no_excluded,
            fail_len: self.fail_len,
            filter_len: self.filter_len,
            unmap_len: self.unmap_len,
            add_pg: !self.no_pg,
            keep_tag: self.keep_tag,
        };

        let arg_list = std::env::args().collect::<Vec<_>>().join(" ");

        let workers = std::num::NonZero::new(self.common.thread_count())
            .unwrap_or(std::num::NonZero::<usize>::MIN);

        let output_path = (self.output != "-").then(|| std::path::PathBuf::from(&self.output));
        let stats = ampliconclip(
            &self.input,
            output_path.as_deref(),
            &self.bed,
            &opts,
            &arg_list,
            workers,
        )?;

        if self.common.json {
            eprintln!(
                "{}",
                serde_json::to_string(&stats)
                    .map_err(|e| RsomicsError::InvalidInput(format!("JSON: {e}")))?
            );
        } else if !self.common.quiet {
            eprintln!(
                "TOTAL READS: {}\nTOTAL CLIPPED: {}\nFORWARD CLIPPED: {}\nREVERSE CLIPPED: {}\n\
                 BOTH CLIPPED: {}\nNOT CLIPPED: {}\nEXCLUDED: {}\nFILTERED: {}\nFAILED: {}\nWRITTEN: {}",
                stats.total,
                stats.clipped,
                stats.forward_clipped,
                stats.reverse_clipped,
                stats.both_clipped,
                stats.not_clipped,
                stats.excluded,
                stats.filtered,
                stats.failed,
                stats.written,
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
    tagline: "Clip amplicon primer regions off aligned reads given a primer BED.",
    origin: Some(Origin {
        upstream: "samtools ampliconclip",
        upstream_license: "MIT",
        our_license: "MIT OR Apache-2.0",
        paper_doi: None,
    }),
    usage_lines: &["-b primers.bed <input.bam> [-o output.bam] [--hard-clip] [--both-ends]"],
    sections: &[Section {
        title: "OPTIONS",
        flags: &[
            FlagSpec {
                short: Some('b'),
                long: "bed",
                aliases: &[],
                value: Some("FILE"),
                type_hint: None,
                required: true,
                default: None,
                description: "BED file of primer regions to clip.",
                why_default: None,
            },
            FlagSpec {
                short: Some('o'),
                long: "output",
                aliases: &[],
                value: Some("FILE"),
                type_hint: None,
                required: false,
                default: Some("-"),
                description: "Output BAM file.",
                why_default: Some("stdout"),
            },
            FlagSpec {
                short: None,
                long: "hard-clip",
                aliases: &[],
                value: None,
                type_hint: None,
                required: false,
                default: None,
                description: "Hard clip primers (remove SEQ/QUAL) instead of the default soft clip.",
                why_default: None,
            },
            FlagSpec {
                short: None,
                long: "both-ends",
                aliases: &[],
                value: None,
                type_hint: None,
                required: false,
                default: None,
                description: "Clip on both the 5' and 3' ends.",
                why_default: None,
            },
            FlagSpec {
                short: None,
                long: "strand",
                aliases: &[],
                value: None,
                type_hint: None,
                required: false,
                default: None,
                description: "Use strand data from the BED to match read direction.",
                why_default: None,
            },
            FlagSpec {
                short: None,
                long: "tolerance",
                aliases: &[],
                value: Some("INT"),
                type_hint: None,
                required: false,
                default: Some("5"),
                description: "Match a region within this many bases.",
                why_default: None,
            },
            FlagSpec {
                short: None,
                long: "no-PG",
                aliases: &[],
                value: None,
                type_hint: None,
                required: false,
                default: None,
                description: "Do not add a @PG line to the header.",
                why_default: None,
            },
        ],
    }],
    examples: &[
        Example {
            description: "Soft clip primers from the 5' end (default)",
            command: "rsomics-bam-ampliconclip -b primers.bed in.bam -o clipped.bam",
        },
        Example {
            description: "Hard clip both ends",
            command: "rsomics-bam-ampliconclip -b primers.bed --hard-clip --both-ends in.bam -o clipped.bam",
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
