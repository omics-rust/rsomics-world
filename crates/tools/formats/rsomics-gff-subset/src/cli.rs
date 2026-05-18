use clap::Parser;
use rsomics_common::{CommonFlags, Result, RsomicsError, ToolMeta};
use rsomics_gff_subset::gff_subset;
use rsomics_help::{Example, FlagSpec, HelpSpec, Origin, Section};
use std::path::PathBuf;
pub const META: ToolMeta = ToolMeta {
    name: env!("CARGO_PKG_NAME"),
    version: env!("CARGO_PKG_VERSION"),
};
#[derive(Parser, Debug)]
#[command(name = "rsomics-gff-subset", version, about, long_about = None, disable_help_flag = true)]
pub struct Cli {
    pub input: PathBuf,
    #[arg(long = "type")]
    feature_type: String,
    #[arg(short = 'o', long = "output", default_value = "-")]
    output: String,
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
        let count = gff_subset(&self.input, &mut out, &self.feature_type)?;
        if !self.common.quiet {
            eprintln!("{count} features");
        }
        Ok(())
    }
}
pub static HELP: HelpSpec = HelpSpec {
    name: META.name,
    version: META.version,
    tagline: "Extract GFF features by type.",
    origin: Some(Origin {
        upstream: "grep/awk on GFF column 3",
        upstream_license: "N/A",
        our_license: "MIT OR Apache-2.0",
        paper_doi: None,
    }),
    usage_lines: &["<input.gff> --type gene [-o genes.gff]"],
    sections: &[Section {
        title: "OPTIONS",
        flags: &[FlagSpec {
            short: None,
            long: "type",
            aliases: &[],
            value: Some("<type>"),
            type_hint: Some("String"),
            required: true,
            default: None,
            description: "Feature type to extract.",
            why_default: None,
        }],
    }],
    examples: &[Example {
        description: "Extract genes",
        command: "rsomics-gff-subset genes.gff --type gene",
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
