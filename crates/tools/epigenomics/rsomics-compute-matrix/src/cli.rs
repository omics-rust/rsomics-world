use std::path::PathBuf;

use clap::{Parser, Subcommand};
use rsomics_common::{CommonFlags, Result, Tool, ToolMeta};
use rsomics_help::{Example, FlagSpec, HelpSpec, Origin, Section};

use rsomics_compute_matrix::{BinAvg, MatrixParams, Mode, RefPoint, compute_matrix, read_bed};

pub const META: ToolMeta = ToolMeta {
    name: env!("CARGO_PKG_NAME"),
    version: env!("CARGO_PKG_VERSION"),
};

#[derive(Parser, Debug)]
#[command(
    name = "rsomics-compute-matrix",
    version,
    about,
    long_about = None,
    disable_help_flag = true
)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Command,

    #[command(flatten)]
    pub common: CommonFlags,
}

#[derive(Subcommand, Debug)]
pub enum Command {
    /// Score flanks around a per-region reference point (TSS, TES or center).
    #[command(name = "reference-point")]
    ReferencePoint(RefPointArgs),
    /// Scale each region body to a fixed bin count, with optional flanks.
    #[command(name = "scale-regions")]
    ScaleRegions(ScaleArgs),
}

#[derive(clap::Args, Debug)]
pub struct RefPointArgs {
    /// bigWig signal file.
    #[arg(short = 'S', long = "score-file")]
    pub score_file: PathBuf,

    /// BED file of regions (BED6; single group).
    #[arg(short = 'R', long = "regions-file")]
    pub regions_file: PathBuf,

    /// Output gzipped matrix file.
    #[arg(short = 'o', long = "out-file-name")]
    pub output: PathBuf,

    /// Reference point within each region: TSS, TES or center.
    #[arg(long = "reference-point", default_value = "TSS")]
    pub reference_point: RefPoint,

    /// Distance upstream of the reference point.
    #[arg(
        short = 'b',
        long = "before-region-start-length",
        default_value_t = 500
    )]
    pub upstream: u32,

    /// Distance downstream of the reference point.
    #[arg(
        short = 'a',
        long = "after-region-start-length",
        default_value_t = 1500
    )]
    pub downstream: u32,

    #[command(flatten)]
    pub shared: SharedArgs,
}

#[derive(clap::Args, Debug)]
pub struct ScaleArgs {
    /// bigWig signal file.
    #[arg(short = 'S', long = "score-file")]
    pub score_file: PathBuf,

    /// BED file of regions (BED6; single group).
    #[arg(short = 'R', long = "regions-file")]
    pub regions_file: PathBuf,

    /// Output gzipped matrix file.
    #[arg(short = 'o', long = "out-file-name")]
    pub output: PathBuf,

    /// Length each region body is scaled to.
    #[arg(short = 'm', long = "region-body-length", default_value_t = 1000)]
    pub body: u32,

    /// Distance upstream of the region start.
    #[arg(short = 'b', long = "before-region-start-length", default_value_t = 0)]
    pub upstream: u32,

    /// Distance downstream of the region end.
    #[arg(short = 'a', long = "after-region-start-length", default_value_t = 0)]
    pub downstream: u32,

    #[command(flatten)]
    pub shared: SharedArgs,
}

#[derive(clap::Args, Debug)]
pub struct SharedArgs {
    /// Bin width in bases.
    #[arg(long = "bin-size", default_value_t = 10)]
    pub bin_size: u32,

    /// Per-bin statistic.
    #[arg(long = "average-type-bins", default_value = "mean")]
    pub average_type_bins: BinAvg,

    /// Treat missing data (NaN) as zero.
    #[arg(long = "missing-data-as-zero", default_value_t = false)]
    pub missing_data_as_zero: bool,

    /// Skip regions whose binned mean is zero.
    #[arg(long = "skip-zeros", default_value_t = false)]
    pub skip_zeros: bool,

    /// Skip a region if any bin is <= this value.
    #[arg(long = "min-threshold")]
    pub min_threshold: Option<f64>,

    /// Skip a region if any bin is >= this value.
    #[arg(long = "max-threshold")]
    pub max_threshold: Option<f64>,

    /// Multiply all values by this factor.
    #[arg(long = "scale", default_value_t = 1.0)]
    pub scale: f64,

    /// Sample label written to the header (default: bigWig basename).
    #[arg(long = "samples-label")]
    pub samples_label: Option<String>,
}

impl Tool for Cli {
    fn meta() -> ToolMeta {
        META
    }
    fn common(&self) -> &CommonFlags {
        &self.common
    }

