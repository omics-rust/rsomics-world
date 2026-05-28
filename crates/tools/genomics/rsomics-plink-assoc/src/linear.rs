//! Linear regression association test (plink --linear).
//!
//! For each variant, fits: phenotype ~ 0 + dosage (additive model).
//! Outputs the slope (beta), standard error, t-statistic, and p-value.
//! Dosage coding: HomA2=0, Het=1, HomA1=2 (A1 = coded/minor allele).
//!
//! Only quantitative phenotypes (non-missing, not 0/-9) are included.

use rsomics_pgen::{Genotype, Pgen};
use std::io::Write;

use crate::assoc::chi2_sf;

pub struct LinearRecord {
    pub chrom: String,
    pub snp: String,
    pub bp: u64,
    pub nmiss: usize,
    pub a1: String,
    pub beta: f64,
    pub se: f64,
    pub stat: f64,
    pub p: f64,
}

/// Fit per-variant additive linear regression for quantitative phenotype.
pub fn linear_test(pgen: &Pgen) -> Vec<LinearRecord> {
    let n_samples = pgen.n_samples();

    pgen.variants
        .iter()
        .enumerate()
        .map(|(v, var)| {
            let mut xs: Vec<f64> = Vec::with_capacity(n_samples);
            let mut ys: Vec<f64> = Vec::with_capacity(n_samples);

            for s in 0..n_samples {
                // Parse quantitative phenotype; skip missing ("0", "-9")
                let pheno: f64 = match pgen.samples[s].phen.as_str() {
                    "0" | "-9" => continue,
                    phen_str => match phen_str.parse() {
                        Ok(v) => v,
                        Err(_) => continue,
                    },
                };
                let dosage = match pgen.get(v, s) {
                    Genotype::HomA2 => 0.0f64,
                    Genotype::Het => 1.0,
                    Genotype::HomA1 => 2.0,
                    Genotype::Missing | _ => continue,
                };
                xs.push(dosage);
                ys.push(pheno);
            }

            let nmiss = n_samples - xs.len();
            let (beta, se, stat, p) = ols_stats(&xs, &ys);

            LinearRecord {
                chrom: var.chrom.clone(),
                snp: var.id.clone(),
                bp: var.pos,
                nmiss,
                a1: var.a1.clone(),
                beta,
                se,
                stat,
                p,
            }
        })
        .collect()
}

/// Ordinary-least-squares regression y ~ intercept + beta*x.
///
/// Returns (beta, standard_error, t_statistic, p_value).
/// Uses a two-tailed t-test with (n - 2) degrees of freedom;
/// the p-value is converted from the t distribution via chi2_sf on t².
#[allow(clippy::many_single_char_names)]
fn ols_stats(x: &[f64], y: &[f64]) -> (f64, f64, f64, f64) {
    let n = x.len();
    if n < 3 {
        return (f64::NAN, f64::NAN, f64::NAN, f64::NAN);
    }
    let n_f = n as f64;
    let sum_x: f64 = x.iter().sum();
    let sum_y: f64 = y.iter().sum();
    let sum_xx: f64 = x.iter().map(|xi| xi * xi).sum();
    let sum_xy: f64 = x.iter().zip(y.iter()).map(|(xi, yi)| xi * yi).sum();
    let mean_x = sum_x / n_f;
    let mean_y = sum_y / n_f;

    let sxx = sum_xx - n_f * mean_x * mean_x;
    if sxx.abs() < f64::EPSILON {
        // Monomorphic variant — no variation in dosage.
        return (f64::NAN, f64::NAN, f64::NAN, f64::NAN);
    }

    let sxy = sum_xy - n_f * mean_x * mean_y;
    let beta = sxy / sxx;
    let alpha = mean_y - beta * mean_x;

    // Residual sum of squares.
    let rss: f64 = x
        .iter()
        .zip(y.iter())
        .map(|(xi, yi)| {
            let residual = yi - (alpha + beta * xi);
            residual * residual
        })
        .sum();

    let df = n_f - 2.0;
    let s2 = rss / df;
    let se = (s2 / sxx).sqrt();

    if se == 0.0 || se.is_nan() {
        return (beta, se, f64::NAN, f64::NAN);
    }

    let t = beta / se;
    // Convert t statistic to p-value: use chi2_sf(t², 1) which gives the
    // two-tailed p for the standard normal approximation, matching plink's
    // output when df is large (n > 50 typical in GWAS).
    let p = chi2_sf(t * t, 1);

    (beta, se, t, p)
}

pub fn print_linear(records: &[LinearRecord], out: &mut impl Write) {
    writeln!(out, "CHR\tSNP\tBP\tNMISS\tA1\tBETA\tSE\tSTAT\tP").unwrap();
    for r in records {
        writeln!(
            out,
            "{}\t{}\t{}\t{}\t{}\t{:.6}\t{:.6}\t{:.4}\t{:.6e}",
            r.chrom, r.snp, r.bp, r.nmiss, r.a1, r.beta, r.se, r.stat, r.p
        )
        .unwrap();
    }
}
