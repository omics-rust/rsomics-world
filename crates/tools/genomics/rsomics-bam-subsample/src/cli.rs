use std::path::PathBuf;

use clap::Parser;
use rsomics_common::{CommonFlags, Result, RsomicsError, Tool, ToolMeta};
use rsomics_help::{Example, FlagSpec, HelpSpec, Origin, Section};

use rsomics_bam_subsample::subsample_bam;

pub const META: ToolMeta = ToolMeta {
    name: env!("CARGO_PKG_NAME"),
    version: env!("CARGO_PKG_VERSION"),
};

#[derive(Parser, Debug)]
#[command(
    name = "rsomics-bam-subsample",
    version,
    about,
    long_about = None,
    disable_help_flag = true
)]
pub struct Cli {
    pub input: PathBuf,
    #[arg(short = 'f', long, default_value_t = 0.1)]
    fraction: f64,
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
        let kept = subsample_bam(&self.input, &mut out, self.fraction, self.common.seed_rng())?;
        if !self.common.quiet {
            eprintln!("{kept} records kept");
        }
        Ok(())
    }
}

pub static HELP: HelpSpec = HelpSpec {
    name: env!("CARGO_PKG_NAME"),
    version: env!("CARGO_PKG_VERSION"),
    tagline: "Random downsampling of BAM/SAM records by fraction.",
    origin: Some(Origin {
        upstream: "samtools view -s",
        upstream_license: "MIT",
        our_license: "MIT OR Apache-2.0",
        paper_doi: Some("10.1093/bioinformatics/btp352"),
    }),
    usage_lines: &["<input.bam> -f <fraction> [-o output.bam]"],
    sections: &[Section {
        title: "OPTIONS",
        flags: &[FlagSpec {
            short: Some('f'),
            long: "fraction",
            aliases: &[],
            value: Some("<float>"),
            type_hint: Some("f64"),
            required: false,
            default: Some("0.1"),
            description: "Fraction of reads to keep (0.0–1.0).",
            why_default: None,
        }],
    }],
    examples: &[
        Example {
            description: "Keep 10% of reads",
            command: "rsomics-bam-subsample input.bam -f 0.1 -o sub.bam",
        },
        Example {
            description: "Keep 50% with fixed seed",
            command: "rsomics-bam-subsample input.bam -f 0.5 --seed 123 -o half.bam",
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
