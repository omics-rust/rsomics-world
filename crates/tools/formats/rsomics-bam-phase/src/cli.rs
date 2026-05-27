use std::path::PathBuf;

use clap::Parser;
use rsomics_common::{CommonFlags, Result, RsomicsError, Tool, ToolMeta};
use rsomics_help::{Example, FlagSpec, HelpSpec, Origin, Section};

use rsomics_bam_phase::{PhaseOpts, phase};

pub const META: ToolMeta = ToolMeta {
    name: env!("CARGO_PKG_NAME"),
    version: env!("CARGO_PKG_VERSION"),
};

#[derive(Parser, Debug)]
#[command(
    name = "rsomics-bam-phase",
    version,
    about,
    long_about = None,
    disable_help_flag = true
)]
pub struct Cli {
    /// Input coordinate-sorted BAM file.
    pub input: PathBuf,

    /// DP window length for local haplotype states.
    #[arg(short = 'k', long = "window", default_value_t = 13)]
    pub window: usize,

    /// Output BAM prefix (creates <prefix>.0.bam, <prefix>.1.bam, <prefix>.chimera.bam).
    #[arg(short = 'b', long = "bam-prefix")]
    pub bam_prefix: Option<PathBuf>,

    /// Minimum het phred-LOD threshold.
    #[arg(long = "min-lod", default_value_t = 37)]
    pub min_lod: u32,

    /// Minimum base quality.
    #[arg(short = 'Q', long = "min-bq", default_value_t = 13)]
    pub min_bq: u8,

    /// Maximum pileup depth per site.
    #[arg(short = 'D', long = "max-depth", default_value_t = 256)]
    pub max_depth: usize,

    /// Disable chimera-fragment detection and flipping.
    #[arg(short = 'F', long = "no-fix-chimera", default_value_t = false)]
    pub no_fix_chimera: bool,

    /// Route ambiguously phased reads to chimera output (instead of random haplotype).
    #[arg(short = 'A', long = "drop-ambiguous", default_value_t = false)]
    pub drop_ambiguous: bool,

    #[command(flatten)]
    pub common: CommonFlags,
}

impl Cli {
    pub fn execute(self) -> Result<()> {
        let opts = PhaseOpts {
            k: self.window,
            bam_prefix: self.bam_prefix,
            min_var_lod: self.min_lod,
            min_base_q: self.min_bq,
            max_depth: self.max_depth,
            fix_chimera: !self.no_fix_chimera,
            drop_ambiguous: self.drop_ambiguous,
        };

        let workers = std::num::NonZero::new(self.common.thread_count())
            .unwrap_or(std::num::NonZero::<usize>::MIN);

        let mut stdout = std::io::stdout().lock();
        let stats = phase(&self.input, &mut stdout, &opts, workers)?;

        if self.common.json {
            eprintln!(
                "{}",
                serde_json::to_string(&stats)
                    .map_err(|e| RsomicsError::InvalidInput(format!("JSON: {e}")))?
            );
        } else if !self.common.quiet {
            eprintln!(
                "{} records, {} het sites, {} phase sets, {} masked sites",
                stats.records_in, stats.het_sites, stats.phase_sets, stats.masked_sites,
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
    tagline: "Phase heterozygous SNPs from aligned reads.",
    origin: Some(Origin {
        upstream: "samtools phase",
        upstream_license: "MIT",
        our_license: "MIT OR Apache-2.0",
        paper_doi: None,
    }),
    usage_lines: &["<input.bam> [-k 13] [--min-lod 37] [-Q 13] [-D 256] [-b PREFIX] [-F] [-A]"],
    sections: &[Section {
        title: "OPTIONS",
        flags: &[
            FlagSpec {
                short: Some('k'),
                long: "window",
                aliases: &[],
                value: Some("INT"),
                type_hint: Some("int"),
                required: false,
                default: Some("13"),
                description: "DP window length for local haplotype patterns.",
                why_default: Some(
                    "Matches samtools phase default. Larger k gives better phasing at higher compute cost.",
                ),
            },
            FlagSpec {
                short: Some('b'),
                long: "bam-prefix",
                aliases: &[],
                value: Some("STR"),
                type_hint: Some("str"),
                required: false,
                default: None,
                description: "BAM output prefix. Creates <prefix>.0.bam, <prefix>.1.bam, <prefix>.chimera.bam.",
                why_default: None,
            },
            FlagSpec {
                short: None,
                long: "min-lod",
                aliases: &[],
                value: Some("INT"),
                type_hint: Some("int"),
                required: false,
                default: Some("37"),
                description: "Minimum het phred-LOD to call a site as heterozygous.",
                why_default: Some("Matches samtools phase default."),
            },
            FlagSpec {
                short: Some('Q'),
                long: "min-bq",
                aliases: &[],
                value: Some("INT"),
                type_hint: Some("int"),
                required: false,
                default: Some("13"),
                description: "Minimum base quality for pileup allele accumulation.",
                why_default: Some("Matches samtools phase default."),
            },
            FlagSpec {
                short: Some('D'),
                long: "max-depth",
                aliases: &[],
                value: Some("INT"),
                type_hint: Some("int"),
                required: false,
                default: Some("256"),
                description: "Skip pileup sites with depth exceeding this value.",
                why_default: Some("Matches samtools phase default."),
            },
            FlagSpec {
                short: Some('F'),
                long: "no-fix-chimera",
                aliases: &[],
                value: None,
                type_hint: None,
                required: false,
                default: None,
                description: "Disable chimeric-fragment detection and flipping.",
                why_default: None,
            },
            FlagSpec {
                short: Some('A'),
                long: "drop-ambiguous",
                aliases: &[],
                value: None,
                type_hint: None,
                required: false,
                default: None,
                description: "Route ambiguously phased reads to chimera output rather than random haplotype.",
                why_default: None,
            },
        ],
    }],
    examples: &[
        Example {
            description: "Phase reads and write text output to stdout",
            command: "rsomics-bam-phase sorted.bam",
        },
        Example {
            description: "Phase and split reads into haplotype BAMs",
            command: "rsomics-bam-phase sorted.bam -b hap",
        },
        Example {
            description: "Lower LOD threshold for low-coverage data",
            command: "rsomics-bam-phase sorted.bam --min-lod 20",
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
