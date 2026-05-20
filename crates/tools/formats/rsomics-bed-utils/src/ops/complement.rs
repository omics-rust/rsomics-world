use std::fs::File;
use std::io::{BufRead, BufReader, Write};
use std::path::Path;

use rsomics_common::{Result, RsomicsError};
use rsomics_intervals::{IntervalSet, bed, complement as iv_complement};

pub fn complement(input: &Path, genome: &Path, output: &mut dyn Write) -> Result<()> {
    let intervals = bed::read(File::open(input).map_err(RsomicsError::Io)?)?;
    let chrom_sizes = read_genome(genome)?;
    let set: IntervalSet = intervals.into_iter().collect();
    let out = iv_complement(&set, &chrom_sizes);
    bed::write_bed3(output, out.iter().cloned())
}

fn read_genome(path: &Path) -> Result<Vec<(String, u64)>> {
    let f = File::open(path).map_err(RsomicsError::Io)?;
    let mut out = Vec::new();
    for (lineno, line) in BufReader::new(f).lines().enumerate() {
        let line = line.map_err(RsomicsError::Io)?;
        let trimmed = line.trim_end();
        if trimmed.is_empty() || trimmed.starts_with('#') {
            continue;
        }
        let mut fields = trimmed.split('\t');
        let chrom = fields.next().ok_or_else(|| {
            RsomicsError::InvalidInput(format!("genome line {}: missing chrom", lineno + 1))
        })?;
        let size_s = fields.next().ok_or_else(|| {
            RsomicsError::InvalidInput(format!("genome line {}: missing size", lineno + 1))
        })?;
        let size: u64 = size_s.parse().map_err(|_| {
            RsomicsError::InvalidInput(format!("genome line {}: bad size {size_s:?}", lineno + 1))
        })?;
        out.push((chrom.to_string(), size));
    }
    Ok(out)
}
