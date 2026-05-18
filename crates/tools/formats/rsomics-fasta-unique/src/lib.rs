use std::collections::HashSet;
use std::io::{BufWriter, Write};
use std::path::Path;

use needletail::parse_fastx_file;
use rsomics_common::{Result, RsomicsError};

pub fn unique_fasta(input: &Path, output: &mut dyn Write) -> Result<(u64, u64)> {
    if std::fs::metadata(input).is_ok_and(|m| m.len() == 0) {
        return Err(RsomicsError::InvalidInput("empty file".into()));
    }

    let mut reader = parse_fastx_file(input)
        .map_err(|e| RsomicsError::InvalidInput(format!("{}: {e}", input.display())))?;

    let mut out = BufWriter::with_capacity(256 * 1024, output);
    let mut seen: HashSet<u64> = HashSet::new();
    let mut total: u64 = 0;
    let mut kept: u64 = 0;

    while let Some(record) = reader.next() {
        let record = record.map_err(|e| RsomicsError::InvalidInput(format!("reading: {e}")))?;
        total += 1;

        let hash = simple_hash(&record.seq());
        if seen.contains(&hash) {
            continue;
        }
        seen.insert(hash);

        out.write_all(b">").map_err(RsomicsError::Io)?;
        out.write_all(record.id()).map_err(RsomicsError::Io)?;
        out.write_all(b"\n").map_err(RsomicsError::Io)?;
        out.write_all(&record.seq()).map_err(RsomicsError::Io)?;
        out.write_all(b"\n").map_err(RsomicsError::Io)?;
        kept += 1;
    }

    out.flush().map_err(RsomicsError::Io)?;
    Ok((total, kept))
}

fn simple_hash(data: &[u8]) -> u64 {
    let mut h: u64 = 0xcbf2_9ce4_8422_2325;
    for &b in data {
        h ^= u64::from(b);
        h = h.wrapping_mul(0x0100_0000_01b3);
    }
    h
}
