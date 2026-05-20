use std::collections::HashSet;
use std::fs::File;
use std::io::{BufRead, BufReader, BufWriter, Write};
use std::path::Path;

use rsomics_common::{Result, RsomicsError};

pub fn extract_fastq(
    input: &Path,
    names_path: &Path,
    output: &mut dyn Write,
    exclude: bool,
) -> Result<u64> {
    let names = load_names(names_path)?;

    let file = File::open(input)
        .map_err(|e| RsomicsError::InvalidInput(format!("{}: {e}", input.display())))?;
    let reader = BufReader::new(file);
    let mut out = BufWriter::with_capacity(256 * 1024, output);
    let mut lines = reader.lines();
    let mut count: u64 = 0;

    while let Some(header) = lines.next() {
        let header = header.map_err(RsomicsError::Io)?;
        let seq = next_line(&mut lines)?;
        let plus = next_line(&mut lines)?;
        let qual = next_line(&mut lines)?;

        let name = header
            .split_once(|c: char| c.is_whitespace())
            .map_or(header.as_str(), |(n, _)| n)
            .trim_start_matches('@');

        let in_set = names.contains(name);
        let keep = if exclude { !in_set } else { in_set };

        if keep {
            writeln!(out, "{header}").map_err(RsomicsError::Io)?;
            writeln!(out, "{seq}").map_err(RsomicsError::Io)?;
            writeln!(out, "{plus}").map_err(RsomicsError::Io)?;
            writeln!(out, "{qual}").map_err(RsomicsError::Io)?;
            count += 1;
        }
    }

    out.flush().map_err(RsomicsError::Io)?;
    Ok(count)
}

fn next_line(lines: &mut std::io::Lines<BufReader<File>>) -> Result<String> {
    lines
        .next()
        .ok_or_else(|| RsomicsError::InvalidInput("truncated FASTQ".into()))?
        .map_err(RsomicsError::Io)
}

fn load_names(path: &Path) -> Result<HashSet<String>> {
    let file = File::open(path)
        .map_err(|e| RsomicsError::InvalidInput(format!("{}: {e}", path.display())))?;
    let reader = BufReader::new(file);
    let mut names = HashSet::new();
    for line in reader.lines() {
        let line = line.map_err(RsomicsError::Io)?;
        let name = line.trim().to_string();
        if !name.is_empty() {
            names.insert(name);
        }
    }
    Ok(names)
}
