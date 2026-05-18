use rsomics_common::{Result, RsomicsError};
use std::collections::BTreeMap;
use std::fs::File;
use std::io::{BufRead, BufReader, BufWriter, Write};
use std::path::Path;
pub fn gff_sources(input: &Path, output: &mut dyn Write) -> Result<u64> {
    let file = File::open(input)
        .map_err(|e| RsomicsError::InvalidInput(format!("{}: {e}", input.display())))?;
    let reader = BufReader::new(file);
    let mut sources: BTreeMap<String, u64> = BTreeMap::new();
    for line in reader.lines() {
        let line = line.map_err(RsomicsError::Io)?;
        if line.starts_with('#') || line.is_empty() {
            continue;
        }
        if let Some(source) = line.split('\t').nth(1) {
            *sources.entry(source.to_string()).or_insert(0) += 1;
        }
    }
    let mut out = BufWriter::with_capacity(64 * 1024, output);
    for (source, count) in &sources {
        writeln!(out, "{source}\t{count}").map_err(RsomicsError::Io)?;
    }
    out.flush().map_err(RsomicsError::Io)?;
    Ok(sources.len() as u64)
}
