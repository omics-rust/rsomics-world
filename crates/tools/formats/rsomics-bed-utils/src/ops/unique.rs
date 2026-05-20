use rsomics_common::{Result, RsomicsError};
use std::collections::HashSet;
use std::fs::File;
use std::io::{BufRead, BufReader, BufWriter, Write};
use std::path::Path;
pub fn bed_unique(input: &Path, output: &mut dyn Write) -> Result<(u64, u64)> {
    let file = File::open(input)
        .map_err(|e| RsomicsError::InvalidInput(format!("{}: {e}", input.display())))?;
    let reader = BufReader::new(file);
    let mut out = BufWriter::with_capacity(64 * 1024, output);
    let mut seen: HashSet<String> = HashSet::new();
    let mut total: u64 = 0;
    let mut kept: u64 = 0;
    for line in reader.lines() {
        let line = line.map_err(RsomicsError::Io)?;
        if line.starts_with('#') || line.is_empty() {
            continue;
        }
        total += 1;
        let key = line.split('\t').take(3).collect::<Vec<_>>().join("\t");
        if seen.insert(key) {
            writeln!(out, "{line}").map_err(RsomicsError::Io)?;
            kept += 1;
        }
    }
    out.flush().map_err(RsomicsError::Io)?;
    Ok((total, kept))
}
