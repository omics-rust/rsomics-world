use rsomics_common::{Result, RsomicsError};
use std::fs::File;
use std::io::{BufRead, BufReader, BufWriter, Write};
use std::path::Path;
pub fn vcf_biallelic(input: &Path, output: &mut dyn Write) -> Result<(u64, u64)> {
    let file = File::open(input)
        .map_err(|e| RsomicsError::InvalidInput(format!("{}: {e}", input.display())))?;
    let reader = BufReader::new(file);
    let mut out = BufWriter::with_capacity(64 * 1024, output);
    let mut total: u64 = 0;
    let mut biallelic: u64 = 0;
    for line in reader.lines() {
        let line = line.map_err(RsomicsError::Io)?;
        if line.starts_with('#') {
            writeln!(out, "{line}").map_err(RsomicsError::Io)?;
            continue;
        }
        total += 1;
        if let Some(alt) = line.split('\t').nth(4)
            && !alt.contains(',')
        {
            writeln!(out, "{line}").map_err(RsomicsError::Io)?;
            biallelic += 1;
        }
    }
    out.flush().map_err(RsomicsError::Io)?;
    Ok((total, biallelic))
}
