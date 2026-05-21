use std::collections::BTreeMap;
use std::fs::File;
use std::io::{BufRead, BufReader, BufWriter, Write};
use std::path::Path;

use rsomics_common::{Result, RsomicsError};

pub fn merge_counts(inputs: &[&Path], output: &mut dyn Write) -> Result<u64> {
    if inputs.is_empty() {
        return Err(RsomicsError::InvalidInput("no input files".into()));
    }

    let mut gene_counts: BTreeMap<String, Vec<u64>> = BTreeMap::new();
    let mut sample_names: Vec<String> = Vec::new();

    for (idx, path) in inputs.iter().enumerate() {
        let name = path
            .file_stem()
            .unwrap_or_default()
            .to_string_lossy()
            .to_string();
        sample_names.push(name);

        let file = File::open(path)
            .map_err(|e| RsomicsError::InvalidInput(format!("{}: {e}", path.display())))?;
        let reader = BufReader::new(file);

        for line in reader.lines() {
            let line = line.map_err(RsomicsError::Io)?;
            if line.starts_with('#') || line.starts_with("Geneid") || line.starts_with("__") {
                continue;
            }
            let parts: Vec<&str> = line.split('\t').collect();
            if parts.len() < 2 {
                continue;
            }
            let gene = parts[0].to_string();
            let count: u64 = parts.last().unwrap_or(&"0").parse().unwrap_or(0);

            let entry = gene_counts
                .entry(gene)
                .or_insert_with(|| vec![0; inputs.len()]);
            if entry.len() <= idx {
                entry.resize(inputs.len(), 0);
            }
            entry[idx] = count;
        }
    }

    let mut out = BufWriter::new(output);
    write!(out, "gene").map_err(RsomicsError::Io)?;
    for name in &sample_names {
        write!(out, "\t{name}").map_err(RsomicsError::Io)?;
    }
    writeln!(out).map_err(RsomicsError::Io)?;

    let mut total_genes = 0u64;
    for (gene, counts) in &gene_counts {
        write!(out, "{gene}").map_err(RsomicsError::Io)?;
        for c in counts {
            write!(out, "\t{c}").map_err(RsomicsError::Io)?;
        }
        writeln!(out).map_err(RsomicsError::Io)?;
        total_genes += 1;
    }

    out.flush().map_err(RsomicsError::Io)?;
    Ok(total_genes)
}
