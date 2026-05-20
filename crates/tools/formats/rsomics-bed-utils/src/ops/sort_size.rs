use rsomics_common::{Result, RsomicsError};
use std::fs::File;
use std::io::{BufRead, BufReader, BufWriter, Write};
use std::path::Path;

pub fn sort_by_size(input: &Path, output: &mut dyn Write, descending: bool) -> Result<u64> {
    let file = File::open(input)
        .map_err(|e| RsomicsError::InvalidInput(format!("{}: {e}", input.display())))?;
    let reader = BufReader::new(file);
    let mut entries: Vec<(u64, String)> = Vec::new();
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
        entries.push((end.saturating_sub(start), line));
    }
    if descending {
        entries.sort_by_key(|entry| std::cmp::Reverse(entry.0));
    } else {
        entries.sort_by_key(|(size, _)| *size);
    }
    let mut out = BufWriter::with_capacity(64 * 1024, output);
    for (_, line) in &entries {
        writeln!(out, "{line}").map_err(RsomicsError::Io)?;
    }
    out.flush().map_err(RsomicsError::Io)?;
    Ok(entries.len() as u64)
}
