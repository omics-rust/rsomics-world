use std::fs::File;
use std::io::{BufReader, BufWriter};
use std::path::PathBuf;

use clap::Parser;
use rsomics_common::{CommonFlags, Result, RsomicsError, Tool, ToolMeta};

use rsomics_derep_prefix::derep_prefix;
use rsomics_derep_prefix::fasta::{FastaWidth, write_fasta};

pub const META: ToolMeta = ToolMeta {
    name: env!("CARGO_PKG_NAME"),
    version: env!("CARGO_PKG_VERSION"),
};

#[derive(Parser, Debug)]
#[command(
    name = "rsomics-derep-prefix",
    version,
    about = "Prefix FASTA dereplication (vsearch --derep_prefix port)",
    long_about = None,
    disable_help_flag = true
)]
pub struct Cli {
    /// Input FASTA file (use `-` for stdin).
    pub input: PathBuf,

    /// Output FASTA file (use `-` for stdout).
    #[arg(short = 'o', long)]
    pub output: PathBuf,

    /// Parse `;size=N` abundance annotations from input headers.
    #[arg(long = "sizein", default_value_t = false)]
    pub sizein: bool,

    /// Discard sequences shorter than this length (vsearch default 32).
    #[arg(long = "minseqlength", default_value_t = 32)]
    pub minseqlength: usize,

    /// Discard sequences longer than this length (vsearch default 50000).
    #[arg(long = "maxseqlength", default_value_t = 50000)]
    pub maxseqlength: usize,

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

        let (clusters, discarded) = derep_prefix(
            reader.as_mut(),
            self.sizein,
            self.minseqlength,
            self.maxseqlength,
        )
        .map_err(|e| RsomicsError::InvalidInput(e.to_string()))?;

        let mut writer: Box<dyn std::io::Write> = if self.output.as_os_str() == "-" {
            Box::new(BufWriter::new(std::io::stdout()))
        } else {
            Box::new(BufWriter::new(
                File::create(&self.output).map_err(RsomicsError::Io)?,
            ))
        };

        let width = FastaWidth(self.fasta_width);
        let unique = clusters.len();
        let mut total_in: u64 = 0;
        for cluster in &clusters {
            total_in += cluster.abundance;
            write_fasta(
                writer.as_mut(),
                &cluster.label,
                cluster.abundance,
                &cluster.seq,
                width,
            )
            .map_err(RsomicsError::Io)?;
        }

        if !self.common.quiet {
            if discarded > 0 {
                eprintln!(
                    "minseqlength {}: {discarded} sequence(s) discarded.",
                    self.minseqlength
                );
            }
            eprintln!("{total_in} seqs in, {unique} unique sequences out");
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
