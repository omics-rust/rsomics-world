use std::path::PathBuf;

use clap::Parser;
use rsomics_common::{CommonFlags, Result, RsomicsError, Tool, ToolMeta};
use rsomics_help::{Example, FlagSpec, HelpSpec, Origin, Section};

use rsomics_peak_count::peak_counts;

pub const META: ToolMeta = ToolMeta {
    name: env!("CARGO_PKG_NAME"),
    version: env!("CARGO_PKG_VERSION"),
};

#[derive(Parser, Debug)]
#[command(name = "rsomics-peak-count", version, about, long_about = None, disable_help_flag = true)]
pub struct Cli {
    pub bam: PathBuf,
    #[arg(short = 'b', long)]
    bed: PathBuf,
    #[arg(long, default_value_t = 0)]
    min_mapq: u8,
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
        let mut out: Box<dyn std::io::Write> = if self.output == "-" {
            Box::new(std::io::stdout().lock())
        } else {
            Box::new(std::fs::File::create(&self.output).map_err(RsomicsError::Io)?)
        };
        let total = peak_counts(&self.bam, &self.bed, &mut out, self.min_mapq)?;
        if !self.common.quiet {
            eprintln!("{total} reads counted in peaks");
        }
        Ok(())
    }
}

pub static HELP: HelpSpec = HelpSpec {
    name: env!("CARGO_PKG_NAME"),
    version: env!("CARGO_PKG_VERSION"),
    tagline: "Count BAM reads per BED peak region — ChIP-seq/ATAC-seq quantification.",
    origin: Some(Origin {
        upstream: "bedtools multicov / featureCounts",
        upstream_license: "MIT / GPL-3",
        our_license: "MIT OR Apache-2.0",
        paper_doi: None,
    }),
    usage_lines: &["<input.bam> -b <peaks.bed> [-o counts.tsv]"],
    sections: &[Section {
        title: "OPTIONS",
        flags: &[
            FlagSpec {
                short: Some('b'),
                long: "bed",
                aliases: &[],
                value: Some("<path>"),
                type_hint: Some("PathBuf"),
                required: true,
                default: None,
                description: "BED file with peak regions.",
                why_default: None,
            },
            FlagSpec {
                short: None,
                long: "min-mapq",
                aliases: &[],
                value: Some("<u8>"),
                type_hint: Some("u8"),
                required: false,
                default: Some("0"),
                description: "Minimum mapping quality.",
                why_default: None,
            },
        ],
    }],
    examples: &[Example {
        description: "Count reads per peak",
        command: "rsomics-peak-count aligned.bam -b peaks.bed -o counts.tsv",
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
