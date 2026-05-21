use rsomics_common::{Result, RsomicsError};
use std::fs::File;
use std::io::{BufRead, BufReader, BufWriter, Write};
use std::path::Path;

pub fn filter_low_counts(
    input: &Path,
    min_count: u64,
    min_samples: usize,
    output: &mut dyn Write,
) -> Result<(u64, u64)> {
    let file = File::open(input)
        .map_err(|e| RsomicsError::InvalidInput(format!("{}: {e}", input.display())))?;
    let reader = BufReader::new(file);
    let mut out = BufWriter::new(output);
    let mut lines = reader.lines();

    let header = lines
        .next()
        .ok_or_else(|| RsomicsError::InvalidInput("empty file".into()))?
        .map_err(RsomicsError::Io)?;
    writeln!(out, "{header}").map_err(RsomicsError::Io)?;

    let mut total = 0u64;
    let mut kept = 0u64;

    for line in lines {
        let line = line.map_err(RsomicsError::Io)?;
        let parts: Vec<&str> = line.split('\t').collect();
        if parts.len() < 2 {
            continue;
        }
        total += 1;

        let above = parts[1..]
            .iter()
            .filter(|s| s.parse::<u64>().unwrap_or(0) >= min_count)
            .count();

        if above >= min_samples {
            writeln!(out, "{line}").map_err(RsomicsError::Io)?;
            kept += 1;
        }
    }

    out.flush().map_err(RsomicsError::Io)?;
    Ok((total, kept))
}
