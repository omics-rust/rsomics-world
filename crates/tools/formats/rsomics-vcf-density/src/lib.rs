#![allow(clippy::cast_precision_loss)]
use rsomics_common::{Result, RsomicsError};
use std::collections::BTreeMap;
use std::fs::File;
use std::io::{BufRead, BufReader, BufWriter, Write};
use std::path::Path;
pub fn vcf_density(input: &Path, output: &mut dyn Write, window: u64) -> Result<u64> {
    let file = File::open(input)
        .map_err(|e| RsomicsError::InvalidInput(format!("{}: {e}", input.display())))?;
    let reader = BufReader::new(file);
    let mut bins: BTreeMap<(String, u64), u64> = BTreeMap::new();
    for line in reader.lines() {
        let line = line.map_err(RsomicsError::Io)?;
        if line.starts_with('#') || line.is_empty() {
            continue;
        }
        let mut fields = line.split('\t');
        if let (Some(chrom), Some(pos_str)) = (fields.next(), fields.next()) {
            if let Ok(pos) = pos_str.parse::<u64>() {
                let bin = pos / window;
                *bins.entry((chrom.to_string(), bin)).or_insert(0) += 1;
            }
        }
    }
    let mut out = BufWriter::with_capacity(64 * 1024, output);
    for ((chrom, bin), count) in &bins {
        let start = bin * window;
        writeln!(out, "{chrom}\t{start}\t{}\t{count}", start + window).map_err(RsomicsError::Io)?;
    }
    out.flush().map_err(RsomicsError::Io)?;
    Ok(bins.len() as u64)
}
