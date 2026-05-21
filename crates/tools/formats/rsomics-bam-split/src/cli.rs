use std::path::PathBuf;

use clap::Parser;
use rsomics_common::{CommonFlags, Result, Tool, ToolMeta};
use rsomics_help::{Example, FlagSpec, HelpSpec, Origin, Section};

use rsomics_bam_split::split_by_reference;

pub const META: ToolMeta = ToolMeta {
    name: env!("CARGO_PKG_NAME"),
    version: env!("CARGO_PKG_VERSION"),
};

#[derive(Parser, Debug)]
#[command(
    name = "rsomics-bam-split",
    version,
    about,
    long_about = None,
    disable_help_flag = true
)]
pub struct Cli {
    /// Input BAM file.
    pub input: PathBuf,

    /// Output prefix (creates <prefix>.<RG>.bam per read group).
    #[arg(short = 'o', long = "output-prefix")]
    output_prefix: PathBuf,

    #[command(flatten)]
    pub common: CommonFlags,
}

impl Cli {
    pub fn execute(self) -> Result<()> {
        let counts = split_by_reference(&self.input, &self.output_prefix)?;

        if self.common.json {
            let j = serde_json::json!(counts);
            eprintln!("{j}");
        } else {
            for (rg, count) in &counts {
                eprintln!("{rg}\t{count}");
            }
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
    tagline: "Split BAM by read group.",
    origin: Some(Origin {
        upstream: "samtools split",
        upstream_license: "MIT",
        our_license: "MIT OR Apache-2.0",
        paper_doi: Some("10.1093/bioinformatics/btp352"),
    }),
    usage_lines: &["<input.bam> -o <prefix>"],
    sections: &[Section {
        title: "OPTIONS",
        flags: &[FlagSpec {
            short: Some('o'),
            long: "output-prefix",
            aliases: &[],
            value: Some("<prefix>"),
            type_hint: Some("Path"),
            required: true,
            default: None,
            description: "Output prefix (creates <prefix>.<RG>.bam per read group).",
            why_default: None,
        }],
    }],
    examples: &[Example {
        description: "Split by read group",
        command: "rsomics-bam-split input.bam -o output/split",
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
