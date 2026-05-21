use std::io::Write;
use std::path::Path;

use rsomics_common::{Result, RsomicsError};

#[allow(clippy::cast_precision_loss)]
pub fn consensus(input: &Path, threshold: f64, output: &mut dyn Write) -> Result<usize> {
    let mut reader = needletail::parse_fastx_file(input)
        .map_err(|e| RsomicsError::InvalidInput(format!("{}: {e}", input.display())))?;

    let mut seqs: Vec<Vec<u8>> = Vec::new();
    while let Some(result) = reader.next() {
        let record =
            result.map_err(|e| RsomicsError::InvalidInput(format!("reading record: {e}")))?;
        seqs.push(record.seq().to_vec());
    }

    if seqs.is_empty() {
        return Err(RsomicsError::InvalidInput("empty alignment".into()));
    }

    let aln_len = seqs[0].len();
    let n = seqs.len();
    let mut cons = Vec::with_capacity(aln_len);

    for col in 0..aln_len {
        let mut counts = [0u32; 256];
        for seq in &seqs {
            if col < seq.len() {
                let b = seq[col].to_ascii_uppercase();
                counts[b as usize] += 1;
            }
        }

        let best = counts
            .iter()
            .enumerate()
            .filter(|(i, _)| *i != b'-' as usize && *i != b'.' as usize)
            .max_by_key(|(_, c)| *c)
            .map(|(i, c)| (i as u8, *c))
            .unwrap_or((b'N', 0));

        if best.1 as f64 / n as f64 >= threshold {
            cons.push(best.0);
        } else {
            cons.push(b'N');
        }
    }

    writeln!(output, ">consensus").map_err(RsomicsError::Io)?;
    output.write_all(&cons).map_err(RsomicsError::Io)?;
    writeln!(output).map_err(RsomicsError::Io)?;

    Ok(cons.len())
}
