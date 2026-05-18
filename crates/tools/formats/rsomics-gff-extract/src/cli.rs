use std::path::PathBuf;

use clap::Parser;
use rsomics_common::{CommonFlags, Result, RsomicsError, ToolMeta};
use rsomics_help::{Example, FlagSpec, HelpSpec, Origin, Section};

use rsomics_gff_extract::extract_attributes;

pub const META: ToolMeta = ToolMeta {
    name: env!("CARGO_PKG_NAME"),
    version: env!("CARGO_PKG_VERSION"),
};

#[derive(Parser, Debug)]
#[command(
    name = "rsomics-gff-extract",
    version,
    about,
    long_about = None,
    disable_help_flag = true
)]
pub struct Cli {
    /// Input GFF/GTF file.
    pub input: PathBuf,

    /// Output file (default stdout).
    #[arg(short = 'o', long = "output", default_value = "-")]
    output: String,

    /// Attribute keys to extract (comma-separated).
    #[arg(short = 'k', long = "keys", default_value = "gene_id,gene_name")]
    keys: String,

    /// Only extract from this feature type (e.g. gene, exon).
    #[arg(long = "feature-type")]
    feature_type: Option<String>,

    #[command(flatten)]
    pub common: CommonFlags,
}

impl Cli {
    pub fn execute(self) -> Result<()> {
        let keys: Vec<String> = self.keys.split(',').map(|s| s.trim().to_string()).collect();

        let mut out: Box<dyn std::io::Write> = if self.output == "-" {
            Box::new(std::io::stdout().lock())
        } else {
            Box::new(std::fs::File::create(&self.output).map_err(RsomicsError::Io)?)
        };

        let count = extract_attributes(&self.input, &mut out, &keys, self.feature_type.as_deref())?;

        if !self.common.quiet {
            eprintln!("{count} records");
        }

        Ok(())
    }
}

pub static HELP: HelpSpec = HelpSpec {
    name: META.name,
    version: META.version,
    tagline: "Extract specific attributes from GFF/GTF records.",
    origin: Some(Origin {
        upstream: "awk on GFF column 9",
        upstream_license: "N/A",
        our_license: "MIT OR Apache-2.0",
        paper_doi: None,
    }),
    usage_lines: &["<input.gff> [-k gene_id,gene_name] [--feature-type gene]"],
    sections: &[Section {
        title: "OPTIONS",
        flags: &[
            FlagSpec {
                short: Some('k'),
                long: "keys",
                aliases: &[],
                value: Some("<keys>"),
                type_hint: Some("String"),
                required: false,
                default: Some("gene_id,gene_name"),
                description: "Comma-separated attribute keys to extract.",
                why_default: None,
            },
            FlagSpec {
                short: None,
                long: "feature-type",
                aliases: &[],
                value: Some("<type>"),
                type_hint: Some("String"),
                required: false,
                default: None,
                description: "Only extract from this feature type.",
                why_default: None,
            },
        ],
    }],
    examples: &[Example {
        description: "Extract gene IDs and names from gene features",
        command: "rsomics-gff-extract genes.gtf -k gene_id,gene_name --feature-type gene",
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
