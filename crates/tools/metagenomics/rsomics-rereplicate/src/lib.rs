//! Rereplicate: expand `;size=N` abundance annotations back into N individual
//! FASTA records.  Inverse of `vsearch --derep_fulllength`.
//!
//! Behaviour matches `vsearch --rereplicate` v2.31.0 (BSD-2):
//!
//! - Each input record with `;size=N` is emitted N times in order.
//! - The `;size=N` annotation is **stripped** from the output header (vsearch
//!   sets `opt_xsize=true` unconditionally for this command).
//! - With `--sizeout`, each copy receives `;size=1` appended instead.
//! - Records with no `;size=` annotation are treated as abundance 1 (emitted
//!   once), with a warning logged.
//! - No `minseqlength` or `maxseqlength` filtering is applied (those options
//!   are not in the `--rereplicate` allowed-option set).
//! - Sequence bytes are preserved exactly (case + U kept); the tool does not
//!   normalise for this operation.
//! - FASTA output wraps at `fasta_width` columns (default 80).
//! - Output order mirrors input order; each record's copies appear together.

use std::io::{BufRead, Write};

/// One parsed FASTA record.
#[derive(Debug)]
pub struct FastaRecord {
    /// Label with the first `;size=N` token stripped.
    pub label: String,
    /// Abundance parsed from `;size=N`; `None` if annotation was absent.
    pub abundance: Option<u64>,
    /// Raw sequence bytes as they appeared in the file.
    pub seq: Vec<u8>,
}

/// Parse FASTA records from a buffered reader.
///
/// Each record yields `(raw_label, raw_seq_bytes)`.  Multi-line sequences are
/// concatenated.  Empty lines between records are skipped.
pub fn parse_fasta(reader: &mut dyn BufRead) -> anyhow::Result<Vec<(String, Vec<u8>)>> {
    let mut records: Vec<(String, Vec<u8>)> = Vec::new();
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
                records.push((label, current_seq.clone()));
                current_seq.clear();
            }
            current_label = Some(rest.to_owned());
        } else if current_label.is_some() {
            current_seq.extend_from_slice(trimmed.as_bytes());
        }
    }
    if let Some(label) = current_label.take() {
        records.push((label, current_seq));
    }

    Ok(records)
}

/// Strip the first `;size=N` token from a label.
///
/// Returns `(stripped_label, Some(N))` when found, `(original, None)` otherwise.
#[must_use]
pub fn strip_size(label: &str) -> (String, Option<u64>) {
    let parts: Vec<&str> = label.split(';').collect();
    let mut size_value: Option<u64> = None;
    let mut kept: Vec<&str> = Vec::with_capacity(parts.len());

    for part in &parts {
        if size_value.is_none()
            && part.starts_with("size=")
            && let Ok(n) = part["size=".len()..].parse::<u64>()
        {
            size_value = Some(n);
            continue;
        }
        kept.push(part);
    }

    (kept.join(";"), size_value)
}

/// Write one FASTA record to `out`, wrapping sequence at `width` columns.
/// `width == 0` means no wrapping.
pub fn write_record(
    out: &mut dyn Write,
    label: &str,
    sizeout: bool,
    seq: &[u8],
    width: usize,
) -> std::io::Result<()> {
    if sizeout {
        writeln!(out, ">{label};size=1")?;
    } else {
        writeln!(out, ">{label}")?;
    }
    if width == 0 {
        out.write_all(seq)?;
        writeln!(out)?;
    } else {
        for chunk in seq.chunks(width) {
            out.write_all(chunk)?;
            writeln!(out)?;
        }
    }
    Ok(())
}

