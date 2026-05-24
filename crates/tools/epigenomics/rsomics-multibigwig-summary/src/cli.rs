use std::path::PathBuf;

use clap::Parser;
use rsomics_common::{CommonFlags, Result, RsomicsError, Tool, ToolMeta};
use rsomics_help::{Example, FlagSpec, HelpSpec, Origin, Section};

use rsomics_multibigwig_summary::{
    DEFAULT_BIN_SIZE, SummaryOpts, summarize_bed, summarize_bins, write_raw_counts,
};

pub const META: ToolMeta = ToolMeta {
    name: env!("CARGO_PKG_NAME"),
    version: env!("CARGO_PKG_VERSION"),
};

#[derive(Parser, Debug)]
#[command(
    name = "rsomics-multibigwig-summary",
    version,
    about,
    long_about = None,
    disable_help_flag = true
)]
pub struct Cli {
    /// Input bigWig files (two or more), deeptools `-b/--bigwigfiles`.
    #[arg(short = 'b', long = "bigwigfiles", num_args = 1.., required = true)]
    pub bigwigfiles: Vec<PathBuf>,

    /// Per-bin / per-region mean-signal table (deeptools `--outRawCounts`).
    /// `-` for stdout. This is the value-exact oracle; the `.npz` matrix output
    /// is scoped out (it is an opaque archive consumed only by the plot tools).
    #[arg(long = "out-raw-counts", short = 'o')]
    pub out_raw_counts: String,

    /// Count per supplied BED region instead of per genome bin (deeptools
    /// `BED-file --BED`). When given, `--bin-size` is ignored.
    #[arg(long = "bed")]
    pub bed: Option<PathBuf>,

    /// Bin width in bases for `bins` mode (deeptools default 10000).
    #[arg(long = "bin-size", default_value_t = DEFAULT_BIN_SIZE)]
    pub bin_size: u32,

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
        let opts = SummaryOpts {
            bin_size: self.bin_size,
        };

        let matrix = match &self.bed {
            Some(bed) => summarize_bed(&self.bigwigfiles, bed, &opts)?,
            None => summarize_bins(&self.bigwigfiles, &opts)?,
        };

        let mut out: Box<dyn std::io::Write> = if self.out_raw_counts == "-" {
            Box::new(std::io::stdout().lock())
        } else {
            Box::new(std::fs::File::create(&self.out_raw_counts).map_err(RsomicsError::Io)?)
        };
        write_raw_counts(&mut out, &matrix)?;

        if !self.common.quiet {
            eprintln!(
                "{} rows × {} samples written",
                matrix.regions.len(),
                matrix.labels.len()
            );
        }
        Ok(())
    }
}

pub static HELP: HelpSpec = HelpSpec {
    name: env!("CARGO_PKG_NAME"),
    version: env!("CARGO_PKG_VERSION"),
    tagline: "Multi-bigWig per-bin / per-region mean-signal matrix (deeptools multiBigwigSummary port).",
    origin: Some(Origin {
        upstream: "deeptools multiBigwigSummary",
        upstream_license: "MIT",
        our_license: "MIT OR Apache-2.0",
        paper_doi: Some("10.1093/nar/gkw257"),
    }),
    usage_lines: &[
        "-b a.bw b.bw -o counts.tab [--bin-size 10000]",
        "-b a.bw b.bw --bed regions.bed -o counts.tab",
    ],
    sections: &[Section {
        title: "OPTIONS",
        flags: &[
            FlagSpec {
                short: Some('b'),
                long: "bigwigfiles",
                aliases: &[],
                value: Some("<file>..."),
                type_hint: Some("path"),
                required: true,
                default: None,
                description: "Input bigWig files (two or more).",
                why_default: None,
            },
            FlagSpec {
                short: Some('o'),
                long: "out-raw-counts",
                aliases: &[],
                value: Some("<file|->"),
                type_hint: Some("path"),
                required: true,
                default: None,
                description: "Per-bin / per-region signal table (deeptools --outRawCounts).",
                why_default: None,
            },
            FlagSpec {
                short: None,
                long: "bed",
                aliases: &[],
                value: Some("<file>"),
                type_hint: Some("path"),
                required: false,
                default: None,
                description: "Compute mean signal per BED region instead of per genome bin.",
                why_default: None,
            },
            FlagSpec {
                short: None,
                long: "bin-size",
                aliases: &[],
                value: Some("<u32>"),
                type_hint: Some("u32"),
                required: false,
                default: Some("10000"),
                description: "Bin width in bases (bins mode).",
                why_default: None,
            },
        ],
    }],
    examples: &[
        Example {
            description: "Per-10kb-bin mean signal across two bigWigs",
            command: "rsomics-multibigwig-summary -b a.bw b.bw -o counts.tab",
        },
        Example {
            description: "Per-peak mean signal from a BED file",
            command: "rsomics-multibigwig-summary -b a.bw b.bw --bed peaks.bed -o counts.tab",
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
