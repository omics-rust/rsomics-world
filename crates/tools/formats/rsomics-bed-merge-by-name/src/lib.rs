use rsomics_common::{Result, RsomicsError};
use std::collections::BTreeMap;
use std::fs::File;
use std::io::{BufRead, BufReader, BufWriter, Write};
use std::path::Path;

pub fn merge_by_name(input: &Path, output: &mut dyn Write) -> Result<u64> {
    let file = File::open(input)
        .map_err(|e| RsomicsError::InvalidInput(format!("{}: {e}", input.display())))?;
    let reader = BufReader::new(file);
    let mut groups: BTreeMap<String, (String, u64, u64)> = BTreeMap::new();
    for line in reader.lines() {
        let line = line.map_err(RsomicsError::Io)?;
        if line.starts_with('#') || line.is_empty() {
            continue;
        }
        let fields: Vec<&str> = line.split('\t').collect();
        if fields.len() < 4 {
            continue;
        }
        let chrom = fields[0].to_string();
        let start: u64 = fields[1].parse().unwrap_or(0);
        let end: u64 = fields[2].parse().unwrap_or(0);
        let name = fields[3].to_string();
        let key = format!("{chrom}\t{name}");
        groups
            .entry(key)
            .and_modify(|(_, s, e)| {
                *s = (*s).min(start);
                *e = (*e).max(end);
            })
            .or_insert((chrom, start, end));
    }
    let mut out = BufWriter::with_capacity(64 * 1024, output);
    for (key, (chrom, start, end)) in &groups {
        let name = key.split('\t').nth(1).unwrap_or(".");
        writeln!(out, "{chrom}\t{start}\t{end}\t{name}").map_err(RsomicsError::Io)?;
    }
    out.flush().map_err(RsomicsError::Io)?;
    Ok(groups.len() as u64)
}
