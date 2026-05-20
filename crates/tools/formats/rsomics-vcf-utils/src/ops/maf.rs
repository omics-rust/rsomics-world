#![allow(clippy::cast_precision_loss)]
use rsomics_common::{Result, RsomicsError};
use std::fs::File;
use std::io::{BufRead, BufReader, BufWriter, Write};
use std::path::Path;

pub fn vcf_maf(
    input: &Path,
    output: &mut dyn Write,
    min_maf: f64,
    max_maf: f64,
) -> Result<u64> {
    let file = File::open(input)
        .map_err(|e| RsomicsError::InvalidInput(format!("{}: {e}", input.display())))?;
    let reader = BufReader::new(file);
    let mut out = BufWriter::with_capacity(64 * 1024, output);
    let mut kept = 0u64;
    for line in reader.lines() {
        let line = line.map_err(RsomicsError::Io)?;
        if line.starts_with('#') {
            writeln!(out, "{line}").map_err(RsomicsError::Io)?;
            continue;
        }
        let af: f64 = line
            .split('\t')
            .nth(7)
            .and_then(|info| {
                info.split(';')
                    .find(|s| s.starts_with("AF="))
                    .map(|s| s[3..].parse().unwrap_or(0.0))
            })
            .unwrap_or(0.0);
        let maf = af.min(1.0 - af);
        if maf >= min_maf && maf <= max_maf {
            writeln!(out, "{line}").map_err(RsomicsError::Io)?;
            kept += 1;
        }
    }
    out.flush().map_err(RsomicsError::Io)?;
    Ok(kept)
}
