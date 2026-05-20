use std::fs::File;
use std::io::{BufRead, BufReader, BufWriter, Write};
use std::path::Path;

use rsomics_common::{Result, RsomicsError};

pub fn deinterleave(input: &Path, out1: &mut dyn Write, out2: &mut dyn Write) -> Result<u64> {
    let file = File::open(input)
        .map_err(|e| RsomicsError::InvalidInput(format!("{}: {e}", input.display())))?;
    let reader = BufReader::new(file);
    let mut o1 = BufWriter::with_capacity(256 * 1024, out1);
    let mut o2 = BufWriter::with_capacity(256 * 1024, out2);
    let mut lines = reader.lines();
    let mut pairs: u64 = 0;

    while let Some(h1) = lines.next() {
        let h1 = h1.map_err(RsomicsError::Io)?;
        let s1 = next_line(&mut lines)?;
        let p1 = next_line(&mut lines)?;
        let q1 = next_line(&mut lines)?;

        let Some(h2) = lines.next() else { break };
        let h2 = h2.map_err(RsomicsError::Io)?;
        let s2 = next_line(&mut lines)?;
        let p2 = next_line(&mut lines)?;
        let q2 = next_line(&mut lines)?;

        writeln!(o1, "{h1}").map_err(RsomicsError::Io)?;
        writeln!(o1, "{s1}").map_err(RsomicsError::Io)?;
        writeln!(o1, "{p1}").map_err(RsomicsError::Io)?;
        writeln!(o1, "{q1}").map_err(RsomicsError::Io)?;

        writeln!(o2, "{h2}").map_err(RsomicsError::Io)?;
        writeln!(o2, "{s2}").map_err(RsomicsError::Io)?;
        writeln!(o2, "{p2}").map_err(RsomicsError::Io)?;
        writeln!(o2, "{q2}").map_err(RsomicsError::Io)?;

        pairs += 1;
    }

    o1.flush().map_err(RsomicsError::Io)?;
    o2.flush().map_err(RsomicsError::Io)?;
    Ok(pairs)
}

fn next_line<B: BufRead>(lines: &mut std::io::Lines<B>) -> Result<String> {
    lines
        .next()
        .ok_or_else(|| RsomicsError::InvalidInput("truncated FASTQ".into()))?
        .map_err(RsomicsError::Io)
}
