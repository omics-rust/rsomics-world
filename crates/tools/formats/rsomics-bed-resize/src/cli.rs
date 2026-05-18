use clap::Parser;
use rsomics_bed_resize::bed_resize;
use rsomics_common::{CommonFlags, Result, RsomicsError, ToolMeta};
use rsomics_help::{Example, HelpSpec, Origin, Section};
use std::path::PathBuf;
pub const META: ToolMeta = ToolMeta {
    name: env!("CARGO_PKG_NAME"),
    version: env!("CARGO_PKG_VERSION"),
};
#[derive(Parser, Debug)]
#[command(name = "rsomics-bed-resize", version, about, long_about = None, disable_help_flag = true)]
pub struct Cli {
    pub input: PathBuf,
    #[arg(short = 'o', long = "output", default_value = "-")]
    output: String,
    #[arg(short = 's', long = "size", default_value_t = 500)]
    size: u64,
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
        let _ = bed_resize(&self.input, &mut out, self.size)?;
        Ok(())
    }
}
pub static HELP: HelpSpec = HelpSpec {
    name: META.name,
    version: META.version,
    tagline: "rsomics-bed-resize",
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
        command: "rsomics-bed-resize input",
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
