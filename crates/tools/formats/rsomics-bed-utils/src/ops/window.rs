use std::collections::BTreeMap;
use std::fs::File;
use std::io::{BufRead, BufReader, BufWriter, Write};
use std::path::Path;

use rsomics_common::{Result, RsomicsError};

struct Interval {
    line: String,
    start: u64,
    end: u64,
}

pub fn window_bed(
    a_path: &Path,
    b_path: &Path,
    output: &mut dyn Write,
    window: u64,
) -> Result<u64> {
    let b_intervals = load_intervals(b_path)?;

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

        let a_start = start.saturating_sub(window);
        let a_end = end + window;

        if let Some(chr_intervals) = b_intervals.get(chrom) {
            for iv in chr_intervals {
                if a_start < iv.end && a_end > iv.start {
                    writeln!(out, "{line}\t{}", iv.line).map_err(RsomicsError::Io)?;
                    count += 1;
                }
            }
        }
    }

    out.flush().map_err(RsomicsError::Io)?;
    Ok(count)
}

fn load_intervals(path: &Path) -> Result<BTreeMap<String, Vec<Interval>>> {
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
        by_chrom.entry(chrom).or_default().push(Interval {
            line: line.clone(),
            start,
            end,
        });
    }

    for ivs in by_chrom.values_mut() {
        ivs.sort_by_key(|i| i.start);
    }

    Ok(by_chrom)
}
