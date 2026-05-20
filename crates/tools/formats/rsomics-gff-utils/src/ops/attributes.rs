use rsomics_common::{Result, RsomicsError};
use std::collections::BTreeSet;
use std::fs::File;
use std::io::{BufRead, BufReader, BufWriter, Write};
use std::path::Path;

pub fn gff_attributes(input: &Path, output: &mut dyn Write) -> Result<u64> {
    let file = File::open(input)
        .map_err(|e| RsomicsError::InvalidInput(format!("{}: {e}", input.display())))?;
    let reader = BufReader::new(file);
    let mut keys: BTreeSet<String> = BTreeSet::new();
    for line in reader.lines() {
        let line = line.map_err(RsomicsError::Io)?;
        if line.starts_with('#') || line.is_empty() {
            continue;
        }
        if let Some(attrs) = line.split('\t').nth(8) {
            for part in attrs.split(';') {
                let part = part.trim();
                if let Some(key) = part.split(['=', ' ']).next()
                    && !key.is_empty()
                {
                    keys.insert(key.to_string());
                }
            }
        }
    }
    let mut out = BufWriter::with_capacity(64 * 1024, output);
    for key in &keys {
        writeln!(out, "{key}").map_err(RsomicsError::Io)?;
    }
    out.flush().map_err(RsomicsError::Io)?;
    Ok(keys.len() as u64)
}
