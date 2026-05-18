#![allow(clippy::cast_precision_loss)]

use std::fs::File;
use std::io::{BufRead, BufReader, BufWriter, Write};
use std::path::Path;

use rsomics_common::{Result, RsomicsError};

pub fn fastq_gc(input: &Path, output: &mut dyn Write) -> Result<u64> {
    let file = File::open(input)
        .map_err(|e| RsomicsError::InvalidInput(format!("{}: {e}", input.display())))?;
    let reader = BufReader::new(file);
    let mut out = BufWriter::with_capacity(64 * 1024, output);
    let mut lines = reader.lines();
    let mut count: u64 = 0;

    writeln!(out, "read\tgc_pct\tlen").map_err(RsomicsError::Io)?;

    while let Some(header) = lines.next() {
        let header = header.map_err(RsomicsError::Io)?;
        let seq = next_line(&mut lines)?;
        let _plus = next_line(&mut lines)?;
        let _qual = next_line(&mut lines)?;

        let name = header
            .split_once(|c: char| c.is_whitespace())
            .map_or(header.as_str(), |(n, _)| n)
            .trim_start_matches('@');

        let len = seq.len();
        let gc = seq
            .bytes()
            .filter(|&b| b == b'G' || b == b'g' || b == b'C' || b == b'c')
            .count();
        let gc_pct = if len > 0 {
            gc as f64 / len as f64 * 100.0
        } else {
            0.0
        };

        writeln!(out, "{name}\t{gc_pct:.2}\t{len}").map_err(RsomicsError::Io)?;
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
