use std::path::PathBuf;

use clap::{Parser, Subcommand};
use rsomics_common::{CommonFlags, Result, RsomicsError, Tool, ToolMeta};
use rsomics_help::{Example, HelpSpec, Origin};

use rsomics_fastq_utils::ops;

pub const META: ToolMeta = ToolMeta {
    name: env!("CARGO_PKG_NAME"),
    version: env!("CARGO_PKG_VERSION"),
};

#[derive(Parser)]
#[command(
    name = "rsomics-fastq-utils",
    version,
    about = "FASTQ utility toolkit",
    disable_help_flag = true
)]
pub struct Cli {
    #[command(subcommand)]
    command: Command,
    #[command(flatten)]
    pub common: CommonFlags,
}

#[derive(Subcommand)]
enum Command {
    /// Count FASTQ records (gz-transparent)
    Count { input: Vec<PathBuf> },
    /// Split interleaved FASTQ into R1/R2
    Deinterleave {
        input: PathBuf,
        #[arg(long)]
        out1: PathBuf,
        #[arg(long)]
        out2: PathBuf,
    },
    /// Extract reads by name from FASTQ
    Extract {
        input: PathBuf,
        #[arg(short = 'l', long)]
        list: PathBuf,
        #[arg(long)]
        exclude: bool,
        #[arg(short = 'o', long, default_value = "-")]
        output: String,
    },
    /// Compute per-read GC content
    Gc {
        input: PathBuf,
        #[arg(short = 'o', long, default_value = "-")]
        output: String,
    },
    /// Filter FASTQ records by read name regex
    Grep {
        input: PathBuf,
        #[arg(short = 'p', long)]
        pattern: String,
        #[arg(long)]
        invert_match: bool,
        #[arg(short = 'o', long, default_value = "-")]
        output: String,
    },
    /// Output the first N FASTQ records
    Head {
        input: PathBuf,
        #[arg(short = 'n', long, default_value_t = 10)]
        num: u64,
        #[arg(short = 'o', long, default_value = "-")]
        output: String,
    },
    /// Interleave paired FASTQ R1/R2 into a single stream
    Interleave {
        #[arg(short = 'i', long)]
        in1: PathBuf,
        #[arg(short = 'I', long)]
        in2: PathBuf,
        #[arg(short = 'o', long, default_value = "-")]
        output: String,
    },
    /// Output per-read lengths
    Len {
        input: PathBuf,
        #[arg(long)]
        tab: bool,
        #[arg(short = 'o', long, default_value = "-")]
        output: String,
    },
    /// Rename reads with sequential IDs
    Rename {
        input: PathBuf,
        #[arg(long, default_value = "read_")]
        prefix: String,
        #[arg(short = 'o', long, default_value = "-")]
        output: String,
    },
    /// Reverse-complement sequences
    Revcomp {
        input: PathBuf,
        #[arg(short = 'o', long, default_value = "-")]
        output: String,
    },
    /// Random subsample of records
    Sample {
        input: PathBuf,
        #[arg(short = 'p', long, default_value_t = 0.1)]
        proportion: f64,
        #[arg(long, default_value_t = 42)]
        seed: u64,
        #[arg(short = 'o', long, default_value = "-")]
        output: String,
    },
    /// Randomly shuffle record order
    Shuffle {
        input: PathBuf,
        #[arg(long, default_value_t = 42)]
        seed: u64,
        #[arg(short = 'o', long, default_value = "-")]
        output: String,
    },
    /// Sort reads by name or length
    Sort {
        input: PathBuf,
        #[arg(short = 'l', long)]
        by_length: bool,
        #[arg(short = 'L', long)]
        by_length_desc: bool,
        #[arg(short = 'o', long, default_value = "-")]
        output: String,
    },
    /// Convert to tab-separated name+seq+qual
    Tab {
        input: PathBuf,
        #[arg(short = 'o', long, default_value = "-")]
        output: String,
    },
    /// Convert FASTQ to FASTA (strip quality)
    ToFasta {
        input: PathBuf,
        #[arg(short = 'o', long, default_value = "-")]
        output: String,
    },
    /// Sliding-window quality statistics per read
    Window {
        input: PathBuf,
        #[arg(short = 'w', long, default_value_t = 10)]
        window: usize,
        #[arg(short = 'o', long, default_value = "-")]
        output: String,
    },
}

fn open_output(path: &str) -> Result<Box<dyn std::io::Write>> {
    if path == "-" {
        Ok(Box::new(std::io::stdout().lock()))
    } else {
        Ok(Box::new(
            std::fs::File::create(path).map_err(RsomicsError::Io)?,
        ))
    }
}

