use std::fs::File;
use std::io::{BufRead, BufReader, BufWriter, Write};
use std::path::Path;

use rsomics_common::{Result, RsomicsError};

pub fn bed_spacing(input: &Path, output: &mut dyn Write) -> Result<u64> {
    let file = File::open(input)
        .map_err(|e| RsomicsError::InvalidInput(format!("{}: {e}", input.display())))?;
    let reader = BufReader::new(file);
    let mut out = BufWriter::with_capacity(64 * 1024, output);

    let mut prev_chrom = String::new();
    let mut prev_end: u64 = 0;
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

        if chrom == prev_chrom && start >= prev_end {
            let gap = start.saturating_sub(prev_end);
            writeln!(out, "{chrom}\t{prev_end}\t{start}\t{gap}").map_err(RsomicsError::Io)?;
            count += 1;
        }

        prev_chrom = chrom.to_string();
        prev_end = end;
    }

    out.flush().map_err(RsomicsError::Io)?;
    Ok(count)
}
