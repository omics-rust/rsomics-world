use std::io::BufWriter;
use std::num::NonZero;
use std::path::PathBuf;

use clap::Parser;
use rsomics_common::{CommonFlags, Result, RsomicsError, Tool, ToolMeta};
use rsomics_help::{Example, FlagSpec, HelpSpec, Origin, Section};

use rsomics_bam_ampliconstats::stats::{AmpStatsArgs, MAX_DEPTH_LEVELS};

pub const META: ToolMeta = ToolMeta {
    name: env!("CARGO_PKG_NAME"),
    version: env!("CARGO_PKG_VERSION"),
};

#[derive(Parser, Debug)]
#[command(
    name = "rsomics-bam-ampliconstats",
    version,
    about,
    long_about = None,
    disable_help_flag = true
)]
pub struct Cli {
    /// Primer BED file (6-column, strand required).
    #[arg(value_name = "PRIMERS.BED")]
    primers: PathBuf,

    /// Input BAM file(s).
    #[arg(value_name = "INPUT.BAM", required = true)]
    inputs: Vec<PathBuf>,

    /// Only include reads with all FLAGS present.
    #[arg(
        short = 'f',
        long = "required-flag",
        default_value = "0",
        value_name = "INT"
    )]
    flag_require: u16,

    /// Only include reads with none of these FLAGS present [0xB04].
    #[arg(short = 'F', long = "filter-flag", value_name = "INT")]
    flag_filter: Option<String>,

    /// Margin for matching primer positions [30].
    #[arg(
        short = 'm',
        long = "pos-margin",
        default_value = "30",
        value_name = "INT"
    )]
    pos_margin: i64,

    /// Minimum base depth(s), comma-separated [1].
    #[arg(
        short = 'd',
        long = "min-depth",
        default_value = "1",
        value_name = "INT[,INT]"
    )]
    min_depth: String,

    /// Maximum number of amplicons [1000].
    #[arg(
        short = 'a',
        long = "max-amplicons",
        default_value = "1000",
        value_name = "INT"
    )]
    max_amplicons: usize,

    /// Maximum amplicon length [1000].
    #[arg(
        short = 'l',
        long = "max-amplicon-length",
        default_value = "1000",
        value_name = "INT"
    )]
    max_amp_len: usize,

    /// Add/subtract from TLEN (use when clipping but no fixmate step) [0].
    #[arg(long = "tlen-adjust", default_value = "0", value_name = "INT")]
    tlen_adj: i64,

    /// Minimum template coord frequency for recording [10].
    #[arg(
        short = 'c',
        long = "tcoord-min-count",
        default_value = "10",
        value_name = "INT"
    )]
    tcoord_min_count: u32,

    /// Bin template start/end positions into multiples of INT [1].
    #[arg(
        short = 'b',
        long = "tcoord-bin",
        default_value = "1",
        value_name = "INT"
    )]
    tcoord_bin: i64,

    /// Merge depth values within ±FRACTION [0.01].
    #[arg(
        short = 'D',
        long = "depth-bin",
        default_value = "0.01",
        value_name = "FRACTION"
    )]
    depth_bin: f64,

    /// Force single-ref (<=1.12) output format.
    #[arg(short = 'S', long = "single-ref")]
    single_ref: bool,

    #[command(flatten)]
    pub common: CommonFlags,
}

