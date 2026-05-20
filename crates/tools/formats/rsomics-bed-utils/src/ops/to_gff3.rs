use rsomics_common::{Result, RsomicsError};
use std::fs::File;
use std::io::{BufRead, BufReader, BufWriter, Write};
use std::path::Path;

pub fn bed_to_gff3(input: &Path, output: &mut dyn Write) -> Result<u64> {
    let file = File::open(input)
        .map_err(|e| RsomicsError::InvalidInput(format!("{}: {e}", input.display())))?;
    let reader = BufReader::new(file);
    let mut out = BufWriter::with_capacity(64 * 1024, output);
    writeln!(out, "##gff-version 3").map_err(RsomicsError::Io)?;
    let mut count: u64 = 0;
    for line in reader.lines() {
        let line = line.map_err(RsomicsError::Io)?;
        if line.starts_with('#') || line.is_empty() {
            continue;
        }
        let f: Vec<&str> = line.split('\t').collect();
        if f.len() < 3 {
            continue;
        }
        let chrom = f[0];
        let start: u64 = f[1].parse::<u64>().unwrap_or(0) + 1;
        let end = f[2];
        let name = f.get(3).unwrap_or(&".");
        let score = f.get(4).unwrap_or(&".");
        let strand = f.get(5).unwrap_or(&".");
        writeln!(
            out,
            "{chrom}\tBED\tregion\t{start}\t{end}\t{score}\t{strand}\t.\tID={name}"
        )
        .map_err(RsomicsError::Io)?;
        count += 1;
    }
    out.flush().map_err(RsomicsError::Io)?;
    Ok(count)
}
