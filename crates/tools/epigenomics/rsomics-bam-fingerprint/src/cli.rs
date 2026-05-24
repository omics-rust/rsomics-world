use std::path::PathBuf;

use clap::Parser;
use rsomics_common::{CommonFlags, Result, RsomicsError, Tool, ToolMeta};
use rsomics_help::{Example, FlagSpec, HelpSpec, Origin, Section};

use rsomics_bam_fingerprint::{
    FingerprintOpts, fingerprint, write_fingerprint, write_quality_metrics, write_raw_counts,
};

pub const META: ToolMeta = ToolMeta {
    name: env!("CARGO_PKG_NAME"),
    version: env!("CARGO_PKG_VERSION"),
};

#[derive(Parser, Debug)]
#[command(
    name = "rsomics-bam-fingerprint",
    version,
    about,
    long_about = None,
    disable_help_flag = true
)]
pub struct Cli {
    /// Input indexed BAM file (coordinate-sorted).
    #[arg(short = 'b', long = "bam")]
    pub bam: PathBuf,

    /// Per-bin read-count table (deeptools `--outRawCounts`). `-` for stdout.
    #[arg(long = "out-raw-counts")]
    pub out_raw_counts: Option<String>,

    /// Cumulative fingerprint curve (rank vs cumulative fraction). `-` for stdout.
    #[arg(long = "out-fingerprint")]
    pub out_fingerprint: Option<String>,

    /// Quality-metrics summary (AUC, X-intercept, elbow). `-` for stdout.
    #[arg(long = "out-quality-metrics")]
    pub out_quality_metrics: Option<String>,

    /// Window size in bp to sample the genome (deeptools default: 500).
    #[arg(long = "bin-size", short = 's', default_value_t = 500)]
    pub bin_size: u32,

    /// Number of bins sampled across the genome (deeptools default: 500000).
    #[arg(long = "number-of-samples", short = 'n', default_value_t = 500_000)]
    pub number_of_samples: u64,

    /// Minimum mapping quality (deeptools default: 0 = no filter).
    #[arg(long = "min-mapq", default_value_t = 0)]
    pub min_mapq: u8,

    /// Skip reads with any of these FLAG bits set (deeptools `--samFlagExclude`).
    #[arg(long = "sam-flag-exclude", default_value = "0")]
    pub sam_flag_exclude: String,

    /// Keep only reads with all of these FLAG bits set (deeptools `--samFlagInclude`).
    #[arg(long = "sam-flag-include", default_value = "0")]
    pub sam_flag_include: String,

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
        if self.out_raw_counts.is_none()
            && self.out_fingerprint.is_none()
            && self.out_quality_metrics.is_none()
        {
            return Err(RsomicsError::InvalidInput(
                "at least one of --out-raw-counts, --out-fingerprint or --out-quality-metrics is required".into(),
            ));
        }

        let opts = FingerprintOpts {
            bin_size: self.bin_size,
            number_of_samples: self.number_of_samples,
            min_mapq: self.min_mapq,
            sam_flag_exclude: parse_flag(&self.sam_flag_exclude, "--sam-flag-exclude")?,
            sam_flag_include: parse_flag(&self.sam_flag_include, "--sam-flag-include")?,
        };

        let workers = std::num::NonZero::new(self.common.thread_count())
            .unwrap_or(std::num::NonZero::<usize>::MIN);

        let fp = fingerprint(&self.bam, &opts, workers)?;
        let label = self.bam.to_string_lossy().into_owned();

        if let Some(path) = &self.out_raw_counts {
            let mut out = open_sink(path)?;
            write_raw_counts(&mut out, &label, &fp)?;
        }
        if let Some(path) = &self.out_fingerprint {
            let mut out = open_sink(path)?;
            write_fingerprint(&mut out, &fp)?;
        }
        if let Some(path) = &self.out_quality_metrics {
            let mut out = open_sink(path)?;
            write_quality_metrics(&mut out, &label, &fp)?;
        }

