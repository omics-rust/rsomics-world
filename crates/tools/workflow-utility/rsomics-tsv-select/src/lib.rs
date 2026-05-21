use std::fs::File;
use std::io::{BufRead, BufReader, BufWriter, Write};
use std::path::Path;

use rsomics_common::{Result, RsomicsError};

pub fn select_columns(input: &Path, columns: &[String], output: &mut dyn Write) -> Result<u64> {
    let file = File::open(input)
        .map_err(|e| RsomicsError::InvalidInput(format!("{}: {e}", input.display())))?;
    let reader = BufReader::new(file);
    let mut out = BufWriter::new(output);
    let mut lines = reader.lines();

    let header = lines
        .next()
        .ok_or_else(|| RsomicsError::InvalidInput("empty file".into()))?
        .map_err(RsomicsError::Io)?;
    let headers: Vec<&str> = header.split('\t').collect();

    let indices: Vec<usize> = columns
        .iter()
        .map(|col| {
            if let Ok(idx) = col.parse::<usize>() {
                if idx > 0 && idx <= headers.len() {
                    Ok(idx - 1)
                } else {
                    Err(RsomicsError::InvalidInput(format!(
                        "column index {idx} out of range (1-{})",
                        headers.len()
                    )))
                }
            } else {
                headers
                    .iter()
                    .position(|h| *h == col.as_str())
                    .ok_or_else(|| {
                        RsomicsError::InvalidInput(format!("column '{col}' not found in header"))
                    })
            }
        })
        .collect::<Result<Vec<_>>>()?;

    let selected_headers: Vec<&str> = indices.iter().map(|&i| headers[i]).collect();
    writeln!(out, "{}", selected_headers.join("\t")).map_err(RsomicsError::Io)?;

    let mut count = 0u64;
    for line in lines {
        let line = line.map_err(RsomicsError::Io)?;
        let fields: Vec<&str> = line.split('\t').collect();
        let selected: Vec<&str> = indices
            .iter()
            .map(|&i| fields.get(i).copied().unwrap_or(""))
            .collect();
        writeln!(out, "{}", selected.join("\t")).map_err(RsomicsError::Io)?;
        count += 1;
    }

    out.flush().map_err(RsomicsError::Io)?;
    Ok(count)
}
