use clap::Parser;
use rsomics_common::{CommonFlags, Result, RsomicsError, ToolMeta};
use rsomics_fastq_deinterleave::deinterleave;
use rsomics_help::{Example, HelpSpec, Origin, Section};
use std::path::PathBuf;
pub const META: ToolMeta = ToolMeta {
    name: env!("CARGO_PKG_NAME"),
    version: env!("CARGO_PKG_VERSION"),
};
#[derive(Parser, Debug)]
#[command(name = "rsomics-fastq-deinterleave", version, about, long_about = None, disable_help_flag = true)]
pub struct Cli {
    pub input: PathBuf,
    #[arg(long = "out1")]
    out1: PathBuf,
    #[arg(long = "out2")]
    out2: PathBuf,
    #[command(flatten)]
    pub common: CommonFlags,
}
impl Cli {
    pub fn execute(self) -> Result<()> {
        let mut o1 = std::fs::File::create(&self.out1).map_err(RsomicsError::Io)?;
        let mut o2 = std::fs::File::create(&self.out2).map_err(RsomicsError::Io)?;
        let pairs = deinterleave(&self.input, &mut o1, &mut o2)?;
        if !self.common.quiet {
            eprintln!("{pairs} pairs");
        }
        Ok(())
    }
}
pub static HELP: HelpSpec = HelpSpec {
    name: META.name,
    version: META.version,
    tagline: "Split interleaved FASTQ into R1/R2.",
    origin: Some(Origin {
        upstream: "BBTools reformat.sh / seqkit split2",
        upstream_license: "MIT",
        our_license: "MIT OR Apache-2.0",
        paper_doi: None,
    }),
    usage_lines: &["<interleaved.fq> --out1 R1.fq --out2 R2.fq"],
    sections: &[Section {
        title: "OPTIONS",
        flags: &[],
    }],
    examples: &[Example {
        description: "Deinterleave",
        command: "rsomics-fastq-deinterleave interleaved.fq --out1 R1.fq --out2 R2.fq",
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
