use std::collections::BTreeMap;
use std::fs::File;
use std::io::{BufRead, BufReader, BufWriter, Write};
use std::path::Path;

use rsomics_common::{Result, RsomicsError};

struct Feature {
    start: u64,
    end: u64,
    name: String,
}

pub fn annotate_bed(
    bed_path: &Path,
    gff_path: &Path,
    output: &mut dyn Write,
    feature_type: &str,
    attribute: &str,
) -> Result<u64> {
    let features = load_features(gff_path, feature_type, attribute)?;

    let file = File::open(bed_path)
        .map_err(|e| RsomicsError::InvalidInput(format!("{}: {e}", bed_path.display())))?;
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
        let mid = u64::midpoint(start, end);

        let nearest = find_nearest(&features, chrom, mid);

        write!(out, "{line}\t").map_err(RsomicsError::Io)?;
        match nearest {
            Some((name, dist)) => writeln!(out, "{name}\t{dist}"),
            None => writeln!(out, ".\t."),
        }
        .map_err(RsomicsError::Io)?;
        count += 1;
    }

    out.flush().map_err(RsomicsError::Io)?;
    Ok(count)
}

fn load_features(
    path: &Path,
    feature_type: &str,
    attribute: &str,
) -> Result<BTreeMap<String, Vec<Feature>>> {
    let file = File::open(path)
        .map_err(|e| RsomicsError::InvalidInput(format!("{}: {e}", path.display())))?;
    let reader = BufReader::new(file);
    let mut by_chrom: BTreeMap<String, Vec<Feature>> = BTreeMap::new();

    for line in reader.lines() {
        let line = line.map_err(RsomicsError::Io)?;
        if line.starts_with('#') || line.is_empty() {
            continue;
        }
        let fields: Vec<&str> = line.split('\t').collect();
        if fields.len() < 9 || fields[2] != feature_type {
            continue;
        }
        let chrom = fields[0].to_string();
        let start: u64 = fields[3].parse().unwrap_or(0);
        let end: u64 = fields[4].parse().unwrap_or(0);
        let name = extract_attr(fields[8], attribute).unwrap_or_default();
        if name.is_empty() {
            continue;
        }
        by_chrom
            .entry(chrom)
            .or_default()
            .push(Feature { start, end, name });
    }

    for feats in by_chrom.values_mut() {
        feats.sort_by_key(|f| f.start);
    }

    Ok(by_chrom)
}

fn find_nearest(
    features: &BTreeMap<String, Vec<Feature>>,
    chrom: &str,
    pos: u64,
) -> Option<(String, u64)> {
    let feats = features.get(chrom)?;
    let mut best_name = None;
    let mut best_dist = u64::MAX;

    for f in feats {
        let dist = if pos < f.start {
            f.start.saturating_sub(pos)
        } else {
            pos.saturating_sub(f.end)
        };
        if dist < best_dist {
            best_dist = dist;
            best_name = Some(f.name.clone());
        }
        if dist == 0 {
            break;
        }
    }

    best_name.map(|n| (n, best_dist))
}

fn extract_attr(attrs: &str, key: &str) -> Option<String> {
    for part in attrs.split(';') {
        let part = part.trim();
        let Some(rest) = part.strip_prefix(key) else {
            continue;
        };
        if rest.starts_with('=') || rest.starts_with(' ') {
            let val = rest[1..].trim().trim_matches('"');
            if !val.is_empty() {
                return Some(val.to_string());
            }
        }
    }
    None
}
