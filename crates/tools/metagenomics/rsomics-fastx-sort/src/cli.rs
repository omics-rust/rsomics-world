use std::fs::File;
use std::io::{BufReader, BufWriter};
use std::path::PathBuf;

use clap::{Parser, ValueEnum};
use rsomics_common::{CommonFlags, Result, RsomicsError, Tool, ToolMeta};

use rsomics_fastx_sort::fasta::{FastaWidth, write_fasta_raw, write_fasta_with_size};
use rsomics_fastx_sort::{read_records, sort_by_length, sort_by_size};

pub const META: ToolMeta = ToolMeta {
    name: env!("CARGO_PKG_NAME"),
    version: env!("CARGO_PKG_VERSION"),
};

#[derive(Clone, Debug, ValueEnum)]
pub enum SortMode {
    /// Sort by abundance (`;size=N`) descending — vsearch `--sortbysize`.
    Size,
    /// Sort by sequence length descending — vsearch `--sortbylength`.
    Length,
}

#[derive(Parser, Debug)]
#[command(
    name = "rsomics-fastx-sort",
    version,
    about = "Deterministic FASTA sorting by abundance or length (vsearch --sortbysize / --sortbylength port)",
    long_about = None,
    disable_help_flag = true
)]
pub struct Cli {
    /// Input FASTA file (use `-` for stdin).
    pub input: PathBuf,

    /// Output FASTA file (use `-` for stdout).
    #[arg(short = 'o', long)]
    pub output: PathBuf,

    /// Sort mode: `size` (abundance descending) or `length` (length descending).
    #[arg(long, value_enum)]
    pub mode: SortMode,

    /// Append `;size=N` to each output header (strips existing annotation first).
    #[arg(long = "sizeout", default_value_t = false)]
    pub sizeout: bool,

    /// Accepted for compatibility; vsearch always parses `;size=N` for sort operations.
    #[arg(long = "sizein", default_value_t = false)]
    pub sizein: bool,

    /// Discard sequences shorter than this length.
    ///
    /// vsearch default for sort operations is 1 (unlike derep/search which use 32).
    #[arg(long = "minseqlength", default_value_t = 1)]
    pub minseqlength: usize,

    /// Discard sequences longer than this length.
    #[arg(long = "maxseqlength", default_value_t = 50000)]
    pub maxseqlength: usize,

    /// For `--mode size`: discard sequences with abundance below this.
    #[arg(long = "minsize", default_value_t = 0)]
    pub minsize: u64,

    /// For `--mode size`: discard sequences with abundance above this.
    #[arg(long = "maxsize", default_value_t = u64::MAX)]
    pub maxsize: u64,

    /// FASTA line wrap width; 0 = no wrapping.
    #[arg(long = "fasta-width", default_value_t = 80)]
    pub fasta_width: usize,

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
        let mut reader: Box<dyn std::io::BufRead> = if self.input.as_os_str() == "-" {
            Box::new(std::io::BufReader::new(std::io::stdin()))
        } else {
            Box::new(BufReader::new(File::open(&self.input).map_err(|e| {
                RsomicsError::InvalidInput(format!("{}: {e}", self.input.display()))
            })?))
        };

        let (mut records, discarded) =
            read_records(reader.as_mut(), self.minseqlength, self.maxseqlength)
                .map_err(|e| RsomicsError::InvalidInput(e.to_string()))?;

        match self.mode {
            SortMode::Size => {
                // Apply minsize/maxsize filters before sorting.
                records.retain(|r| r.abundance >= self.minsize && r.abundance <= self.maxsize);
                sort_by_size(&mut records);
            }
            SortMode::Length => {
                sort_by_length(&mut records);
            }
        }

        let mut writer: Box<dyn std::io::Write> = if self.output.as_os_str() == "-" {
            Box::new(BufWriter::new(std::io::stdout()))
        } else {
            Box::new(BufWriter::new(
                File::create(&self.output).map_err(RsomicsError::Io)?,
            ))
        };

        let width = FastaWidth(self.fasta_width);
        for record in &records {
            if self.sizeout {
                write_fasta_with_size(
                    writer.as_mut(),
                    &record.stripped_header,
                    record.abundance,
                    &record.seq,
                    width,
                )
                .map_err(RsomicsError::Io)?;
            } else {
                write_fasta_raw(writer.as_mut(), &record.raw_header, &record.seq, width)
                    .map_err(RsomicsError::Io)?;
            }
        }

        if !self.common.quiet {
            let total = records.len() + discarded;
            if discarded > 0 {
                eprintln!(
                    "minseqlength {}: {discarded} sequence(s) discarded.",
                    self.minseqlength
                );
            }
            eprintln!("{total} seqs in, {} seqs written", records.len());
        }

        Ok(())
    }
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
