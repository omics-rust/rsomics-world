use std::collections::BTreeMap;
use std::fs::File;
use std::io::{BufRead, BufReader, BufWriter, Write};
use std::path::Path;

use rsomics_common::{Result, RsomicsError};

pub fn list_features(input: &Path, output: &mut dyn Write) -> Result<u64> {
    let file = File::open(input)
        .map_err(|e| RsomicsError::InvalidInput(format!("{}: {e}", input.display())))?;
    let reader = BufReader::new(file);
    let mut counts: BTreeMap<String, u64> = BTreeMap::new();

    for line in reader.lines() {
        let line = line.map_err(RsomicsError::Io)?;
        if line.starts_with('#') || line.is_empty() {
            continue;
        }
        if let Some(feature) = line.split('\t').nth(2) {
            *counts.entry(feature.to_string()).or_insert(0) += 1;
        }
    }

    let mut out = BufWriter::with_capacity(64 * 1024, output);
    for (feature, count) in &counts {
        writeln!(out, "{feature}\t{count}").map_err(RsomicsError::Io)?;
    }
    out.flush().map_err(RsomicsError::Io)?;

    Ok(counts.len() as u64)
}
