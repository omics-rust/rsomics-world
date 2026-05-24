//! Minimal FASTA parser and writer.

use std::io::{BufRead, Write};

/// Line-wrap width for FASTA output.  0 means no wrapping.
#[derive(Clone, Copy, Debug)]
pub struct FastaWidth(pub usize);

impl FastaWidth {
    pub const DEFAULT: FastaWidth = FastaWidth(80);
}

/// Parse FASTA from a buffered reader.
///
/// Yields `Ok((raw_label, raw_seq_bytes))` for each record.  Multi-line
/// sequences are concatenated.
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

/// Write a record with the original header unchanged (no `--sizeout`).
pub fn write_fasta_raw(
    out: &mut dyn Write,
    header: &str,
    seq: &[u8],
    width: FastaWidth,
) -> std::io::Result<()> {
    writeln!(out, ">{header}")?;
    write_seq(out, seq, width)
}

/// Write a record with `;size=N` stripped from header and reappended (`--sizeout`).
pub fn write_fasta_with_size(
    out: &mut dyn Write,
    stripped_header: &str,
    abundance: u64,
    seq: &[u8],
    width: FastaWidth,
) -> std::io::Result<()> {
    writeln!(out, ">{stripped_header};size={abundance}")?;
    write_seq(out, seq, width)
}

fn write_seq(out: &mut dyn Write, seq: &[u8], width: FastaWidth) -> std::io::Result<()> {
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
