use std::fs::File;
use std::io::{BufReader, BufWriter};
use std::path::PathBuf;

use clap::Parser;
use rsomics_common::{CommonFlags, Result, RsomicsError, Tool, ToolMeta};

use rsomics_rereplicate::rereplicate;

pub const META: ToolMeta = ToolMeta {
    name: env!("CARGO_PKG_NAME"),
    version: env!("CARGO_PKG_VERSION"),
};

#[derive(Parser, Debug)]
#[command(
    name = "rsomics-rereplicate",
    version,
    about = "Expand ;size=N abundance back into N copies (vsearch --rereplicate port)",
    long_about = None,
    disable_help_flag = true
)]
pub struct Cli {
    /// Input FASTA file (use `-` for stdin).
    pub input: PathBuf,

    /// Output FASTA file (use `-` for stdout).
    #[arg(short = 'o', long)]
    pub output: PathBuf,

    /// Append `;size=1` to each output record (vsearch --sizeout).
    #[arg(long = "sizeout", default_value_t = false)]
    pub sizeout: bool,

    /// FASTA sequence line wrap width; 0 = no wrapping.
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

        let mut writer: Box<dyn std::io::Write> = if self.output.as_os_str() == "-" {
            Box::new(BufWriter::new(std::io::stdout()))
        } else {
            Box::new(BufWriter::new(
                File::create(&self.output).map_err(RsomicsError::Io)?,
            ))
        };

        let (amplicons, reads, missing) = rereplicate(
            reader.as_mut(),
            writer.as_mut(),
            self.sizeout,
            self.fasta_width,
        )
        .map_err(|e| RsomicsError::InvalidInput(e.to_string()))?;

        if !self.common.quiet {
            if missing > 0 {
                eprintln!(
                    "WARNING: Missing abundance information for some input sequences, assumed 1"
                );
            }
            eprintln!("Rereplicated {reads} reads from {amplicons} amplicons");
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
