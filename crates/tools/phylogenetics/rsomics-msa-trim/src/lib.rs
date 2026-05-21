use std::io::{BufWriter, Write};
use std::path::Path;

use rsomics_common::{Result, RsomicsError};

#[allow(clippy::cast_precision_loss)]
pub fn trim_msa(
    input: &Path,
    max_gap_fraction: f64,
    output: &mut dyn Write,
) -> Result<(usize, usize)> {
    let mut reader = needletail::parse_fastx_file(input)
        .map_err(|e| RsomicsError::InvalidInput(format!("{}: {e}", input.display())))?;

    let mut names: Vec<String> = Vec::new();
    let mut seqs: Vec<Vec<u8>> = Vec::new();

    while let Some(result) = reader.next() {
        let record =
            result.map_err(|e| RsomicsError::InvalidInput(format!("reading record: {e}")))?;
        let name = std::str::from_utf8(record.id())
            .map_err(|e| RsomicsError::InvalidInput(format!("non-UTF8 name: {e}")))?
            .to_string();
        names.push(name);
        seqs.push(record.seq().to_vec());
    }

    if seqs.is_empty() {
        return Err(RsomicsError::InvalidInput("empty alignment".into()));
    }

    let aln_len = seqs[0].len();
    let n_seqs = seqs.len();

    let mut keep = vec![false; aln_len];
    let mut kept = 0usize;

    for col in 0..aln_len {
        let gaps = seqs
            .iter()
            .filter(|s| col < s.len() && matches!(s[col], b'-' | b'.'))
            .count();
        let frac = gaps as f64 / n_seqs as f64;
        if frac <= max_gap_fraction {
            keep[col] = true;
            kept += 1;
        }
    }

    let mut out = BufWriter::new(output);
    for (i, name) in names.iter().enumerate() {
        writeln!(out, ">{name}").map_err(RsomicsError::Io)?;
        let trimmed: Vec<u8> = seqs[i]
            .iter()
            .enumerate()
            .filter(|(j, _)| *j < keep.len() && keep[*j])
            .map(|(_, &b)| b)
            .collect();
        out.write_all(&trimmed).map_err(RsomicsError::Io)?;
        writeln!(out).map_err(RsomicsError::Io)?;
    }
    out.flush().map_err(RsomicsError::Io)?;

    Ok((aln_len, kept))
}
