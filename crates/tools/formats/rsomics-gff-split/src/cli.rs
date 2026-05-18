use std::path::PathBuf;

use clap::Parser;
use rsomics_common::{CommonFlags, Result, ToolMeta};
use rsomics_help::{Example, FlagSpec, HelpSpec, Origin, Section};

use rsomics_gff_split::split_gff;

pub const META: ToolMeta = ToolMeta {
    name: env!("CARGO_PKG_NAME"),
    version: env!("CARGO_PKG_VERSION"),
};

#[derive(Parser, Debug)]
#[command(
    name = "rsomics-gff-split",
    version,
    about,
    long_about = None,
    disable_help_flag = true
)]
pub struct Cli {
    pub input: PathBuf,
    #[arg(short = 'o', long = "output-prefix")]
    prefix: PathBuf,
    #[command(flatten)]
    pub common: CommonFlags,
}

impl Cli {
    pub fn execute(self) -> Result<()> {
        let counts = split_gff(&self.input, &self.prefix)?;
        if !self.common.quiet {
            for (chrom, count) in &counts {
                eprintln!("{chrom}\t{count}");
            }
        }
        Ok(())
    }
}

pub static HELP: HelpSpec = HelpSpec {
    name: META.name,
    version: META.version,
    tagline: "Split GFF by chromosome.",
    origin: Some(Origin {
        upstream: "awk on GFF column 1",
        upstream_license: "MIT",
        our_license: "MIT OR Apache-2.0",
        paper_doi: None,
    }),
    usage_lines: &["<input.gff> -o <prefix>"],
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
            description: "Output prefix (creates <prefix>.<chrom>.gff).",
            why_default: None,
        }],
    }],
    examples: &[Example {
        description: "Split by chromosome",
        command: "rsomics-gff-split genes.gff -o split/out",
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
