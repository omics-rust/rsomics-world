use std::path::PathBuf;

use clap::Parser;
use rsomics_common::{CommonFlags, Result, RsomicsError, Tool, ToolMeta};

use rsomics_tin::{TinOutput, TinSummary, compute_tin, summarise};

pub const META: ToolMeta = ToolMeta {
    name: env!("CARGO_PKG_NAME"),
    version: env!("CARGO_PKG_VERSION"),
};

#[derive(Parser, Debug)]
#[command(
    name = "rsomics-tin",
    version,
    about = "Compute Transcript Integrity Number (TIN) for each transcript in a BED12 gene model",
    long_about = "Compute TIN for each transcript (or gene) in BED12 format from a sorted, \
indexed BAM file.\n\n\
TIN is an entropy-based score (0–100) measuring coverage uniformity across the mRNA:\n  \
TIN = 100 × exp(H) / n  where H = Shannon entropy over n sampled mRNA positions\n\n\
Outputs:\n  <bam_basename>.tin.xls        per-transcript: geneID, chrom, tx_start, tx_end, TIN\n  \
<bam_basename>.summary.txt    sample-level mean/median/stdev TIN"
)]
pub struct Cli {
    /// Input BAM file(s). Sorted and indexed. Comma-separated, or a directory, or a text file
    /// listing BAM paths (one per line).
    #[arg(short = 'i', long = "input")]
    pub input_files: String,

    /// Reference gene model in BED12 format
    #[arg(short = 'r', long = "refgene")]
    pub refgene: PathBuf,

    /// Minimum number of reads mapped to a transcript (transcripts below → TIN=0)
    #[arg(short = 'c', long = "minCov", default_value = "10")]
    pub min_cov: u64,

    /// Number of equally-spaced mRNA positions to sample per transcript.
    /// Halved until smaller than the transcript mRNA length.
    #[arg(short = 'n', long = "sample-size", default_value = "100")]
    pub sample_size: usize,

    #[command(flatten)]
    pub common: CommonFlags,
}

impl Tool for Cli {
    fn meta() -> ToolMeta {
        META
    }

    fn common(&self) -> &CommonFlags {
        &self.common
    }

    fn execute(self) -> Result<()> {
        let bam_paths = resolve_bam_inputs(&self.input_files)?;

        for bam_path in &bam_paths {
            eprintln!("Processing {} ...", bam_path.display());
            let result = compute_tin(bam_path, &self.refgene, self.min_cov, self.sample_size)?;
            write_output(bam_path, &result)?;

            if self.common.json {
                let summary = build_summary(&result);
                println!(
                    "{}",
                    serde_json::to_string_pretty(&summary)
                        .map_err(|e| RsomicsError::InvalidInput(e.to_string()))?
                );
            }
        }

        Ok(())
    }
}

/// Resolve the `-i` argument: single BAM, comma-separated BAMs, directory, or text file.
fn resolve_bam_inputs(input: &str) -> Result<Vec<PathBuf>> {
    use std::path::Path;

    let p = Path::new(input);

    if p.is_dir() {
        let mut bams: Vec<PathBuf> = std::fs::read_dir(p)
            .map_err(RsomicsError::Io)?
            .filter_map(std::result::Result::ok)
            .map(|e| e.path())
            .filter(|p| p.extension().and_then(|e| e.to_str()) == Some("bam"))
            .collect();
        bams.sort();
        return Ok(bams);
    }

    if p.is_file() && !input.contains(',') {
        // If the path itself ends in .bam, it's a direct BAM file — don't try to read it as text.
        let ext = p
            .extension()
            .and_then(|e| e.to_str())
            .unwrap_or("")
            .to_ascii_lowercase();
        if ext == "bam" {
            return Ok(vec![p.to_path_buf()]);
        }

        // Otherwise it may be a text file listing BAM paths (one per line).
        let content =
            std::fs::read_to_string(p).map_err(|e| RsomicsError::Io(std::io::Error::other(e)))?;
        let first_line = content.lines().next().unwrap_or("");
        let trimmed = first_line.trim_end();
        let looks_like_bam_list =
            trimmed.len() >= 4 && trimmed[trimmed.len() - 4..].eq_ignore_ascii_case(".bam");
        if looks_like_bam_list {
            let paths: Vec<PathBuf> = content
                .lines()
                .map(str::trim)
                .filter(|l| !l.is_empty())
                .map(PathBuf::from)
                .collect();
            return Ok(paths);
        }
        return Ok(vec![p.to_path_buf()]);
    }

    // Comma-separated list
    Ok(input.split(',').map(|s| PathBuf::from(s.trim())).collect())
}

/// Format an f64 like Python's `str(float)`: integer-valued floats keep a
/// trailing `.0` (Rust's `{}` drops it), so tin.py's `0.0`/`100.0` are matched.
/// Non-integer values already share the shortest round-trip repr with Python.
fn py_float(x: f64) -> String {
    let s = format!("{x}");
    if s.contains(['.', 'e', 'E']) || !x.is_finite() {
        s
    } else {
        format!("{s}.0")
    }
}

fn write_output(bam_path: &std::path::Path, result: &[TinOutput]) -> Result<()> {
    use std::io::Write;

    let base = bam_path
        .file_stem()
        .and_then(|s| s.to_str())
        .unwrap_or("out");

    // Per-transcript XLS file
    let xls_path = format!("{base}.tin.xls");
    let mut xls = std::fs::File::create(&xls_path).map_err(|e| {
        RsomicsError::Io(std::io::Error::other(format!("creating {xls_path}: {e}")))
    })?;
    writeln!(xls, "geneID\tchrom\ttx_start\ttx_end\tTIN").map_err(RsomicsError::Io)?;
    for row in result {
        writeln!(
            xls,
            "{}\t{}\t{}\t{}\t{}",
            row.gene_id,
            row.chrom,
            row.tx_start,
            row.tx_end,
            py_float(row.tin)
        )
        .map_err(RsomicsError::Io)?;
    }

    // Summary file
    let summary = build_summary(result);
    let sum_path = format!("{base}.summary.txt");
    let mut sf = std::fs::File::create(&sum_path).map_err(|e| {
        RsomicsError::Io(std::io::Error::other(format!("creating {sum_path}: {e}")))
    })?;
    writeln!(sf, "Bam_file\tTIN(mean)\tTIN(median)\tTIN(stdev)").map_err(RsomicsError::Io)?;
    writeln!(
        sf,
        "{}\t{}\t{}\t{}",
        bam_path.file_name().and_then(|n| n.to_str()).unwrap_or("?"),
        py_float(summary.mean),
        py_float(summary.median),
        py_float(summary.stdev),
    )
    .map_err(RsomicsError::Io)?;

    Ok(())
}

fn build_summary(result: &[TinOutput]) -> TinSummary {
    summarise(result)
}

#[cfg(test)]
mod tests {
    use super::*;
    use clap::CommandFactory;

    #[test]
    fn cli_debug_assert() {
        Cli::command().debug_assert();
    }
}
