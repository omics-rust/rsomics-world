use clap::Parser;
use rsomics_common::{CommonFlags, Result, ToolMeta};
use rsomics_help::{Example, FlagSpec, HelpSpec, Origin, Section};
use std::path::PathBuf;

pub const META: ToolMeta = ToolMeta {
    name: env!("CARGO_PKG_NAME"),
    version: env!("CARGO_PKG_VERSION"),
};

#[derive(Parser, Debug)]
#[command(name = "rsomics-minimap2", version, about, long_about = None, disable_help_flag = true)]
pub struct Cli {
    /// Reference FASTA (will be indexed).
    #[arg(value_name = "REF")]
    reference: PathBuf,
    /// Query FASTA/FASTQ.
    #[arg(value_name = "QUERY")]
    query: PathBuf,
    /// Preset: sr (short-read), map-ont (ONT), map-hifi (HiFi), map-pb (PacBio CLR).
    #[arg(short = 'x', long = "preset", default_value = "map-ont")]
    preset: String,
    #[command(flatten)]
    pub common: CommonFlags,
}

impl Cli {
    pub fn execute(self) -> Result<()> {
        let output = rsomics_minimap2::align(&self.reference, &self.query, &self.preset)?;
        print!("{output}");
        Ok(())
    }
}

pub static HELP: HelpSpec = HelpSpec {
    name: META.name,
    version: META.version,
    tagline: "Long/short-read aligner (minimap2 FFI wrapper, Quadrant ②).",
    origin: Some(Origin {
        upstream: "minimap2",
        upstream_license: "MIT",
        our_license: "MIT OR Apache-2.0",
        paper_doi: Some("10.1093/bioinformatics/bty191"),
    }),
    usage_lines: &["-x PRESET <REF.fa> <QUERY.fq>"],
    sections: &[Section {
        title: "OPTIONS",
        flags: &[
            FlagSpec {
                short: None,
                long: "REF",
                aliases: &[],
                value: Some("<path>"),
                type_hint: Some("Path"),
                required: true,
                default: None,
                description: "Reference FASTA.",
                why_default: None,
            },
            FlagSpec {
                short: None,
                long: "QUERY",
                aliases: &[],
                value: Some("<path>"),
                type_hint: Some("Path"),
                required: true,
                default: None,
                description: "Query reads.",
                why_default: None,
            },
            FlagSpec {
                short: Some('x'),
                long: "preset",
                aliases: &[],
                value: Some("<str>"),
                type_hint: Some("String"),
                required: false,
                default: Some("map-ont"),
                description: "Preset: sr, map-ont, map-hifi, map-pb.",
                why_default: None,
            },
        ],
    }],
    examples: &[Example {
        description: "Align ONT reads",
        command: "rsomics-minimap2 -x map-ont ref.fa reads.fq > aligned.paf",
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
