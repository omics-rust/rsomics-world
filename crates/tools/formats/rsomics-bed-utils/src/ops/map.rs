#![allow(clippy::cast_precision_loss)]

use std::collections::BTreeMap;
use std::fs::File;
use std::io::{BufRead, BufReader, BufWriter, Write};
use std::path::Path;

use rsomics_common::{Result, RsomicsError};

struct Interval {
    start: u64,
    end: u64,
    value: f64,
}

pub fn map_bed(
    a_path: &Path,
    b_path: &Path,
    output: &mut dyn Write,
    op: &str,
    col: usize,
) -> Result<u64> {
    let b_ivs = load_valued_intervals(b_path, col)?;

    let file = File::open(a_path)
        .map_err(|e| RsomicsError::InvalidInput(format!("{}: {e}", a_path.display())))?;
    let reader = BufReader::new(file);
    let mut out = BufWriter::with_capacity(64 * 1024, output);
    let mut count: u64 = 0;

    for line in reader.lines() {
        let line = line.map_err(RsomicsError::Io)?;
        if line.starts_with('#') || line.is_empty() {
            continue;
        }
        let fields: Vec<&str> = line.split('\t').collect();
        if fields.len() < 3 {
            continue;
        }
        let chrom = fields[0];
        let start: u64 = fields[1].parse().unwrap_or(0);
        let end: u64 = fields[2].parse().unwrap_or(0);

        let mut vals: Vec<f64> = Vec::new();
        if let Some(chr_ivs) = b_ivs.get(chrom) {
            for iv in chr_ivs {
                if start < iv.end && end > iv.start {
                    vals.push(iv.value);
                }
            }
        }

        let result = aggregate(&vals, op);
        writeln!(out, "{line}\t{result}").map_err(RsomicsError::Io)?;
        count += 1;
    }

    out.flush().map_err(RsomicsError::Io)?;
    Ok(count)
}

fn aggregate(vals: &[f64], op: &str) -> String {
    if vals.is_empty() {
        return ".".to_string();
    }
    match op {
        "mean" => {
            let s: f64 = vals.iter().sum();
            format!("{:.6}", s / vals.len() as f64)
        }
        "min" => format!("{:.6}", vals.iter().copied().fold(f64::INFINITY, f64::min)),
        "max" => format!(
            "{:.6}",
            vals.iter().copied().fold(f64::NEG_INFINITY, f64::max)
        ),
        "count" => format!("{}", vals.len()),
        _ => format!("{:.6}", vals.iter().sum::<f64>()),
    }
}

fn load_valued_intervals(path: &Path, col: usize) -> Result<BTreeMap<String, Vec<Interval>>> {
    let file = File::open(path)
        .map_err(|e| RsomicsError::InvalidInput(format!("{}: {e}", path.display())))?;
    let reader = BufReader::new(file);
    let mut by_chrom: BTreeMap<String, Vec<Interval>> = BTreeMap::new();

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
        let value: f64 = fields
            .get(col.saturating_sub(1))
            .and_then(|s| s.parse().ok())
            .unwrap_or(0.0);
        by_chrom
            .entry(chrom)
            .or_default()
            .push(Interval { start, end, value });
    }

    for ivs in by_chrom.values_mut() {
        ivs.sort_by_key(|i| i.start);
    }

    Ok(by_chrom)
}
