#![allow(clippy::cast_precision_loss)]

use std::collections::BTreeMap;
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::path::Path;

use rsomics_common::{Result, RsomicsError};
use serde::Serialize;

struct Interval {
    start: u64,
    end: u64,
}

#[derive(Debug, Default, Serialize)]
pub struct OverlapStats {
    pub a_count: u64,
    pub b_count: u64,
    pub a_with_overlap: u64,
    pub b_with_overlap: u64,
    pub a_bases: u64,
    pub b_bases: u64,
    pub overlap_bases: u64,
    pub jaccard: f64,
}

pub fn compute_overlap(a_path: &Path, b_path: &Path) -> Result<OverlapStats> {
    let a_ivs = load_intervals(a_path)?;
    let b_ivs = load_intervals(b_path)?;

    let mut stats = OverlapStats::default();

    for (chrom, a_list) in &a_ivs {
        stats.a_count += a_list.len() as u64;
        for a in a_list {
            stats.a_bases += a.end.saturating_sub(a.start);
        }

        if let Some(b_list) = b_ivs.get(chrom) {
            for a in a_list {
                let mut has_overlap = false;
                for b in b_list {
                    if a.start < b.end && a.end > b.start {
                        let os = a.start.max(b.start);
                        let oe = a.end.min(b.end);
                        stats.overlap_bases += oe.saturating_sub(os);
                        has_overlap = true;
                    }
                }
                if has_overlap {
                    stats.a_with_overlap += 1;
                }
            }
        }
    }

    for (chrom, b_list) in &b_ivs {
        stats.b_count += b_list.len() as u64;
        for b in b_list {
            stats.b_bases += b.end.saturating_sub(b.start);
        }

        if let Some(a_list) = a_ivs.get(chrom) {
            for b in b_list {
                for a in a_list {
                    if b.start < a.end && b.end > a.start {
                        stats.b_with_overlap += 1;
                        break;
                    }
                }
            }
        }
    }

    let union = stats
        .a_bases
        .saturating_add(stats.b_bases)
        .saturating_sub(stats.overlap_bases);
    stats.jaccard = if union > 0 {
        stats.overlap_bases as f64 / union as f64
    } else {
        0.0
    };

    Ok(stats)
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
        by_chrom
            .entry(chrom)
            .or_default()
            .push(Interval { start, end });
    }

    for ivs in by_chrom.values_mut() {
        ivs.sort_by_key(|i| i.start);
    }

    Ok(by_chrom)
}
