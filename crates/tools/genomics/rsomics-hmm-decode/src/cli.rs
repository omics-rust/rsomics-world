use clap::Parser;
use rsomics_common::{CommonFlags, Result, RsomicsError, Tool, ToolMeta};
use rsomics_help::{Example, FlagSpec, HelpSpec, Section};
use rsomics_hmm_decode::decode_observations;
use std::path::PathBuf;

pub const META: ToolMeta = ToolMeta {
    name: env!("CARGO_PKG_NAME"),
    version: env!("CARGO_PKG_VERSION"),
};

#[derive(Parser, Debug)]
#[command(name = "rsomics-hmm-decode", version, about, long_about = None, disable_help_flag = true)]
pub struct Cli {
    #[arg(short = 'm', long)]
    model: PathBuf,
    pub observations: PathBuf,
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
        let n = decode_observations(&self.model, &self.observations, &mut out)?;
        if !self.common.quiet {
            eprintln!("{n} sequences decoded");
        }
        Ok(())
    }
}

pub static HELP: HelpSpec = HelpSpec {
    name: env!("CARGO_PKG_NAME"),
    version: env!("CARGO_PKG_VERSION"),
    tagline: "Viterbi-decode observation sequences with a discrete HMM.",
    origin: None,
    usage_lines: &["-m <model.json> <observations.txt> [-o states.txt]"],
    sections: &[Section {
        title: "OPTIONS",
        flags: &[FlagSpec {
            short: Some('m'),
            long: "model",
            aliases: &[],
            value: Some("<path>"),
            type_hint: Some("PathBuf"),
            required: true,
            default: None,
            description: "HMM model JSON ({pi, trans, emit, n_symbols}).",
            why_default: None,
        }],
    }],
    examples: &[Example {
        description: "Decode chromatin states",
        command: "rsomics-hmm-decode -m cpg_model.json observations.txt -o states.txt",
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
