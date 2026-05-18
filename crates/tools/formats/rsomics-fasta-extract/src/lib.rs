use std::collections::HashSet;
use std::fs::File;
use std::io::{BufRead, BufReader, BufWriter, Write};
use std::path::Path;

use needletail::parse_fastx_file;
use rsomics_common::{Result, RsomicsError};

pub fn extract_fasta(
    input: &Path,
    names_path: &Path,
    output: &mut dyn Write,
    exclude: bool,
) -> Result<u64> {
    let names = load_names(names_path)?;

    if std::fs::metadata(input).is_ok_and(|m| m.len() == 0) {
        return Err(RsomicsError::InvalidInput("empty file".into()));
    }

    let mut reader = parse_fastx_file(input)
        .map_err(|e| RsomicsError::InvalidInput(format!("{}: {e}", input.display())))?;

    let mut out = BufWriter::with_capacity(256 * 1024, output);
    let mut count: u64 = 0;

    while let Some(record) = reader.next() {
        let record = record.map_err(|e| RsomicsError::InvalidInput(format!("reading: {e}")))?;
        let id = std::str::from_utf8(record.id())
            .unwrap_or("unknown")
            .to_string();

        let in_set = names.contains(&id);
        let keep = if exclude { !in_set } else { in_set };

        if keep {
            out.write_all(b">").map_err(RsomicsError::Io)?;
            out.write_all(record.id()).map_err(RsomicsError::Io)?;
            writeln!(out).map_err(RsomicsError::Io)?;
            out.write_all(&record.seq()).map_err(RsomicsError::Io)?;
            writeln!(out).map_err(RsomicsError::Io)?;
            count += 1;
        }
    }

    out.flush().map_err(RsomicsError::Io)?;
    Ok(count)
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
