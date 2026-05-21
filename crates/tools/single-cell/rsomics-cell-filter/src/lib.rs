use std::fs::File;
use std::io::{BufRead, BufReader, BufWriter, Write};
use std::path::Path;

use rsomics_common::{Result, RsomicsError};

pub struct FilterCriteria {
    pub min_genes: u64,
    pub min_umis: u64,
    pub max_mito_frac: f64,
}

pub struct FilterResult {
    pub total: u64,
    pub passed: u64,
    pub failed: u64,
}

pub fn filter_cells(
    input: &Path,
    criteria: &FilterCriteria,
    output: &mut dyn Write,
) -> Result<FilterResult> {
    let file = File::open(input)
        .map_err(|e| RsomicsError::InvalidInput(format!("{}: {e}", input.display())))?;
    let reader = BufReader::new(file);
    let mut out = BufWriter::new(output);
    let mut lines = reader.lines();

    if let Some(header) = lines.next() {
        let header = header.map_err(RsomicsError::Io)?;
        writeln!(out, "{header}").map_err(RsomicsError::Io)?;
    }

    let mut total = 0u64;
    let mut passed = 0u64;

    for line in lines {
        let line = line.map_err(RsomicsError::Io)?;
        let parts: Vec<&str> = line.split('\t').collect();
        if parts.len() < 4 {
            continue;
        }
        total += 1;

        let genes: u64 = parts.get(1).and_then(|s| s.parse().ok()).unwrap_or(0);
        let umis: u64 = parts.get(2).and_then(|s| s.parse().ok()).unwrap_or(0);
        let mito: f64 = parts.get(3).and_then(|s| s.parse().ok()).unwrap_or(1.0);

        if genes >= criteria.min_genes
            && umis >= criteria.min_umis
            && mito <= criteria.max_mito_frac
        {
            writeln!(out, "{line}").map_err(RsomicsError::Io)?;
            passed += 1;
        }
    }

    out.flush().map_err(RsomicsError::Io)?;
    Ok(FilterResult {
        total,
        passed,
        failed: total - passed,
    })
}
