use std::path::PathBuf;
use std::process::ExitCode;

use clap::{Parser, Subcommand};
use rsomics_common::{CommonFlags, Result, RsomicsError};

use rsomics_fasta_utils::ops;

#[derive(Parser)]
#[command(name = "rsomics-fasta-utils", version, about = "FASTA utility toolkit")]
struct Cli {
    #[command(subcommand)]
    command: Command,
    #[command(flatten)]
    common: CommonFlags,
}

#[derive(Subcommand)]
enum Command {
    /// Count sequences
    Count { input: PathBuf },
    /// List unique sequence names (chromosomes)
    Chroms {
        input: PathBuf,
        #[arg(short = 'o', long, default_value = "-")]
        output: String,
    },
    /// Nucleotide composition
    Composition { input: PathBuf },
    /// GC content per sequence
    GcContent {
        input: PathBuf,
        #[arg(short = 'o', long, default_value = "-")]
        output: String,
    },
    /// Sequence lengths
    Len {
        input: PathBuf,
        #[arg(long)]
        tab: bool,
        #[arg(short = 'o', long, default_value = "-")]
        output: String,
    },
    /// Rename sequences with prefix
    Rename {
        input: PathBuf,
        #[arg(short = 'p', long)]
        prefix: String,
        #[arg(short = 'o', long, default_value = "-")]
        output: String,
    },
    /// Reverse complement
    Revcomp {
        input: PathBuf,
        #[arg(short = 'o', long, default_value = "-")]
        output: String,
    },
    /// Convert to tab-separated (name\tsequence)
    Tab {
        input: PathBuf,
        #[arg(short = 'o', long, default_value = "-")]
        output: String,
    },
    /// Convert to BED (one interval per sequence)
    ToBed {
        input: PathBuf,
        #[arg(short = 'o', long, default_value = "-")]
        output: String,
    },
    /// Convert FASTA to FASTQ (with dummy quality)
    ToFastq {
        input: PathBuf,
        #[arg(long, default_value_t = b'I')]
        qual_char: u8,
        #[arg(short = 'o', long, default_value = "-")]
        output: String,
    },
    /// Deduplicate sequences
    Unique {
        input: PathBuf,
        #[arg(short = 'o', long, default_value = "-")]
        output: String,
    },
    /// Convert to uppercase
    Upper {
        input: PathBuf,
        #[arg(short = 'o', long, default_value = "-")]
        output: String,
    },
    /// Re-wrap sequences to fixed line width
    Wrap {
        input: PathBuf,
        #[arg(short = 'w', long, default_value_t = 80)]
        width: usize,
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

fn run(cli: Cli) -> Result<()> {
    match cli.command {
        Command::Count { input } => {
            let n = ops::count::count(&input)?;
            println!("{n}");
        }
        Command::Chroms { input, output } => {
            let mut out = open_output(&output)?;
            ops::chroms::fasta_chroms(&input, &mut out)?;
        }
        Command::Composition { input } => {
            let c = ops::composition::fasta_composition(&input)?;
            println!("A\t{}", c.a);
            println!("C\t{}", c.c);
            println!("G\t{}", c.g);
            println!("T\t{}", c.t);
            println!("N\t{}", c.n);
            println!("other\t{}", c.other);
            println!("total\t{}", c.total);
        }
        Command::GcContent { input, output } => {
            let mut out = open_output(&output)?;
            ops::gc_content::fasta_gc_content(&input, &mut out)?;
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
        Command::Tab { input, output } => {
            let mut out = open_output(&output)?;
            ops::tab::fasta_to_tab(&input, &mut out)?;
        }
        Command::ToBed { input, output } => {
            let mut out = open_output(&output)?;
            ops::to_bed::fasta_to_bed(&input, &mut out)?;
        }
        Command::ToFastq {
            input,
            qual_char,
            output,
        } => {
            let mut out = open_output(&output)?;
            ops::to_fastq::convert(&input, qual_char, &mut out)?;
        }
        Command::Unique { input, output } => {
            let mut out = open_output(&output)?;
            let (total, unique) = ops::unique::unique_fasta(&input, &mut out)?;
            eprintln!("{total} total, {unique} unique");
        }
        Command::Upper { input, output } => {
            let mut out = open_output(&output)?;
            ops::upper::uppercase(&input, &mut out)?;
        }
        Command::Wrap {
            input,
            width,
            output,
        } => {
            let mut out = open_output(&output)?;
            ops::wrap::fasta_wrap(&input, &mut out, width)?;
        }
    }
    Ok(())
}

fn main() -> ExitCode {
    let cli = Cli::parse();
    let common = cli.common.clone();
    rsomics_common::run(
        &common,
        rsomics_common::ToolMeta {
            name: "rsomics-fasta-utils",
            version: env!("CARGO_PKG_VERSION"),
        },
        || run(cli),
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use clap::CommandFactory;

    #[test]
    fn cli_debug_assert() {
        Cli::command().debug_assert();
    }
}
