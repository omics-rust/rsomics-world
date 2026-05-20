#![allow(clippy::cast_precision_loss)]

use std::io::{BufWriter, Write};
use std::path::Path;

use needletail::parse_fastx_file;
use rsomics_common::{Result, RsomicsError};

pub fn window_stats(
    input: &Path,
    output: &mut dyn Write,
    window_size: usize,
    step: usize,
) -> Result<u64> {
    if std::fs::metadata(input).is_ok_and(|m| m.len() == 0) {
        return Err(RsomicsError::InvalidInput("empty file".into()));
    }

    let mut reader = parse_fastx_file(input)
        .map_err(|e| RsomicsError::InvalidInput(format!("{}: {e}", input.display())))?;

    let mut out = BufWriter::with_capacity(256 * 1024, output);
    writeln!(out, "chrom\tstart\tend\tgc_pct\tlen").map_err(RsomicsError::Io)?;

    let mut count: u64 = 0;

    while let Some(record) = reader.next() {
        let record = record.map_err(|e| RsomicsError::InvalidInput(format!("reading: {e}")))?;
        let id = std::str::from_utf8(record.id()).unwrap_or("unknown");
        let seq = record.seq();
        let seq_len = seq.len();

        let mut pos = 0;
        while pos + window_size <= seq_len {
            let window = &seq[pos..pos + window_size];
            let gc = window
                .iter()
                .filter(|&&b| b == b'G' || b == b'g' || b == b'C' || b == b'c')
                .count();
            let gc_pct = gc as f64 / window_size as f64 * 100.0;

            writeln!(
                out,
                "{id}\t{pos}\t{}\t{gc_pct:.2}\t{window_size}",
                pos + window_size
            )
            .map_err(RsomicsError::Io)?;
            count += 1;
            pos += step;
        }
    }

    out.flush().map_err(RsomicsError::Io)?;
    Ok(count)
}
