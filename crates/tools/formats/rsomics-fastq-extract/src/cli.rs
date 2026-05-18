use std::path::PathBuf;

use clap::Parser;
use rsomics_common::{CommonFlags, Result, RsomicsError, ToolMeta};
use rsomics_help::{Example, FlagSpec, HelpSpec, Origin, Section};

use rsomics_fastq_extract::extract_fastq;

pub const META: ToolMeta = ToolMeta {
    name: env!("CARGO_PKG_NAME"),
    version: env!("CARGO_PKG_VERSION"),
};

#[derive(Parser, Debug)]
#[command(
    name = "rsomics-fastq-extract",
    version,
    about,
    long_about = None,
    disable_help_flag = true
)]
pub struct Cli {
    pub input: PathBuf,
    #[arg(short = 'l', long = "list")]
    names: PathBuf,
    #[arg(short = 'o', long = "output", default_value = "-")]
    output: String,
    #[arg(long = "exclude")]
    exclude: bool,
    #[command(flatten)]
    pub common: CommonFlags,
}

impl Cli {
    pub fn execute(self) -> Result<()> {
        let mut out: Box<dyn std::io::Write> = if self.output == "-" {
            Box::new(std::io::stdout().lock())
        } else {
            Box::new(std::fs::File::create(&self.output).map_err(RsomicsError::Io)?)
        };
        let count = extract_fastq(&self.input, &self.names, &mut out, self.exclude)?;
        if !self.common.quiet {
            eprintln!("{count} reads extracted");
        }
        Ok(())
    }
}

pub static HELP: HelpSpec = HelpSpec {
    name: META.name,
    version: META.version,
    tagline: "Extract reads by name from FASTQ.",
    origin: Some(Origin {
        upstream: "seqtk subseq / filterbyname.sh",
        upstream_license: "MIT",
        our_license: "MIT OR Apache-2.0",
        paper_doi: None,
    }),
    usage_lines: &["<input.fq> -l <names.txt> [--exclude] [-o out.fq]"],
    sections: &[Section {
        title: "OPTIONS",
        flags: &[
            FlagSpec {
                short: Some('l'),
                long: "list",
                aliases: &[],
                value: Some("<path>"),
                type_hint: Some("Path"),
                required: true,
                default: None,
                description: "File with read names (one per line).",
                why_default: None,
            },
            FlagSpec {
                short: None,
                long: "exclude",
                aliases: &[],
                value: None,
                type_hint: None,
                required: false,
                default: None,
                description: "Exclude listed names instead of including.",
                why_default: None,
            },
        ],
    }],
    examples: &[Example {
        description: "Extract specific reads",
        command: "rsomics-fastq-extract reads.fq -l names.txt -o subset.fq",
    }],
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
