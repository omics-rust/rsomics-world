use std::path::PathBuf;

use clap::Parser;
use rsomics_common::{CommonFlags, Result, RsomicsError, Tool, ToolMeta};
use rsomics_help::{Example, FlagSpec, HelpSpec, Origin, Section};

use rsomics_vcf_norm::normalize_vcf;

pub const META: ToolMeta = ToolMeta {
    name: env!("CARGO_PKG_NAME"),
    version: env!("CARGO_PKG_VERSION"),
};

#[derive(Parser, Debug)]
#[command(
    name = "rsomics-vcf-norm",
    version,
    about,
    long_about = None,
    disable_help_flag = true
)]
pub struct Cli {
    /// Input VCF file.
    pub input: PathBuf,

    /// Output file (default stdout).
    #[arg(short = 'o', long = "output", default_value = "-")]
    output: String,

    /// Split multiallelic sites into biallelic records.
    #[arg(short = 'm', long = "multiallelics")]
    split_multiallelic: bool,

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

        let stats = normalize_vcf(&self.input, &mut out, self.split_multiallelic)?;

        if !self.common.quiet {
            eprintln!("{} records, {} split", stats.total, stats.split);
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
    tagline: "Normalize VCF variants and split multiallelic sites.",
    origin: Some(Origin {
        upstream: "bcftools norm",
        upstream_license: "MIT",
        our_license: "MIT OR Apache-2.0",
        paper_doi: Some("10.1093/bioinformatics/btp352"),
    }),
    usage_lines: &["<input.vcf> [-o output.vcf] [-m]"],
    sections: &[Section {
        title: "OPTIONS",
        flags: &[FlagSpec {
            short: Some('m'),
            long: "multiallelics",
            aliases: &[],
            value: None,
            type_hint: None,
            required: false,
            default: None,
            description: "Split multiallelic sites into biallelic records.",
            why_default: None,
        }],
    }],
    examples: &[Example {
        description: "Split multiallelic variants",
        command: "rsomics-vcf-norm -m input.vcf -o normalized.vcf",
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
