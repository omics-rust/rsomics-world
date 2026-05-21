use rsomics_common::{Result, RsomicsError};
use std::fs::File;
use std::io::{BufRead, BufReader, BufWriter, Write};
use std::path::Path;

pub fn vcf_to_bed(input: &Path, output: &mut dyn Write) -> Result<u64> {
    let file = File::open(input)
        .map_err(|e| RsomicsError::InvalidInput(format!("{}: {e}", input.display())))?;
    let reader = BufReader::new(file);
    let mut out = BufWriter::new(output);
    let mut count = 0u64;

    for line in reader.lines() {
        let line = line.map_err(RsomicsError::Io)?;
        if line.starts_with('#') {
            continue;
        }
        let parts: Vec<&str> = line.split('\t').collect();
        if parts.len() < 5 {
            continue;
        }
        let chrom = parts[0];
        let pos: u64 = parts[1].parse().unwrap_or(0);
        let ref_allele = parts[3];
        let start = pos.saturating_sub(1);
        let end = start + ref_allele.len() as u64;
        writeln!(out, "{chrom}\t{start}\t{end}").map_err(RsomicsError::Io)?;
        count += 1;
    }
    out.flush().map_err(RsomicsError::Io)?;
    Ok(count)
}
