use std::path::{Path, PathBuf};

use clap::Parser;
use rsomics_common::{CommonFlags, Result, RsomicsError, Tool, ToolMeta};
use rsomics_help::{Example, FlagSpec, HelpSpec, Origin, Section};

use rsomics_bam_reset::{ResetOpts, reset};

pub const META: ToolMeta = ToolMeta {
    name: env!("CARGO_PKG_NAME"),
    version: env!("CARGO_PKG_VERSION"),
};

#[derive(Parser, Debug)]
#[command(
    name = "rsomics-bam-reset",
    version,
    about,
    long_about = None,
    disable_help_flag = true
)]
pub struct Cli {
    /// Input BAM file.
    pub input: PathBuf,

    /// Output BAM file (default stdout).
    #[arg(short = 'o', long = "output", default_value = "-")]
    output: String,

    /// Aux tags to remove, in addition to the default aligner set
    /// (AS,CC,CG,CP,H1,H2,HI,H0,IH,MC,MD,MQ,NM,SA,TS). Comma-separated.
    #[arg(short = 'x', long = "remove-tag", conflicts_with = "keep_tag")]
    remove_tag: Option<String>,

    /// Aux tags to retain; all others are removed. Comma-separated.
    /// Equivalent to samtools' `-x ^STR`.
    #[arg(long = "keep-tag", conflicts_with = "remove_tag")]
    keep_tag: Option<String>,

    /// Drop @RG header lines and the RG aux tag.
    #[arg(long = "no-RG")]
    no_rg: bool,

    /// Do not add a provenance @PG line for this command.
    #[arg(long = "no-PG")]
    no_pg: bool,

    /// Drop the @PG line with this ID and every @PG line after it.
    #[arg(long = "reject-PG", value_name = "ID")]
    reject_pg: Option<String>,

    /// Keep the duplicate flag (0x400) instead of clearing it.
    #[arg(long = "dupflag")]
    dupflag: bool,

    #[command(flatten)]
    pub common: CommonFlags,
}

fn parse_tag_list(s: &str) -> Result<Vec<[u8; 2]>> {
    s.split(',')
        .filter(|t| !t.is_empty())
        .map(|t| {
            let b = t.as_bytes();
            if b.len() != 2 {
                return Err(RsomicsError::InvalidInput(format!(
                    "aux tag must be exactly 2 characters: {t:?}"
                )));
            }
            Ok([b[0], b[1]])
        })
        .collect()
}

impl Cli {
    pub fn execute(self) -> Result<()> {
        let remove_tags = self
            .remove_tag
            .as_deref()
            .map(parse_tag_list)
            .transpose()?
            .unwrap_or_default();
        let keep_tags = self
            .keep_tag
            .as_deref()
            .map(parse_tag_list)
            .transpose()?
            .unwrap_or_default();

        let opts = ResetOpts {
            remove_tags,
            keep_tags,
            no_rg: self.no_rg,
            no_pg: self.no_pg,
            reject_pg: self.reject_pg,
            keep_dupflag: self.dupflag,
        };

        let output_path: Option<&Path> = if self.output == "-" {
            None
        } else {
            Some(Path::new(&self.output))
        };

        let workers = std::num::NonZero::new(self.common.thread_count())
            .unwrap_or(std::num::NonZero::<usize>::MIN);

        let args_cl = format!("rsomics-bam-reset {}", self.input.display());

        let count = reset(&self.input, output_path, &opts, &args_cl, workers)?;

        if !self.common.quiet {
            eprintln!("{count} records written");
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
    tagline: "Revert aligner changes in BAM reads back to their unaligned state.",
    origin: Some(Origin {
        upstream: "samtools reset",
        upstream_license: "MIT",
        our_license: "MIT OR Apache-2.0",
        paper_doi: Some("10.1093/bioinformatics/btp352"),
    }),
    usage_lines: &[
        "<input.bam> [-o out.bam]",
        "<input.bam> --keep-tag RG,BC -o out.bam",
        "<input.bam> -x XS,YT --no-RG -o out.bam",
    ],
    sections: &[Section {
        title: "OPTIONS",
        flags: &[
            FlagSpec {
                short: Some('x'),
                long: "remove-tag",
                aliases: &[],
                value: Some("STR"),
                type_hint: None,
                required: false,
                default: None,
                description: "Extra aux tags to remove (comma-separated), on top of the default aligner set.",
                why_default: None,
            },
            FlagSpec {
                short: None,
                long: "keep-tag",
                aliases: &[],
                value: Some("STR"),
                type_hint: None,
                required: false,
                default: None,
                description: "Aux tags to retain; all others removed (comma-separated).",
                why_default: None,
            },
            FlagSpec {
                short: None,
                long: "no-RG",
                aliases: &[],
                value: None,
                type_hint: None,
                required: false,
                default: None,
                description: "Drop @RG header lines and the RG aux tag.",
                why_default: None,
            },
            FlagSpec {
                short: None,
                long: "no-PG",
                aliases: &[],
                value: None,
                type_hint: None,
                required: false,
                default: None,
                description: "Do not add a provenance @PG line for this command.",
                why_default: None,
            },
            FlagSpec {
                short: None,
                long: "reject-PG",
                aliases: &[],
                value: Some("ID"),
                type_hint: None,
                required: false,
                default: None,
                description: "Drop the @PG with this ID and every @PG after it.",
                why_default: None,
            },
            FlagSpec {
                short: None,
                long: "dupflag",
                aliases: &[],
                value: None,
                type_hint: None,
                required: false,
                default: None,
                description: "Keep the duplicate flag (0x400) instead of clearing it.",
                why_default: None,
            },
        ],
    }],
    examples: &[
        Example {
            description: "Revert an aligned BAM to unaligned reads",
            command: "rsomics-bam-reset aligned.bam -o reset.bam",
        },
        Example {
            description: "Keep only RG and barcode tags",
            command: "rsomics-bam-reset aligned.bam --keep-tag RG,BC -o reset.bam",
        },
        Example {
            description: "Also strip RG, both header and tag",
            command: "rsomics-bam-reset aligned.bam --no-RG -o reset.bam",
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

    #[test]
    fn parse_tag_list_ok() {
        assert_eq!(
            parse_tag_list("NM,MD,AS").unwrap(),
            vec![*b"NM", *b"MD", *b"AS"]
        );
    }

    #[test]
    fn parse_tag_list_rejects_bad_len() {
        assert!(parse_tag_list("NMM").is_err());
    }
}
