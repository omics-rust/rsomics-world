use std::collections::HashMap;
use std::fs::File;
use std::io::{BufRead, BufReader, BufWriter, Write};
use std::path::Path;

use rsomics_common::{Result, RsomicsError};

pub fn rename_bed(input: &Path, map_path: &Path, output: &mut dyn Write) -> Result<u64> {
    let mapping = load_mapping(map_path)?;

    let file = File::open(input)
        .map_err(|e| RsomicsError::InvalidInput(format!("{}: {e}", input.display())))?;
    let reader = BufReader::new(file);
    let mut out = BufWriter::with_capacity(64 * 1024, output);
    let mut count: u64 = 0;

    for line in reader.lines() {
        let line = line.map_err(RsomicsError::Io)?;
        if line.starts_with('#') || line.is_empty() {
            writeln!(out, "{line}").map_err(RsomicsError::Io)?;
            continue;
        }
        let fields: Vec<&str> = line.split('\t').collect();
        if fields.is_empty() {
            writeln!(out, "{line}").map_err(RsomicsError::Io)?;
            continue;
        }

        let old_name = fields[0];
        let new_name = mapping.get(old_name).map_or(old_name, |s| s.as_str());

        write!(out, "{new_name}").map_err(RsomicsError::Io)?;
        for field in &fields[1..] {
            write!(out, "\t{field}").map_err(RsomicsError::Io)?;
        }
        writeln!(out).map_err(RsomicsError::Io)?;
        count += 1;
    }

    out.flush().map_err(RsomicsError::Io)?;
    Ok(count)
}

fn load_mapping(path: &Path) -> Result<HashMap<String, String>> {
    let file = File::open(path)
        .map_err(|e| RsomicsError::InvalidInput(format!("{}: {e}", path.display())))?;
    let reader = BufReader::new(file);
    let mut map = HashMap::new();

    for line in reader.lines() {
        let line = line.map_err(RsomicsError::Io)?;
        if line.starts_with('#') || line.is_empty() {
            continue;
        }
        let parts: Vec<&str> = line.split('\t').collect();
        if parts.len() >= 2 {
            map.insert(parts[0].to_string(), parts[1].to_string());
        }
    }

    Ok(map)
}