impl Tool for Cli {
    fn meta() -> ToolMeta {
        META
    }

    fn common(&self) -> &CommonFlags {
        &self.common
    }

    #[allow(clippy::too_many_lines)]
    fn execute(self) -> Result<()> {
        match self.command {
            Command::Count { input } => {
                let mut total = 0u64;
                for path in &input {
                    total += ops::count::count(path)?;
                }
                println!("{total}");
            }
            Command::Deinterleave { input, out1, out2 } => {
                let mut w1 = std::io::BufWriter::new(
                    std::fs::File::create(&out1).map_err(RsomicsError::Io)?,
                );
                let mut w2 = std::io::BufWriter::new(
                    std::fs::File::create(&out2).map_err(RsomicsError::Io)?,
                );
                ops::deinterleave::deinterleave(&input, &mut w1, &mut w2)?;
            }
            Command::Extract {
                input,
                list,
                exclude,
                output,
            } => {
                let mut out = open_output(&output)?;
                ops::extract::extract_fastq(&input, &list, &mut out, exclude)?;
            }
            Command::Gc { input, output } => {
                let mut out = open_output(&output)?;
                ops::gc::fastq_gc(&input, &mut out)?;
            }
            Command::Grep {
                input,
                pattern,
                invert_match,
                output,
            } => {
                let mut out = open_output(&output)?;
                ops::grep::grep(&input, &pattern, invert_match, &mut out)?;
            }
            Command::Head { input, num, output } => {
                let mut out = open_output(&output)?;
                ops::head::head(&input, num, &mut out)?;
            }
            Command::Interleave { in1, in2, output } => {
                let mut out = open_output(&output)?;
                ops::interleave::interleave(&in1, &in2, &mut out)?;
            }
            Command::Len { input, tab, output } => {
                let mut out = open_output(&output)?;
                ops::len::lengths(&input, tab, &mut out)?;
            }
            Command::Rename {
                input,
                prefix,
                output,
            } => {
                let mut out = open_output(&output)?;
                ops::rename::rename(&input, &prefix, &mut out)?;
            }
            Command::Revcomp { input, output } => {
                let mut out = open_output(&output)?;
                ops::revcomp::revcomp(&input, &mut out)?;
            }
            Command::Sample {
                input,
                proportion,
                seed,
                output,
            } => {
                let mut out = open_output(&output)?;
                ops::sample::sample(&input, proportion, seed, &mut out)?;
            }
            Command::Shuffle {
                input,
                seed,
                output,
            } => {
                let mut out = open_output(&output)?;
                ops::shuffle::shuffle_fastq(&input, &mut out, seed)?;
            }
            Command::Sort {
                input,
                by_length,
                by_length_desc,
                output,
            } => {
                let key = if by_length_desc {
                    ops::sort::SortKey::LengthDesc
                } else if by_length {
                    ops::sort::SortKey::Length
                } else {
                    ops::sort::SortKey::Name
                };
                let mut out = open_output(&output)?;
                ops::sort::sort(&input, key, &mut out)?;
            }
            Command::Tab { input, output } => {
                let mut out = open_output(&output)?;
                ops::tab::fastq_to_tab(&input, &mut out)?;
            }
            Command::ToFasta { input, output } => {
                let mut out = open_output(&output)?;
                ops::to_fasta::convert(&input, &mut out)?;
            }
            Command::Window {
                input,
                window,
                output,
            } => {
                let mut out = open_output(&output)?;
                ops::window::fastq_window(&input, &mut out, window)?;
            }
        }
        Ok(())
    }
}

pub static HELP: HelpSpec = HelpSpec {
    name: env!("CARGO_PKG_NAME"),
    version: env!("CARGO_PKG_VERSION"),
    tagline: "FASTQ utility toolkit — lightweight subcommands for counting, filtering, converting, and inspecting FASTQ files.",
    origin: Some(Origin {
        upstream: "seqkit / seqtk",
        upstream_license: "MIT",
        our_license: "MIT OR Apache-2.0",
        paper_doi: None,
    }),
    usage_lines: &["<COMMAND> [OPTIONS] <input>"],
    sections: &[],
    examples: &[
        Example {
            description: "Count records",
            command: "rsomics-fastq-utils count reads.fq.gz",
        },
        Example {
            description: "First 100 reads",
            command: "rsomics-fastq-utils head -n 100 reads.fq",
        },
        Example {
            description: "Convert to FASTA",
            command: "rsomics-fastq-utils to-fasta reads.fq -o reads.fa",
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
