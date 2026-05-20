#![allow(clippy::cast_precision_loss)]
use rsomics_common::{Result, RsomicsError};
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::path::Path;

pub struct TsTv {
    pub ts: u64,
    pub tv: u64,
    pub ratio: f64,
}

pub fn vcf_ts_tv(input: &Path) -> Result<TsTv> {
    let file = File::open(input)
        .map_err(|e| RsomicsError::InvalidInput(format!("{}: {e}", input.display())))?;
    let reader = BufReader::new(file);
    let (mut ts, mut tv) = (0u64, 0u64);
    for line in reader.lines() {
        let line = line.map_err(RsomicsError::Io)?;
        if line.starts_with('#') || line.is_empty() {
            continue;
        }
        let fields: Vec<&str> = line.split('\t').collect();
        if fields.len() < 5 || fields[3].len() != 1 {
            continue;
        }
        for alt in fields[4].split(',') {
            if alt.len() != 1 {
                continue;
            }
            match (fields[3].as_bytes()[0], alt.as_bytes()[0]) {
                (b'A', b'G') | (b'G', b'A') | (b'C', b'T') | (b'T', b'C') => ts += 1,
                (b'A' | b'G', b'C' | b'T') | (b'C' | b'T', b'A' | b'G') => tv += 1,
                _ => {}
            }
        }
    }
    let ratio = if tv > 0 { ts as f64 / tv as f64 } else { 0.0 };
    Ok(TsTv { ts, tv, ratio })
}
