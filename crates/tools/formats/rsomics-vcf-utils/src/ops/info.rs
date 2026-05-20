use std::fs::File;
use std::io::{BufRead, BufReader, BufWriter, Write};
use std::path::Path;

use rsomics_common::{Result, RsomicsError};

pub fn extract_info(input: &Path, output: &mut dyn Write, key: &str) -> Result<u64> {
    let file = File::open(input)
        .map_err(|e| RsomicsError::InvalidInput(format!("{}: {e}", input.display())))?;
    let reader = BufReader::new(file);
    let mut out = BufWriter::with_capacity(64 * 1024, output);
    let mut count: u64 = 0;
    let prefix = format!("{key}=");

    for line in reader.lines() {
        let line = line.map_err(RsomicsError::Io)?;
        if line.starts_with('#') || line.is_empty() {
            continue;
        }
        let fields: Vec<&str> = line.split('\t').collect();
        if fields.len() < 8 {
            continue;
        }

        let chrom = fields[0];
        let pos = fields[1];
        let info = fields[7];

        let value = info
            .split(';')
            .find(|s| s.starts_with(&prefix))
            .map_or(".", |s| &s[prefix.len()..]);

        writeln!(out, "{chrom}\t{pos}\t{value}").map_err(RsomicsError::Io)?;
        count += 1;
    }

    out.flush().map_err(RsomicsError::Io)?;
    Ok(count)
}
