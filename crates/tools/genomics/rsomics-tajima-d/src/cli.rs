use clap::Parser;
use rsomics_common::{CommonFlags, Result, Tool, ToolMeta};
use rsomics_help::{Example, FlagSpec, HelpSpec, Origin, Section};
use rsomics_tajima_d::load_sfs;
use std::path::PathBuf;

pub const META: ToolMeta = ToolMeta {
    name: env!("CARGO_PKG_NAME"),
    version: env!("CARGO_PKG_VERSION"),
};

#[derive(Parser, Debug)]
#[command(name = "rsomics-tajima-d", version, about, long_about = None, disable_help_flag = true)]
pub struct Cli {
    pub input: PathBuf,
    #[arg(short = 'n', long)]
    samples: u64,
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
        let (counts, _) = load_sfs(&self.input)?;
        let d = rsomics_popgen_core::diversity::tajimas_d(&counts, self.samples)
            .map_err(|e| rsomics_common::RsomicsError::InvalidInput(format!("{e}")))?;
        println!("{d:.6}");
        Ok(())
    }
}

pub static HELP: HelpSpec = HelpSpec {
    name: env!("CARGO_PKG_NAME"),
    version: env!("CARGO_PKG_VERSION"),
    tagline: "Compute Tajima's D from derived allele counts.",
    origin: Some(Origin {
        upstream: "vcftools / PopGenome",
        upstream_license: "GPL-3 / GPL-2",
        our_license: "MIT OR Apache-2.0",
        paper_doi: Some("10.1093/genetics/123.3.585"),
    }),
    usage_lines: &["<sfs.tsv> -n <num_samples>"],
    sections: &[Section {
        title: "OPTIONS",
        flags: &[FlagSpec {
            short: Some('n'),
            long: "samples",
            aliases: &[],
            value: Some("<int>"),
            type_hint: Some("u64"),
            required: true,
            default: None,
            description: "Number of haploid samples.",
            why_default: None,
        }],
    }],
    examples: &[Example {
        description: "Tajima's D from SFS",
        command: "rsomics-tajima-d sfs.tsv -n 100",
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
