#![allow(clippy::cast_precision_loss)]
use rsomics_common::{Result, RsomicsError};
use std::fs::File;
use std::io::{BufRead, BufReader, BufWriter, Write};
use std::path::Path;
pub fn bed_coverage_hist(input: &Path, output: &mut dyn Write) -> Result<u64> {
    let file = File::open(input)
        .map_err(|e| RsomicsError::InvalidInput(format!("{}: {e}", input.display())))?;
    let reader = BufReader::new(file);
    let mut sizes: Vec<u64> = Vec::new();
    for line in reader.lines() {
        let line = line.map_err(RsomicsError::Io)?;
        if line.starts_with('#') || line.is_empty() {
            continue;
        }
        let fields: Vec<&str> = line.split('\t').collect();
        if fields.len() < 3 {
            continue;
        }
        let start: u64 = fields[1].parse().unwrap_or(0);
        let end: u64 = fields[2].parse().unwrap_or(0);
        sizes.push(end.saturating_sub(start));
    }
    sizes.sort_unstable();
    let mut out = BufWriter::with_capacity(64 * 1024, output);
    let total: u64 = sizes.iter().sum();
    let n = sizes.len();
    writeln!(out, "count\t{n}").map_err(RsomicsError::Io)?;
    writeln!(out, "total_bp\t{total}").map_err(RsomicsError::Io)?;
    if !sizes.is_empty() {
        writeln!(out, "min\t{}", sizes[0]).map_err(RsomicsError::Io)?;
        writeln!(out, "max\t{}", sizes[n - 1]).map_err(RsomicsError::Io)?;
        writeln!(out, "mean\t{:.1}", total as f64 / n as f64).map_err(RsomicsError::Io)?;
        writeln!(out, "median\t{}", sizes[n / 2]).map_err(RsomicsError::Io)?;
    }
    out.flush().map_err(RsomicsError::Io)?;
    Ok(n as u64)
}
