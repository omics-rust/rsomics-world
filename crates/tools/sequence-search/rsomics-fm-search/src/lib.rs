use std::io::{BufWriter, Write};
use std::path::Path;

use rsomics_common::{Result, RsomicsError};
use rsomics_fm_index::FmIndex;

pub fn search_fasta(
    fasta: &Path,
    pattern: &[u8],
    count_only: bool,
    output: &mut dyn Write,
) -> Result<u64> {
    let mut reader = needletail::parse_fastx_file(fasta)
        .map_err(|e| RsomicsError::InvalidInput(format!("{}: {e}", fasta.display())))?;

    let mut out = BufWriter::new(output);
    let mut total_hits = 0u64;

    while let Some(result) = reader.next() {
        let record =
            result.map_err(|e| RsomicsError::InvalidInput(format!("reading record: {e}")))?;
        let name = std::str::from_utf8(record.id())
            .map_err(|e| RsomicsError::InvalidInput(format!("name: {e}")))?;
        let seq = record.seq();
        let upper: Vec<u8> = seq.iter().map(u8::to_ascii_uppercase).collect();

        let index = FmIndex::build(&upper)
            .map_err(|e| RsomicsError::InvalidInput(format!("index build: {e}")))?;

        if count_only {
            let n = index.count(pattern);
            if n > 0 {
                writeln!(out, "{name}\t{n}").map_err(RsomicsError::Io)?;
                total_hits += n as u64;
            }
        } else {
            let positions = index.locate(pattern);
            for pos in &positions {
                writeln!(out, "{name}\t{pos}\t{}", pos + pattern.len())
                    .map_err(RsomicsError::Io)?;
            }
            total_hits += positions.len() as u64;
        }
    }

    out.flush().map_err(RsomicsError::Io)?;
    Ok(total_hits)
}
