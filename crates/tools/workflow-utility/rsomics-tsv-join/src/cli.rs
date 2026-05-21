use clap::Parser;
use rsomics_common::{CommonFlags, Result, RsomicsError, Tool, ToolMeta};
use rsomics_help::{Example, FlagSpec, HelpSpec, Section};
use rsomics_tsv_join::inner_join;
use std::path::PathBuf;

pub const META: ToolMeta = ToolMeta {
    name: env!("CARGO_PKG_NAME"),
    version: env!("CARGO_PKG_VERSION"),
};

#[derive(Parser, Debug)]
#[command(name = "rsomics-tsv-join", version, about, long_about = None, disable_help_flag = true)]
pub struct Cli {
    pub left: PathBuf,
    pub right: PathBuf,
    #[arg(short = 'k', long)]
    key: String,
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
        let n = inner_join(&self.left, &self.right, &self.key, &mut out)?;
        if !self.common.quiet {
            eprintln!("{n} joined rows");
        }
        Ok(())
    }
}

pub static HELP: HelpSpec = HelpSpec {
    name: env!("CARGO_PKG_NAME"),
    version: env!("CARGO_PKG_VERSION"),
    tagline: "Inner-join two TSV files by key column.",
    origin: None,
    usage_lines: &["<left.tsv> <right.tsv> -k <column_name>"],
    sections: &[Section {
        title: "OPTIONS",
        flags: &[FlagSpec {
            short: Some('k'),
            long: "key",
            aliases: &[],
            value: Some("<col>"),
            type_hint: Some("String"),
            required: true,
            default: None,
            description: "Key column name (must exist in both files).",
            why_default: None,
        }],
    }],
    examples: &[Example {
        description: "Join expression with annotation",
        command: "rsomics-tsv-join expr.tsv annot.tsv -k gene -o merged.tsv",
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
