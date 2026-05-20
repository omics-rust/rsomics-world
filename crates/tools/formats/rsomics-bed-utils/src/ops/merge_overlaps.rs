use rsomics_common::{Result, RsomicsError};
use std::collections::BTreeMap;
use std::fs::File;
use std::io::{BufRead, BufReader, BufWriter, Write};
use std::path::Path;
pub fn bed_merge_overlaps(input: &Path, output: &mut dyn Write) -> Result<u64> {
    let file = File::open(input)
        .map_err(|e| RsomicsError::InvalidInput(format!("{}: {e}", input.display())))?;
    let reader = BufReader::new(file);
    let mut by_chrom: BTreeMap<String, Vec<(u64, u64)>> = BTreeMap::new();
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
        by_chrom.entry(chrom).or_default().push((s, e));
    }
    let mut out = BufWriter::with_capacity(64 * 1024, output);
    let mut count: u64 = 0;
    for (chrom, mut ivs) in by_chrom {
        ivs.sort_by_key(|(s, _)| *s);
        let mut merged: Vec<(u64, u64)> = Vec::new();
        for (s, e) in ivs {
            if let Some(last) = merged.last_mut()
                && s <= last.1
            {
                last.1 = last.1.max(e);
                continue;
            }
            merged.push((s, e));
        }
        for (s, e) in &merged {
            writeln!(out, "{chrom}\t{s}\t{e}").map_err(RsomicsError::Io)?;
            count += 1;
        }
    }
    out.flush().map_err(RsomicsError::Io)?;
    Ok(count)
}
