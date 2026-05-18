use std::fs::File;
use std::io::{BufRead, BufReader, BufWriter, Write};
use std::path::Path;

use rsomics_common::{Result, RsomicsError};

pub fn sample_bed(input: &Path, output: &mut dyn Write, count: usize, seed: u64) -> Result<u64> {
    let file = File::open(input)
        .map_err(|e| RsomicsError::InvalidInput(format!("{}: {e}", input.display())))?;
    let reader = BufReader::new(file);

    let mut lines: Vec<String> = Vec::new();
    for line in reader.lines() {
        let line = line.map_err(RsomicsError::Io)?;
        if line.starts_with('#') || line.is_empty() {
            continue;
        }
        lines.push(line);
    }

    let n = count.min(lines.len());
    let mut rng = SimpleRng(seed);
    for i in (1..lines.len()).rev() {
        let j = rng.next_usize(i + 1);
        lines.swap(i, j);
    }

    let mut out = BufWriter::with_capacity(64 * 1024, output);
    for line in &lines[..n] {
        writeln!(out, "{line}").map_err(RsomicsError::Io)?;
    }
    out.flush().map_err(RsomicsError::Io)?;

    Ok(n as u64)
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
