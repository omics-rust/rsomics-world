use std::path::PathBuf;

use clap::Parser;
use rsomics_common::{CommonFlags, Result, RsomicsError, ToolMeta};
use rsomics_help::{Example, HelpSpec, Origin, Section};

use rsomics_bed_overlap::compute_overlap;

pub const META: ToolMeta = ToolMeta {
    name: env!("CARGO_PKG_NAME"),
    version: env!("CARGO_PKG_VERSION"),
};

#[derive(Parser, Debug)]
#[command(
    name = "rsomics-bed-overlap",
    version,
    about,
    long_about = None,
    disable_help_flag = true
)]
pub struct Cli {
    /// First BED file (A).
    pub a: PathBuf,

    /// Second BED file (B).
    pub b: PathBuf,

    #[command(flatten)]
    pub common: CommonFlags,
}

impl Cli {
    pub fn execute(self) -> Result<()> {
        let stats = compute_overlap(&self.a, &self.b)?;

        if self.common.json {
            serde_json::to_writer_pretty(std::io::stdout(), &stats)
                .map_err(|e| RsomicsError::InvalidInput(format!("{e}")))?;
            println!();
        } else {
            println!("A intervals:\t{}", stats.a_count);
            println!("B intervals:\t{}", stats.b_count);
            println!("A with overlap:\t{}", stats.a_with_overlap);
            println!("B with overlap:\t{}", stats.b_with_overlap);
            println!("A bases:\t{}", stats.a_bases);
            println!("B bases:\t{}", stats.b_bases);
            println!("Overlap bases:\t{}", stats.overlap_bases);
            println!("Jaccard:\t{:.6}", stats.jaccard);
        }

        Ok(())
    }
}

pub static HELP: HelpSpec = HelpSpec {
    name: META.name,
    version: META.version,
    tagline: "Compute overlap statistics between two BED files.",
    origin: Some(Origin {
        upstream: "bedtools jaccard + bedtools intersect -wa",
        upstream_license: "MIT",
        our_license: "MIT OR Apache-2.0",
        paper_doi: None,
    }),
    usage_lines: &["<A.bed> <B.bed> [--json]"],
    sections: &[Section {
        title: "OPTIONS",
        flags: &[],
    }],
    examples: &[Example {
        description: "Compute overlap statistics",
        command: "rsomics-bed-overlap peaks.bed genes.bed",
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
