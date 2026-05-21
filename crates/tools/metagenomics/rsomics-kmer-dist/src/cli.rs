use std::path::PathBuf;

use clap::Parser;
use rsomics_common::{CommonFlags, Result, RsomicsError, Tool, ToolMeta};
use rsomics_help::{Example, FlagSpec, HelpSpec, Section};

use rsomics_kmer_dist::{DistMetric, pairwise_distances};

pub const META: ToolMeta = ToolMeta {
    name: env!("CARGO_PKG_NAME"),
    version: env!("CARGO_PKG_VERSION"),
};

#[derive(Parser, Debug)]
#[command(name = "rsomics-kmer-dist", version, about, long_about = None, disable_help_flag = true)]
pub struct Cli {
    #[arg(required = true, num_args = 2..)]
    inputs: Vec<PathBuf>,
    #[arg(short = 'k', long, default_value_t = 21)]
    kmer_size: usize,
    #[arg(short = 'm', long, default_value = "jaccard")]
    metric: String,
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
        let metric = match self.metric.as_str() {
            "jaccard" => DistMetric::Jaccard,
            "bray-curtis" | "braycurtis" => DistMetric::BrayCurtis,
            "cosine" => DistMetric::Cosine,
            other => {
                return Err(RsomicsError::InvalidInput(format!(
                    "unknown metric '{other}': use jaccard/bray-curtis/cosine"
                )));
            }
        };
        let paths: Vec<&std::path::Path> = self.inputs.iter().map(PathBuf::as_path).collect();
        let mut out: Box<dyn std::io::Write> = if self.output == "-" {
            Box::new(std::io::stdout().lock())
        } else {
            Box::new(std::fs::File::create(&self.output).map_err(RsomicsError::Io)?)
        };
        pairwise_distances(&paths, self.kmer_size, &metric, &mut out)
    }
}

pub static HELP: HelpSpec = HelpSpec {
    name: env!("CARGO_PKG_NAME"),
    version: env!("CARGO_PKG_VERSION"),
    tagline: "Pairwise k-mer frequency distance between samples.",
    origin: None,
    usage_lines: &["<file1> <file2> [file3 ...] [-k 21] [-m jaccard]"],
    sections: &[Section {
        title: "OPTIONS",
        flags: &[
            FlagSpec {
                short: Some('k'),
                long: "kmer-size",
                aliases: &[],
                value: Some("<int>"),
                type_hint: Some("usize"),
                required: false,
                default: Some("21"),
                description: "K-mer size.",
                why_default: None,
            },
            FlagSpec {
                short: Some('m'),
                long: "metric",
                aliases: &[],
                value: Some("<name>"),
                type_hint: Some("String"),
                required: false,
                default: Some("jaccard"),
                description: "Distance metric: jaccard, bray-curtis, cosine.",
                why_default: None,
            },
        ],
    }],
    examples: &[Example {
        description: "Jaccard distance between two samples",
        command: "rsomics-kmer-dist sample1.fq sample2.fq -k 21 -m jaccard",
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
