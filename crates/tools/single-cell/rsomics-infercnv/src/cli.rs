use std::path::PathBuf;

use clap::Parser;
use rsomics_common::{CommonFlags, Result, RsomicsError, Tool, ToolMeta};
use rsomics_help::{Example, FlagSpec, HelpSpec, Origin, Section};

use rsomics_infercnv::{infer_cnv, load_gene_order, load_matrix, write_cnv};

pub const META: ToolMeta = ToolMeta {
    name: env!("CARGO_PKG_NAME"),
    version: env!("CARGO_PKG_VERSION"),
};

#[derive(Parser, Debug)]
#[command(
    name = "rsomics-infercnv",
    version,
    about,
    long_about = None,
    disable_help_flag = true
)]
pub struct Cli {
    /// Expression matrix (TSV: genes × cells, first col = gene name, first row = barcodes).
    #[arg(long = "matrix")]
    matrix: PathBuf,

    /// Gene annotation file (GTF/GFF) for genomic ordering.
    #[arg(long = "gtf")]
    gtf: PathBuf,

    /// File listing reference (normal) cell barcodes, one per line.
    #[arg(long = "ref-cells")]
    ref_cells: Option<PathBuf>,

    /// Sliding window size for smoothing (number of genes).
    #[arg(long = "window", default_value_t = 101)]
    window: usize,

    /// Output file (default stdout).
    #[arg(short = 'o', long = "output", default_value = "-")]
    output: String,

    #[command(flatten)]
    pub common: CommonFlags,
}

impl Cli {
    pub fn execute(self) -> Result<()> {
        let gene_order = load_gene_order(&self.gtf)?;
        let (gene_names, cell_names, expr) = load_matrix(&self.matrix)?;

        let ref_indices = if let Some(ref_path) = &self.ref_cells {
            let ref_barcodes: Vec<String> = std::fs::read_to_string(ref_path)
                .map_err(|e| RsomicsError::InvalidInput(format!("{}: {e}", ref_path.display())))?
                .lines()
                .map(|l| l.trim().to_string())
                .filter(|l| !l.is_empty())
                .collect();
            cell_names
                .iter()
                .enumerate()
                .filter(|(_, name)| ref_barcodes.contains(name))
                .map(|(i, _)| i)
                .collect()
        } else {
            Vec::new()
        };

        let (ordered_genes, cnv) = infer_cnv(
            &gene_names,
            &cell_names,
            &expr,
            &gene_order,
            &ref_indices,
            self.window,
        )?;

        let mut out: Box<dyn std::io::Write> = if self.output == "-" {
            Box::new(std::io::stdout().lock())
        } else {
            Box::new(std::fs::File::create(&self.output).map_err(RsomicsError::Io)?)
        };
        write_cnv(&ordered_genes, &cell_names, &cnv, &mut out)?;

        if !self.common.json {
            eprintln!(
                "{} genes × {} cells → CNV matrix",
                ordered_genes.len(),
                cell_names.len()
            );
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
    tagline: "Infer copy-number variations from scRNA-seq expression.",
    origin: Some(Origin {
        upstream: "inferCNV (Broad/Trinity CTAT)",
        upstream_license: "BSD-3-Clause",
        our_license: "MIT OR Apache-2.0",
        paper_doi: None,
    }),
    usage_lines: &["--matrix EXPR.tsv --gtf genes.gtf [--ref-cells normal.txt] [-o cnv.tsv]"],
    sections: &[Section {
        title: "OPTIONS",
        flags: &[
            FlagSpec {
                short: None,
                long: "matrix",
                aliases: &[],
                value: Some("<path>"),
                type_hint: Some("Path"),
                required: true,
                default: None,
                description: "Expression matrix (genes × cells TSV).",
                why_default: None,
            },
            FlagSpec {
                short: None,
                long: "gtf",
                aliases: &[],
                value: Some("<path>"),
                type_hint: Some("Path"),
                required: true,
                default: None,
                description: "Gene annotation GTF/GFF.",
                why_default: None,
            },
            FlagSpec {
                short: None,
                long: "ref-cells",
                aliases: &[],
                value: Some("<path>"),
                type_hint: Some("Path"),
                required: false,
                default: None,
                description: "Reference (normal) cell barcodes file.",
                why_default: None,
            },
            FlagSpec {
                short: None,
                long: "window",
                aliases: &[],
                value: Some("<N>"),
                type_hint: Some("usize"),
                required: false,
                default: Some("101"),
                description: "Smoothing window size (genes).",
                why_default: Some("inferCNV default"),
            },
        ],
    }],
    examples: &[Example {
        description: "Infer CNV with reference normals",
        command: "rsomics-infercnv --matrix counts.tsv --gtf genes.gtf --ref-cells normals.txt -o cnv.tsv",
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
