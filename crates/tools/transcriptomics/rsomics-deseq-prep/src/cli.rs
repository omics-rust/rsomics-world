use clap::Parser;
use rsomics_common::{CommonFlags, Result, RsomicsError, Tool, ToolMeta};
use rsomics_deseq_prep::filter_low_counts;
use rsomics_help::{Example, FlagSpec, HelpSpec, Section};
use std::path::PathBuf;

pub const META: ToolMeta = ToolMeta {
    name: env!("CARGO_PKG_NAME"),
    version: env!("CARGO_PKG_VERSION"),
};

#[derive(Parser, Debug)]
#[command(name = "rsomics-deseq-prep", version, about, long_about = None, disable_help_flag = true)]
pub struct Cli {
    pub input: PathBuf,
    #[arg(long, default_value_t = 10)]
    min_count: u64,
    #[arg(long, default_value_t = 2)]
    min_samples: usize,
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
            filter_low_counts(&self.input, self.min_count, self.min_samples, &mut out)?;
        if !self.common.quiet {
            eprintln!("{kept}/{total} genes kept");
        }
        Ok(())
    }
}

pub static HELP: HelpSpec = HelpSpec {
    name: env!("CARGO_PKG_NAME"),
    version: env!("CARGO_PKG_VERSION"),
    tagline: "Filter low-count genes from a count matrix — pre-DESeq2 preparation.",
    origin: None,
    usage_lines: &["<counts.tsv> [--min-count 10] [--min-samples 2] [-o filtered.tsv]"],
    sections: &[Section {
        title: "OPTIONS",
        flags: &[
            FlagSpec {
                short: None,
                long: "min-count",
                aliases: &[],
                value: Some("<int>"),
                type_hint: Some("u64"),
                required: false,
                default: Some("10"),
                description: "Minimum count threshold per sample.",
                why_default: None,
            },
            FlagSpec {
                short: None,
                long: "min-samples",
                aliases: &[],
                value: Some("<int>"),
                type_hint: Some("usize"),
                required: false,
                default: Some("2"),
                description: "Minimum samples meeting threshold.",
                why_default: None,
            },
        ],
    }],
    examples: &[Example {
        description: "Keep genes with ≥10 counts in ≥3 samples",
        command: "rsomics-deseq-prep counts.tsv --min-count 10 --min-samples 3 -o filtered.tsv",
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
