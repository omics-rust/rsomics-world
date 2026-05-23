use std::path::PathBuf;

use clap::Parser;
use rsomics_common::{CommonFlags, Result, RsomicsError, Tool, ToolMeta};
use rsomics_help::{Example, FlagSpec, HelpSpec, Origin, Section};

use rsomics_vcf_index::{IndexKind, index_vcf};

pub const META: ToolMeta = ToolMeta {
    name: env!("CARGO_PKG_NAME"),
    version: env!("CARGO_PKG_VERSION"),
};

#[derive(Parser, Debug)]
#[command(
    name = "rsomics-vcf-index",
    version,
    about,
    long_about = None,
    disable_help_flag = true
)]
pub struct Cli {
    /// Input bgzipped VCF file (.vcf.gz).
    #[arg(value_name = "INPUT")]
    pub input: PathBuf,

    /// Write a tabix (.tbi) index instead of the default CSI index.
    #[arg(long = "tbi", default_value_t = false)]
    pub tbi: bool,

    /// Overwrite an existing index without error.
    #[arg(short = 'f', long = "force", default_value_t = false)]
    pub force: bool,

    /// Output index path (default: <INPUT>.csi or <INPUT>.tbi).
    #[arg(short = 'o', long = "output")]
    pub output: Option<PathBuf>,

    #[command(flatten)]
    pub common: CommonFlags,
}

impl Cli {
    pub fn execute(self) -> Result<()> {
        let kind = if self.tbi {
            IndexKind::Tbi
        } else {
            IndexKind::Csi
        };

        let dst = self.output.unwrap_or_else(|| {
            let ext = if self.tbi { "tbi" } else { "csi" };
            let mut p = self.input.clone();
            let new_name = format!(
                "{}.{ext}",
                p.file_name().unwrap_or_default().to_string_lossy()
            );
            p.set_file_name(new_name);
            p
        });

        if !self.force && dst.exists() {
            return Err(RsomicsError::InvalidInput(format!(
                "index already exists: {} (use --force to overwrite)",
                dst.display()
            )));
        }

        index_vcf(&self.input, &dst, kind).map_err(RsomicsError::Io)?;

        if !self.common.quiet {
            eprintln!("wrote {}", dst.display());
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
    tagline: "Index a bgzipped VCF (.csi/.tbi).",
    origin: Some(Origin {
        upstream: "bcftools index / tabix -p vcf",
        upstream_license: "MIT",
        our_license: "MIT OR Apache-2.0",
        paper_doi: Some("10.1093/gigascience/giab008"),
    }),
    usage_lines: &["[OPTIONS] <INPUT.vcf.gz>"],
    sections: &[Section {
        title: "OPTIONS",
        flags: &[
            FlagSpec {
                short: None,
                long: "INPUT",
                aliases: &[],
                value: Some("<path>"),
                type_hint: Some("Path"),
                required: true,
                default: None,
                description: "Input bgzipped VCF (.vcf.gz).",
                why_default: None,
            },
            FlagSpec {
                short: None,
                long: "tbi",
                aliases: &[],
                value: None,
                type_hint: Some("Flag"),
                required: false,
                default: Some("off"),
                description: "Write a tabix (.tbi) index (default: CSI).",
                why_default: None,
            },
            FlagSpec {
                short: Some('f'),
                long: "force",
                aliases: &[],
                value: None,
                type_hint: Some("Flag"),
                required: false,
                default: Some("off"),
                description: "Overwrite an existing index.",
                why_default: None,
            },
            FlagSpec {
                short: Some('o'),
                long: "output",
                aliases: &[],
                value: Some("<path>"),
                type_hint: Some("Path"),
                required: false,
                default: Some("<INPUT>.csi / <INPUT>.tbi"),
                description: "Output index path.",
                why_default: None,
            },
        ],
    }],
    examples: &[
        Example {
            description: "Build a CSI index (default)",
            command: "rsomics-vcf-index sample.vcf.gz",
        },
        Example {
            description: "Build a tabix index",
            command: "rsomics-vcf-index --tbi sample.vcf.gz",
        },
        Example {
            description: "Overwrite an existing index",
            command: "rsomics-vcf-index --force sample.vcf.gz",
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
