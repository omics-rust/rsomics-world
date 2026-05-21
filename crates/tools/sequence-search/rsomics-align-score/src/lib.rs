use rsomics_align_core::{Op, ScoreParams, needleman_wunsch, smith_waterman};
use rsomics_common::{Result, RsomicsError};
use std::io::Write;
use std::path::Path;

pub enum AlignMode {
    Global,
    Local,
}

#[allow(clippy::cast_precision_loss)]
fn identity(ops: &[Op]) -> f64 {
    if ops.is_empty() {
        return 0.0;
    }
    let matches = ops.iter().filter(|o| matches!(o, Op::Match)).count();
    matches as f64 / ops.len() as f64
}

pub fn align_pairs(
    fasta: &Path,
    mode: &AlignMode,
    params: &ScoreParams,
    output: &mut dyn Write,
) -> Result<u64> {
    let mut reader = needletail::parse_fastx_file(fasta)
        .map_err(|e| RsomicsError::InvalidInput(format!("{}: {e}", fasta.display())))?;

    let mut seqs: Vec<(String, Vec<u8>)> = Vec::new();
    while let Some(result) = reader.next() {
        let record = result.map_err(|e| RsomicsError::InvalidInput(format!("read: {e}")))?;
        let name = std::str::from_utf8(record.id())
            .map_err(|e| RsomicsError::InvalidInput(format!("name: {e}")))?
            .to_string();
        seqs.push((name, record.seq().to_vec()));
    }

    let mut count = 0u64;
    writeln!(output, "seq1\tseq2\tscore\tidentity").map_err(RsomicsError::Io)?;

    for i in 0..seqs.len() {
        for j in (i + 1)..seqs.len() {
            let aln = match mode {
                AlignMode::Global => needleman_wunsch(&seqs[i].1, &seqs[j].1, params),
                AlignMode::Local => smith_waterman(&seqs[i].1, &seqs[j].1, params),
            }
            .map_err(|e| RsomicsError::InvalidInput(format!("align: {e}")))?;

            writeln!(
                output,
                "{}\t{}\t{}\t{:.4}",
                seqs[i].0,
                seqs[j].0,
                aln.score,
                identity(&aln.ops)
            )
            .map_err(RsomicsError::Io)?;
            count += 1;
        }
    }
    Ok(count)
}