impl Cli {
    pub fn execute(self) -> Result<()> {
        let flag_filter = if let Some(s) = &self.flag_filter {
            parse_flag(s)?
        } else {
            AmpStatsArgs::default().flag_filter
        };

        let min_depth = parse_min_depth(&self.min_depth)?;

        let args = AmpStatsArgs {
            flag_require: self.flag_require,
            flag_filter,
            max_delta: self.pos_margin,
            min_depth,
            max_amp: self.max_amplicons + 1,
            max_amp_len: self.max_amp_len + 1,
            tlen_adj: self.tlen_adj,
            depth_bin: self.depth_bin,
            tcoord_min_count: self.tcoord_min_count,
            tcoord_bin: self.tcoord_bin.max(1),
            workers: NonZero::new(self.common.thread_count()).unwrap_or(NonZero::<usize>::MIN),
            single_ref: self.single_ref,
        };

        let bam_refs: Vec<&std::path::Path> = self.inputs.iter().map(|p| p.as_path()).collect();

        let argv_str = format!(
            "rsomics-bam-ampliconstats {} {}",
            self.primers.display(),
            bam_refs
                .iter()
                .map(|p| p.display().to_string())
                .collect::<Vec<_>>()
                .join(" ")
        );

        let stdout = std::io::stdout();
        let mut out = BufWriter::new(stdout.lock());

        rsomics_bam_ampliconstats::stats::run(
            &args,
            &self.primers,
            &bam_refs,
            &argv_str,
            &mut out,
        )?;

        Ok(())
    }
}

fn parse_flag(s: &str) -> Result<u16> {
    if let Some(hex) = s.strip_prefix("0x").or_else(|| s.strip_prefix("0X")) {
        u16::from_str_radix(hex, 16)
            .map_err(|e| RsomicsError::InvalidInput(format!("bad flag '{s}': {e}")))
    } else {
        s.parse::<u16>()
            .map_err(|e| RsomicsError::InvalidInput(format!("bad flag '{s}': {e}")))
    }
}

fn parse_min_depth(s: &str) -> Result<[u32; MAX_DEPTH_LEVELS]> {
    let mut depths = [0u32; MAX_DEPTH_LEVELS];
    let mut d = 0usize;
    for part in s.split(',') {
        if d >= MAX_DEPTH_LEVELS {
            break;
        }
        depths[d] = part
            .trim()
            .parse()
            .map_err(|e| RsomicsError::InvalidInput(format!("bad min-depth '{part}': {e}")))?;
        d += 1;
    }
    if d == 0 {
        depths[0] = 1;
    }
    Ok(depths)
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
    tagline: "Amplicon sequencing statistics from a primer BED and one or more BAMs.",
    origin: Some(Origin {
        upstream: "samtools ampliconstats",
        upstream_license: "MIT",
        our_license: "MIT OR Apache-2.0",
        paper_doi: None,
    }),
    usage_lines: &["<PRIMERS.BED> <INPUT.BAM> [INPUT2.BAM ...]"],
    sections: &[Section {
        title: "OPTIONS",
        flags: &[
            FlagSpec {
                short: Some('f'),
                long: "required-flag",
                aliases: &[],
                value: Some("<INT>"),
                type_hint: Some("u16"),
                required: false,
                default: Some("0"),
                description: "Only include reads with all FLAGS set.",
                why_default: None,
            },
            FlagSpec {
                short: Some('F'),
                long: "filter-flag",
                aliases: &[],
                value: Some("<INT>"),
                type_hint: Some("u16"),
                required: false,
                default: Some("0xB04"),
                description: "Exclude reads with any of these FLAGS.",
                why_default: None,
            },
            FlagSpec {
                short: Some('m'),
                long: "pos-margin",
                aliases: &[],
                value: Some("<INT>"),
                type_hint: Some("i64"),
                required: false,
                default: Some("30"),
                description: "Margin for matching read start/end to primer positions.",
                why_default: None,
            },
            FlagSpec {
                short: Some('d'),
                long: "min-depth",
                aliases: &[],
                value: Some("<INT[,INT]>"),
                type_hint: Some("String"),
                required: false,
                default: Some("1"),
                description: "Minimum depth thresholds for coverage reporting, comma-separated.",
                why_default: None,
            },
        ],
    }],
    examples: &[Example {
        description: "Run on a single amplicon BAM",
        command: "rsomics-bam-ampliconstats primers.bed amplicon.bam",
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
