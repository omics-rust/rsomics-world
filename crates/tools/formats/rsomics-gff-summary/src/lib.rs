use rsomics_common::{Result, RsomicsError};
use std::collections::BTreeMap;
use std::fs::File;
use std::io::{BufRead, BufReader, BufWriter, Write};
use std::path::Path;
pub fn gff_summary(input: &Path, output: &mut dyn Write) -> Result<u64> {
    let file = File::open(input)
        .map_err(|e| RsomicsError::InvalidInput(format!("{}: {e}", input.display())))?;
    let reader = BufReader::new(file);
    let mut types: BTreeMap<String, u64> = BTreeMap::new();
    let mut chroms: BTreeMap<String, u64> = BTreeMap::new();
    let mut total: u64 = 0;
    for line in reader.lines() {
        let line = line.map_err(RsomicsError::Io)?;
        if line.starts_with('#') || line.is_empty() {
            continue;
        }
        total += 1;
        let f: Vec<&str> = line.split('\t').collect();
        if f.len() >= 3 {
            *types.entry(f[2].to_string()).or_insert(0) += 1;
        }
        if !f.is_empty() {
            *chroms.entry(f[0].to_string()).or_insert(0) += 1;
        }
    }
    let mut out = BufWriter::with_capacity(64 * 1024, output);
    writeln!(out, "total_features\t{total}").map_err(RsomicsError::Io)?;
    writeln!(out, "chromosomes\t{}", chroms.len()).map_err(RsomicsError::Io)?;
    writeln!(out, "feature_types\t{}", types.len()).map_err(RsomicsError::Io)?;
    writeln!(out, "\n# Per feature type:").map_err(RsomicsError::Io)?;
    for (t, c) in &types {
        writeln!(out, "{t}\t{c}").map_err(RsomicsError::Io)?;
    }
    out.flush().map_err(RsomicsError::Io)?;
    Ok(total)
}
