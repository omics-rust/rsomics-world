use std::collections::HashMap;
use std::io::{BufWriter, Write};
use std::path::Path;

use needletail::parse_fastx_file;
use rsomics_common::{Result, RsomicsError};

pub fn count_kmers(input: &Path, output: &mut dyn Write, k: usize) -> Result<u64> {
    if std::fs::metadata(input).is_ok_and(|m| m.len() == 0) {
        return Err(RsomicsError::InvalidInput("empty file".into()));
    }

    let mut reader = parse_fastx_file(input)
        .map_err(|e| RsomicsError::InvalidInput(format!("{}: {e}", input.display())))?;

    let mut counts: HashMap<Vec<u8>, u64> = HashMap::new();

    while let Some(record) = reader.next() {
        let record = record.map_err(|e| RsomicsError::InvalidInput(format!("reading: {e}")))?;
        let seq = record.seq();
        if seq.len() < k {
            continue;
        }
        for window in seq.windows(k) {
            if window
                .iter()
                .all(|b| matches!(b, b'A' | b'C' | b'G' | b'T' | b'a' | b'c' | b'g' | b't'))
            {
                let upper: Vec<u8> = window.iter().map(u8::to_ascii_uppercase).collect();
                *counts.entry(upper).or_insert(0) += 1;
            }
        }
    }

    let mut sorted: Vec<_> = counts.into_iter().collect();
    sorted.sort_by(|a, b| b.1.cmp(&a.1).then(a.0.cmp(&b.0)));

    let mut out = BufWriter::with_capacity(64 * 1024, output);
    let mut total: u64 = 0;
    for (kmer, count) in &sorted {
        let s = std::str::from_utf8(kmer).unwrap_or("?");
        writeln!(out, "{s}\t{count}").map_err(RsomicsError::Io)?;
        total += count;
    }
    out.flush().map_err(RsomicsError::Io)?;

    Ok(total)
}
