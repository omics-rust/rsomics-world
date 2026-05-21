use std::fs::File;
use std::io::{BufRead, BufReader};
use std::path::Path;

use rsomics_common::{Result, RsomicsError};

pub fn load_sfs(path: &Path) -> Result<(Vec<u64>, u64)> {
    let file = File::open(path)
        .map_err(|e| RsomicsError::InvalidInput(format!("{}: {e}", path.display())))?;
    let reader = BufReader::new(file);
    let mut counts = Vec::new();
    let mut n_samples = 0u64;

    for line in reader.lines() {
        let line = line.map_err(RsomicsError::Io)?;
        let line = line.trim();
        if line.is_empty() || line.starts_with('#') {
            continue;
        }
        let parts: Vec<&str> = line.split('\t').collect();
        if parts.len() >= 2 {
            let count: u64 = parts[0]
                .parse()
                .map_err(|e| RsomicsError::InvalidInput(format!("bad count: {e}")))?;
            let total: u64 = parts[1]
                .parse()
                .map_err(|e| RsomicsError::InvalidInput(format!("bad n: {e}")))?;
            counts.push(count);
            n_samples = n_samples.max(total);
        } else {
            let count: u64 = parts[0]
                .parse()
                .map_err(|e| RsomicsError::InvalidInput(format!("bad count: {e}")))?;
            counts.push(count);
        }
    }
    Ok((counts, n_samples))
}
