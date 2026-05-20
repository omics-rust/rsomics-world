use rsomics_common::{Result, RsomicsError};
use std::fs::File;
use std::io::{BufRead, BufReader, BufWriter, Write};
use std::path::Path;
pub fn bed_sort_name(input: &Path, output: &mut dyn Write) -> Result<u64> {
    let file = File::open(input)
        .map_err(|e| RsomicsError::InvalidInput(format!("{}: {e}", input.display())))?;
    let reader = BufReader::new(file);
    let mut entries: Vec<(String, String)> = Vec::new();
    for line in reader.lines() {
        let line = line.map_err(RsomicsError::Io)?;
        if line.starts_with('#') || line.is_empty() {
            continue;
        }
        let name = line.split('\t').nth(3).unwrap_or("").to_string();
        entries.push((name, line));
    }
    entries.sort_by(|a, b| a.0.cmp(&b.0));
    let mut out = BufWriter::with_capacity(64 * 1024, output);
    for (_, line) in &entries {
        writeln!(out, "{line}").map_err(RsomicsError::Io)?;
    }
    out.flush().map_err(RsomicsError::Io)?;
    Ok(entries.len() as u64)
}
