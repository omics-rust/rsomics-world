use rsomics_common::{Result, RsomicsError};
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::path::Path;
pub fn bed_total_bp(input: &Path) -> Result<u64> {
    let file = File::open(input)
        .map_err(|e| RsomicsError::InvalidInput(format!("{}: {e}", input.display())))?;
    let reader = BufReader::new(file);
    let mut count: u64 = 0;
    for line in reader.lines() {
        let line = line.map_err(RsomicsError::Io)?;
        if line.starts_with('#') || line.is_empty() {
            continue;
        }
        let f: Vec<&str> = line.split('\t').collect();
        if f.len() >= 3 {
            let s: u64 = f[1].parse().unwrap_or(0);
            let e: u64 = f[2].parse().unwrap_or(0);
            count += e.saturating_sub(s);
        }
    }
    Ok(count)
}
