use std::path::PathBuf;

use clap::Parser;
use rsomics_common::{CommonFlags, Result, RsomicsError, Tool, ToolMeta};
use rsomics_help::{Example, HelpSpec, Origin};

use rsomics_vcf_convert::{OutputFormat, convert, vcf_to_haplegendsample};

pub const META: ToolMeta = ToolMeta {
    name: env!("CARGO_PKG_NAME"),
    version: env!("CARGO_PKG_VERSION"),
};

/// bcftools convert output-type codes mapped to our enum.
fn parse_output_type(s: &str) -> std::result::Result<OutputFormat, String> {
    // Accept "v", "z", "z0"–"z9". Reject "b"/"u" with informative error.
    match s {
        "b" | "u" => Err(
            "BCF binary output (-O b/u) is not implemented in rsomics-vcf-convert 0.1.0; \
             use `bcftools convert -O b/u` instead"
                .into(),
        ),
        "v" => Ok(OutputFormat::VcfText),
        s if s.starts_with('z') => Ok(OutputFormat::VcfGz),
        _ => Err(format!(
            "unknown output type '{s}'; expected v, z, z0-z9, b, or u"
        )),
    }
}

#[derive(Parser)]
#[command(
    name = "rsomics-vcf-convert",
    version,
    about = "Convert VCF/VCF.gz to another format — Rust port of bcftools convert",
    disable_help_flag = true
)]
pub struct Cli {
    /// Input VCF or VCF.gz file ("-" for stdin not supported in 0.1.0)
    pub input: PathBuf,

    /// Output file [stdout]
    #[arg(short = 'o', long, default_value = "-")]
    pub output: String,

    /// Output type: v (VCF text), z/z0-z9 (bgzipped VCF) [v]
    ///
    /// -O b (BCF binary) and -O u (uncompressed BCF) are not implemented
    /// in 0.1.0; use `bcftools convert` for those.
    #[arg(short = 'O', long = "output-type", value_name = "TYPE",
          default_value = "v",
          value_parser = parse_output_type)]
    pub output_type: OutputFormat,

    /// Export to HAP/LEGEND/SAMPLE: <PREFIX> or <HAP-FILE>,<LEGEND-FILE>,<SAMPLE-FILE>
    ///
    /// Matches bcftools `-h`/`--haplegendsample`. Use the long form here
    /// because `-h` is reserved for help output.
    #[arg(long, value_name = "SPEC")]
    pub haplegendsample: Option<String>,

    #[command(flatten)]
    pub common: CommonFlags,
}

/// Resolve HAP/LEGEND/SAMPLE output paths from a bcftools-style spec.
///
/// The spec is either a bare prefix (three files are derived by appending
/// `.hap.gz`, `.legend.gz`, `.sample`) or three comma-separated file paths.
fn resolve_hls_paths(spec: &str) -> (String, String, String) {
    if spec.contains(',') {
        let mut parts = spec.splitn(3, ',');
        let hap = parts.next().unwrap_or("-").to_owned();
        let leg = parts.next().unwrap_or("-").to_owned();
        let samp = parts.next().unwrap_or("-").to_owned();
        (hap, leg, samp)
    } else {
        (
            format!("{spec}.hap"),
            format!("{spec}.legend"),
            format!("{spec}.samples"),
        )
    }
}

fn open_write(path: &str) -> Result<Box<dyn std::io::Write>> {
    if path == "-" {
        Ok(Box::new(std::io::stdout().lock()))
    } else {
        std::fs::File::create(path)
            .map(|f| Box::new(f) as Box<dyn std::io::Write>)
            .map_err(|e| RsomicsError::InvalidInput(format!("{path}: {e}")))
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
        if let Some(spec) = &self.haplegendsample {
            let (hap_path, leg_path, samp_path) = resolve_hls_paths(spec);
            let mut hap = open_write(&hap_path)?;
            let mut leg = open_write(&leg_path)?;
            let mut samp = open_write(&samp_path)?;
            let (n, skipped) =
                vcf_to_haplegendsample(&self.input, &mut *hap, &mut *leg, &mut *samp)?;
            if skipped > 0 {
                eprintln!("warning: {skipped} multi-allelic records skipped");
            }
            eprintln!("wrote {n} variants to {hap_path}, {leg_path}, {samp_path}");
            return Ok(());
        }

        let mut out = open_write(&self.output)?;
        convert(&self.input, &mut *out, self.output_type)?;
        Ok(())
    }
}

pub static HELP: HelpSpec = HelpSpec {
    name: env!("CARGO_PKG_NAME"),
    version: env!("CARGO_PKG_VERSION"),
    tagline: "Convert VCF/VCF.gz — plain text ↔ bgzipped, optional HAP/LEGEND/SAMPLE export.",
    origin: Some(Origin {
        upstream: "bcftools convert",
        upstream_license: "MIT",
        our_license: "MIT OR Apache-2.0",
        paper_doi: None,
    }),
    usage_lines: &["[OPTIONS] <input>"],
    sections: &[],
    examples: &[
        Example {
            description: "Compress a plain VCF to bgzipped VCF",
            command: "rsomics-vcf-convert -O z input.vcf -o output.vcf.gz",
        },
        Example {
            description: "Decompress a VCF.gz to plain VCF",
            command: "rsomics-vcf-convert -O v input.vcf.gz -o output.vcf",
        },
        Example {
            description: "Export to HAP/LEGEND/SAMPLE with prefix (bcftools -h equivalent)",
            command: "rsomics-vcf-convert --haplegendsample chr22 input.vcf.gz",
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
