use std::path::PathBuf;

use clap::Parser;
use rsomics_common::{CommonFlags, Result, RsomicsError, ToolMeta};
use rsomics_help::{Example, FlagSpec, HelpSpec, Origin, Section};

use rsomics_featurecounts::{CountOpts, count_reads, load_features, write_counts};

pub const META: ToolMeta = ToolMeta {
    name: env!("CARGO_PKG_NAME"),
    version: env!("CARGO_PKG_VERSION"),
};

#[derive(Parser, Debug)]
#[command(
    name = "rsomics-featurecounts",
    version,
    about,
    long_about = None,
    disable_help_flag = true
)]
pub struct Cli {
    /// Input BAM file.
    #[arg(short = 'a', long = "annotation")]
    annotation: PathBuf,

    /// Input BAM file(s).
    pub inputs: Vec<PathBuf>,

    /// Output file (default stdout).
    #[arg(short = 'o', long = "output", default_value = "-")]
    output: String,

    /// Feature type to count (GFF column 3).
    #[arg(long = "feature-type", default_value = "exon")]
    feature_type: String,

    /// Attribute to group by (GFF column 9 key).
    #[arg(long = "attribute", default_value = "gene_id")]
    attribute: String,

    /// Minimum mapping quality.
    #[arg(long = "min-mapq", default_value_t = 0)]
    min_mapq: u8,

    #[command(flatten)]
    pub common: CommonFlags,
}

impl Cli {
    pub fn execute(self) -> Result<()> {
        let opts = CountOpts {
            feature_type: self.feature_type,
            attribute: self.attribute,
            min_mapq: self.min_mapq,
            ..Default::default()
        };

        let features = load_features(&self.annotation, &opts)?;

        let mut out: Box<dyn std::io::Write> = if self.output == "-" {
            Box::new(std::io::stdout().lock())
        } else {
            Box::new(std::fs::File::create(&self.output).map_err(RsomicsError::Io)?)
        };

        for bam_path in &self.inputs {
            let (counts, summary) = count_reads(bam_path, &features, &opts)?;

            if self.common.json {
                let j = serde_json::json!({
                    "file": bam_path.display().to_string(),
                    "summary": summary,
                    "counts": counts,
                });
                serde_json::to_writer_pretty(&mut out, &j)
                    .map_err(|e| RsomicsError::InvalidInput(format!("JSON: {e}")))?;
                writeln!(out).map_err(RsomicsError::Io)?;
            } else {
                write_counts(&counts, &features, &mut out)?;
                eprintln!(
                    "{}: {} assigned, {} no_feature, {} ambiguous / {} total",
                    bam_path.display(),
                    summary.assigned,
                    summary.unassigned_no_features,
                    summary.unassigned_ambiguity,
                    summary.total_reads,
                );
            }
        }

        Ok(())
    }
}

use std::io::Write;

pub static HELP: HelpSpec = HelpSpec {
    name: META.name,
    version: META.version,
    tagline: "Count reads over genomic features from BAM + GFF.",
    origin: Some(Origin {
        upstream: "featureCounts (Subread)",
        upstream_license: "GPL-3.0",
        our_license: "MIT OR Apache-2.0",
        paper_doi: Some("10.1093/bioinformatics/btt656"),
    }),
    usage_lines: &["-a annotation.gtf <input.bam> [-o counts.tsv]"],
    sections: &[Section {
        title: "OPTIONS",
        flags: &[
            FlagSpec {
                short: Some('a'),
                long: "annotation",
                aliases: &[],
                value: Some("<path>"),
                type_hint: Some("Path"),
                required: true,
                default: None,
                description: "Gene annotation file (GTF/GFF).",
                why_default: None,
            },
            FlagSpec {
                short: None,
                long: "feature-type",
                aliases: &[],
                value: Some("<type>"),
                type_hint: Some("String"),
                required: false,
                default: Some("exon"),
                description: "GFF feature type to count.",
                why_default: Some("featureCounts default"),
            },
            FlagSpec {
                short: None,
                long: "attribute",
                aliases: &[],
                value: Some("<key>"),
                type_hint: Some("String"),
                required: false,
                default: Some("gene_id"),
                description: "GFF attribute to group features by.",
                why_default: Some("featureCounts default"),
            },
        ],
    }],
    examples: &[Example {
        description: "Count reads per gene",
        command: "rsomics-featurecounts -a genes.gtf input.bam -o counts.tsv",
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
