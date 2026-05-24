use std::num::NonZero;
use std::path::PathBuf;

use clap::Parser;
use rsomics_common::{CommonFlags, Result, RsomicsError, Tool, ToolMeta};
use rsomics_help::{Example, FlagSpec, HelpSpec, Origin, Section};

use rsomics_multibam_summary::{
    DEFAULT_BIN_SIZE, SummaryOpts, summarize_bed, summarize_bins, write_raw_counts,
};

pub const META: ToolMeta = ToolMeta {
    name: env!("CARGO_PKG_NAME"),
    version: env!("CARGO_PKG_VERSION"),
};

#[derive(Parser, Debug)]
#[command(
    name = "rsomics-multibam-summary",
    version,
    about,
    long_about = None,
    disable_help_flag = true
)]
pub struct Cli {
    /// Input BAM files (two or more; same reference), deeptools `-b/--bamfiles`.
    #[arg(short = 'b', long = "bamfiles", num_args = 1.., required = true)]
    pub bamfiles: Vec<PathBuf>,

    /// Per-bin / per-region read-count table (deeptools `--outRawCounts`).
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

    /// Minimum mapping quality (deeptools default 0 = no filter).
    #[arg(long = "min-mapq", default_value_t = 0)]
    pub min_mapq: u8,

    /// Skip reads with any of these FLAG bits set (hex or decimal). deeptools
    /// default 0 (no skip — secondary/supplementary/duplicate reads are kept).
    #[arg(long = "skip-flags", default_value = "0")]
    pub skip_flags: String,

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
            skip_flags: parse_flag_hex(&self.skip_flags)?,
            min_mapq: self.min_mapq,
        };

        let workers = NonZero::new(self.common.thread_count()).unwrap_or(NonZero::<usize>::MIN);

        let matrix = match &self.bed {
            Some(bed) => summarize_bed(&self.bamfiles, bed, &opts, workers)?,
            None => summarize_bins(&self.bamfiles, &opts, workers)?,
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

fn parse_flag_hex(s: &str) -> Result<u16> {
    let trimmed = s.trim();
    let result = if let Some(hex) = trimmed
        .strip_prefix("0x")
        .or_else(|| trimmed.strip_prefix("0X"))
    {
        u16::from_str_radix(hex, 16)
    } else {
        trimmed.parse::<u16>()
    };
    result.map_err(|e| RsomicsError::InvalidInput(format!("invalid --skip-flags '{s}': {e}")))
}

pub static HELP: HelpSpec = HelpSpec {
    name: env!("CARGO_PKG_NAME"),
    version: env!("CARGO_PKG_VERSION"),
    tagline: "Multi-BAM per-bin / per-region read-count matrix (deeptools multiBamSummary port).",
    origin: Some(Origin {
        upstream: "deeptools multiBamSummary",
        upstream_license: "MIT",
        our_license: "MIT OR Apache-2.0",
        paper_doi: Some("10.1093/nar/gkw257"),
    }),
    usage_lines: &[
        "-b a.bam b.bam -o counts.tab [--bin-size 10000]",
        "-b a.bam b.bam --bed regions.bed -o counts.tab",
    ],
    sections: &[Section {
        title: "OPTIONS",
        flags: &[
            FlagSpec {
                short: Some('b'),
                long: "bamfiles",
                aliases: &[],
                value: Some("<file>..."),
                type_hint: Some("path"),
                required: true,
                default: None,
                description: "Input BAM files (two or more; same reference).",
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
                description: "Per-bin / per-region count table (deeptools --outRawCounts).",
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
                description: "Count per BED region instead of per genome bin.",
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
            FlagSpec {
                short: None,
                long: "min-mapq",
                aliases: &[],
                value: Some("<u8>"),
                type_hint: Some("u8"),
                required: false,
                default: Some("0"),
                description: "Minimum mapping quality.",
                why_default: None,
            },
            FlagSpec {
                short: None,
                long: "skip-flags",
                aliases: &[],
                value: Some("<hex|int>"),
                type_hint: Some("str"),
                required: false,
                default: Some("0"),
                description: "Skip reads with these FLAG bits. Use 0x400 for duplicates.",
                why_default: None,
            },
        ],
    }],
    examples: &[
        Example {
            description: "Per-10kb-bin counts across two BAMs",
            command: "rsomics-multibam-summary -b a.bam b.bam -o counts.tab",
        },
        Example {
            description: "Per-peak counts from a BED file",
            command: "rsomics-multibam-summary -b a.bam b.bam --bed peaks.bed -o counts.tab",
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

    #[test]
    fn parse_flag_hex_ok() {
        assert_eq!(parse_flag_hex("0x400").unwrap(), 0x400);
        assert_eq!(parse_flag_hex("1024").unwrap(), 1024);
        assert_eq!(parse_flag_hex("0").unwrap(), 0);
    }
}
