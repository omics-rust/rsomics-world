#![allow(clippy::cast_precision_loss)]
use needletail::parse_fastx_file;
use rsomics_common::{Result, RsomicsError};
use std::io::{BufWriter, Write};
use std::path::Path;
pub fn fasta_gc_content(input: &Path, output: &mut dyn Write) -> Result<u64> {
    if std::fs::metadata(input).is_ok_and(|m| m.len() == 0) {
        return Err(RsomicsError::InvalidInput("empty file".into()));
    }
    let mut reader = parse_fastx_file(input)
        .map_err(|e| RsomicsError::InvalidInput(format!("{}: {e}", input.display())))?;
    let mut out = BufWriter::with_capacity(64 * 1024, output);
    writeln!(out, "name\tlen\tgc_pct").map_err(RsomicsError::Io)?;
    let mut count: u64 = 0;
    while let Some(record) = reader.next() {
        let record = record.map_err(|e| RsomicsError::InvalidInput(format!("reading: {e}")))?;
        let id = std::str::from_utf8(record.id()).unwrap_or("?");
        let seq = record.seq();
        let len = seq.len();
        let gc = seq
            .iter()
            .filter(|&&b| b == b'G' || b == b'g' || b == b'C' || b == b'c')
            .count();
        let pct = if len > 0 {
            gc as f64 / len as f64 * 100.0
        } else {
            0.0
        };
        writeln!(out, "{id}\t{len}\t{pct:.2}").map_err(RsomicsError::Io)?;
        count += 1;
    }
    out.flush().map_err(RsomicsError::Io)?;
    Ok(count)
}
