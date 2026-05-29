//! Pairwise LD (r²) computation from PLINK1 binary filesets.
//!
//! Implements PLINK's `--r2` operation: for every pair of variants within a
//! sliding window (or across the full chromosome for all-pairs mode), compute
//! the squared Pearson correlation r² of the additive dosage vectors, skipping
//! samples missing in either variant.
//!
//! Output is a tab-separated table with one row per pair:
//!
//! ```text
//! CHR_A  BP_A  SNP_A  CHR_B  BP_B  SNP_B  R2
//! ```
//!
//! which matches the PLINK1 `--r2` `.ld` file format exactly.

#![allow(clippy::cast_precision_loss)]

use std::io::Write;

use rsomics_pgen::{Genotype, Pgen};

/// Compute r² between variants `vi` and `vj` over non-missing samples.
///
/// Returns 0.0 when fewer than 2 samples are available or one variant is
/// monomorphic (zero variance).
pub fn r2(pgen: &Pgen, vi: usize, vj: usize) -> f64 {
    let n_samp = pgen.n_samples();
    let mut xi = Vec::with_capacity(n_samp);
    let mut xj = Vec::with_capacity(n_samp);

    for s in 0..n_samp {
        let gi = pgen.get(vi, s);
        let gj = pgen.get(vj, s);
        let di = match gi {
            Genotype::HomA1 => 2.0f64,
            Genotype::Het => 1.0,
            Genotype::HomA2 => 0.0,
            _ => continue,
        };
        let dj = match gj {
            Genotype::HomA1 => 2.0f64,
            Genotype::Het => 1.0,
            Genotype::HomA2 => 0.0,
            _ => continue,
        };
        xi.push(di);
        xj.push(dj);
    }

    let n = xi.len();
    if n < 2 {
        return 0.0;
    }
    let mean_i: f64 = xi.iter().sum::<f64>() / n as f64;
    let mean_j: f64 = xj.iter().sum::<f64>() / n as f64;
    let mut cov = 0.0f64;
    let mut var_i = 0.0f64;
    let mut var_j = 0.0f64;
    for k in 0..n {
        let di = xi[k] - mean_i;
        let dj = xj[k] - mean_j;
        cov += di * dj;
        var_i += di * di;
        var_j += dj * dj;
    }
    let denom = var_i * var_j;
    if denom <= 0.0 {
        0.0
    } else {
        (cov * cov) / denom
    }
}

/// Compute pairwise r² for all variant pairs within a sliding window and
/// write to `out` in PLINK1 `.ld` format.
///
/// # Parameters
/// - `window_size`: maximum variant-pair distance (number of variants).
///   `0` means compute all pairs on the same chromosome (no window limit).
/// - `min_r2`: only output pairs with r² ≥ this value (default 0.0 = all pairs).
pub fn compute_ld<W: Write>(
    pgen: &Pgen,
    window_size: usize,
    min_r2: f64,
    out: &mut W,
) -> anyhow::Result<()> {
    let vars = &pgen.variants;

    // Group variant indices by chromosome in first-seen order.
    let mut chrom_groups: Vec<Vec<usize>> = Vec::new();
    let mut chrom_index: std::collections::HashMap<&str, usize> = std::collections::HashMap::new();

    for (i, v) in vars.iter().enumerate() {
        match chrom_index.get(v.chrom.as_str()) {
            Some(&idx) => chrom_groups[idx].push(i),
            None => {
                chrom_index.insert(v.chrom.as_str(), chrom_groups.len());
                chrom_groups.push(vec![i]);
            }
        }
    }

    for group in &chrom_groups {
        let n = group.len();
        for (a, &vi) in group.iter().enumerate() {
            let j_end = if window_size == 0 {
                n
            } else {
                (a + 1 + window_size).min(n)
            };
            for &vj in &group[a + 1..j_end] {
                let r2_val = r2(pgen, vi, vj);
                if r2_val < min_r2 {
                    continue;
                }
                let va = &vars[vi];
                let vb = &vars[vj];
                writeln!(
                    out,
                    "{}\t{}\t{}\t{}\t{}\t{}\t{:.6}",
                    va.chrom, va.pos, va.id, vb.chrom, vb.pos, vb.id, r2_val
                )?;
            }
        }
    }
    Ok(())
}
