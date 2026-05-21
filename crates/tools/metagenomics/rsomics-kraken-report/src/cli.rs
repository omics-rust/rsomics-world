use clap::Parser;
use rsomics_common::{CommonFlags, Result, RsomicsError, Tool, ToolMeta};
use rsomics_help::{Example, FlagSpec, HelpSpec, Section};
use rsomics_kraken_report::{parse_report, top_taxa};
use std::path::PathBuf;

pub const META: ToolMeta = ToolMeta {
    name: env!("CARGO_PKG_NAME"),
    version: env!("CARGO_PKG_VERSION"),
};

#[derive(Parser, Debug)]
#[command(name = "rsomics-kraken-report", version, about, long_about = None, disable_help_flag = true)]
pub struct Cli {
    pub input: PathBuf,
    #[arg(short = 'r', long, default_value = "S")]
    rank: String,
    #[arg(short = 'n', long, default_value_t = 20)]
    top: usize,
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
        let entries = parse_report(&self.input)?;
        let mut out: Box<dyn std::io::Write> = if self.output == "-" {
            Box::new(std::io::stdout().lock())
        } else {
            Box::new(std::fs::File::create(&self.output).map_err(RsomicsError::Io)?)
        };
        top_taxa(&entries, &self.rank, self.top, &mut out)?;
        if !self.common.quiet {
            eprintln!("{} entries parsed", entries.len());
        }
        Ok(())
    }
}
pub static HELP: HelpSpec = HelpSpec {
    name: env!("CARGO_PKG_NAME"),
    version: env!("CARGO_PKG_VERSION"),
    tagline: "Parse Kraken2 report — top taxa by rank.",
    origin: None,
    usage_lines: &["<kraken.report> [-r S] [-n 20]"],
    sections: &[Section {
        title: "OPTIONS",
        flags: &[
            FlagSpec {
                short: Some('r'),
                long: "rank",
                aliases: &[],
                value: Some("<code>"),
                type_hint: Some("String"),
                required: false,
                default: Some("S"),
                description: "Taxonomic rank code (S=species, G=genus, etc.).",
                why_default: None,
            },
            FlagSpec {
                short: Some('n'),
                long: "top",
                aliases: &[],
                value: Some("<int>"),
                type_hint: Some("usize"),
                required: false,
                default: Some("20"),
                description: "Number of top taxa to show.",
                why_default: None,
            },
        ],
    }],
    examples: &[Example {
        description: "Top 10 species",
        command: "rsomics-kraken-report report.txt -r S -n 10",
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
