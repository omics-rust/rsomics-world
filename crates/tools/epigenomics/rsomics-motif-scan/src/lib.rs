use regex::bytes::Regex;
use rsomics_common::{Result, RsomicsError};
use std::io::{BufWriter, Write};
use std::path::Path;

#[must_use]
pub fn iupac_to_regex(motif: &str) -> String {
    let mut re = String::new();
    for c in motif.chars() {
        let expanded = match c.to_ascii_uppercase() {
            'A' => "[Aa]",
            'C' => "[Cc]",
            'G' => "[Gg]",
            'T' => "[Tt]",
            'R' => "[AaGg]",
            'Y' => "[CcTt]",
            'S' => "[GgCc]",
            'W' => "[AaTt]",
            'K' => "[GgTt]",
            'M' => "[AaCc]",
            'B' => "[CcGgTt]",
            'D' => "[AaGgTt]",
            'H' => "[AaCcTt]",
            'V' => "[AaCcGg]",
            'N' => "[AaCcGgTt]",
            other => {
                re.push(other);
                continue;
            }
        };
        re.push_str(expanded);
    }
    re
}

pub fn scan_motif(fasta: &Path, motif: &str, output: &mut dyn Write) -> Result<u64> {
    let pattern = iupac_to_regex(motif);
    let re = Regex::new(&pattern)
        .map_err(|e| RsomicsError::InvalidInput(format!("bad motif regex: {e}")))?;

    let mut reader = needletail::parse_fastx_file(fasta)
        .map_err(|e| RsomicsError::InvalidInput(format!("{}: {e}", fasta.display())))?;

    let mut out = BufWriter::new(output);
    let mut count = 0u64;

    while let Some(result) = reader.next() {
        let record = result.map_err(|e| RsomicsError::InvalidInput(format!("read: {e}")))?;
        let name = std::str::from_utf8(record.id())
            .map_err(|e| RsomicsError::InvalidInput(format!("name: {e}")))?;
        let seq = record.seq();

        for m in re.find_iter(&seq) {
            writeln!(out, "{name}\t{}\t{}\t{motif}", m.start(), m.end())
                .map_err(RsomicsError::Io)?;
            count += 1;
        }
    }

    out.flush().map_err(RsomicsError::Io)?;
    Ok(count)
}
