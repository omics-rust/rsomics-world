use std::path::PathBuf;

use clap::Parser;
use rsomics_common::{CommonFlags, Result, RsomicsError, ToolMeta};
use rsomics_help::{Example, FlagSpec, HelpSpec, Origin, Section};

use rsomics_bed_annotate::annotate_bed;

pub const META: ToolMeta = ToolMeta {
    name: env!("CARGO_PKG_NAME"),
    version: env!("CARGO_PKG_VERSION"),
};

#[derive(Parser, Debug)]
#[command(
    name = "rsomics-bed-annotate",
    version,
    about,
    long_about = None,
    disable_help_flag = true
)]
pub struct Cli {
    /// Input BED file.
    pub input: PathBuf,

    /// Gene annotation GFF/GTF file.
    #[arg(short = 'g', long = "gff")]
    gff: PathBuf,

    /// Output file (default stdout).
    #[arg(short = 'o', long = "output", default_value = "-")]
    output: String,

    /// GFF feature type to match.
    #[arg(long = "feature-type", default_value = "gene")]
    feature_type: String,

    /// GFF attribute to use as label.
    #[arg(long = "attribute", default_value = "gene_name")]
    attribute: String,

    #[command(flatten)]
    pub common: CommonFlags,
}

impl Cli {
    pub fn execute(self) -> Result<()> {
        let mut out: Box<dyn std::io::Write> = if self.output == "-" {
            Box::new(std::io::stdout().lock())
        } else {
            Box::new(std::fs::File::create(&self.output).map_err(RsomicsError::Io)?)
        };

        let count = annotate_bed(
            &self.input,
            &self.gff,
            &mut out,
            &self.feature_type,
            &self.attribute,
        )?;

        if !self.common.quiet {
            eprintln!("{count} intervals annotated");
        }

        Ok(())
    }
}

pub static HELP: HelpSpec = HelpSpec {
    name: META.name,
    version: META.version,
    tagline: "Annotate BED intervals with nearest GFF features.",
    origin: Some(Origin {
        upstream: "bedtools closest + awk",
        upstream_license: "MIT",
        our_license: "MIT OR Apache-2.0",
        paper_doi: None,
    }),
    usage_lines: &["<input.bed> -g genes.gtf [-o annotated.bed]"],
    sections: &[Section {
        title: "OPTIONS",
        flags: &[
            FlagSpec {
                short: Some('g'),
                long: "gff",
                aliases: &[],
                value: Some("<path>"),
                type_hint: Some("Path"),
                required: true,
                default: None,
                description: "Gene annotation GFF/GTF file.",
                why_default: None,
            },
            FlagSpec {
                short: None,
                long: "feature-type",
                aliases: &[],
                value: Some("<type>"),
                type_hint: Some("String"),
                required: false,
                default: Some("gene"),
                description: "GFF feature type to match.",
                why_default: None,
            },
            FlagSpec {
                short: None,
                long: "attribute",
                aliases: &[],
                value: Some("<key>"),
                type_hint: Some("String"),
                required: false,
                default: Some("gene_name"),
                description: "GFF attribute to use as label.",
                why_default: None,
            },
        ],
    }],
    examples: &[Example {
        description: "Annotate peaks with nearest gene",
        command: "rsomics-bed-annotate peaks.bed -g genes.gtf -o annotated.bed",
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
