use std::fs::File;
use std::io::{BufRead, BufReader, BufWriter, Write};
use std::path::Path;

use rsomics_common::{Result, RsomicsError};

pub fn tail(input: &Path, output: &mut dyn Write, n: usize) -> Result<u64> {
    let file = File::open(input)
        .map_err(|e| RsomicsError::InvalidInput(format!("{}: {e}", input.display())))?;
    let reader = BufReader::new(file);

    let mut headers: Vec<String> = Vec::new();
    let mut records: Vec<String> = Vec::new();

    for line in reader.lines() {
        let line = line.map_err(RsomicsError::Io)?;
        if line.starts_with('#') {
            headers.push(line);
        } else if !line.is_empty() {
            records.push(line);
        }
    }

    let start = records.len().saturating_sub(n);
    let mut out = BufWriter::with_capacity(64 * 1024, output);

    for h in &headers {
        writeln!(out, "{h}").map_err(RsomicsError::Io)?;
    }
    for line in &records[start..] {
        writeln!(out, "{line}").map_err(RsomicsError::Io)?;
    }

    out.flush().map_err(RsomicsError::Io)?;
    Ok((records.len() - start) as u64)
}
