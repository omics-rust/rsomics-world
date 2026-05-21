use std::fs::File;
use std::io::{BufReader, BufWriter, Write};
use std::path::Path;

use rsomics_common::{Result, RsomicsError};
use rsomics_intervals::Interval;

pub enum MaskMode {
    Soft,
    Hard,
}

fn load_bed(path: &Path) -> Result<Vec<Interval>> {
    let file = File::open(path)
        .map_err(|e| RsomicsError::InvalidInput(format!("{}: {e}", path.display())))?;
    rsomics_intervals::bed::read(BufReader::new(file))
        .map_err(|e| RsomicsError::InvalidInput(format!("reading BED: {e}")))
}

pub fn mask_fasta(
    fasta: &Path,
    bed_path: &Path,
    mode: &MaskMode,
    output: &mut dyn Write,
) -> Result<u64> {
    let intervals = load_bed(bed_path)?;
    let mut out = BufWriter::with_capacity(64 * 1024, output);
    let mut masked_bases: u64 = 0;

    let mut reader = needletail::parse_fastx_file(fasta)
        .map_err(|e| RsomicsError::InvalidInput(format!("{}: {e}", fasta.display())))?;

    while let Some(result) = reader.next() {
        let record =
            result.map_err(|e| RsomicsError::InvalidInput(format!("reading record: {e}")))?;
        let name = std::str::from_utf8(record.id())
            .map_err(|e| RsomicsError::InvalidInput(format!("non-UTF8 name: {e}")))?;
        let seq = record.seq();

        writeln!(out, ">{name}").map_err(RsomicsError::Io)?;

        let chrom_ivs: Vec<&Interval> = intervals.iter().filter(|iv| iv.chrom == name).collect();
        let mut seq_out: Vec<u8> = seq.to_vec();

        for iv in &chrom_ivs {
            let start = usize::try_from(iv.start).unwrap_or(usize::MAX);
            let end = usize::try_from(iv.end)
                .unwrap_or(usize::MAX)
                .min(seq_out.len());
            for b in &mut seq_out[start..end] {
                match mode {
                    MaskMode::Soft => *b = b.to_ascii_lowercase(),
                    MaskMode::Hard => *b = b'N',
                }
                masked_bases += 1;
            }
        }

        out.write_all(&seq_out).map_err(RsomicsError::Io)?;
        out.write_all(b"\n").map_err(RsomicsError::Io)?;
    }

    out.flush().map_err(RsomicsError::Io)?;
    Ok(masked_bases)
}
