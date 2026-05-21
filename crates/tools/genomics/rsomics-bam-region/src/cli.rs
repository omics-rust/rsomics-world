use std::path::PathBuf;

use clap::Parser;
use rsomics_common::{CommonFlags, Result, RsomicsError, Tool, ToolMeta};
use rsomics_help::{Example, FlagSpec, HelpSpec, Origin, Section};

use rsomics_bam_region::extract_region;

pub const META: ToolMeta = ToolMeta {
    name: env!("CARGO_PKG_NAME"),
    version: env!("CARGO_PKG_VERSION"),
};

#[derive(Parser, Debug)]
#[command(
    name = "rsomics-bam-region",
    version,
    about,
    long_about = None,
    disable_help_flag = true
)]
pub struct Cli {
    pub input: PathBuf,
    #[arg(short = 'r', long)]
    region: String,
    #[arg(short = 'c', long)]
    count: bool,
    #[arg(short = 'o', long, default_value = "-")]
    output: String,
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
        let mut out: Box<dyn std::io::Write> = if self.count || self.output == "-" {
            Box::new(std::io::stdout().lock())
        } else {
            Box::new(std::fs::File::create(&self.output).map_err(RsomicsError::Io)?)
        };
        let n = extract_region(&self.input, &self.region, &mut out, self.count)?;
        if !self.common.quiet {
            eprintln!("{n} records");
        }
        Ok(())
    }
}

pub static HELP: HelpSpec = HelpSpec {
    name: env!("CARGO_PKG_NAME"),
    version: env!("CARGO_PKG_VERSION"),
    tagline: "Extract BAM reads overlapping a genomic region — indexed random access.",
    origin: Some(Origin {
        upstream: "samtools view -L / samtools view <region>",
        upstream_license: "MIT",
        our_license: "MIT OR Apache-2.0",
        paper_doi: Some("10.1093/bioinformatics/btp352"),
    }),
    usage_lines: &["<input.bam> -r <chr:start-end> [-o output.bam]"],
    sections: &[Section {
        title: "OPTIONS",
        flags: &[
            FlagSpec {
                short: Some('r'),
                long: "region",
                aliases: &[],
                value: Some("<chr:start-end>"),
                type_hint: Some("String"),
                required: true,
                default: None,
                description: "Genomic region (e.g., chr1:1000-2000). Requires .bai index.",
                why_default: None,
            },
            FlagSpec {
                short: Some('c'),
                long: "count",
                aliases: &[],
                value: None,
                type_hint: Some("bool"),
                required: false,
                default: None,
                description: "Print count only instead of BAM output.",
                why_default: None,
            },
        ],
    }],
    examples: &[
        Example {
            description: "Extract reads in region",
            command: "rsomics-bam-region input.bam -r chr1:1000-2000 -o region.bam",
        },
        Example {
            description: "Count reads in region",
            command: "rsomics-bam-region input.bam -r chr1:1000-2000 -c",
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
