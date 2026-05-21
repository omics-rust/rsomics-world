use std::fs::File;
use std::io::{BufRead, BufReader, BufWriter, Write};
use std::path::Path;

use rsomics_common::{Result, RsomicsError};

pub struct RankStats {
    pub total_barcodes: usize,
    pub knee_idx: usize,
    pub knee_count: u64,
}

pub fn barcode_rank(input: &Path, output: &mut dyn Write) -> Result<RankStats> {
    let file = File::open(input)
        .map_err(|e| RsomicsError::InvalidInput(format!("{}: {e}", input.display())))?;
    let reader = BufReader::new(file);

    let mut counts: Vec<u64> = Vec::new();
    for line in reader.lines() {
        let line = line.map_err(RsomicsError::Io)?;
        let line = line.trim();
        if line.is_empty() || line.starts_with('#') {
            continue;
        }
        let parts: Vec<&str> = line.split('\t').collect();
        let count: u64 = parts
            .last()
            .unwrap_or(&"0")
            .parse()
            .map_err(|e| RsomicsError::InvalidInput(format!("bad count: {e}")))?;
        counts.push(count);
    }

    counts.sort_unstable_by(|a, b| b.cmp(a));

    let knee_idx = find_knee(&counts);
    let knee_count = counts.get(knee_idx).copied().unwrap_or(0);

    let mut out = BufWriter::new(output);
    writeln!(out, "rank\tcount").map_err(RsomicsError::Io)?;
    for (i, &c) in counts.iter().enumerate() {
        writeln!(out, "{}\t{c}", i + 1).map_err(RsomicsError::Io)?;
    }
    out.flush().map_err(RsomicsError::Io)?;

    Ok(RankStats {
        total_barcodes: counts.len(),
        knee_idx,
        knee_count,
    })
}

fn find_knee(sorted_desc: &[u64]) -> usize {
    if sorted_desc.len() < 3 {
        return 0;
    }
    let mut max_dist = 0.0f64;
    let mut knee = 0;
    let n = sorted_desc.len();

    #[allow(clippy::cast_precision_loss)]
    let (x0, y0) = (0.0, sorted_desc[0] as f64);
    #[allow(clippy::cast_precision_loss)]
    let (x1, y1) = (n as f64, *sorted_desc.last().unwrap() as f64);
    let line_len = ((x1 - x0).powi(2) + (y1 - y0).powi(2)).sqrt();

    #[allow(clippy::cast_precision_loss)]
    for (i, &val) in sorted_desc.iter().enumerate() {
        let px = i as f64;
        let py = val as f64;
        let dist = ((y1 - y0) * px - (x1 - x0) * py + x1 * y0 - y1 * x0).abs() / line_len;
        if dist > max_dist {
            max_dist = dist;
            knee = i;
        }
    }
    knee
}
