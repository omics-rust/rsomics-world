use rsomics_common::{Result, RsomicsError};
use std::fs::File;
use std::io::{BufRead, BufReader, BufWriter, Write};
use std::path::Path;

#[allow(clippy::cast_precision_loss)]
pub fn ld_matrix(input: &Path, output: &mut dyn Write) -> Result<u64> {
    let file = File::open(input)
        .map_err(|e| RsomicsError::InvalidInput(format!("{}: {e}", input.display())))?;
    let reader = BufReader::new(file);

    let mut variant_names: Vec<String> = Vec::new();
    let mut genotypes: Vec<Vec<f64>> = Vec::new();

    for line in reader.lines() {
        let line = line.map_err(RsomicsError::Io)?;
        let parts: Vec<&str> = line.split('\t').collect();
        if parts.len() < 2 {
            continue;
        }
        variant_names.push(parts[0].to_string());
        let geno: Vec<f64> = parts[1..].iter().filter_map(|s| s.parse().ok()).collect();
        genotypes.push(geno);
    }

    let n_var = genotypes.len();
    let mut out = BufWriter::new(output);
    writeln!(out, "var1\tvar2\tr_squared").map_err(RsomicsError::Io)?;

    let mut count = 0u64;
    for i in 0..n_var {
        for j in (i + 1)..n_var {
            let r2 = compute_r2(&genotypes[i], &genotypes[j]);
            writeln!(out, "{}\t{}\t{r2:.6}", variant_names[i], variant_names[j])
                .map_err(RsomicsError::Io)?;
            count += 1;
        }
    }

    out.flush().map_err(RsomicsError::Io)?;
    Ok(count)
}

#[allow(clippy::cast_precision_loss)]
fn compute_r2(a: &[f64], b: &[f64]) -> f64 {
    let n = a.len().min(b.len());
    if n == 0 {
        return 0.0;
    }

    let mean_a: f64 = a[..n].iter().sum::<f64>() / n as f64;
    let mean_b: f64 = b[..n].iter().sum::<f64>() / n as f64;

    let mut cov = 0.0f64;
    let mut var_a = 0.0f64;
    let mut var_b = 0.0f64;

    for k in 0..n {
        let da = a[k] - mean_a;
        let db = b[k] - mean_b;
        cov += da * db;
        var_a += da * da;
        var_b += db * db;
    }

    let denom = var_a * var_b;
    if denom == 0.0 {
        0.0
    } else {
        (cov * cov) / denom
    }
}
