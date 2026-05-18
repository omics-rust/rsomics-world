use rsomics_common::{Result, RsomicsError};
use std::collections::BTreeMap;
use std::fs::File;
use std::io::{BufRead, BufReader, BufWriter, Write};
use std::path::Path;
pub fn bed_total_span(input: &Path, output: &mut dyn Write) -> Result<u64> {
    let file = File::open(input)
        .map_err(|e| RsomicsError::InvalidInput(format!("{}: {e}", input.display())))?;
    let reader = BufReader::new(file);
    let mut spans: BTreeMap<String, (u64, u64)> = BTreeMap::new();
    for line in reader.lines() {
        let line = line.map_err(RsomicsError::Io)?;
        if line.starts_with('#') || line.is_empty() {
            continue;
        }
        let f: Vec<&str> = line.split('\t').collect();
        if f.len() < 3 {
            continue;
        }
        let chrom = f[0].to_string();
        let start: u64 = f[1].parse().unwrap_or(0);
        let end: u64 = f[2].parse().unwrap_or(0);
        let entry = spans.entry(chrom).or_insert((u64::MAX, 0));
        entry.0 = entry.0.min(start);
        entry.1 = entry.1.max(end);
    }
    let mut out = BufWriter::with_capacity(64 * 1024, output);
    let mut total: u64 = 0;
    for (chrom, (min_s, max_e)) in &spans {
        let span = max_e.saturating_sub(*min_s);
        writeln!(out, "{chrom}\t{span}").map_err(RsomicsError::Io)?;
        total += span;
    }
    writeln!(out, "total\t{total}").map_err(RsomicsError::Io)?;
    out.flush().map_err(RsomicsError::Io)?;
    Ok(spans.len() as u64)
}
