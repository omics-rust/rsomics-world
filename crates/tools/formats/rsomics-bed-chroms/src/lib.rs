use rsomics_common::{Result, RsomicsError};
use std::collections::BTreeSet;
use std::fs::File;
use std::io::{BufRead, BufReader, BufWriter, Write};
use std::path::Path;

pub fn bed_chroms(input: &Path, output: &mut dyn Write) -> Result<u64> {
    let file = File::open(input)
        .map_err(|e| RsomicsError::InvalidInput(format!("{}: {e}", input.display())))?;
    let reader = BufReader::new(file);
    let mut chroms: BTreeSet<String> = BTreeSet::new();
    for line in reader.lines() {
        let line = line.map_err(RsomicsError::Io)?;
        if line.starts_with('#') || line.is_empty() {
            continue;
        }
        if let Some(chrom) = line.split('\t').next() {
            chroms.insert(chrom.to_string());
        }
    }
    let mut out = BufWriter::with_capacity(64 * 1024, output);
    for chrom in &chroms {
        writeln!(out, "{chrom}").map_err(RsomicsError::Io)?;
    }
    out.flush().map_err(RsomicsError::Io)?;
    Ok(chroms.len() as u64)
}
