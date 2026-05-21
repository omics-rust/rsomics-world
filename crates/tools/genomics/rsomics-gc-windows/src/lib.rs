use std::io::{BufWriter, Write};
use std::path::Path;

use rsomics_common::{Result, RsomicsError};

pub fn gc_windows(fasta: &Path, window: usize, step: usize, output: &mut dyn Write) -> Result<u64> {
    let mut out = BufWriter::with_capacity(64 * 1024, output);
    let mut count: u64 = 0;
    let step = if step == 0 { window } else { step };

    let mut reader = needletail::parse_fastx_file(fasta)
        .map_err(|e| RsomicsError::InvalidInput(format!("{}: {e}", fasta.display())))?;

    while let Some(result) = reader.next() {
        let record =
            result.map_err(|e| RsomicsError::InvalidInput(format!("reading record: {e}")))?;
        let name = std::str::from_utf8(record.id())
            .map_err(|e| RsomicsError::InvalidInput(format!("non-UTF8 name: {e}")))?;
        let seq = record.seq();
        let seq_len = seq.len();

        let mut start = 0;
        while start + window <= seq_len {
            let w = &seq[start..start + window];
            let gc = w
                .iter()
                .filter(|&&b| matches!(b, b'G' | b'g' | b'C' | b'c'))
                .count();
            let n = w.iter().filter(|&&b| matches!(b, b'N' | b'n')).count();
            let effective = window - n;
            #[allow(clippy::cast_precision_loss)]
            let pct = if effective > 0 {
                gc as f64 / effective as f64
            } else {
                0.0
            };
            writeln!(out, "{name}\t{start}\t{}\t{pct:.4}", start + window)
                .map_err(RsomicsError::Io)?;
            count += 1;
            start += step;
        }
    }

    out.flush().map_err(RsomicsError::Io)?;
    Ok(count)
}
