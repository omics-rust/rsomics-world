use clap::Parser;
use rsomics_common::{CommonFlags, Result, RsomicsError, Tool, ToolMeta};
use rsomics_de_volcano::annotate_de;
use rsomics_help::{Example, FlagSpec, HelpSpec, Section};
use std::path::PathBuf;

pub const META: ToolMeta = ToolMeta {
    name: env!("CARGO_PKG_NAME"),
    version: env!("CARGO_PKG_VERSION"),
};

#[derive(Parser, Debug)]
#[command(name = "rsomics-de-volcano", version, about, long_about = None, disable_help_flag = true)]
pub struct Cli {
    pub input: PathBuf,
    #[arg(long, default_value = "padj")]
    pval_col: String,
    #[arg(long, default_value = "log2FoldChange")]
    lfc_col: String,
    #[arg(long, default_value_t = 0.05)]
    pval_threshold: f64,
    #[arg(long, default_value_t = 1.0)]
    lfc_threshold: f64,
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
        let (up, down, ns) = annotate_de(
            &self.input,
            &self.pval_col,
            &self.lfc_col,
            self.pval_threshold,
            self.lfc_threshold,
            &mut out,
        )?;
        if !self.common.quiet {
            eprintln!("{up} up, {down} down, {ns} NS");
        }
        Ok(())
    }
}

pub static HELP: HelpSpec = HelpSpec {
    name: env!("CARGO_PKG_NAME"),
    version: env!("CARGO_PKG_VERSION"),
    tagline: "Annotate DE results for volcano plots — UP/DOWN/NS categories.",
    origin: None,
    usage_lines: &["<deseq_results.tsv> [--pval-col padj] [--lfc-col log2FoldChange]"],
    sections: &[Section {
        title: "OPTIONS",
        flags: &[
            FlagSpec {
                short: None,
                long: "pval-threshold",
                aliases: &[],
                value: Some("<float>"),
                type_hint: Some("f64"),
                required: false,
                default: Some("0.05"),
                description: "Adjusted p-value cutoff.",
                why_default: None,
            },
            FlagSpec {
                short: None,
                long: "lfc-threshold",
                aliases: &[],
                value: Some("<float>"),
                type_hint: Some("f64"),
                required: false,
                default: Some("1.0"),
                description: "Log2 fold change cutoff.",
                why_default: None,
            },
        ],
    }],
    examples: &[Example {
        description: "Annotate DESeq2 output",
        command: "rsomics-de-volcano results.tsv --pval-col padj --lfc-col log2FoldChange -o volcano.tsv",
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
