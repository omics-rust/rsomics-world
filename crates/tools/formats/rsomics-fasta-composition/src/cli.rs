#![allow(clippy::cast_precision_loss)]
use clap::Parser;
use rsomics_common::{CommonFlags, Result, ToolMeta};
use rsomics_fasta_composition::fasta_composition;
use rsomics_help::{Example, HelpSpec, Origin, Section};
use std::path::PathBuf;
pub const META: ToolMeta = ToolMeta {
    name: env!("CARGO_PKG_NAME"),
    version: env!("CARGO_PKG_VERSION"),
};
#[derive(Parser, Debug)]
#[command(name = "rsomics-fasta-composition", version, about, long_about = None, disable_help_flag = true)]
pub struct Cli {
    pub input: PathBuf,
    #[command(flatten)]
    pub common: CommonFlags,
}
impl Cli {
    pub fn execute(self) -> Result<()> {
        let comp = fasta_composition(&self.input)?;
        println!("A\t{}", comp.a);
        println!("C\t{}", comp.c);
        println!("G\t{}", comp.g);
        println!("T\t{}", comp.t);
        println!("N\t{}", comp.n);
        println!("other\t{}", comp.other);
        println!("total\t{}", comp.total);
        if comp.total > 0 {
            println!(
                "GC%\t{:.2}",
                (comp.g + comp.c) as f64 / comp.total as f64 * 100.0
            );
        }
        Ok(())
    }
}
pub static HELP: HelpSpec = HelpSpec {
    name: META.name,
    version: META.version,
    tagline: "Base composition from FASTA.",
    origin: Some(Origin {
        upstream: "seqkit stats",
        upstream_license: "MIT",
        our_license: "MIT OR Apache-2.0",
        paper_doi: None,
    }),
    usage_lines: &["<input.fa>"],
    sections: &[Section {
        title: "OPTIONS",
        flags: &[],
    }],
    examples: &[Example {
        description: "Base counts",
        command: "rsomics-fasta-composition genome.fa",
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
