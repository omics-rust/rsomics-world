use rsomics_common::{Result, RsomicsError};
use std::fs::File;
use std::io::{BufRead, BufReader, BufWriter, Write};
use std::path::Path;
pub fn bed_promoters(
    input: &Path,
    output: &mut dyn Write,
    upstream: u64,
    downstream: u64,
) -> Result<u64> {
    let file = File::open(input)
        .map_err(|e| RsomicsError::InvalidInput(format!("{}: {e}", input.display())))?;
    let reader = BufReader::new(file);
    let mut out = BufWriter::with_capacity(64 * 1024, output);
    let mut count: u64 = 0;
    for line in reader.lines() {
        let line = line.map_err(RsomicsError::Io)?;
        if line.starts_with('#') || line.is_empty() {
            continue;
        }
        let fields: Vec<&str> = line.split('\t').collect();
        if fields.len() < 3 {
            continue;
        }
        let chrom = fields[0];
        let start: u64 = fields[1].parse().unwrap_or(0);
        let end: u64 = fields[2].parse().unwrap_or(0);
        let strand = fields.get(5).unwrap_or(&"+");
        let (prom_start, prom_end) = if *strand == "-" {
            (end, end + upstream + downstream)
        } else {
            (start.saturating_sub(upstream), start + downstream)
        };
        write!(out, "{chrom}\t{prom_start}\t{prom_end}").map_err(RsomicsError::Io)?;
        for f in &fields[3..] {
            write!(out, "\t{f}").map_err(RsomicsError::Io)?;
        }
        writeln!(out).map_err(RsomicsError::Io)?;
        count += 1;
    }
    out.flush().map_err(RsomicsError::Io)?;
    Ok(count)
}
