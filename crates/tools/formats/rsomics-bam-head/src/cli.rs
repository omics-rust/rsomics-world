use std::path::PathBuf;

use clap::Parser;
use rsomics_common::{CommonFlags, Result, RsomicsError, Tool, ToolMeta};
use rsomics_help::{Example, FlagSpec, HelpSpec, Origin, Section};

use rsomics_bam_head::{HeadOpts, head};

pub const META: ToolMeta = ToolMeta {
    name: env!("CARGO_PKG_NAME"),
    version: env!("CARGO_PKG_VERSION"),
};

#[derive(Parser, Debug)]
#[command(
    name = "rsomics-bam-head",
    version,
    about,
    long_about = None,
    disable_help_flag = true
)]
pub struct Cli {
    /// Input BAM file.
    pub input: PathBuf,

    /// Output SAM file (default stdout).
    #[arg(short = 'o', long = "output", default_value = "-")]
    output: String,

    /// Print only the first INT header lines (default: all).
    #[arg(short = 'H', long = "headers")]
    headers: Option<u64>,

    /// Also print the first INT alignment records (default: 0).
    #[arg(short = 'n', long = "records", default_value_t = 0)]
    records: u64,

    #[command(flatten)]
    pub common: CommonFlags,
}

impl Cli {
    pub fn execute(self) -> Result<()> {
        let opts = HeadOpts {
            header_lines: self.headers,
            records: self.records,
        };

        let output_path = (self.output != "-").then(|| PathBuf::from(&self.output));
        let stats = head(&self.input, output_path.as_deref(), &opts)?;

        if self.common.json {
            eprintln!(
                "{}",
                serde_json::to_string(&stats)
                    .map_err(|e| RsomicsError::InvalidInput(format!("JSON: {e}")))?
            );
        }

        Ok(())
    }
}

impl Tool for Cli {
    fn meta() -> ToolMeta {
        META
    }

    fn common(&self) -> &CommonFlags {
        &self.common
    }

    fn execute(self) -> Result<()> {
        self.execute()
    }
}

pub static HELP: HelpSpec = HelpSpec {
    name: META.name,
    version: META.version,
    tagline: "Print the header and the first N alignment records of a BAM as SAM.",
    origin: Some(Origin {
        upstream: "samtools head",
        upstream_license: "MIT",
        our_license: "MIT OR Apache-2.0",
        paper_doi: None,
    }),
    usage_lines: &["<in.bam> [-H header-lines] [-n records] [-o out.sam]"],
    sections: &[Section {
        title: "OPTIONS",
        flags: &[
            FlagSpec {
                short: Some('H'),
                long: "headers",
                aliases: &[],
                value: Some("INT"),
                type_hint: None,
                required: false,
                default: Some("all"),
                description: "Print only the first INT header lines.",
                why_default: None,
            },
            FlagSpec {
                short: Some('n'),
                long: "records",
                aliases: &[],
                value: Some("INT"),
                type_hint: None,
                required: false,
                default: Some("0"),
                description: "Also print the first INT alignment records.",
                why_default: None,
            },
        ],
    }],
    examples: &[
        Example {
            description: "Print all header lines",
            command: "rsomics-bam-head in.bam",
        },
        Example {
            description: "Print the header and first 5 records",
            command: "rsomics-bam-head -n 5 in.bam",
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