    fn execute(self) -> Result<()> {
        let threads = self.common.thread_count();
        let quiet = self.common.quiet;
        let (mode, score_file, regions_file, output, upstream, downstream, body, shared) =
            match self.command {
                Command::ReferencePoint(a) => (
                    Mode::ReferencePoint(a.reference_point),
                    a.score_file,
                    a.regions_file,
                    a.output,
                    a.upstream,
                    a.downstream,
                    0,
                    a.shared,
                ),
                Command::ScaleRegions(a) => (
                    Mode::ScaleRegions,
                    a.score_file,
                    a.regions_file,
                    a.output,
                    a.upstream,
                    a.downstream,
                    a.body,
                    a.shared,
                ),
            };

        let sample_label = shared
            .samples_label
            .unwrap_or_else(|| smart_label(&score_file));
        let group_label = "genes".to_string();

        let params = MatrixParams {
            mode,
            upstream,
            downstream,
            body,
            bin_size: shared.bin_size,
            bin_avg: shared.average_type_bins,
            missing_data_as_zero: shared.missing_data_as_zero,
            min_threshold: shared.min_threshold,
            max_threshold: shared.max_threshold,
            scale: shared.scale,
            skip_zeros: shared.skip_zeros,
            nan_after_end: false,
            proc_number: threads,
            sample_label,
            group_label,
        };

        let regions = read_bed(&regions_file)?;
        let (written, no_score) = compute_matrix(&score_file, &regions, &params, &output)?;
        if !quiet {
            eprintln!("{written} regions written, {no_score} without scores");
        }
        Ok(())
    }
}

/// deeptools `smartLabels`: file basename minus its extension.
fn smart_label(path: &std::path::Path) -> String {
    path.file_stem().map_or_else(
        || path.to_string_lossy().into_owned(),
        |s| s.to_string_lossy().into_owned(),
    )
}

pub static HELP: HelpSpec = HelpSpec {
    name: env!("CARGO_PKG_NAME"),
    version: env!("CARGO_PKG_VERSION"),
    tagline: "bigWig signal → score matrix over BED regions (deeptools computeMatrix port).",
    origin: Some(Origin {
        upstream: "deeptools computeMatrix",
        upstream_license: "MIT",
        our_license: "MIT OR Apache-2.0",
        paper_doi: Some("10.1093/nar/gkw257"),
    }),
    usage_lines: &[
        "reference-point -S signal.bw -R regions.bed -o matrix.gz --reference-point TSS -b 1000 -a 1000 --bin-size 50",
        "scale-regions -S signal.bw -R regions.bed -o matrix.gz -m 1000 -b 500 -a 500 --bin-size 50",
    ],
    sections: &[Section {
        title: "OPTIONS",
        flags: &[
            FlagSpec {
                short: Some('S'),
                long: "score-file",
                aliases: &[],
                value: Some("<file.bw>"),
                type_hint: Some("path"),
                required: true,
                default: None,
                description: "bigWig signal file.",
                why_default: None,
            },
            FlagSpec {
                short: Some('R'),
                long: "regions-file",
                aliases: &[],
                value: Some("<file.bed>"),
                type_hint: Some("path"),
                required: true,
                default: None,
                description: "BED6 regions file (single group).",
                why_default: None,
            },
            FlagSpec {
                short: Some('o'),
                long: "out-file-name",
                aliases: &[],
                value: Some("<matrix.gz>"),
                type_hint: Some("path"),
                required: true,
                default: None,
                description: "Gzipped output matrix.",
                why_default: None,
            },
            FlagSpec {
                short: Some('b'),
                long: "before-region-start-length",
                aliases: &["upstream"],
                value: Some("<bp>"),
                type_hint: Some("u32"),
                required: false,
                default: Some("500 (ref-point) / 0 (scale)"),
                description: "Upstream flank length.",
                why_default: None,
            },
            FlagSpec {
                short: Some('a'),
                long: "after-region-start-length",
                aliases: &["downstream"],
                value: Some("<bp>"),
                type_hint: Some("u32"),
                required: false,
                default: Some("1500 (ref-point) / 0 (scale)"),
                description: "Downstream flank length.",
                why_default: None,
            },
            FlagSpec {
                short: Some('m'),
                long: "region-body-length",
                aliases: &[],
                value: Some("<bp>"),
                type_hint: Some("u32"),
                required: false,
                default: Some("1000"),
                description: "scale-regions: body scaled to this length.",
                why_default: None,
            },
            FlagSpec {
                short: None,
                long: "bin-size",
                aliases: &[],
                value: Some("<bp>"),
                type_hint: Some("u32"),
                required: false,
                default: Some("10"),
                description: "Bin width.",
                why_default: None,
            },
            FlagSpec {
                short: None,
                long: "reference-point",
                aliases: &[],
                value: Some("<TSS|TES|center>"),
                type_hint: Some("str"),
                required: false,
                default: Some("TSS"),
                description: "reference-point anchor.",
                why_default: None,
            },
            FlagSpec {
                short: None,
                long: "average-type-bins",
                aliases: &[],
                value: Some("<stat>"),
                type_hint: Some("str"),
                required: false,
                default: Some("mean"),
                description: "Per-bin statistic: mean|median|min|max|std|sum.",
                why_default: None,
            },
            FlagSpec {
                short: None,
                long: "missing-data-as-zero",
                aliases: &[],
                value: None,
                type_hint: Some("flag"),
                required: false,
                default: Some("false"),
                description: "Treat NaN as 0.",
                why_default: None,
            },
        ],
    }],
    examples: &[
        Example {
            description: "TSS-centred matrix, 50 bp bins, 1 kb flanks",
            command: "rsomics-compute-matrix reference-point -S signal.bw -R genes.bed -o m.gz --reference-point TSS -b 1000 -a 1000 --bin-size 50",
        },
        Example {
            description: "Scaled gene bodies with 500 bp flanks",
            command: "rsomics-compute-matrix scale-regions -S signal.bw -R genes.bed -o m.gz -m 1000 -b 500 -a 500 --bin-size 50",
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
