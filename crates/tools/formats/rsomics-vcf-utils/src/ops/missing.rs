#![allow(clippy::cast_precision_loss)]
use rsomics_common::{Result, RsomicsError};
use std::fs::File;
use std::io::{BufRead, BufReader, BufWriter, Write};
use std::path::Path;

pub fn vcf_missing(input: &Path, output: &mut dyn Write) -> Result<u64> {
    let file = File::open(input)
        .map_err(|e| RsomicsError::InvalidInput(format!("{}: {e}", input.display())))?;
    let reader = BufReader::new(file);
    let mut out = BufWriter::with_capacity(64 * 1024, output);
    let mut samples: Vec<String> = Vec::new();
    let mut missing: Vec<u64> = Vec::new();
    let mut total: u64 = 0;
    for line in reader.lines() {
        let line = line.map_err(RsomicsError::Io)?;
        if line.starts_with("#CHROM") {
            let fields: Vec<&str> = line.split('\t').collect();
            for s in fields.iter().skip(9) {
                samples.push(s.to_string());
            }
            missing.resize(samples.len(), 0);
            continue;
        }
        if line.starts_with('#') || line.is_empty() {
            continue;
        }
        total += 1;
        let fields: Vec<&str> = line.split('\t').collect();
        for (i, gt_field) in fields.iter().skip(9).enumerate() {
            let gt = gt_field.split(':').next().unwrap_or(".");
            if gt.contains('.') && i < missing.len() {
                missing[i] += 1;
            }
        }
    }
    writeln!(out, "sample\tmissing\ttotal\tpct").map_err(RsomicsError::Io)?;
    for (i, s) in samples.iter().enumerate() {
        let m = missing.get(i).copied().unwrap_or(0);
        let pct = if total > 0 {
            m as f64 / total as f64 * 100.0
        } else {
            0.0
        };
        writeln!(out, "{s}\t{m}\t{total}\t{pct:.2}").map_err(RsomicsError::Io)?;
    }
    out.flush().map_err(RsomicsError::Io)?;
    Ok(samples.len() as u64)
}
