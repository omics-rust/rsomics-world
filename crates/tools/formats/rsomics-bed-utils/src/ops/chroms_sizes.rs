use rsomics_common::{Result, RsomicsError};
use std::collections::BTreeMap;
use std::fs::File;
use std::io::{BufRead, BufReader, BufWriter, Write};
use std::path::Path;
pub fn bed_chroms_sizes(input: &Path, output: &mut dyn Write) -> Result<u64> {
    let file = File::open(input)
        .map_err(|e| RsomicsError::InvalidInput(format!("{}: {e}", input.display())))?;
    let reader = BufReader::new(file);
    let mut sizes: BTreeMap<String, u64> = BTreeMap::new();
    for line in reader.lines() {
        let line = line.map_err(RsomicsError::Io)?;
        if line.starts_with('#') || line.is_empty() {
            continue;
        }
        let f: Vec<&str> = line.split('\t').collect();
        if f.len() < 3 {
            continue;
        }
        let chrom = f[0].to_string();
        let s: u64 = f[1].parse().unwrap_or(0);
        let e: u64 = f[2].parse().unwrap_or(0);
        *sizes.entry(chrom).or_insert(0) += e.saturating_sub(s);
    }
    let mut out = BufWriter::with_capacity(64 * 1024, output);
    for (chrom, bp) in &sizes {
        writeln!(out, "{chrom}\t{bp}").map_err(RsomicsError::Io)?;
    }
    out.flush().map_err(RsomicsError::Io)?;
    Ok(sizes.len() as u64)
}
