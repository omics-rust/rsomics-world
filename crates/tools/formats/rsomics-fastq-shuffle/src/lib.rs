use std::fs::File;
use std::io::{BufRead, BufReader, BufWriter, Write};
use std::path::Path;

use rsomics_common::{Result, RsomicsError};

pub fn shuffle_fasta(input: &Path, output: &mut dyn Write, seed: u64) -> Result<u64> {
    let file = File::open(input)
        .map_err(|e| RsomicsError::InvalidInput(format!("{}: {e}", input.display())))?;
    let reader = BufReader::new(file);
    let mut lines = reader.lines();

    let mut records: Vec<[String; 4]> = Vec::new();
    while let Some(h) = lines.next() {
        let h = h.map_err(RsomicsError::Io)?;
        if h.is_empty() {
            continue;
        }
        let s = next_line(&mut lines)?;
        let p = next_line(&mut lines)?;
        let q = next_line(&mut lines)?;
        records.push([h, s, p, q]);
    }

    let mut rng = SimpleRng(seed);
    for i in (1..records.len()).rev() {
        let j = rng.next_usize(i + 1);
        records.swap(i, j);
    }

    let mut out = BufWriter::with_capacity(256 * 1024, output);
    for rec in &records {
        for line in rec {
            writeln!(out, "{line}").map_err(RsomicsError::Io)?;
        }
    }
    out.flush().map_err(RsomicsError::Io)?;

    Ok(records.len() as u64)
}

fn next_line<B: BufRead>(lines: &mut std::io::Lines<B>) -> Result<String> {
    lines
        .next()
        .ok_or_else(|| RsomicsError::InvalidInput("truncated FASTQ".into()))?
        .map_err(RsomicsError::Io)
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
