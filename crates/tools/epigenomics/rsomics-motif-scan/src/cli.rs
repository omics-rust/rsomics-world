use clap::Parser;
use rsomics_common::{CommonFlags, Result, RsomicsError, Tool, ToolMeta};
use rsomics_help::{Example, FlagSpec, HelpSpec, Section};
use rsomics_motif_scan::scan_motif;
use std::path::PathBuf;

pub const META: ToolMeta = ToolMeta {
    name: env!("CARGO_PKG_NAME"),
    version: env!("CARGO_PKG_VERSION"),
};

#[derive(Parser, Debug)]
#[command(name = "rsomics-motif-scan", version, about, long_about = None, disable_help_flag = true)]
pub struct Cli {
    pub fasta: PathBuf,
    #[arg(short = 'm', long)]
    motif: String,
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
        let n = scan_motif(&self.fasta, &self.motif, &mut out)?;
        if !self.common.quiet {
            eprintln!("{n} matches");
        }
        Ok(())
    }
}

pub static HELP: HelpSpec = HelpSpec {
    name: env!("CARGO_PKG_NAME"),
    version: env!("CARGO_PKG_VERSION"),
    tagline: "Scan FASTA for IUPAC DNA motif occurrences — BED output.",
    origin: None,
    usage_lines: &["<ref.fa> -m <IUPAC_motif> [-o matches.bed]"],
    sections: &[Section {
        title: "OPTIONS",
        flags: &[FlagSpec {
            short: Some('m'),
            long: "motif",
            aliases: &[],
            value: Some("<IUPAC>"),
            type_hint: Some("String"),
            required: true,
            default: None,
            description: "IUPAC motif (e.g., CANNTG for E-box).",
            why_default: None,
        }],
    }],
    examples: &[
        Example {
            description: "Find E-box motifs",
            command: "rsomics-motif-scan genome.fa -m CANNTG -o ebox.bed",
        },
        Example {
            description: "Find CpG sites",
            command: "rsomics-motif-scan genome.fa -m CG",
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
