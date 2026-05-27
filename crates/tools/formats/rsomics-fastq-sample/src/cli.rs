use std::path::PathBuf;

use clap::Parser;
use rsomics_common::{CommonFlags, Result, RsomicsError, Tool, ToolMeta};
use rsomics_fastq_sample::{SampleMode, run_pe, run_se};
use rsomics_help::{Example, FlagSpec, HelpSpec, Origin, Section};

pub const META: ToolMeta = ToolMeta {
    name: env!("CARGO_PKG_NAME"),
    version: env!("CARGO_PKG_VERSION"),
};

#[derive(Parser, Debug)]
#[command(name = "rsomics-fastq-sample", disable_help_flag = true)]
pub struct Cli {
    /// Input FASTQ (R1 for paired-end; gz autodetected)
    pub input: PathBuf,
    /// Output path for sampled reads (R1 for paired-end)
    #[arg(short = 'o', long)]
    output: PathBuf,
    /// R2 input for paired-end sampling
    #[arg(short = 'I', long = "in2")]
    in2: Option<PathBuf>,
    /// R2 output for paired-end sampling (required when --in2 is set)
    #[arg(short = 'O', long = "out2")]
    out2: Option<PathBuf>,
    /// Sample fraction (Bernoulli: 0 < p <= 1; mutually exclusive with -n)
    #[arg(short = 'p', long, conflicts_with = "exact")]
    fraction: Option<f64>,
    /// Sample exactly N records (reservoir; mutually exclusive with -p)
    #[arg(short = 'n', long = "number", conflicts_with = "fraction")]
    exact: Option<u64>,
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
        let mode = match (self.fraction, self.exact) {
            (Some(p), None) => {
                if !(0.0 < p && p <= 1.0) {
                    return Err(RsomicsError::ConfigError(format!(
                        "--fraction must be in (0, 1]; got {p}"
                    )));
                }
                SampleMode::Fraction(p)
            }
            (None, Some(n)) => {
                if n == 0 {
                    return Err(RsomicsError::ConfigError(
                        "--number must be > 0".to_string(),
                    ));
                }
                SampleMode::Exact(n)
            }
            (None, None) => {
                return Err(RsomicsError::ConfigError(
                    "one of --fraction (-p) or --number (-n) is required".to_string(),
                ));
            }
            (Some(_), Some(_)) => unreachable!("clap conflicts_with prevents this"),
        };

        let seed = self.common.seed_rng();

        match (self.in2, self.out2) {
            (Some(r2), Some(o2)) => {
                let res = run_pe(&self.input, &r2, &self.output, &o2, mode, seed)?;
                if !self.common.quiet {
                    eprintln!("{}/{} pairs kept", res.kept, res.total);
                }
            }
            (None, None) => {
                let res = run_se(&self.input, &self.output, mode, seed)?;
                if !self.common.quiet {
                    eprintln!("{}/{} reads kept", res.kept, res.total);
                }
            }
            _ => {
                return Err(RsomicsError::ConfigError(
                    "--in2 and --out2 must both be set (or both absent) for paired-end mode"
                        .to_string(),
                ));
            }
        }
        Ok(())
    }
}

#[expect(dead_code)]
pub const HELP: HelpSpec = HelpSpec {
    name: META.name,
    version: META.version,
    tagline: "Random subsample FASTQ records by fraction or exact count (seqkit/seqtk sample compat).",
    origin: Some(Origin {
        upstream: "seqkit + seqtk",
        upstream_license: "MIT",
        our_license: "MIT OR Apache-2.0",
        paper_doi: Some("10.1371/journal.pone.0163962"),
    }),
    usage_lines: &[
        "[OPTIONS] -p <FRACTION> -o <OUT> <INPUT>",
        "[OPTIONS] -n <N>        -o <OUT> <INPUT>",
    ],
    sections: &[Section {
        title: "OPTIONS",
        flags: &[
            FlagSpec {
                short: None,
                long: "input",
                aliases: &[],
                value: Some("<path>"),
                type_hint: Some("Path"),
                required: true,
                default: None,
                description: "Input FASTQ (gz/bz2/xz autodetect). Positional.",
                why_default: None,
            },
            FlagSpec {
                short: Some('o'),
                long: "output",
                aliases: &[],
                value: Some("<path>"),
                type_hint: Some("Path"),
                required: true,
                default: None,
                description: "Output FASTQ; add .gz suffix for compressed output",
                why_default: None,
            },
            FlagSpec {
                short: Some('p'),
                long: "fraction",
                aliases: &[],
                value: Some("<f64>"),
                type_hint: Some("f64"),
                required: false,
                default: None,
                description: "Keep each record independently with this probability (Bernoulli; 0 < p ≤ 1)",
                why_default: None,
            },
            FlagSpec {
                short: Some('n'),
                long: "number",
                aliases: &[],
                value: Some("<u64>"),
                type_hint: Some("u64"),
                required: false,
                default: None,
                description: "Keep exactly N records (reservoir sampling; requires single pass)",
                why_default: None,
            },
            FlagSpec {
                short: Some('I'),
                long: "in2",
                aliases: &[],
                value: Some("<path>"),
                type_hint: Some("Path"),
                required: false,
                default: None,
                description: "R2 input for paired-end mode (must pair with --out2)",
                why_default: None,
            },
            FlagSpec {
                short: Some('O'),
                long: "out2",
                aliases: &[],
                value: Some("<path>"),
                type_hint: Some("Path"),
                required: false,
                default: None,
                description: "R2 output for paired-end mode",
                why_default: None,
            },
            FlagSpec {
                short: Some('h'),
                long: "help",
                aliases: &[],
                value: None,
                type_hint: Some("bool"),
                required: false,
                default: None,
                description: "Show this help (add --plain or --json for alt modes)",
                why_default: None,
            },
        ],
    }],
    examples: &[
        Example {
            description: "Keep 10 % of reads by Bernoulli sampling",
            command: "rsomics-fastq-sample -p 0.1 -o out.fq.gz input.fq.gz",
        },
        Example {
            description: "Keep exactly 100 000 reads (reservoir sampling)",
            command: "rsomics-fastq-sample -n 100000 -o out.fq input.fq",
        },
        Example {
            description: "Paired-end 10 % subsample",
            command: "rsomics-fastq-sample -p 0.1 -o r1.fq.gz -I r2.fq.gz -O r2_out.fq.gz r1.fq.gz",
        },
    ],
    json_result_schema_doc: None,
};

#[cfg(test)]
mod tests {
    use clap::CommandFactory;

    #[test]
    fn cli_definition_is_valid() {
        super::Cli::command().debug_assert();
    }
}
