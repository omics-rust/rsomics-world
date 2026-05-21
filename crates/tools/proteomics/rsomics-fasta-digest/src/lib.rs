use std::io::{BufWriter, Write};
use std::path::Path;

use rsomics_common::{Result, RsomicsError};

pub enum Enzyme {
    Trypsin,
    LysC,
    Chymotrypsin,
}

impl Enzyme {
    fn cleaves_after(&self, aa: u8) -> bool {
        match self {
            Self::Trypsin => aa == b'K' || aa == b'R',
            Self::LysC => aa == b'K',
            Self::Chymotrypsin => matches!(aa, b'F' | b'W' | b'Y' | b'L'),
        }
    }
}

pub fn digest(
    input: &Path,
    enzyme: &Enzyme,
    missed_cleavages: usize,
    min_len: usize,
    max_len: usize,
    output: &mut dyn Write,
) -> Result<u64> {
    let mut reader = needletail::parse_fastx_file(input)
        .map_err(|e| RsomicsError::InvalidInput(format!("{}: {e}", input.display())))?;

    let mut out = BufWriter::with_capacity(64 * 1024, output);
    let mut peptide_count: u64 = 0;

    while let Some(result) = reader.next() {
        let record =
            result.map_err(|e| RsomicsError::InvalidInput(format!("reading record: {e}")))?;
        let name = std::str::from_utf8(record.id())
            .map_err(|e| RsomicsError::InvalidInput(format!("name: {e}")))?;
        let seq = record.seq();
        let seq_upper: Vec<u8> = seq.iter().map(u8::to_ascii_uppercase).collect();

        let mut sites: Vec<usize> = vec![0];
        for (i, &aa) in seq_upper.iter().enumerate() {
            if enzyme.cleaves_after(aa) {
                sites.push(i + 1);
            }
        }
        sites.push(seq_upper.len());

        for i in 0..sites.len() - 1 {
            for mc in 0..=missed_cleavages {
                let end_idx = i + 1 + mc;
                if end_idx >= sites.len() {
                    break;
                }
                let start = sites[i];
                let end = sites[end_idx];
                let pep = &seq_upper[start..end];
                if pep.len() >= min_len && pep.len() <= max_len {
                    writeln!(
                        out,
                        "{name}\t{start}\t{end}\t{}",
                        std::str::from_utf8(pep).unwrap_or("?")
                    )
                    .map_err(RsomicsError::Io)?;
                    peptide_count += 1;
                }
            }
        }
    }

    out.flush().map_err(RsomicsError::Io)?;
    Ok(peptide_count)
}
