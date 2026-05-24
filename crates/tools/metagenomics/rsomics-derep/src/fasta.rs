//! Minimal FASTA parser and writer for the derep use-case.
//!
//! The parser yields `(label, sequence_bytes)` pairs where the sequence is
//! the raw bytes from the file (not yet normalised).  Multi-line sequences
//! are concatenated.
//!
//! The writer wraps sequence lines at `fasta_width` characters (vsearch
//! default 80).

use std::io::{BufRead, Write};

/// Line-wrap width for FASTA output.  0 means no wrapping.
#[derive(Clone, Copy, Debug)]
pub struct FastaWidth(pub usize);

impl FastaWidth {
    pub const DEFAULT: FastaWidth = FastaWidth(80);
}

/// Parse FASTA from a buffered reader.
///
/// Yields `Ok((label, raw_seq_bytes))` for each record.
pub fn parse_fasta(
    reader: &mut dyn BufRead,
) -> anyhow::Result<impl Iterator<Item = anyhow::Result<(String, Vec<u8>)>>> {
    let mut records: Vec<anyhow::Result<(String, Vec<u8>)>> = Vec::new();
    let mut current_label: Option<String> = None;
    let mut current_seq: Vec<u8> = Vec::new();
    let mut line = String::new();

    loop {
        line.clear();
        let n = reader.read_line(&mut line)?;
        if n == 0 {
            break;
        }
        let trimmed = line.trim_end_matches(['\n', '\r']);
        if trimmed.is_empty() {
            continue;
        }
        if let Some(rest) = trimmed.strip_prefix('>') {
            if let Some(label) = current_label.take() {
                records.push(Ok((label, current_seq.clone())));
                current_seq.clear();
            }
            current_label = Some(rest.to_owned());
        } else if current_label.is_some() {
            current_seq.extend_from_slice(trimmed.as_bytes());
        }
    }
    if let Some(label) = current_label.take() {
        records.push(Ok((label, current_seq)));
    }

    Ok(records.into_iter())
}

/// Write a dereplicated record as FASTA with `;size=N` appended to the label.
pub fn write_fasta(
    out: &mut dyn Write,
    label: &str,
    abundance: u64,
    seq: &[u8],
    width: FastaWidth,
) -> std::io::Result<()> {
    writeln!(out, ">{label};size={abundance}")?;
    let w = width.0;
    if w == 0 {
        out.write_all(seq)?;
        writeln!(out)?;
    } else {
        for chunk in seq.chunks(w) {
            out.write_all(chunk)?;
            writeln!(out)?;
        }
    }
    Ok(())
}
