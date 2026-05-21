use clap::Parser;
use rsomics_align_core::ScoreParams;
use rsomics_align_score::{AlignMode, align_pairs};
use rsomics_common::{CommonFlags, Result, RsomicsError, Tool, ToolMeta};
use rsomics_help::{Example, FlagSpec, HelpSpec, Section};
use std::path::PathBuf;

pub const META: ToolMeta = ToolMeta {
    name: env!("CARGO_PKG_NAME"),
    version: env!("CARGO_PKG_VERSION"),
};

#[derive(Parser, Debug)]
#[command(name = "rsomics-align-score", version, about, long_about = None, disable_help_flag = true)]
pub struct Cli {
    pub fasta: PathBuf,
    #[arg(long)]
    local: bool,
    #[arg(long, default_value_t = 1)]
    match_score: i32,
    #[arg(long, default_value_t = -1)]
    mismatch: i32,
    #[arg(long, default_value_t = -2)]
    gap_open: i32,
    #[arg(long, default_value_t = -1)]
    gap_extend: i32,
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
        let mode = if self.local {
            AlignMode::Local
        } else {
            AlignMode::Global
        };
        let params = ScoreParams {
            match_score: self.match_score,
            mismatch: self.mismatch,
            gap_open: self.gap_open,
            gap_extend: self.gap_extend,
        };
        let mut out: Box<dyn std::io::Write> = if self.output == "-" {
            Box::new(std::io::stdout().lock())
        } else {
            Box::new(std::fs::File::create(&self.output).map_err(RsomicsError::Io)?)
        };
        let n = align_pairs(&self.fasta, &mode, &params, &mut out)?;
        if !self.common.quiet {
            eprintln!("{n} pairs aligned");
        }
        Ok(())
    }
}

pub static HELP: HelpSpec = HelpSpec {
    name: env!("CARGO_PKG_NAME"),
    version: env!("CARGO_PKG_VERSION"),
    tagline: "Pairwise sequence alignment — NW global or SW local.",
    origin: None,
    usage_lines: &["<seqs.fa> [--local] [--match-score 1] [--mismatch -1]"],
    sections: &[Section {
        title: "OPTIONS",
        flags: &[FlagSpec {
            short: None,
            long: "local",
            aliases: &[],
            value: None,
            type_hint: Some("bool"),
            required: false,
            default: None,
            description: "Use Smith-Waterman local alignment (default: NW global).",
            why_default: None,
        }],
    }],
    examples: &[
        Example {
            description: "Global alignment",
            command: "rsomics-align-score seqs.fa",
        },
        Example {
            description: "Local alignment",
            command: "rsomics-align-score seqs.fa --local",
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
