use std::num::NonZero;
use std::path::PathBuf;

use clap::Parser;
use rsomics_common::{CommonFlags, Result, RsomicsError, Tool, ToolMeta};
use rsomics_help::{Example, FlagSpec, HelpSpec, Origin, Section};

use rsomics_bam_consensus::{ConsensusOpts, DEFAULT_EXCL_FLAGS, consensus};

pub const META: ToolMeta = ToolMeta {
    name: env!("CARGO_PKG_NAME"),
    version: env!("CARGO_PKG_VERSION"),
};

#[derive(Parser, Debug)]
#[command(
    name = "rsomics-bam-consensus",
    version,
    about,
    long_about = None,
    disable_help_flag = true
)]
pub struct Cli {
    /// Input coordinate-sorted BAM file.
    pub input: PathBuf,

    /// Output file (default stdout).
    #[arg(short = 'o', long = "output", default_value = "-")]
    output: String,

    /// Use base qualities as weights in the vote (samtools `--use-qual`).
    #[arg(long = "use-qual")]
    use_qual: bool,

    /// Minimum base quality to count a base.
    #[arg(long = "min-BQ", default_value_t = 0)]
    min_bq: u8,

    /// Minimum depth to emit a non-N call.
    #[arg(short = 'd', long = "min-depth", default_value_t = 1)]
    min_depth: u32,

    /// Minimum fraction of total score the called allele(s) must reach.
    #[arg(short = 'c', long = "call-fract", default_value_t = 0.75)]
    call_fract: f64,

    /// Minimum ratio of second-best to best score to call a het.
    #[arg(short = 'H', long = "het-fract", default_value_t = 0.5)]
    het_fract: f64,

    /// Enable IUPAC ambiguity codes for heterozygous sites.
    #[arg(short = 'A', long = "ambig")]
    ambig: bool,

    /// Emit deletion (`*`) bases in the consensus.
    #[arg(long = "show-del")]
    show_del: bool,

    /// Wrap FASTA output at this line length.
    #[arg(short = 'l', long = "line-len", default_value_t = 70)]
    line_len: usize,

    /// Exclude reads with any of these FLAG bits (default UNMAP,SECONDARY,QCFAIL,DUP).
    #[arg(long = "ff", default_value_t = DEFAULT_EXCL_FLAGS)]
    excl_flags: u16,

    /// Include only reads with any of these FLAG bits set (0 = no filter).
    #[arg(long = "rf", default_value_t = 0)]
    incl_flags: u16,

    /// Minimum mapping quality.
    #[arg(long = "min-MQ", default_value_t = 0)]
    min_mapq: u8,

    #[command(flatten)]
    pub common: CommonFlags,
}

impl Cli {
    pub fn execute(self) -> Result<()> {
        let opts = ConsensusOpts {
            use_qual: self.use_qual,
            min_qual: self.min_bq,
            min_depth: self.min_depth,
            call_fract: self.call_fract,
            het_fract: self.het_fract,
            ambig: self.ambig,
            show_del: self.show_del,
            line_len: self.line_len,
            excl_flags: self.excl_flags,
            incl_flags: self.incl_flags,
            min_mapq: self.min_mapq,
        };

        let workers = NonZero::new(self.common.thread_count()).unwrap_or(NonZero::<usize>::MIN);

        let stats = if self.output == "-" {
            let mut out = std::io::stdout().lock();
            consensus(&self.input, &mut out, &opts, workers)?
        } else {
            let file = std::fs::File::create(&self.output).map_err(|e| {
                RsomicsError::InvalidInput(format!("creating {}: {e}", self.output))
            })?;
            let mut out = std::io::BufWriter::new(file);
            consensus(&self.input, &mut out, &opts, workers)?
        };

        if self.common.json {
            eprintln!(
                "{}",
                serde_json::to_string(&stats)
                    .map_err(|e| RsomicsError::InvalidInput(format!("JSON: {e}")))?
            );
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
    tagline: "FASTA consensus from a sorted BAM (simple-mode port of samtools consensus).",
    origin: Some(Origin {
        upstream: "samtools consensus",
        upstream_license: "MIT",
        our_license: "MIT OR Apache-2.0",
        paper_doi: None,
    }),
    usage_lines: &["<input.bam> [-o out.fa] [--use-qual] [--min-depth N]"],
    sections: &[Section {
        title: "OPTIONS",
        flags: &[
            FlagSpec {
                short: Some('o'),
                long: "output",
                aliases: &[],
                value: Some("FILE"),
                type_hint: None,
                required: false,
                default: Some("-"),
                description: "Write FASTA consensus to FILE (default: stdout).",
                why_default: None,
            },
            FlagSpec {
                short: None,
                long: "use-qual",
                aliases: &[],
                value: None,
                type_hint: None,
                required: false,
                default: None,
                description: "Weight each base by its quality score.",
                why_default: None,
            },
            FlagSpec {
                short: Some('d'),
                long: "min-depth",
                aliases: &[],
                value: Some("INT"),
                type_hint: None,
                required: false,
                default: Some("1"),
                description: "Minimum depth to call a non-N base.",
                why_default: None,
            },
            FlagSpec {
                short: Some('c'),
                long: "call-fract",
                aliases: &[],
                value: Some("FLOAT"),
                type_hint: None,
                required: false,
                default: Some("0.75"),
                description: "Minimum fraction of total score for a call.",
                why_default: None,
            },
            FlagSpec {
                short: Some('H'),
                long: "het-fract",
                aliases: &[],
                value: Some("FLOAT"),
                type_hint: None,
                required: false,
                default: Some("0.5"),
                description: "Min ratio of 2nd-best to best score to call a het (needs -A).",
                why_default: None,
            },
            FlagSpec {
                short: Some('A'),
                long: "ambig",
                aliases: &[],
                value: None,
                type_hint: None,
                required: false,
                default: None,
                description: "Enable IUPAC ambiguity codes for het sites.",
                why_default: None,
            },
            FlagSpec {
                short: Some('l'),
                long: "line-len",
                aliases: &[],
                value: Some("INT"),
                type_hint: None,
                required: false,
                default: Some("70"),
                description: "FASTA line wrap length.",
                why_default: None,
            },
        ],
    }],
    examples: &[
        Example {
            description: "FASTA consensus with default settings",
            command: "rsomics-bam-consensus sorted.bam",
        },
        Example {
            description: "Quality-weighted consensus requiring depth ≥ 10",
            command: "rsomics-bam-consensus -q -d 10 sorted.bam -o consensus.fa",
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
