use std::num::NonZero;
use std::path::PathBuf;

use clap::Parser;
use rsomics_common::{CommonFlags, Result, RsomicsError, Tool, ToolMeta};
use rsomics_help::{Example, FlagSpec, HelpSpec, Origin, Section};

use rsomics_bam_checksum::{ChecksumOpts, run_checksum};

pub const META: ToolMeta = ToolMeta {
    name: env!("CARGO_PKG_NAME"),
    version: env!("CARGO_PKG_VERSION"),
};

#[derive(Parser, Debug)]
#[command(
    name = "rsomics-bam-checksum",
    version,
    about,
    long_about = None,
    disable_help_flag = true
)]
pub struct Cli {
    /// Input BAM file.
    #[arg(value_name = "INPUT")]
    input: PathBuf,

    /// BAM flag bits to include in checksums.
    #[arg(
        short = 'b',
        long = "flag-mask",
        default_value = "193",
        value_name = "INT"
    )]
    flag_mask: u16,

    /// Exclude records with any of these flags set.
    #[arg(
        short = 'F',
        long = "exclude-flags",
        default_value = "2304",
        value_name = "INT"
    )]
    excl_flags: u16,

    /// Require records to have all of these flags set.
    #[arg(
        short = 'f',
        long = "require-flags",
        default_value = "0",
        value_name = "INT"
    )]
    req_flags: u16,

    /// Do not reverse-complement sequences on the reverse strand.
    #[arg(short = 'c', long = "no-rev-comp")]
    no_rev_comp: bool,

    /// Comma-separated aux tags to checksum.
    #[arg(
        long = "aux-tags",
        default_value = "BC,FI,QT,RT,TC",
        value_name = "TAGS"
    )]
    aux_tags: String,

    #[command(flatten)]
    pub common: CommonFlags,
}

impl Cli {
    pub fn execute(self) -> Result<()> {
        let tags = parse_tags(&self.aux_tags)?;
        let workers = NonZero::new(self.common.thread_count()).unwrap_or(NonZero::<usize>::MIN);

        let opts = ChecksumOpts {
            flag_mask: self.flag_mask,
            excl_flags: self.excl_flags,
            req_flags: self.req_flags,
            rev_comp: !self.no_rev_comp,
            tags,
            workers,
        };

        let result = run_checksum(&self.input, &opts)?;
        print!("{result}");
        Ok(())
    }
}

fn parse_tags(s: &str) -> Result<Vec<[u8; 2]>> {
    s.split(',')
        .filter(|t| !t.is_empty())
        .map(|t| {
            let b = t.as_bytes();
            if b.len() != 2 {
                return Err(RsomicsError::InvalidInput(format!(
                    "aux tag must be exactly 2 ASCII characters, got: {t}"
                )));
            }
            Ok([b[0], b[1]])
        })
        .collect()
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
    tagline: "Order-independent BAM checksum (samtools checksum compatible).",
    origin: Some(Origin {
        upstream: "samtools checksum",
        upstream_license: "MIT",
        our_license: "MIT OR Apache-2.0",
        paper_doi: None,
    }),
    usage_lines: &["<INPUT.bam>", "-t BC,QT <INPUT.bam>"],
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
                description: "Input BAM file (positional).",
                why_default: None,
            },
            FlagSpec {
                short: Some('b'),
                long: "flag-mask",
                aliases: &[],
                value: Some("<int>"),
                type_hint: Some("u16"),
                required: false,
                default: Some("193"),
                description: "BAM flag bits to include in checksums (decimal). Default 193 = 0x0c1 = PAIRED|READ1|READ2.",
                why_default: None,
            },
            FlagSpec {
                short: Some('F'),
                long: "exclude-flags",
                aliases: &[],
                value: Some("<int>"),
                type_hint: Some("u16"),
                required: false,
                default: Some("2304"),
                description: "Skip records with any of these flags set. Default 2304 = 0x900 = SECONDARY|SUPPLEMENTARY.",
                why_default: None,
            },
            FlagSpec {
                short: Some('f'),
                long: "require-flags",
                aliases: &[],
                value: Some("<int>"),
                type_hint: Some("u16"),
                required: false,
                default: Some("0"),
                description: "Skip records not having all of these flags.",
                why_default: None,
            },
            FlagSpec {
                short: Some('c'),
                long: "no-rev-comp",
                aliases: &[],
                value: None,
                type_hint: None,
                required: false,
                default: Some("off"),
                description: "Do not reverse-complement sequences on the reverse strand.",
                why_default: None,
            },
            FlagSpec {
                short: None,
                long: "aux-tags",
                aliases: &[],
                value: Some("<tags>"),
                type_hint: Some("String"),
                required: false,
                default: Some("BC,FI,QT,RT,TC"),
                description: "Comma-separated aux tags to include in the +aux checksum.",
                why_default: None,
            },
        ],
    }],
    examples: &[
        Example {
            description: "Compute checksum",
            command: "rsomics-bam-checksum aligned.bam",
        },
        Example {
            description: "With 4 threads",
            command: "rsomics-bam-checksum -t 4 aligned.bam",
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
