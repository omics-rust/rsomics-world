use std::path::PathBuf;

use clap::Parser;
use rsomics_common::{CommonFlags, Result, RsomicsError, ToolMeta};
use rsomics_help::{Example, HelpSpec, Origin, Section};

use rsomics_fasta_n50::compute_n50;

pub const META: ToolMeta = ToolMeta {
    name: env!("CARGO_PKG_NAME"),
    version: env!("CARGO_PKG_VERSION"),
};

#[derive(Parser, Debug)]
#[command(
    name = "rsomics-fasta-n50",
    version,
    about,
    long_about = None,
    disable_help_flag = true
)]
pub struct Cli {
    /// Input FASTA file.
    pub input: PathBuf,

    #[command(flatten)]
    pub common: CommonFlags,
}

impl Cli {
    pub fn execute(self) -> Result<()> {
        let stats = compute_n50(&self.input)?;

        if self.common.json {
            serde_json::to_writer_pretty(std::io::stdout(), &stats)
                .map_err(|e| RsomicsError::InvalidInput(format!("{e}")))?;
            println!();
        } else {
            println!("sequences\t{}", stats.num_seqs);
            println!("total_len\t{}", stats.total_len);
            println!("min_len\t{}", stats.min_len);
            println!("max_len\t{}", stats.max_len);
            println!("mean_len\t{:.1}", stats.mean_len);
            println!("N50\t{}", stats.n50);
            println!("N90\t{}", stats.n90);
            println!("L50\t{}", stats.l50);
            println!("L90\t{}", stats.l90);
            println!("GC%\t{:.2}", stats.gc_pct);
        }

        Ok(())
    }
}

pub static HELP: HelpSpec = HelpSpec {
    name: META.name,
    version: META.version,
    tagline: "Compute N50, L50, and assembly statistics from FASTA.",
    origin: Some(Origin {
        upstream: "assembly-stats / QUAST",
        upstream_license: "MIT",
        our_license: "MIT OR Apache-2.0",
        paper_doi: None,
    }),
    usage_lines: &["<assembly.fasta> [--json]"],
    sections: &[Section {
        title: "OPTIONS",
        flags: &[],
    }],
    examples: &[
        Example {
            description: "Compute assembly N50",
            command: "rsomics-fasta-n50 assembly.fasta",
        },
        Example {
            description: "JSON output",
            command: "rsomics-fasta-n50 assembly.fasta --json",
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
