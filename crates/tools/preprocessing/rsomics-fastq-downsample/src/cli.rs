use clap::Parser;
use rsomics_common::{CommonFlags, Result, RsomicsError, Tool, ToolMeta};
use rsomics_fastq_downsample::downsample;
use rsomics_help::{Example, FlagSpec, HelpSpec, Section};
use std::path::PathBuf;

pub const META: ToolMeta = ToolMeta {
    name: env!("CARGO_PKG_NAME"),
    version: env!("CARGO_PKG_VERSION"),
};

#[derive(Parser, Debug)]
#[command(name = "rsomics-fastq-downsample", version, about, long_about = None, disable_help_flag = true)]
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
        let (total, kept) =
            downsample(&self.input, self.fraction, self.common.seed_rng(), &mut out)?;
        if !self.common.quiet {
            eprintln!("{kept}/{total} reads kept");
        }
        Ok(())
    }
}

pub static HELP: HelpSpec = HelpSpec {
    name: env!("CARGO_PKG_NAME"),
    version: env!("CARGO_PKG_VERSION"),
    tagline: "Deterministic FASTQ downsampling by fraction.",
    origin: None,
    usage_lines: &["<reads.fq> -f <fraction> [--seed 42] [-o output.fq]"],
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
    examples: &[Example {
        description: "Keep 10% of reads",
        command: "rsomics-fastq-downsample reads.fq.gz -f 0.1 --seed 42 -o sub.fq",
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
