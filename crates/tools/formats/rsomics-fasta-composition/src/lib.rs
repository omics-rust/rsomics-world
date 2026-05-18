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
    let (mut a, mut c, mut g, mut t, mut n, mut other) = (0u64, 0, 0, 0, 0, 0);
    while let Some(record) = reader.next() {
        let record = record.map_err(|e| RsomicsError::InvalidInput(format!("reading: {e}")))?;
        for &b in record.seq().iter() {
            match b.to_ascii_uppercase() {
                b'A' => a += 1,
                b'C' => c += 1,
                b'G' => g += 1,
                b'T' => t += 1,
                b'N' => n += 1,
                _ => other += 1,
            }
        }
    }
    let total = a + c + g + t + n + other;
    Ok(Composition {
        a,
        c,
        g,
        t,
        n,
        other,
        total,
    })
}
