#![allow(clippy::cast_precision_loss)]

use std::fs::File;
use std::io::{BufRead, BufReader, BufWriter, Write};
use std::path::Path;

use rsomics_common::{Result, RsomicsError};

pub fn fastq_window(input: &Path, output: &mut dyn Write, window_size: usize) -> Result<u64> {
    let file = File::open(input)
        .map_err(|e| RsomicsError::InvalidInput(format!("{}: {e}", input.display())))?;
    let reader = BufReader::new(file);
    let mut out = BufWriter::with_capacity(64 * 1024, output);
    let mut lines = reader.lines();
    let mut count: u64 = 0;

    writeln!(out, "read\tpos\tmean_qual").map_err(RsomicsError::Io)?;

    while let Some(header) = lines.next() {
        let header = header.map_err(RsomicsError::Io)?;
        let _seq = next_line(&mut lines)?;
        let _plus = next_line(&mut lines)?;
        let qual = next_line(&mut lines)?;

        let name = header
            .split_once(|c: char| c.is_whitespace())
            .map_or(header.as_str(), |(n, _)| n)
            .trim_start_matches('@');

        let quals: Vec<u8> = qual.bytes().map(|b| b.saturating_sub(33)).collect();

        if quals.len() >= window_size {
            for pos in 0..=(quals.len() - window_size) {
                let sum: u64 = quals[pos..pos + window_size]
                    .iter()
                    .map(|&q| u64::from(q))
                    .sum();
                let mean = sum as f64 / window_size as f64;
                writeln!(out, "{name}\t{pos}\t{mean:.1}").map_err(RsomicsError::Io)?;
            }
        }

        count += 1;
    }

    out.flush().map_err(RsomicsError::Io)?;
    Ok(count)
}

fn next_line(lines: &mut std::io::Lines<BufReader<File>>) -> Result<String> {
    lines
        .next()
        .ok_or_else(|| RsomicsError::InvalidInput("truncated FASTQ".into()))?
        .map_err(RsomicsError::Io)
}
