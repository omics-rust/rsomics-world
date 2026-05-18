use rsomics_common::{Result, RsomicsError};
use std::fs::File;
use std::io::{BufRead, BufReader, BufWriter, Write};
use std::path::Path;

pub fn vcf_samples(input: &Path, output: &mut dyn Write) -> Result<u64> {
    let file = File::open(input)
        .map_err(|e| RsomicsError::InvalidInput(format!("{}: {e}", input.display())))?;
    let reader = BufReader::new(file);
    let mut out = BufWriter::with_capacity(64 * 1024, output);
    let mut count: u64 = 0;
    for line in reader.lines() {
        let line = line.map_err(RsomicsError::Io)?;
        if line.starts_with("#CHROM") {
            let fields: Vec<&str> = line.split('\t').collect();
            for sample in fields.iter().skip(9) {
                writeln!(out, "{sample}").map_err(RsomicsError::Io)?;
                count += 1;
            }
            break;
        }
    }
    out.flush().map_err(RsomicsError::Io)?;
    Ok(count)
}
