#![allow(clippy::cast_precision_loss)]
use needletail::parse_fastx_file;
use rsomics_common::{Result, RsomicsError};
use std::path::Path;

pub struct Composition {
    pub a: u64,
    pub c: u64,
    pub g: u64,
    pub t: u64,
    pub n: u64,
    pub other: u64,
    pub total: u64,
}

pub fn fasta_composition(input: &Path) -> Result<Composition> {
    if std::fs::metadata(input).is_ok_and(|m| m.len() == 0) {
        return Err(RsomicsError::InvalidInput("empty file".into()));
    }
    let mut reader = parse_fastx_file(input)
        .map_err(|e| RsomicsError::InvalidInput(format!("{}: {e}", input.display())))?;
    let (mut count_a, mut count_c, mut count_g, mut count_t, mut count_n, mut other) =
        (0u64, 0, 0, 0, 0, 0);
    while let Some(record) = reader.next() {
        let record = record.map_err(|e| RsomicsError::InvalidInput(format!("reading: {e}")))?;
        for &b in record.seq().iter() {
            match b.to_ascii_uppercase() {
                b'A' => count_a += 1,
                b'C' => count_c += 1,
                b'G' => count_g += 1,
                b'T' => count_t += 1,
                b'N' => count_n += 1,
                _ => other += 1,
            }
        }
    }
    let total = count_a + count_c + count_g + count_t + count_n + other;
    Ok(Composition {
        a: count_a,
        c: count_c,
        g: count_g,
        t: count_t,
        n: count_n,
        other,
        total,
    })
}
