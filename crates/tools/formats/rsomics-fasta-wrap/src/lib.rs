use needletail::parse_fastx_file;
use rsomics_common::{Result, RsomicsError};
use std::io::{BufWriter, Write};
use std::path::Path;
pub fn fasta_wrap(input: &Path, output: &mut dyn Write, width: usize) -> Result<u64> {
    if std::fs::metadata(input).is_ok_and(|m| m.len() == 0) {
        return Err(RsomicsError::InvalidInput("empty file".into()));
    }
    let mut reader = parse_fastx_file(input)
        .map_err(|e| RsomicsError::InvalidInput(format!("{}: {e}", input.display())))?;
    let mut out = BufWriter::with_capacity(256 * 1024, output);
    let mut count: u64 = 0;
    while let Some(record) = reader.next() {
        let record = record.map_err(|e| RsomicsError::InvalidInput(format!("reading: {e}")))?;
        let id = std::str::from_utf8(record.id()).unwrap_or("?");
        writeln!(out, ">{id}").map_err(RsomicsError::Io)?;
        let seq = record.seq();
        for chunk in seq.chunks(width) {
            out.write_all(chunk).map_err(RsomicsError::Io)?;
            out.write_all(b"\n").map_err(RsomicsError::Io)?;
        }
        count += 1;
    }
    out.flush().map_err(RsomicsError::Io)?;
    Ok(count)
}
