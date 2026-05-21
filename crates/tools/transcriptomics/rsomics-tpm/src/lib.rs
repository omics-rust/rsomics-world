use std::collections::HashMap;
use std::fs::File;
use std::io::{BufRead, BufReader, BufWriter, Write};
use std::path::Path;

use rsomics_common::{Result, RsomicsError};

pub fn normalize_tpm(
    counts_path: &Path,
    lengths_path: &Path,
    output: &mut dyn Write,
) -> Result<u64> {
    let lengths = load_lengths(lengths_path)?;

    let file = File::open(counts_path)
        .map_err(|e| RsomicsError::InvalidInput(format!("{}: {e}", counts_path.display())))?;
    let reader = BufReader::new(file);
    let mut lines = reader.lines();

    let header = lines
        .next()
        .ok_or_else(|| RsomicsError::InvalidInput("empty counts file".into()))?
        .map_err(RsomicsError::Io)?;

    let header_parts: Vec<&str> = header.split('\t').collect();
    let n_samples = header_parts.len() - 1;

    let mut genes: Vec<String> = Vec::new();
    let mut raw_counts: Vec<Vec<f64>> = Vec::new();
    let mut gene_lengths: Vec<f64> = Vec::new();

    for line in lines {
        let line = line.map_err(RsomicsError::Io)?;
        let parts: Vec<&str> = line.split('\t').collect();
        if parts.len() < 2 {
            continue;
        }
        let gene = parts[0].to_string();
        let len = lengths.get(&gene).copied().unwrap_or(1000.0);

        let counts: Vec<f64> = parts[1..]
            .iter()
            .map(|s| s.parse::<f64>().unwrap_or(0.0))
            .collect();

        genes.push(gene);
        gene_lengths.push(len);
        raw_counts.push(counts);
    }

    let n_genes = genes.len();
    let mut rpk: Vec<Vec<f64>> = vec![vec![0.0; n_samples]; n_genes];
    let mut col_sums = vec![0.0f64; n_samples];

    for (i, counts) in raw_counts.iter().enumerate() {
        for (j, &c) in counts.iter().enumerate() {
            let r = c / (gene_lengths[i] / 1000.0);
            rpk[i][j] = r;
            col_sums[j] += r;
        }
    }

    let mut out = BufWriter::new(output);
    writeln!(out, "{header}").map_err(RsomicsError::Io)?;

    for (i, gene) in genes.iter().enumerate() {
        write!(out, "{gene}").map_err(RsomicsError::Io)?;
        for j in 0..n_samples {
            let tpm = if col_sums[j] > 0.0 {
                rpk[i][j] / col_sums[j] * 1_000_000.0
            } else {
                0.0
            };
            write!(out, "\t{tpm:.4}").map_err(RsomicsError::Io)?;
        }
        writeln!(out).map_err(RsomicsError::Io)?;
    }

    out.flush().map_err(RsomicsError::Io)?;
    Ok(n_genes as u64)
}

fn load_lengths(path: &Path) -> Result<HashMap<String, f64>> {
    let file = File::open(path)
        .map_err(|e| RsomicsError::InvalidInput(format!("{}: {e}", path.display())))?;
    let reader = BufReader::new(file);
    let mut map = HashMap::new();

    for line in reader.lines() {
        let line = line.map_err(RsomicsError::Io)?;
        if line.starts_with('#') {
            continue;
        }
        let parts: Vec<&str> = line.split('\t').collect();
        if parts.len() >= 2 {
            let gene = parts[0].to_string();
            let len: f64 = parts[1].parse().unwrap_or(1000.0);
            map.insert(gene, len);
        }
    }
    Ok(map)
}
