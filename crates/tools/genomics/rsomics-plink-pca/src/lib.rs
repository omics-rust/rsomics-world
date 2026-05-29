//! PCA and GRM computation from PLINK1 binary filesets.
//!
//! Implements PLINK2's `--pca` operation:
//!
//! 1. Build the standardized genotype matrix X (samples × variants), where
//!    each column (variant) is centered at twice the allele frequency and
//!    scaled by `sqrt(2 * p * (1 - p))`.  Missing genotypes contribute zero
//!    after centering (the PLINK2 mean-imputation default).
//!
//! 2. Compute the Genetic Relationship Matrix (GRM) `G = (1/M) * X * X^T`
//!    (`n_samples` × `n_samples`, symmetric positive semi-definite).
//!
//! 3. Eigendecompose G via faer's self-adjoint EVD, which sorts eigenvalues in
//!    nondecreasing order.  We reverse to get descending order (PC1 first).
//!
//! 4. Output top-k PC scores in PLINK2 `.eigenvec` format
//!    (FID IID PC1 PC2 …) and eigenvalues in `.eigenval` format
//!    (one per line).

#![allow(clippy::cast_precision_loss)]

use std::io::Write;

use faer::{Mat, Par, Side};
use rsomics_pgen::{Genotype, Pgen};

/// Build the standardized genotype matrix (`n_samples` × `n_variants`).
///
/// For variant `j` with allele frequency `p_j`, each sample's dosage
/// `g ∈ {0,1,2}` is standardized as `(g - 2*p_j) / sqrt(2*p_j*(1-p_j))`.
/// Missing genotypes are imputed with mean 0 (PLINK2 default).
/// Monomorphic variants (zero variance) are skipped (dropped from M count).
fn build_std_genotype_matrix(pgen: &Pgen) -> (Mat<f64>, usize) {
    let n_samples = pgen.n_samples();
    let n_vars = pgen.variants.len();

    // Compute allele frequencies and filter out monomorphic variants.
    let mut valid_cols: Vec<(usize, f64, f64)> = Vec::new(); // (vi, mean, scale)
    for vi in 0..n_vars {
        let mut sum = 0u32;
        let mut count = 0u32;
        for si in 0..n_samples {
            match pgen.get(vi, si) {
                Genotype::HomA1 => {
                    sum += 2;
                    count += 1;
                }
                Genotype::Het => {
                    sum += 1;
                    count += 1;
                }
                Genotype::HomA2 => {
                    count += 1;
                }
                _ => {}
            }
        }
        if count == 0 {
            continue;
        }
        let mean = f64::from(sum) / f64::from(count); // = 2 * p
        let freq = mean / 2.0;
        let var = 2.0 * freq * (1.0 - freq);
        if var <= 0.0 {
            continue;
        }
        valid_cols.push((vi, mean, var.sqrt()));
    }

    let m_eff = valid_cols.len();
    let mut geno = Mat::<f64>::zeros(n_samples, m_eff);

    for (col, &(vi, mean, scale)) in valid_cols.iter().enumerate() {
        for si in 0..n_samples {
            let dosage = match pgen.get(vi, si) {
                Genotype::HomA1 => 2.0f64,
                Genotype::Het => 1.0,
                Genotype::HomA2 => 0.0,
                _ => mean, // missing → impute with mean (net-zero contribution)
            };
            *geno.get_mut(si, col) = (dosage - mean) / scale;
        }
    }
    (geno, m_eff)
}

/// Compute the GRM `G = (1/M) * X * X^T` (n×n symmetric).
fn build_grm(geno: &Mat<f64>, m_eff: usize) -> Mat<f64> {
    let n_samples = geno.nrows();
    let m = m_eff as f64;
    let mut grm_mat = Mat::<f64>::zeros(n_samples, n_samples);
    // X * X^T via faer matrix multiply.
    faer::linalg::matmul::matmul(
        grm_mat.as_mut(),
        faer::Accum::Add,
        geno.as_ref(),
        geno.transpose(),
        1.0 / m,
        Par::Seq,
    );
    grm_mat
}

/// Run PCA on the PLINK1 binary fileset and write outputs to `eigenvec_out`
/// and `eigenval_out`.
///
/// # Arguments
/// - `pgen`         — loaded PLINK1 fileset
/// - `n_pcs`        — number of principal components to report
/// - `eigenvec_out` — writer for `.eigenvec` tab-separated output
/// - `eigenval_out` — writer for `.eigenval` output (one value per line)
pub fn run_pca<W1, W2>(
    pgen: &Pgen,
    n_pcs: usize,
    eigenvec_out: &mut W1,
    eigenval_out: &mut W2,
) -> anyhow::Result<()>
where
    W1: Write,
    W2: Write,
{
    let n_samples = pgen.n_samples();
    let n_pcs_actual = n_pcs.min(n_samples.saturating_sub(1));

    let (geno, m_eff) = build_std_genotype_matrix(pgen);
    anyhow::ensure!(m_eff > 0, "no polymorphic variants found in PLINK fileset");
    let grm_mat = build_grm(&geno, m_eff);

    // EVD of the symmetric GRM. faer sorts eigenvalues in *nondecreasing* order.
    // SelfAdjointEigen exposes U() (eigenvectors) and S() (diagonal eigenvalues).
    let evd = grm_mat
        .self_adjoint_eigen(Side::Lower)
        .map_err(|e| anyhow::anyhow!("EVD failed: {e:?}"))?;

    let eig_s = evd.S(); // DiagRef of eigenvalues, nondecreasing
    let eig_u = evd.U(); // MatRef of eigenvectors, column j = eigenvector for eigenvalue j

    let total = eig_u.nrows(); // = n_samples; eigenvalues are in nondecreasing order
    // PC1 = largest eigenvalue → column `total - 1`; PC2 = column `total - 2`; etc.

    // Write eigenvalues (descending).
    let evals: Vec<f64> = eig_s.column_vector().iter().copied().collect();
    for &ev in evals.iter().rev().take(n_pcs_actual) {
        writeln!(eigenval_out, "{ev:.6}")?;
    }

    // Header: #FID IID PC1 PC2 ...
    let pc_header: Vec<String> = (1..=n_pcs_actual).map(|i| format!("PC{i}")).collect();
    writeln!(eigenvec_out, "#FID\tIID\t{}", pc_header.join("\t"))?;

    // Each row = one sample's PC scores.
    for si in 0..n_samples {
        let sample = &pgen.samples[si];
        let scores: Vec<String> = (0..n_pcs_actual)
            .map(|pc_idx| {
                let eig_col = total - 1 - pc_idx; // descending eigenvalue index
                format!("{:.6}", eig_u.get(si, eig_col))
            })
            .collect();
        writeln!(
            eigenvec_out,
            "{}\t{}\t{}",
            sample.fid,
            sample.iid,
            scores.join("\t")
        )?;
    }

    Ok(())
}
