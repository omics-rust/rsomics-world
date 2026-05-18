#![allow(clippy::cast_precision_loss)]
use rsomics_common::{Result, RsomicsError};
use std::fs::File;
use std::io::{BufRead, BufReader, BufWriter, Write};
use std::path::Path;
pub fn vcf_qual_stats(input: &Path, output: &mut dyn Write) -> Result<u64> {
    let file = File::open(input)
        .map_err(|e| RsomicsError::InvalidInput(format!("{}: {e}", input.display())))?;
    let reader = BufReader::new(file);
    let mut quals: Vec<f64> = Vec::new();
    for line in reader.lines() {
        let line = line.map_err(RsomicsError::Io)?;
        if line.starts_with('#') || line.is_empty() {
            continue;
        }
        if let Some(q) = line.split('\t').nth(5).and_then(|s| s.parse::<f64>().ok()) {
            quals.push(q);
        }
    }
    quals.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));
    let mut out = BufWriter::with_capacity(64 * 1024, output);
    let n = quals.len();
    if n > 0 {
        let sum: f64 = quals.iter().sum();
        writeln!(out, "count\t{n}").map_err(RsomicsError::Io)?;
        writeln!(out, "mean\t{:.1}", sum / n as f64).map_err(RsomicsError::Io)?;
        writeln!(out, "min\t{:.1}", quals[0]).map_err(RsomicsError::Io)?;
        writeln!(out, "max\t{:.1}", quals[n - 1]).map_err(RsomicsError::Io)?;
        writeln!(out, "median\t{:.1}", quals[n / 2]).map_err(RsomicsError::Io)?;
    }
    out.flush().map_err(RsomicsError::Io)?;
    Ok(n as u64)
}
