use std::io::{BufWriter, Write};
use std::path::Path;

use needletail::parse_fastx_file;
use rsomics_common::{Result, RsomicsError};

pub fn shuffle_fasta(input: &Path, output: &mut dyn Write, seed: u64) -> Result<u64> {
    if std::fs::metadata(input).is_ok_and(|m| m.len() == 0) {
        return Err(RsomicsError::InvalidInput("empty file".into()));
    }

    let mut reader = parse_fastx_file(input)
        .map_err(|e| RsomicsError::InvalidInput(format!("{}: {e}", input.display())))?;

    let mut records: Vec<(Vec<u8>, Vec<u8>)> = Vec::new();
    while let Some(record) = reader.next() {
        let record = record.map_err(|e| RsomicsError::InvalidInput(format!("reading: {e}")))?;
        records.push((record.id().to_vec(), record.seq().to_vec()));
    }

    let mut rng = SimpleRng(seed);
    for i in (1..records.len()).rev() {
        let j = rng.next_usize(i + 1);
        records.swap(i, j);
    }

    let mut out = BufWriter::with_capacity(256 * 1024, output);
    for (id, seq) in &records {
        out.write_all(b">").map_err(RsomicsError::Io)?;
        out.write_all(id).map_err(RsomicsError::Io)?;
        out.write_all(b"\n").map_err(RsomicsError::Io)?;
        out.write_all(seq).map_err(RsomicsError::Io)?;
        out.write_all(b"\n").map_err(RsomicsError::Io)?;
    }
    out.flush().map_err(RsomicsError::Io)?;

    Ok(records.len() as u64)
}

struct SimpleRng(u64);

impl SimpleRng {
    fn next_u64(&mut self) -> u64 {
        self.0 ^= self.0 << 13;
        self.0 ^= self.0 >> 7;
        self.0 ^= self.0 << 17;
        self.0
    }

    fn next_usize(&mut self, bound: usize) -> usize {
        #[allow(clippy::cast_possible_truncation)]
        let idx = (self.next_u64() % (bound as u64)) as usize;
        idx
    }
}
