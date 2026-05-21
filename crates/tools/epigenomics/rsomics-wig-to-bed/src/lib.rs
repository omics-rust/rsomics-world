use std::fs::File;
use std::io::{BufRead, BufReader, BufWriter, Write};
use std::path::Path;

use rsomics_common::{Result, RsomicsError};

pub fn wig_to_bed(input: &Path, threshold: f64, output: &mut dyn Write) -> Result<u64> {
    let file = File::open(input)
        .map_err(|e| RsomicsError::InvalidInput(format!("{}: {e}", input.display())))?;
    let reader = BufReader::new(file);
    let mut out = BufWriter::new(output);

    let mut chrom = String::new();
    let mut pos: u64 = 0;
    let mut step: u64 = 1;
    let mut span: u64 = 1;
    let mut count: u64 = 0;

    for line in reader.lines() {
        let line = line.map_err(RsomicsError::Io)?;
        let line = line.trim();

        if line.is_empty() || line.starts_with('#') || line.starts_with("track") {
            continue;
        }

        if line.starts_with("fixedStep") || line.starts_with("variableStep") {
            for part in line.split_whitespace() {
                if let Some(val) = part.strip_prefix("chrom=") {
                    chrom = val.to_string();
                } else if let Some(val) = part.strip_prefix("start=") {
                    pos = val.parse().unwrap_or(1) - 1;
                } else if let Some(val) = part.strip_prefix("step=") {
                    step = val.parse().unwrap_or(1);
                } else if let Some(val) = part.strip_prefix("span=") {
                    span = val.parse().unwrap_or(1);
                }
            }
            continue;
        }

        let parts: Vec<&str> = line.split('\t').collect();
        if parts.len() >= 4 {
            let c = parts[0];
            let s: u64 = parts[1].parse().unwrap_or(0);
            let e: u64 = parts[2].parse().unwrap_or(0);
            let v: f64 = parts[3].parse().unwrap_or(0.0);
            if v >= threshold {
                writeln!(out, "{c}\t{s}\t{e}\t{v}").map_err(RsomicsError::Io)?;
                count += 1;
            }
        } else if let Ok(val) = line.parse::<f64>() {
            if val >= threshold {
                writeln!(out, "{chrom}\t{pos}\t{}\t{val}", pos + span).map_err(RsomicsError::Io)?;
                count += 1;
            }
            pos += step;
        }
    }

    out.flush().map_err(RsomicsError::Io)?;
    Ok(count)
}
