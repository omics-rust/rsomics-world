#![allow(clippy::cast_precision_loss)]

use std::path::Path;

use needletail::parse_fastx_file;
use rsomics_common::{Result, RsomicsError};
use serde::Serialize;

#[derive(Debug, Clone, Serialize)]
pub struct AssemblyStats {
    pub num_seqs: u64,
    pub total_len: u64,
    pub min_len: u64,
    pub max_len: u64,
    pub mean_len: f64,
    pub n50: u64,
    pub n90: u64,
    pub l50: u64,
    pub l90: u64,
    pub gc_pct: f64,
}

pub fn compute_n50(input: &Path) -> Result<AssemblyStats> {
    if std::fs::metadata(input).is_ok_and(|m| m.len() == 0) {
        return Err(RsomicsError::InvalidInput("empty file".into()));
    }

    let mut reader = parse_fastx_file(input)
        .map_err(|e| RsomicsError::InvalidInput(format!("{}: {e}", input.display())))?;

    let mut lengths: Vec<u64> = Vec::new();
    let mut gc_count: u64 = 0;
    let mut total: u64 = 0;

    while let Some(record) = reader.next() {
        let record = record.map_err(|e| RsomicsError::InvalidInput(format!("reading: {e}")))?;
        let seq = record.seq();
        let len = seq.len() as u64;
        lengths.push(len);
        total += len;
        for &b in seq.iter() {
            if b == b'G' || b == b'g' || b == b'C' || b == b'c' {
                gc_count += 1;
            }
        }
    }

    if lengths.is_empty() {
        return Err(RsomicsError::InvalidInput("no sequences".into()));
    }

    lengths.sort_unstable_by(|a, b| b.cmp(a));

    let num_seqs = lengths.len() as u64;
    let min_len = *lengths.last().unwrap();
    let max_len = lengths[0];
    let mean_len = total as f64 / num_seqs as f64;
    let gc_pct = if total > 0 {
        gc_count as f64 / total as f64 * 100.0
    } else {
        0.0
    };

    let n50 = compute_nx(&lengths, total, 50);
    let n90 = compute_nx(&lengths, total, 90);
    let l50 = compute_lx(&lengths, total, 50);
    let l90 = compute_lx(&lengths, total, 90);

    Ok(AssemblyStats {
        num_seqs,
        total_len: total,
        min_len,
        max_len,
        mean_len,
        n50,
        n90,
        l50,
        l90,
        gc_pct,
    })
}

fn compute_nx(lengths: &[u64], total: u64, x: u64) -> u64 {
    let threshold = total * x / 100;
    let mut cumsum: u64 = 0;
    for &len in lengths {
        cumsum += len;
        if cumsum >= threshold {
            return len;
        }
    }
    0
}

fn compute_lx(lengths: &[u64], total: u64, x: u64) -> u64 {
    let threshold = total * x / 100;
    let mut cumsum: u64 = 0;
    for (i, &len) in lengths.iter().enumerate() {
        cumsum += len;
        if cumsum >= threshold {
            return (i + 1) as u64;
        }
    }
    0
}
