use std::collections::BTreeMap;
use std::fs::File;
use std::io::{BufRead, BufReader, BufWriter, Write};
use std::path::Path;

use rsomics_common::{Result, RsomicsError};

pub fn bed_union(a_path: &Path, b_path: &Path, output: &mut dyn Write) -> Result<u64> {
    let mut intervals = load_intervals(a_path)?;
    let b_intervals = load_intervals(b_path)?;

    for (chrom, ivs) in b_intervals {
        intervals.entry(chrom).or_default().extend(ivs);
    }

    let mut out = BufWriter::with_capacity(64 * 1024, output);
    let mut count: u64 = 0;

    for (chrom, mut ivs) in intervals {
        ivs.sort_by_key(|(s, _)| *s);
        let merged = merge_intervals(&ivs);
        for (start, end) in &merged {
            writeln!(out, "{chrom}\t{start}\t{end}").map_err(RsomicsError::Io)?;
            count += 1;
        }
    }

    out.flush().map_err(RsomicsError::Io)?;
    Ok(count)
}

fn merge_intervals(ivs: &[(u64, u64)]) -> Vec<(u64, u64)> {
    let mut merged: Vec<(u64, u64)> = Vec::new();
    for &(start, end) in ivs {
        if let Some(last) = merged.last_mut()
            && start <= last.1
        {
            last.1 = last.1.max(end);
            continue;
        }
        merged.push((start, end));
    }
    merged
}

fn load_intervals(path: &Path) -> Result<BTreeMap<String, Vec<(u64, u64)>>> {
    let file = File::open(path)
        .map_err(|e| RsomicsError::InvalidInput(format!("{}: {e}", path.display())))?;
    let reader = BufReader::new(file);
    let mut by_chrom: BTreeMap<String, Vec<(u64, u64)>> = BTreeMap::new();

    for line in reader.lines() {
        let line = line.map_err(RsomicsError::Io)?;
        if line.starts_with('#') || line.is_empty() {
            continue;
        }
        let fields: Vec<&str> = line.split('\t').collect();
        if fields.len() < 3 {
            continue;
        }
        let chrom = fields[0].to_string();
        let start: u64 = fields[1].parse().unwrap_or(0);
        let end: u64 = fields[2].parse().unwrap_or(0);
        by_chrom.entry(chrom).or_default().push((start, end));
    }

    Ok(by_chrom)
}
