use std::path::PathBuf;

use clap::Parser;
use rsomics_common::{CommonFlags, Result, ToolMeta};
use rsomics_help::{Example, FlagSpec, HelpSpec, Origin, Section};

use rsomics_bam_merge::merge_bams;

pub const META: ToolMeta = ToolMeta {
    name: env!("CARGO_PKG_NAME"),
    version: env!("CARGO_PKG_VERSION"),
};

#[derive(Parser, Debug)]
#[command(
    name = "rsomics-bam-merge",
    version,
    about,
    long_about = None,
    disable_help_flag = true
)]
pub struct Cli {
    /// Input BAM files (2 or more).
    #[arg(required = true)]
    inputs: Vec<PathBuf>,

    /// Output BAM file (default stdout).
    #[arg(short = 'o', long = "output", default_value = "-")]
    output: String,

    #[command(flatten)]
    pub common: CommonFlags,
}

impl Cli {
    pub fn execute(self) -> Result<()> {
        let paths: Vec<&std::path::Path> = self.inputs.iter().map(|p| p.as_path()).collect();

        let mut out: Box<dyn std::io::Write> = if self.output == "-" {
            Box::new(std::io::stdout().lock())
        } else {
            Box::new(
                std::fs::File::create(&self.output)
                    .map_err(rsomics_common::RsomicsError::Io)?,
            )
        };

        let count = merge_bams(&paths, &mut out)?;

        if !self.common.json {
            eprintln!("merged {count} records from {} files", self.inputs.len());
        } else {
            let j = serde_json::json!({ "merged_records": count, "input_files": self.inputs.len() });
            eprintln!("{j}");
        }

        Ok(())
    }
}

pub static HELP: HelpSpec = HelpSpec {
    name: META.name,
    version: META.version,
    tagline: "Merge multiple sorted BAM files.",
    origin: Some(Origin {
        upstream: "samtools merge",
        upstream_license: "MIT",
        our_license: "MIT OR Apache-2.0",
        paper_doi: Some("10.1093/bioinformatics/btp352"),
    }),
    usage_lines: &["<input1.bam> <input2.bam> [...] -o merged.bam"],
    sections: &[Section {
        title: "OPTIONS",
        flags: &[FlagSpec {
            short: Some("-o"),
            long: "output",
            aliases: &[],
            value: Some("<path>"),
            type_hint: Some("Path"),
            required: false,
            default: Some("-"),
            description: "Output BAM file.",
            why_default: Some("stdout"),
        }],
    }],
    examples: &[Example {
        description: "Merge two sorted BAM files",
        command: "rsomics-bam-merge sample1.bam sample2.bam -o merged.bam",
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
