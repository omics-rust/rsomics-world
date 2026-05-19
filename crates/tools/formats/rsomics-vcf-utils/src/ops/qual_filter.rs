use rsomics_common::{Result, RsomicsError};
use std::fs::File;
use std::io::{BufRead, BufReader, BufWriter, Write};
use std::path::Path;
pub fn vcf_qual_filter(input: &Path, output: &mut dyn Write, min_qual: f64) -> Result<(u64, u64)> {
    let file = File::open(input)
        .map_err(|e| RsomicsError::InvalidInput(format!("{}: {e}", input.display())))?;
    let reader = BufReader::new(file);
    let mut out = BufWriter::with_capacity(64 * 1024, output);
    let (mut total, mut passed) = (0u64, 0u64);
    for line in reader.lines() {
        let line = line.map_err(RsomicsError::Io)?;
        if line.starts_with('#') {
            writeln!(out, "{line}").map_err(RsomicsError::Io)?;
            continue;
        }
        total += 1;
        let qual: f64 = line
            .split('\t')
            .nth(5)
            .and_then(|s| s.parse().ok())
            .unwrap_or(0.0);
        if qual >= min_qual {
            writeln!(out, "{line}").map_err(RsomicsError::Io)?;
            passed += 1;
        }
    }
    out.flush().map_err(RsomicsError::Io)?;
    Ok((total, passed))
}
