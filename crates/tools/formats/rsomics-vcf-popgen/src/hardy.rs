//! Hardy-Weinberg equilibrium test per site — equivalent to `vcftools --hardy`.
//!
//! Output: CHROM, POS, OBS(HOM1/HET/HOM2), E(HOM1/HET/HOM2), ChiSq_HWE, P_HWE, P_HET_DEFICIT, P_HET_EXCESS
//! P-values use exact Fisher test (mid-p) following Wigginton et al. 2005.

use std::path::Path;

use anyhow::Result;

use crate::vcf::{Allele, VcfReader};

pub struct HardyRecord {
    pub chrom: String,
    pub pos: u64,
    pub obs_hom1: u64,
    pub obs_het: u64,
    pub obs_hom2: u64,
    pub exp_hom1: f64,
    pub exp_het: f64,
    pub exp_hom2: f64,
    pub chisq: f64,
    pub p_hwe: f64,
    pub p_het_deficit: f64,
    pub p_het_excess: f64,
}

/// Exact HWE test (Wigginton et al. 2005 algorithm).
/// Returns (p_hwe, p_het_deficit, p_het_excess).
///
/// The algorithm builds a table of relative probabilities indexed by het count,
/// stepping by 2 (parity matches n_rare). p[max_het] = 1.0, fill downward via
/// recurrence: p[k-2] = p[k] * k*(k-1) / ((n_rare_a-k+2)*(n_chr-n_rare_a-k+2))
fn exact_hwe(obs_hom1: u64, obs_het: u64, obs_hom2: u64) -> (f64, f64, f64) {
    let n = obs_hom1 + obs_het + obs_hom2;
    if n == 0 {
        return (f64::NAN, f64::NAN, f64::NAN);
    }
    // n_rare_a = allele count of the rarer allele.
    let n_rare_a = (2 * obs_hom1 + obs_het).min(2 * obs_hom2 + obs_het);
    let n_chr = 2 * n; // total allele copies

    // Max het count consistent with n_rare_a.
    let max_het = n_rare_a.min(n_chr - n_rare_a);
    // Table indexed 0..=max_het with same parity as max_het.
    // Length = max_het/2 + 1 entries (0, 2, 4, ... or 1, 3, 5, ...).
    let table_len = (max_het / 2 + 1) as usize;
    let mut probs = vec![0.0f64; table_len];
    // Index of max_het in the table.
    let max_idx = table_len - 1;
    probs[max_idx] = 1.0;

    // Fill downward (recurrence from Wigginton et al.):
    // P(k-2) = P(k) * k*(k-1) / ((n_rare_a - k + 2) * (n_chr - n_rare_a - k + 2))
    for t in (0..max_idx).rev() {
        let k = (max_het - 2 * (max_idx - t - 1) as u64) as f64; // actual het count of probs[t+1]
        let denom = (n_rare_a as f64 - k + 2.0) * (n_chr as f64 - n_rare_a as f64 - k + 2.0);
        if denom == 0.0 {
            break;
        }
        probs[t] = probs[t + 1] * k * (k - 1.0) / denom;
    }

    let sum: f64 = probs.iter().sum();
    if sum == 0.0 {
        return (f64::NAN, f64::NAN, f64::NAN);
    }

    // Observed het index in table.
    let obs_het_u = obs_het;
    // obs_het and max_het must have same parity.
    let obs_t = if obs_het_u <= max_het && obs_het_u % 2 == max_het % 2 {
        Some(max_idx - ((max_het - obs_het_u) / 2) as usize)
    } else {
        None
    };

    let obs_p = obs_t.map_or(0.0, |t| probs[t] / sum);

    // Two-sided: sum of all probs ≤ observed prob.
    let p_hwe: f64 = probs.iter().filter(|&&p| p <= obs_p + 1e-15).sum::<f64>() / sum;

    // One-sided deficit (fewer hets than expected = low het table indices).
    // Deficit = sum of probs from 0 up to and including obs_t.
    let p_het_deficit: f64 = match obs_t {
        Some(t) => probs[..=t].iter().sum::<f64>() / sum,
        None => 1.0,
    };

    // One-sided excess (more hets than expected = high het table indices).
    let p_het_excess: f64 = match obs_t {
        Some(t) => probs[t..].iter().sum::<f64>() / sum,
        None => 1.0,
    };

    (
        p_hwe.clamp(0.0, 1.0),
        p_het_deficit.clamp(0.0, 1.0),
        p_het_excess.clamp(0.0, 1.0),
    )
}

pub fn hardy(path: &Path) -> Result<Vec<HardyRecord>> {
    let reader = VcfReader::open(path)?;
    let mut out = Vec::new();

    for rec in reader {
        let rec = rec?;
        // Biallelic SNPs only (vcftools --hardy skips multiallelic).
        if rec.alt_alleles.len() != 1 {
            continue;
        }

        let mut obs_hom1 = 0u64; // ref/ref
        let mut obs_het = 0u64;
        let mut obs_hom2 = 0u64; // alt/alt
        let mut n_called = 0u64;

        for gt in &rec.gts {
            if gt.is_missing() {
                continue;
            }
            n_called += 1;
            match (gt.a1, gt.a2) {
                (Allele::Ref, Allele::Ref) => obs_hom1 += 1,
                (Allele::Alt(_), Allele::Alt(_)) => obs_hom2 += 1,
                _ => obs_het += 1,
            }
        }

        if n_called == 0 {
            continue;
        }

        let n = n_called as f64;
        let p = (2 * obs_hom1 + obs_het) as f64 / (2.0 * n);
        let q = 1.0 - p;
        let exp_hom1 = p * p * n;
        let exp_het = 2.0 * p * q * n;
        let exp_hom2 = q * q * n;

        // Chi-squared statistic (for output; p-values from exact test).
        let chisq = if exp_hom1 > 0.0 {
            (obs_hom1 as f64 - exp_hom1).powi(2) / exp_hom1
        } else {
            0.0
        } + if exp_het > 0.0 {
            (obs_het as f64 - exp_het).powi(2) / exp_het
        } else {
            0.0
        } + if exp_hom2 > 0.0 {
            (obs_hom2 as f64 - exp_hom2).powi(2) / exp_hom2
        } else {
            0.0
        };

        let (p_hwe, p_het_deficit, p_het_excess) = exact_hwe(obs_hom1, obs_het, obs_hom2);

        out.push(HardyRecord {
            chrom: rec.chrom,
            pos: rec.pos,
            obs_hom1,
            obs_het,
            obs_hom2,
            exp_hom1,
            exp_het,
            exp_hom2,
            chisq,
            p_hwe,
            p_het_deficit,
            p_het_excess,
        });
    }

    Ok(out)
}

pub fn print_hardy(records: &[HardyRecord]) {
    println!(
        "CHROM\tPOS\tOBS(HOM1/HET/HOM2)\tE(HOM1/HET/HOM2)\tChiSq_HWE\tP_HWE\tP_HET_DEFICIT\tP_HET_EXCESS"
    );
    for r in records {
        println!(
            "{}\t{}\t{}/{}/{}\t{:.3}/{:.3}/{:.3}\t{:.6}\t{:.6}\t{:.6}\t{:.6}",
            r.chrom,
            r.pos,
            r.obs_hom1,
            r.obs_het,
            r.obs_hom2,
            r.exp_hom1,
            r.exp_het,
            r.exp_hom2,
            r.chisq,
            r.p_hwe,
            r.p_het_deficit,
            r.p_het_excess,
        );
    }
}
