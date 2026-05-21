use clap::Parser;
use rsomics_common::{CommonFlags, Result, RsomicsError, Tool, ToolMeta};
use rsomics_help::{Example, FlagSpec, HelpSpec, Section};
use rsomics_tsv_select::select_columns;
use std::path::PathBuf;

pub const META: ToolMeta = ToolMeta {
    name: env!("CARGO_PKG_NAME"),
    version: env!("CARGO_PKG_VERSION"),
};

#[derive(Parser, Debug)]
#[command(name = "rsomics-tsv-select", version, about, long_about = None, disable_help_flag = true)]
pub struct Cli {
    pub input: PathBuf,
    #[arg(short = 'c', long, num_args = 1.., required = true)]
    columns: Vec<String>,
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
        let n = select_columns(&self.input, &self.columns, &mut out)?;
        if !self.common.quiet {
            eprintln!("{n} rows");
        }
        Ok(())
    }
}

pub static HELP: HelpSpec = HelpSpec {
    name: env!("CARGO_PKG_NAME"),
    version: env!("CARGO_PKG_VERSION"),
    tagline: "Select and reorder TSV columns by name or index.",
    origin: None,
    usage_lines: &["<data.tsv> -c <col1> <col2> ... [-o output.tsv]"],
    sections: &[Section {
        title: "OPTIONS",
        flags: &[FlagSpec {
            short: Some('c'),
            long: "columns",
            aliases: &[],
            value: Some("<names or indices>"),
            type_hint: Some("Vec<String>"),
            required: true,
            default: None,
            description: "Column names or 1-based indices to select.",
            why_default: None,
        }],
    }],
    examples: &[
        Example {
            description: "Select by name",
            command: "rsomics-tsv-select data.tsv -c gene pvalue log2fc",
        },
        Example {
            description: "Select by index",
            command: "rsomics-tsv-select data.tsv -c 1 3 5",
        },
    ],
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
