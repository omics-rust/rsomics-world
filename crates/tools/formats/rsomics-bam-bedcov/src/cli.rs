use std::path::PathBuf;

use clap::Parser;
use rsomics_common::{CommonFlags, Result, RsomicsError, Tool, ToolMeta};
use rsomics_help::{Example, FlagSpec, HelpSpec, Origin, Section};

use rsomics_bam_bedcov::{BedcovOpts, bedcov};

pub const META: ToolMeta = ToolMeta {
    name: env!("CARGO_PKG_NAME"),
    version: env!("CARGO_PKG_VERSION"),
};

#[derive(Parser, Debug)]
#[command(
    name = "rsomics-bam-bedcov",
    version,
    about,
    long_about = None,
    disable_help_flag = true
)]
pub struct Cli {
    /// BED file with query regions.
    pub bed: PathBuf,

    /// Input BAM file(s) — at least one required.
    #[arg(required = true)]
    pub bams: Vec<PathBuf>,

    /// Output file (default stdout).
    #[arg(short = 'o', long = "output", default_value = "-")]
    output: String,

    /// Minimum mapping quality (like samtools bedcov -Q).
    #[arg(short = 'Q', long = "min-mq", default_value_t = 0)]
    min_mapq: u8,

    #[command(flatten)]
    pub common: CommonFlags,
}

impl Cli {
    pub fn execute(self) -> Result<()> {
        let opts = BedcovOpts {
            min_mapq: self.min_mapq,
            skip_flags: 0x704,
        };

        let mut out: Box<dyn std::io::Write> = if self.output == "-" {
            Box::new(std::io::stdout().lock())
        } else {
            Box::new(std::fs::File::create(&self.output).map_err(RsomicsError::Io)?)
        };

        let workers = std::num::NonZero::new(self.common.thread_count())
            .unwrap_or(std::num::NonZero::<usize>::MIN);
        let regions = bedcov(&self.bed, &self.bams, &opts, workers, &mut out)?;

        if self.common.json {
            let j = serde_json::json!({ "regions": regions });
            eprintln!("{j}");
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
    tagline: "Per-BED-region read depth.",
    origin: Some(Origin {
        upstream: "samtools bedcov",
        upstream_license: "MIT",
        our_license: "MIT OR Apache-2.0",
        paper_doi: Some("10.1093/bioinformatics/btp352"),
    }),
    usage_lines: &["<regions.bed> <in1.bam> [in2.bam ...]"],
    sections: &[Section {
        title: "OPTIONS",
        flags: &[FlagSpec {
            short: Some('Q'),
            long: "min-mq",
            aliases: &[],
            value: Some("<INT>"),
            type_hint: Some("u8"),
            required: false,
            default: Some("0"),
            description: "Minimum mapping quality (same as samtools bedcov -Q).",
            why_default: None,
        }],
    }],
    examples: &[Example {
        description: "Compute per-region coverage for one BAM",
        command: "rsomics-bam-bedcov regions.bed aln.bam",
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
