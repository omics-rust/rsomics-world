use std::fs::File;
use std::io::{BufRead, BufReader, BufWriter, Write};
use std::path::Path;

use rsomics_common::{Result, RsomicsError};

pub fn fastq_to_tab(input: &Path, output: &mut dyn Write) -> Result<u64> {
    let file = File::open(input)
        .map_err(|e| RsomicsError::InvalidInput(format!("{}: {e}", input.display())))?;
    let reader = BufReader::new(file);
    let mut out = BufWriter::with_capacity(256 * 1024, output);
    let mut lines = reader.lines();
    let mut count: u64 = 0;

    while let Some(header) = lines.next() {
        let header = header.map_err(RsomicsError::Io)?;
        let seq = next_line(&mut lines)?;
        let _plus = next_line(&mut lines)?;
        let qual = next_line(&mut lines)?;

        let name = header.trim_start_matches('@');
        writeln!(out, "{name}\t{seq}\t{qual}").map_err(RsomicsError::Io)?;
        count += 1;
    }

    out.flush().map_err(RsomicsError::Io)?;
    Ok(count)
}

fn next_line(lines: &mut std::io::Lines<BufReader<File>>) -> Result<String> {
    lines
        .next()
        .ok_or_else(|| RsomicsError::InvalidInput("truncated FASTQ".into()))?
        .map_err(RsomicsError::Io)
}
