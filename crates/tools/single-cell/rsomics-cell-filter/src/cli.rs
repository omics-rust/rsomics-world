use clap::Parser;
use rsomics_cell_filter::{FilterCriteria, filter_cells};
use rsomics_common::{CommonFlags, Result, RsomicsError, Tool, ToolMeta};
use rsomics_help::{Example, FlagSpec, HelpSpec, Section};
use std::path::PathBuf;

pub const META: ToolMeta = ToolMeta {
    name: env!("CARGO_PKG_NAME"),
    version: env!("CARGO_PKG_VERSION"),
};

#[derive(Parser, Debug)]
#[command(name = "rsomics-cell-filter", version, about, long_about = None, disable_help_flag = true)]
pub struct Cli {
    pub input: PathBuf,
    #[arg(long, default_value_t = 200)]
    min_genes: u64,
    #[arg(long, default_value_t = 500)]
    min_umis: u64,
    #[arg(long, default_value_t = 0.2)]
    max_mito: f64,
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
        let criteria = FilterCriteria {
            min_genes: self.min_genes,
            min_umis: self.min_umis,
            max_mito_frac: self.max_mito,
        };
        let mut out: Box<dyn std::io::Write> = if self.output == "-" {
            Box::new(std::io::stdout().lock())
        } else {
            Box::new(std::fs::File::create(&self.output).map_err(RsomicsError::Io)?)
        };
        let r = filter_cells(&self.input, &criteria, &mut out)?;
        if !self.common.quiet {
            eprintln!(
                "{}/{} cells passed ({} removed)",
                r.passed, r.total, r.failed
            );
        }
        Ok(())
    }
}

pub static HELP: HelpSpec = HelpSpec {
    name: env!("CARGO_PKG_NAME"),
    version: env!("CARGO_PKG_VERSION"),
    tagline: "Filter cells by QC metrics — genes, UMIs, mito fraction.",
    origin: None,
    usage_lines: &["<cell_stats.tsv> [--min-genes 200] [--min-umis 500] [--max-mito 0.2]"],
    sections: &[Section {
        title: "OPTIONS",
        flags: &[
            FlagSpec {
                short: None,
                long: "min-genes",
                aliases: &[],
                value: Some("<int>"),
                type_hint: Some("u64"),
                required: false,
                default: Some("200"),
                description: "Minimum detected genes per cell.",
                why_default: None,
            },
            FlagSpec {
                short: None,
                long: "min-umis",
                aliases: &[],
                value: Some("<int>"),
                type_hint: Some("u64"),
                required: false,
                default: Some("500"),
                description: "Minimum UMI count per cell.",
                why_default: None,
            },
            FlagSpec {
                short: None,
                long: "max-mito",
                aliases: &[],
                value: Some("<float>"),
                type_hint: Some("f64"),
                required: false,
                default: Some("0.2"),
                description: "Maximum mitochondrial fraction.",
                why_default: None,
            },
        ],
    }],
    examples: &[Example {
        description: "Standard Seurat-like filter",
        command: "rsomics-cell-filter stats.tsv --min-genes 200 --min-umis 500 --max-mito 0.2 -o filtered.tsv",
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