/// Core rereplicate logic.
///
/// Reads FASTA from `reader` and writes each record expanded `abundance` times
/// to `writer`.  Returns `(amplicons, reads, missing_abundance_count)`.
///
/// `sizeout` appends `;size=1` to each emitted copy.
/// `fasta_width` sets the sequence line wrap column (0 = no wrap).
///
/// Records without a `;size=N` annotation are treated as abundance 1 and
/// counted in `missing_abundance_count`.
pub fn rereplicate(
    reader: &mut dyn BufRead,
    writer: &mut dyn Write,
    sizeout: bool,
    fasta_width: usize,
) -> anyhow::Result<(u64, u64, u64)> {
    let records = parse_fasta(reader)?;
    let mut amplicons: u64 = 0;
    let mut reads: u64 = 0;
    let mut missing: u64 = 0;

    for (raw_label, seq) in records {
        amplicons += 1;
        let (label, size_opt) = strip_size(&raw_label);
        let abundance = if let Some(n) = size_opt {
            n
        } else {
            missing += 1;
            1
        };

        for _ in 0..abundance {
            reads += 1;
            write_record(writer, &label, sizeout, &seq, fasta_width)?;
        }
    }

    Ok((amplicons, reads, missing))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn strip_size_trailing() {
        let (s, v) = strip_size("seq1;size=5");
        assert_eq!(s, "seq1");
        assert_eq!(v, Some(5));
    }

    #[test]
    fn strip_size_middle() {
        let (s, v) = strip_size("seq1;k=v;size=3;extra=x");
        assert_eq!(s, "seq1;k=v;extra=x");
        assert_eq!(v, Some(3));
    }

    #[test]
    fn strip_size_absent() {
        let (s, v) = strip_size("seq1;k=v");
        assert_eq!(s, "seq1;k=v");
        assert_eq!(v, None);
    }

    #[test]
    fn strip_size_non_numeric_kept() {
        let (s, v) = strip_size("seq1;size=abc");
        assert_eq!(s, "seq1;size=abc");
        assert_eq!(v, None);
    }

    #[test]
    fn rereplicate_basic() {
        use std::io::BufReader;
        let input = b">seq1;size=3\nACGT\n>seq2;size=1\nTTTT\n";
        let mut reader = BufReader::new(input.as_ref());
        let mut out: Vec<u8> = Vec::new();
        let (amplicons, reads, missing) = rereplicate(&mut reader, &mut out, false, 80).unwrap();
        assert_eq!(amplicons, 2);
        assert_eq!(reads, 4);
        assert_eq!(missing, 0);
        let s = String::from_utf8(out).unwrap();
        assert_eq!(s, ">seq1\nACGT\n>seq1\nACGT\n>seq1\nACGT\n>seq2\nTTTT\n");
    }

    #[test]
    fn rereplicate_sizeout() {
        use std::io::BufReader;
        let input = b">seq1;size=2\nACGT\n";
        let mut reader = BufReader::new(input.as_ref());
        let mut out: Vec<u8> = Vec::new();
        rereplicate(&mut reader, &mut out, true, 80).unwrap();
        let s = String::from_utf8(out).unwrap();
        assert_eq!(s, ">seq1;size=1\nACGT\n>seq1;size=1\nACGT\n");
    }

    #[test]
    fn rereplicate_missing_abundance_treated_as_one() {
        use std::io::BufReader;
        let input = b">seq1\nACGT\n";
        let mut reader = BufReader::new(input.as_ref());
        let mut out: Vec<u8> = Vec::new();
        let (amplicons, reads, missing) = rereplicate(&mut reader, &mut out, false, 80).unwrap();
        assert_eq!(amplicons, 1);
        assert_eq!(reads, 1);
        assert_eq!(missing, 1);
        let s = String::from_utf8(out).unwrap();
        assert_eq!(s, ">seq1\nACGT\n");
    }

    #[test]
    fn rereplicate_fasta_width() {
        use std::io::BufReader;
        let seq = "ACGTACGTAC"; // 10 bases
        let input = format!(">s;size=1\n{seq}\n");
        let mut reader = BufReader::new(input.as_bytes());
        let mut out: Vec<u8> = Vec::new();
        rereplicate(&mut reader, &mut out, false, 4).unwrap();
        let s = String::from_utf8(out).unwrap();
        // Should wrap at 4: ACGT\nACGT\nAC\n
        assert_eq!(s, ">s\nACGT\nACGT\nAC\n");
    }
}
