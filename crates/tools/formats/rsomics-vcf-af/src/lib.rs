use rsomics_common::{Result, RsomicsError};
use std::fs::File;
use std::io::{BufRead, BufReader, BufWriter, Write};
use std::path::Path;
pub fn vcf_af(input: &Path, output: &mut dyn Write) -> Result<u64> {
    let file = File::open(input)
        .map_err(|e| RsomicsError::InvalidInput(format!("{}: {e}", input.display())))?;
    let reader = BufReader::new(file);
    let mut out = BufWriter::with_capacity(64 * 1024, output);
    writeln!(out, "chrom\tpos\tref\talt\taf").map_err(RsomicsError::Io)?;
    let mut count: u64 = 0;
    for line in reader.lines() {
        let line = line.map_err(RsomicsError::Io)?;
        if line.starts_with('#') || line.is_empty() {
            continue;
        }
        let fields: Vec<&str> = line.split('\t').collect();
        if fields.len() < 8 {
            continue;
        }
        let af = fields[7]
            .split(';')
            .find(|s| s.starts_with("AF="))
            .map(|s| &s[3..])
            .unwrap_or(".");
        writeln!(
            out,
            "{}\t{}\t{}\t{}\t{af}",
            fields[0], fields[1], fields[3], fields[4]
        )
        .map_err(RsomicsError::Io)?;
        count += 1;
    }
    out.flush().map_err(RsomicsError::Io)?;
    Ok(count)
}
