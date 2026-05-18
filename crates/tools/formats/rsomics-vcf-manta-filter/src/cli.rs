use clap::Parser;
use rsomics_common::{CommonFlags, Result, RsomicsError, ToolMeta};
use rsomics_help::{Example, HelpSpec, Origin, Section};
use rsomics_vcf_manta_filter::vcf_manta_filter;
use std::path::PathBuf;
pub const META: ToolMeta = ToolMeta {
    name: env!("CARGO_PKG_NAME"),
    version: env!("CARGO_PKG_VERSION"),
};
#[derive(Parser, Debug)]
#[command(name = "rsomics-vcf-manta-filter", version, about, long_about = None, disable_help_flag = true)]
pub struct Cli {
    pub input: PathBuf,
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
        let _ = vcf_manta_filter(&self.input, &mut out)?;
        Ok(())
    }
}
pub static HELP: HelpSpec = HelpSpec {
    name: META.name,
    version: META.version,
    tagline: "Manta SV caller filter from VCF",
    origin: Some(Origin {
        upstream: "custom",
        upstream_license: "N/A",
        our_license: "MIT OR Apache-2.0",
        paper_doi: None,
    }),
    usage_lines: &["<input> [-o output]"],
    sections: &[Section {
        title: "OPTIONS",
        flags: &[],
    }],
    examples: &[Example {
        description: "Run",
        command: "rsomics-vcf-manta-filter input",
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
