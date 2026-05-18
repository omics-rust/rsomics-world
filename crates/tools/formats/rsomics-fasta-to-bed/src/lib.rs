use needletail::parse_fastx_file;
use rsomics_common::{Result, RsomicsError};
use std::io::{BufWriter, Write};
use std::path::Path;

pub fn fasta_to_bed(input: &Path, output: &mut dyn Write) -> Result<u64> {
    if std::fs::metadata(input).is_ok_and(|m| m.len() == 0) {
        return Err(RsomicsError::InvalidInput("empty file".into()));
    }
    let mut reader = parse_fastx_file(input)
        .map_err(|e| RsomicsError::InvalidInput(format!("{}: {e}", input.display())))?;
    let mut out = BufWriter::with_capacity(64 * 1024, output);
    let mut count: u64 = 0;
    while let Some(record) = reader.next() {
        let record = record.map_err(|e| RsomicsError::InvalidInput(format!("reading: {e}")))?;
        let id = std::str::from_utf8(record.id()).unwrap_or("?");
        writeln!(out, "{id}\t0\t{}", record.seq().len()).map_err(RsomicsError::Io)?;
        count += 1;
    }
    out.flush().map_err(RsomicsError::Io)?;
    Ok(count)
}
