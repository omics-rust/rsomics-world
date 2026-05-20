use rsomics_common::{Result, RsomicsError};
use std::fs::File;
use std::io::{BufRead, BufReader, BufWriter, Write};
use std::path::Path;

pub fn vcf_genotypes(input: &Path, output: &mut dyn Write) -> Result<u64> {
    let file = File::open(input)
        .map_err(|e| RsomicsError::InvalidInput(format!("{}: {e}", input.display())))?;
    let reader = BufReader::new(file);
    let mut out = BufWriter::with_capacity(64 * 1024, output);
    let mut count: u64 = 0;
    for line in reader.lines() {
        let line = line.map_err(RsomicsError::Io)?;
        if line.starts_with("##") {
            continue;
        }
        if line.starts_with("#CHROM") {
            let fields: Vec<&str> = line.split('\t').collect();
            write!(out, "CHROM\tPOS").map_err(RsomicsError::Io)?;
            for s in fields.iter().skip(9) {
                write!(out, "\t{s}").map_err(RsomicsError::Io)?;
            }
            writeln!(out).map_err(RsomicsError::Io)?;
            continue;
        }
        if line.is_empty() {
            continue;
        }
        let fields: Vec<&str> = line.split('\t').collect();
        if fields.len() < 10 {
            continue;
        }
        write!(out, "{}\t{}", fields[0], fields[1]).map_err(RsomicsError::Io)?;
        for gt_field in fields.iter().skip(9) {
            let gt = gt_field.split(':').next().unwrap_or(".");
            write!(out, "\t{gt}").map_err(RsomicsError::Io)?;
        }
        writeln!(out).map_err(RsomicsError::Io)?;
        count += 1;
    }
    out.flush().map_err(RsomicsError::Io)?;
    Ok(count)
}
