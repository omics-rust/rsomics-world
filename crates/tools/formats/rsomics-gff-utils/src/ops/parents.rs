use rsomics_common::{Result, RsomicsError};
use std::collections::BTreeSet;
use std::fs::File;
use std::io::{BufRead, BufReader, BufWriter, Write};
use std::path::Path;

pub fn gff_parents(input: &Path, output: &mut dyn Write) -> Result<u64> {
    let file = File::open(input)
        .map_err(|e| RsomicsError::InvalidInput(format!("{}: {e}", input.display())))?;
    let reader = BufReader::new(file);
    let mut parents: BTreeSet<String> = BTreeSet::new();
    for line in reader.lines() {
        let line = line.map_err(RsomicsError::Io)?;
        if line.starts_with('#') || line.is_empty() {
            continue;
        }
        if let Some(attrs) = line.split('\t').nth(8) {
            for part in attrs.split(';') {
                let part = part.trim();
                if let Some(rest) = part
                    .strip_prefix("Parent=")
                    .or_else(|| part.strip_prefix("Parent "))
                {
                    for p in rest.trim_matches('"').split(',') {
                        parents.insert(p.to_string());
                    }
                }
            }
        }
    }
    let mut out = BufWriter::with_capacity(64 * 1024, output);
    for p in &parents {
        writeln!(out, "{p}").map_err(RsomicsError::Io)?;
    }
    out.flush().map_err(RsomicsError::Io)?;
    Ok(parents.len() as u64)
}
