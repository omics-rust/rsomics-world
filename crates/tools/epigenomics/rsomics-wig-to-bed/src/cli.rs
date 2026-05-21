use clap::Parser;
use rsomics_common::{CommonFlags, Result, RsomicsError, Tool, ToolMeta};
use rsomics_help::{Example, FlagSpec, HelpSpec, Section};
use rsomics_wig_to_bed::wig_to_bed;
use std::path::PathBuf;

pub const META: ToolMeta = ToolMeta {
    name: env!("CARGO_PKG_NAME"),
    version: env!("CARGO_PKG_VERSION"),
};

#[derive(Parser, Debug)]
#[command(name = "rsomics-wig-to-bed", version, about, long_about = None, disable_help_flag = true)]
pub struct Cli {
    pub input: PathBuf,
    #[arg(short = 't', long, default_value_t = 0.0)]
    threshold: f64,
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
        let n = wig_to_bed(&self.input, self.threshold, &mut out)?;
        if !self.common.quiet {
            eprintln!("{n} intervals above threshold");
        }
        Ok(())
    }
}

pub static HELP: HelpSpec = HelpSpec {
    name: env!("CARGO_PKG_NAME"),
    version: env!("CARGO_PKG_VERSION"),
    tagline: "Convert WIG/bedGraph signal to BED intervals above a threshold.",
    origin: None,
    usage_lines: &["<signal.wig> [-t 1.0] [-o peaks.bed]"],
    sections: &[Section {
        title: "OPTIONS",
        flags: &[FlagSpec {
            short: Some('t'),
            long: "threshold",
            aliases: &[],
            value: Some("<float>"),
            type_hint: Some("f64"),
            required: false,
            default: Some("0.0"),
            description: "Minimum signal value to emit.",
            why_default: None,
        }],
    }],
    examples: &[Example {
        description: "Extract regions with signal > 5",
        command: "rsomics-wig-to-bed signal.wig -t 5.0 -o peaks.bed",
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