        if !self.common.quiet {
            eprintln!("{} bins sampled", fp.counts.len());
        }
        Ok(())
    }
}

fn open_sink(path: &str) -> Result<Box<dyn std::io::Write>> {
    if path == "-" {
        Ok(Box::new(std::io::stdout().lock()))
    } else {
        Ok(Box::new(
            std::fs::File::create(path).map_err(RsomicsError::Io)?,
        ))
    }
}

fn parse_flag(s: &str, name: &str) -> Result<u16> {
    let trimmed = s.trim();
    let result = if let Some(hex) = trimmed
        .strip_prefix("0x")
        .or_else(|| trimmed.strip_prefix("0X"))
    {
        u16::from_str_radix(hex, 16)
    } else {
        trimmed.parse::<u16>()
    };
    result.map_err(|e| RsomicsError::InvalidInput(format!("invalid {name} '{s}': {e}")))
}

pub static HELP: HelpSpec = HelpSpec {
    name: env!("CARGO_PKG_NAME"),
    version: env!("CARGO_PKG_VERSION"),
    tagline: "ChIP-enrichment fingerprint (cumulative-coverage Lorenz curve; deeptools plotFingerprint port).",
    origin: Some(Origin {
        upstream: "deeptools plotFingerprint",
        upstream_license: "MIT",
        our_license: "MIT OR Apache-2.0",
        paper_doi: Some("10.1093/nar/gkw257"),
    }),
    usage_lines: &[
        "-b <input.bam> --out-raw-counts counts.tab [--bin-size 500] [--number-of-samples 500000]",
    ],
    sections: &[Section {
        title: "OPTIONS",
        flags: &[
            FlagSpec {
                short: Some('b'),
                long: "bam",
                aliases: &[],
                value: Some("<file>"),
                type_hint: Some("path"),
                required: true,
                default: None,
                description: "Input indexed coordinate-sorted BAM.",
                why_default: None,
            },
            FlagSpec {
                short: None,
                long: "out-raw-counts",
                aliases: &[],
                value: Some("<file|->"),
                type_hint: Some("path"),
                required: false,
                default: None,
                description: "Per-bin read-count table (deeptools --outRawCounts).",
                why_default: None,
            },
            FlagSpec {
                short: None,
                long: "out-fingerprint",
                aliases: &[],
                value: Some("<file|->"),
                type_hint: Some("path"),
                required: false,
                default: None,
                description: "Cumulative fingerprint curve (rank vs fraction).",
                why_default: None,
            },
            FlagSpec {
                short: None,
                long: "out-quality-metrics",
                aliases: &[],
                value: Some("<file|->"),
                type_hint: Some("path"),
                required: false,
                default: None,
                description: "Summary metrics: AUC, X-intercept, elbow.",
                why_default: None,
            },
            FlagSpec {
                short: Some('s'),
                long: "bin-size",
                aliases: &[],
                value: Some("<u32>"),
                type_hint: Some("u32"),
                required: false,
                default: Some("500"),
                description: "Window size in bp to sample the genome.",
                why_default: None,
            },
            FlagSpec {
                short: Some('n'),
                long: "number-of-samples",
                aliases: &[],
                value: Some("<u64>"),
                type_hint: Some("u64"),
                required: false,
                default: Some("500000"),
                description: "Number of bins sampled across the genome.",
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
        ],
    }],
    examples: &[
        Example {
            description: "Emit the per-bin raw-counts table",
            command: "rsomics-bam-fingerprint -b chip.bam --out-raw-counts counts.tab",
        },
        Example {
            description: "Fingerprint curve + summary metrics",
            command: "rsomics-bam-fingerprint -b chip.bam --out-fingerprint fp.tab --out-quality-metrics qc.tab",
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
    fn parse_flag_ok() {
        assert_eq!(parse_flag("0x400", "x").unwrap(), 0x400);
        assert_eq!(parse_flag("1024", "x").unwrap(), 1024);
        assert_eq!(parse_flag("0", "x").unwrap(), 0);
    }
}
