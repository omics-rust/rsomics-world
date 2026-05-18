use std::path::PathBuf;

use clap::Parser;
use rsomics_common::{CommonFlags, Result, RsomicsError, ToolMeta};
use rsomics_help::{Example, FlagSpec, HelpSpec, Origin, Section};

use rsomics_fasta_translate::translate_fasta;

pub const META: ToolMeta = ToolMeta {
    name: env!("CARGO_PKG_NAME"),
    version: env!("CARGO_PKG_VERSION"),
};

#[derive(Parser, Debug)]
#[command(
    name = "rsomics-fasta-translate",
    version,
    about,
    long_about = None,
    disable_help_flag = true
)]
pub struct Cli {
    /// Input DNA/RNA FASTA file.
    pub input: PathBuf,

    /// Output file (default stdout).
    #[arg(short = 'o', long = "output", default_value = "-")]
    output: String,

    /// Reading frames (comma-separated: 1,2,3,-1,-2,-3 or 6 for all).
    #[arg(short = 'f', long = "frames", default_value = "1")]
    frames: String,

    #[command(flatten)]
    pub common: CommonFlags,
}

impl Cli {
    pub fn execute(self) -> Result<()> {
        let frames: Vec<i8> = if self.frames == "6" {
            vec![1, 2, 3, -1, -2, -3]
        } else {
            self.frames
                .split(',')
                .map(|s| {
                    s.trim()
                        .parse::<i8>()
                        .map_err(|_| RsomicsError::InvalidInput(format!("bad frame: {s}")))
                })
                .collect::<Result<Vec<_>>>()?
        };

        let mut out: Box<dyn std::io::Write> = if self.output == "-" {
            Box::new(std::io::stdout().lock())
        } else {
            Box::new(std::fs::File::create(&self.output).map_err(RsomicsError::Io)?)
        };

        let count = translate_fasta(&self.input, &mut out, &frames, 1)?;

        if !self.common.quiet {
            eprintln!("{count} protein sequences");
        }

        Ok(())
    }
}

pub static HELP: HelpSpec = HelpSpec {
    name: META.name,
    version: META.version,
    tagline: "Translate DNA/RNA FASTA to protein sequences.",
    origin: Some(Origin {
        upstream: "transeq (EMBOSS) / seqkit translate",
        upstream_license: "MIT",
        our_license: "MIT OR Apache-2.0",
        paper_doi: None,
    }),
    usage_lines: &["<input.fasta> [-f 6] [-o proteins.fasta]"],
    sections: &[Section {
        title: "OPTIONS",
        flags: &[FlagSpec {
            short: Some('f'),
            long: "frames",
            aliases: &[],
            value: Some("<frames>"),
            type_hint: Some("String"),
            required: false,
            default: Some("1"),
            description: "Reading frames (1-3, -1 to -3, or 6 for all).",
            why_default: None,
        }],
    }],
    examples: &[
        Example {
            description: "Translate frame 1",
            command: "rsomics-fasta-translate input.fasta",
        },
        Example {
            description: "Six-frame translation",
            command: "rsomics-fasta-translate input.fasta -f 6",
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
