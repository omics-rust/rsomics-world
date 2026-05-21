use clap::Parser;
use rsomics_common::{CommonFlags, Result, Tool, ToolMeta};
use rsomics_help::{Example, FlagSpec, HelpSpec, Origin, Section};
use rsomics_tax_assign::{classify_reads, load_kmer_db};
use std::path::PathBuf;

pub const META: ToolMeta = ToolMeta {
    name: env!("CARGO_PKG_NAME"),
    version: env!("CARGO_PKG_VERSION"),
};

#[derive(Parser, Debug)]
#[command(name = "rsomics-tax-assign", version, about, long_about = None, disable_help_flag = true)]
pub struct Cli {
    pub reads: PathBuf,
    #[arg(short = 'd', long)]
    db: PathBuf,
    #[arg(short = 'k', long, default_value_t = 31)]
    kmer_size: usize,
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
        let db = load_kmer_db(&self.db)?;
        let mut out: Box<dyn std::io::Write> = if self.output == "-" {
            Box::new(std::io::stdout().lock())
        } else {
            Box::new(std::fs::File::create(&self.output).map_err(rsomics_common::RsomicsError::Io)?)
        };
        let result = classify_reads(&self.reads, &db, self.kmer_size, &mut out)?;
        if !self.common.quiet {
            eprintln!(
                "{} reads: {} classified ({:.1}%), {} unclassified",
                result.total_reads,
                result.classified,
                if result.total_reads > 0 {
                    result.classified as f64 / result.total_reads as f64 * 100.0
                } else {
                    0.0
                },
                result.unclassified
            );
        }
        Ok(())
    }
}

pub static HELP: HelpSpec = HelpSpec {
    name: env!("CARGO_PKG_NAME"),
    version: env!("CARGO_PKG_VERSION"),
    tagline: "K-mer-based taxonomic classification of sequencing reads.",
    origin: Some(Origin {
        upstream: "Kraken2 / Centrifuge",
        upstream_license: "MIT / GPL-3",
        our_license: "MIT OR Apache-2.0",
        paper_doi: Some("10.1186/s13059-019-1891-0"),
    }),
    usage_lines: &["<reads.fq> -d <kmer_db.tsv> [-k 31] [-o output.tsv]"],
    sections: &[Section {
        title: "OPTIONS",
        flags: &[
            FlagSpec {
                short: Some('d'),
                long: "db",
                aliases: &[],
                value: Some("<path>"),
                type_hint: Some("PathBuf"),
                required: true,
                default: None,
                description: "K-mer database TSV (kmer_hash<TAB>taxid).",
                why_default: None,
            },
            FlagSpec {
                short: Some('k'),
                long: "kmer-size",
                aliases: &[],
                value: Some("<int>"),
                type_hint: Some("usize"),
                required: false,
                default: Some("31"),
                description: "K-mer size (must match database).",
                why_default: None,
            },
        ],
    }],
    examples: &[Example {
        description: "Classify reads",
        command: "rsomics-tax-assign reads.fq -d kraken_db.tsv -o classified.tsv",
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
