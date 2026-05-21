use std::path::PathBuf;

use clap::Parser;
use rsomics_common::{CommonFlags, Result, RsomicsError, Tool, ToolMeta};
use rsomics_help::{Example, HelpSpec, Origin, Section};

use rsomics_fastq_pair::pair_fastq;

pub const META: ToolMeta = ToolMeta {
    name: env!("CARGO_PKG_NAME"),
    version: env!("CARGO_PKG_VERSION"),
};

#[derive(Parser, Debug)]
#[command(
    name = "rsomics-fastq-pair",
    version,
    about,
    long_about = None,
    disable_help_flag = true
)]
pub struct Cli {
    /// Input R1 FASTQ file.
    pub r1: PathBuf,

    /// Input R2 FASTQ file.
    pub r2: PathBuf,

    /// Output R1 (paired).
    #[arg(long = "out1")]
    out1: PathBuf,

    /// Output R2 (paired).
    #[arg(long = "out2")]
    out2: PathBuf,

    #[command(flatten)]
    pub common: CommonFlags,
}

impl Cli {
    pub fn execute(self) -> Result<()> {
        let mut o1 = std::fs::File::create(&self.out1).map_err(RsomicsError::Io)?;
        let mut o2 = std::fs::File::create(&self.out2).map_err(RsomicsError::Io)?;

        let stats = pair_fastq(&self.r1, &self.r2, &mut o1, &mut o2, None)?;

        if !self.common.quiet {
            eprintln!("{} paired, {} singletons", stats.paired, stats.singletons);
        }

        Ok(())
    }
}

impl Tool for Cli {
    fn meta() -> ToolMeta {
        META
    }

    fn common(&self) -> &CommonFlags {
        &self.common
    }

    fn execute(self) -> Result<()> {
        self.execute()
    }
}

pub static HELP: HelpSpec = HelpSpec {
    name: META.name,
    version: META.version,
    tagline: "Re-pair shuffled paired-end FASTQ reads by name.",
    origin: Some(Origin {
        upstream: "fastq_pair / repair.sh (BBTools)",
        upstream_license: "MIT",
        our_license: "MIT OR Apache-2.0",
        paper_doi: None,
    }),
    usage_lines: &["<R1.fq> <R2.fq> --out1 paired_R1.fq --out2 paired_R2.fq"],
    sections: &[Section {
        title: "OPTIONS",
        flags: &[],
    }],
    examples: &[Example {
        description: "Re-pair reads",
        command: "rsomics-fastq-pair R1.fq R2.fq --out1 paired_R1.fq --out2 paired_R2.fq",
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
