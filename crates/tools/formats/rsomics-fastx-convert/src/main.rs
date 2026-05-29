use std::process::ExitCode;

use clap::{Parser, Subcommand};

use rsomics_fastx_convert::{
    fa2fq, fq2fa, fx2tab, open_input, open_input_buffered, open_output, phred_recode, tab2fx,
};

#[derive(Parser, Debug)]
#[command(
    name = "rsomics-fastx-convert",
    version,
    about = "Convert between FASTA/FASTQ formats",
    long_about = None
)]
struct Cli {
    #[command(subcommand)]
    cmd: Cmd,
}

#[derive(Subcommand, Debug)]
enum Cmd {
    /// Convert FASTQ to FASTA (strip quality scores)
    Fq2fa {
        /// Input file ('-' for stdin)
        #[arg(short, long, default_value = "-")]
        input: String,
        /// Output file ('-' for stdout)
        #[arg(short, long, default_value = "-")]
        output: String,
    },
    /// Convert FASTA to FASTQ (add dummy Q40 quality)
    Fa2fq {
        /// Input file ('-' for stdin)
        #[arg(short, long, default_value = "-")]
        input: String,
        /// Output file ('-' for stdout)
        #[arg(short, long, default_value = "-")]
        output: String,
    },
    /// Convert FASTA/FASTQ to tab-separated (name, seq[, qual])
    Fx2tab {
        /// Input file ('-' for stdin)
        #[arg(short, long, default_value = "-")]
        input: String,
        /// Output file ('-' for stdout)
        #[arg(short, long, default_value = "-")]
        output: String,
    },
    /// Convert tab-separated (name, seq[, qual]) back to FASTA/FASTQ
    Tab2fx {
        /// Input file ('-' for stdin)
        #[arg(short, long, default_value = "-")]
        input: String,
        /// Output file ('-' for stdout)
        #[arg(short, long, default_value = "-")]
        output: String,
    },
    /// Re-encode Phred quality scores between Phred+33 and Phred+64
    Phred {
        /// Input file ('-' for stdin)
        #[arg(short, long, default_value = "-")]
        input: String,
        /// Output file ('-' for stdout)
        #[arg(short, long, default_value = "-")]
        output: String,
        /// Phred offset of the input quality encoding (33 = Sanger/Illumina 1.8+, 64 = Illumina 1.3-1.7)
        #[arg(long, default_value = "33")]
        from: u8,
    },
}

fn run(cli: Cli) -> anyhow::Result<()> {
    match cli.cmd {
        Cmd::Fq2fa { input, output } => {
            let inp = open_input(&input)?;
            let mut out = open_output(&output)?;
            fq2fa(inp, &mut out)
        }
        Cmd::Fa2fq { input, output } => {
            let inp = open_input(&input)?;
            let mut out = open_output(&output)?;
            fa2fq(inp, &mut out)
        }
        Cmd::Fx2tab { input, output } => {
            let inp = open_input(&input)?;
            let mut out = open_output(&output)?;
            fx2tab(inp, &mut out)
        }
        Cmd::Tab2fx { input, output } => {
            let inp = open_input_buffered(&input)?;
            let mut out = open_output(&output)?;
            tab2fx(inp, &mut out)
        }
        Cmd::Phred {
            input,
            output,
            from,
        } => {
            let inp = open_input(&input)?;
            let mut out = open_output(&output)?;
            phred_recode(inp, &mut out, from)
        }
    }
}

fn main() -> ExitCode {
    let cli = Cli::parse();
    match run(cli) {
        Ok(()) => ExitCode::SUCCESS,
        Err(e) => {
            eprintln!("error: {e:#}");
            ExitCode::FAILURE
        }
    }
}

#[cfg(test)]
mod tests {
    use super::Cli;
    use clap::CommandFactory;

    #[test]
    fn cli_debug_assert() {
        Cli::command().debug_assert();
    }
}
