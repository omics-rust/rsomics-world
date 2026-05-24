use std::path::PathBuf;

use clap::Parser;
use rsomics_common::{CommonFlags, Result, RsomicsError, Tool, ToolMeta};
use rsomics_help::{Example, FlagSpec, HelpSpec, Origin, Section};

use rsomics_bam_signal::{
    CoverageOpts, Normalisation, OutputFormat, bam_to_bedgraph, bam_to_bigwig,
};

pub const META: ToolMeta = ToolMeta {
    name: env!("CARGO_PKG_NAME"),
    version: env!("CARGO_PKG_VERSION"),
};

#[derive(Parser, Debug)]
#[command(
    name = "rsomics-bam-signal",
    version,
    about,
    long_about = None,
    disable_help_flag = true
)]
pub struct Cli {
    /// Input BAM file (must be sorted; index not required for whole-genome scan).
    pub bam: PathBuf,

    /// Output file (use `-` for stdout, only valid with --out-file-format bedgraph).
    #[arg(short = 'o', long, default_value = "-")]
    pub output: String,

    /// Output format: bedgraph or bigwig. deeptools default is bigwig.
    #[arg(long = "out-file-format", short = 'F', default_value = "bigwig")]
    pub out_file_format: OutputFormat,

    /// Bin size in bases.
    #[arg(long = "bin-size", short = 'b', default_value_t = 50)]
    pub bin_size: u32,

    /// Normalisation method: None, CPM, RPKM, BPM, RPGC.
    #[arg(long = "normalize-using", default_value = "None")]
    pub normalize_using: Normalisation,

    /// Effective genome size for RPGC normalisation.
    #[arg(long = "effective-genome-size")]
    pub effective_genome_size: Option<u64>,

    /// Skip reads with any of these FLAG bits set (hex or decimal).
    /// deeptools default: 0 (no skip). Use 0x400 to skip duplicates.
    #[arg(long = "skip-flags", default_value = "0")]
    pub skip_flags: String,

    /// Minimum mapping quality (deeptools default: 0 = no filter).
    #[arg(long = "min-mapq", default_value_t = 0)]
    pub min_mapq: u8,

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
        let skip_flags = parse_flag_hex(&self.skip_flags)?;

        let opts = CoverageOpts {
            bin_size: self.bin_size,
            skip_flags,
            min_mapq: self.min_mapq,
            normalisation: self.normalize_using,
            effective_genome_size: self.effective_genome_size,
        };

        let workers = std::num::NonZero::new(self.common.thread_count())
            .unwrap_or(std::num::NonZero::<usize>::MIN);

        match self.out_file_format {
            OutputFormat::BigWig => {
                if self.output == "-" {
                    return Err(RsomicsError::InvalidInput(
                        "bigWig output requires a file path (-o <file.bw>); stdout is not supported".into(),
                    ));
                }
                bam_to_bigwig(
                    &self.bam,
                    std::path::Path::new(&self.output),
                    &opts,
                    workers,
                )?;
                if !self.common.quiet {
                    eprintln!("bigWig written to {}", self.output);
                }
            }
            OutputFormat::BedGraph => {
                let mut out: Box<dyn std::io::Write> = if self.output == "-" {
                    Box::new(std::io::stdout().lock())
                } else {
                    Box::new(std::fs::File::create(&self.output).map_err(RsomicsError::Io)?)
                };
                let lines = bam_to_bedgraph(&self.bam, &mut out, &opts, workers)?;
                if !self.common.quiet {
                    eprintln!("{lines} bedGraph lines written");
                }
            }
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
    tagline: "Binned BAM → bedGraph/bigWig signal track (deeptools bamCoverage port).",
    origin: Some(Origin {
        upstream: "deeptools bamCoverage",
        upstream_license: "MIT",
        our_license: "MIT OR Apache-2.0",
        paper_doi: Some("10.1093/nar/gkw257"),
    }),
    usage_lines: &["<input.bam> [-o out.bw] [-F bigwig] [--bin-size 50] [--normalize-using CPM]"],
    sections: &[Section {
        title: "OPTIONS",
        flags: &[
            FlagSpec {
                short: Some('F'),
                long: "out-file-format",
                aliases: &[],
                value: Some("<format>"),
                type_hint: Some("str"),
                required: false,
                default: Some("bigwig"),
                description: "Output format: bedgraph or bigwig (deeptools default: bigwig).",
                why_default: None,
            },
            FlagSpec {
                short: Some('b'),
                long: "bin-size",
                aliases: &[],
                value: Some("<u32>"),
                type_hint: Some("u32"),
                required: false,
                default: Some("50"),
                description: "Bin size in bases.",
                why_default: None,
            },
            FlagSpec {
                short: None,
                long: "normalize-using",
                aliases: &[],
                value: Some("<method>"),
                type_hint: Some("str"),
                required: false,
                default: Some("None"),
                description: "Normalisation: None, CPM, RPKM, BPM, RPGC.",
                why_default: None,
            },
            FlagSpec {
                short: None,
                long: "effective-genome-size",
                aliases: &[],
                value: Some("<u64>"),
                type_hint: Some("u64"),
                required: false,
                default: None,
                description: "Effective genome size for RPGC normalisation.",
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
            description: "Raw binned coverage at 50 bp resolution",
            command: "rsomics-bam-signal in.bam -o signal.bedgraph",
        },
        Example {
            description: "CPM-normalised signal, 100 bp bins",
            command: "rsomics-bam-signal in.bam --bin-size 100 --normalize-using CPM -o cpm.bedgraph",
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
