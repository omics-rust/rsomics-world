use rsomics_common::{Result, RsomicsError};
use std::fs::File;
use std::io::{BufRead, BufReader, BufWriter, Write};
use std::path::Path;
pub fn gff_strand_stats(input: &Path, output: &mut dyn Write) -> Result<u64> {
    let file = File::open(input)
        .map_err(|e| RsomicsError::InvalidInput(format!("{}: {e}", input.display())))?;
    let reader = BufReader::new(file);
    let (mut plus, mut minus, mut none) = (0u64, 0u64, 0u64);
    for line in reader.lines() {
        let line = line.map_err(RsomicsError::Io)?;
        if line.starts_with('#') || line.is_empty() {
            continue;
        }
        match line.split('\t').nth(6) {
            Some("+") => plus += 1,
            Some("-") => minus += 1,
            _ => none += 1,
        }
    }
    let mut out = BufWriter::with_capacity(64 * 1024, output);
    writeln!(out, "+\t{plus}").map_err(RsomicsError::Io)?;
    writeln!(out, "-\t{minus}").map_err(RsomicsError::Io)?;
    writeln!(out, ".\t{none}").map_err(RsomicsError::Io)?;
    out.flush().map_err(RsomicsError::Io)?;
    Ok(plus + minus + none)
}
