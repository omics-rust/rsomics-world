use std::collections::HashSet;
use std::io::{BufWriter, Write};
use std::path::Path;

use needletail::parse_fastx_file;
use rsomics_common::{Result, RsomicsError};

pub fn dedup_fasta(input: &Path, output: &mut dyn Write, by_name: bool) -> Result<(u64, u64)> {
    if std::fs::metadata(input).is_ok_and(|m| m.len() == 0) {
        return Err(RsomicsError::InvalidInput("empty file".into()));
    }

    let mut reader = parse_fastx_file(input)
        .map_err(|e| RsomicsError::InvalidInput(format!("{}: {e}", input.display())))?;

    let mut out = BufWriter::with_capacity(256 * 1024, output);
    let mut seen: HashSet<Vec<u8>> = HashSet::new();
    let mut total: u64 = 0;
    let mut kept: u64 = 0;

    while let Some(record) = reader.next() {
        let record = record.map_err(|e| RsomicsError::InvalidInput(format!("reading: {e}")))?;
        total += 1;

        let key = if by_name {
            record.id().to_vec()
        } else {
            record.seq().to_ascii_uppercase()
        };

        if seen.contains(&key) {
            continue;
        }
        seen.insert(key);

        let id = std::str::from_utf8(record.id()).unwrap_or("?");
        writeln!(out, ">{id}").map_err(RsomicsError::Io)?;
        out.write_all(&record.seq()).map_err(RsomicsError::Io)?;
        writeln!(out).map_err(RsomicsError::Io)?;
        kept += 1;
    }

    out.flush().map_err(RsomicsError::Io)?;
    Ok((total, kept))
}
