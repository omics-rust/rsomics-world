use std::path::PathBuf;
use std::process::ExitCode;

use clap::{Parser, Subcommand};
use rsomics_common::{CommonFlags, Result, RsomicsError, ToolMeta};

use rsomics_fasta_utils::ops;

const META: ToolMeta = ToolMeta {
    name: env!("CARGO_PKG_NAME"),
    version: env!("CARGO_PKG_VERSION"),
};

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
    /// List unique sequence names (chromosomes)
    Chroms {
        input: PathBuf,
        #[arg(short = 'o', long, default_value = "-")]
        output: String,
    },
    /// Nucleotide composition
    Composition { input: PathBuf },
    /// Count sequences
    Count { input: PathBuf },
    /// Deduplicate sequences (by sequence or name)
    Dedup {
        input: PathBuf,
        #[arg(short = 'n', long)]
        by_name: bool,
        #[arg(short = 'o', long, default_value = "-")]
        output: String,
    },
    /// Extract sequences by name list
    Extract {
        input: PathBuf,
        #[arg(short = 'l', long)]
        list: PathBuf,
        #[arg(long)]
        exclude: bool,
        #[arg(short = 'o', long, default_value = "-")]
        output: String,
    },
    /// Filter sequences by length
    Filter {
        input: PathBuf,
        #[arg(short = 'm', long, default_value_t = 0)]
        min_len: usize,
        #[arg(short = 'M', long, default_value_t = 0)]
        max_len: usize,
        #[arg(short = 'o', long, default_value = "-")]
        output: String,
    },
    /// GC content per sequence
    GcContent {
        input: PathBuf,
        #[arg(short = 'o', long, default_value = "-")]
        output: String,
    },
    /// Filter records by name regex
    Grep {
        input: PathBuf,
        #[arg(short = 'p', long)]
        pattern: String,
        #[arg(long)]
        invert_match: bool,
        #[arg(short = 'o', long, default_value = "-")]
        output: String,
    },
    /// Output the first N sequences
    Head {
        input: PathBuf,
        #[arg(short = 'n', long, default_value_t = 10)]
        num: u64,
        #[arg(short = 'o', long, default_value = "-")]
        output: String,
    },
    /// Count k-mer frequencies
    Kmers {
        input: PathBuf,
        #[arg(short = 'k', default_value_t = 21)]
        k: usize,
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
    /// Random subsample of sequences
    Sample {
        input: PathBuf,
        #[arg(short = 'p', long, default_value_t = 0.1)]
        proportion: f64,
        #[arg(long, default_value_t = 42)]
        seed: u64,
        #[arg(short = 'o', long, default_value = "-")]
        output: String,
    },
    /// Randomly shuffle sequence order
    Shuffle {
        input: PathBuf,
        #[arg(long, default_value_t = 42)]
        seed: u64,
        #[arg(short = 'o', long, default_value = "-")]
        output: String,
    },
    /// Sort sequences by name or length
    Sort {
        input: PathBuf,
        #[arg(short = 'l', long)]
        by_length: bool,
        #[arg(short = 'L', long)]
        by_length_desc: bool,
        #[arg(short = 'o', long, default_value = "-")]
        output: String,
    },
    /// Split into multiple files by sequence count
    Split {
        input: PathBuf,
        #[arg(long, default_value_t = 1000)]
        seqs_per_file: u64,
        #[arg(long, default_value = "split_")]
        prefix: String,
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
    /// Deduplicate sequences (keep unique only)
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
    /// Sliding-window GC statistics
    Window {
        input: PathBuf,
        #[arg(short = 'w', long, default_value_t = 10000)]
        window: usize,
        #[arg(short = 's', long, default_value_t = 5000)]
        step: usize,
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

#[allow(clippy::too_many_lines)]
fn run(cli: Cli) -> Result<()> {
    match cli.command {
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
        Command::Count { input } => {
            let n = ops::count::count(&input)?;
            println!("{n}");
        }
        Command::Dedup {
            input,
            by_name,
            output,
        } => {
            let mut out = open_output(&output)?;
            ops::dedup::dedup_fasta(&input, &mut out, by_name)?;
        }
        Command::Extract {
            input,
            list,
            exclude,
            output,
        } => {
            let mut out = open_output(&output)?;
            ops::extract::extract_fasta(&input, &list, &mut out, exclude)?;
        }
        Command::Filter {
            input,
            min_len,
            max_len,
            output,
        } => {
            let mut out = open_output(&output)?;
            ops::filter::filter(&input, min_len, max_len, &mut out)?;
        }
        Command::GcContent { input, output } => {
            let mut out = open_output(&output)?;
            ops::gc_content::fasta_gc_content(&input, &mut out)?;
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
        Command::Kmers { input, k, output } => {
            let mut out = open_output(&output)?;
            ops::kmers::count_kmers(&input, &mut out, k)?;
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
            ops::shuffle::shuffle_fasta(&input, &mut out, seed)?;
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
        Command::Split {
            input,
            seqs_per_file,
            prefix,
        } => {
            let n = ops::split::split_by_count(&input, seqs_per_file, &prefix)?;
            eprintln!("{n} files written");
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
        Command::Window {
            input,
            window,
            step,
            output,
        } => {
            let mut out = open_output(&output)?;
            ops::window::window_stats(&input, &mut out, window, step)?;
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
    rsomics_common::run(&common, META, || run(cli))
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
