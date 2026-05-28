//! Chi-squared allelic association test (plink --assoc).
//!
//! For each variant, constructs a 2×2 allele-count table:
//!   rows = case / control
//!   cols = A1 allele count / A2 allele count
//! Then applies a chi-squared test with 1 degree of freedom.
//! Odds ratio is computed from the 2×2 allele table.
//!
//! Phenotype encoding: 1 = unaffected (control), 2 = affected (case).
//! Samples with phenotype 0 or -9 (missing) are excluded.

use rsomics_pgen::{Genotype, Pgen};
use std::io::Write;

pub struct AssocRecord {
    pub chrom: String,
    pub snp: String,
    pub bp: u64,
    /// A1 = minor/coded allele in PLINK convention
    pub a1: String,
    /// Frequency of A1 in cases
    pub f_a: f64,
    /// Frequency of A1 in controls
    pub f_u: f64,
    pub a2: String,
    pub chisq: f64,
    pub p: f64,
    pub or: f64,
}

/// Run chi-squared allelic test for all variants.
pub fn assoc_test(pgen: &Pgen) -> Vec<AssocRecord> {
    let n_samples = pgen.n_samples();
    pgen.variants
        .iter()
        .enumerate()
        .map(|(v, var)| {
            // a1_case, a2_case = allele counts in cases
            // a1_ctrl, a2_ctrl = allele counts in controls
            let mut a1_case = 0u64;
            let mut a2_case = 0u64;
            let mut a1_ctrl = 0u64;
            let mut a2_ctrl = 0u64;

            for s in 0..n_samples {
                // FAM phenotype: "2" = case, "1" = control, "0"/"-9" = missing
                let (is_case, is_ctrl) = match pgen.samples[s].phen.as_str() {
                    "2" => (true, false),
                    "1" => (false, true),
                    _ => continue,
                };
                let (a1_count, a2_count) = match pgen.get(v, s) {
                    Genotype::HomA1 => (2u64, 0u64),
                    Genotype::Het => (1, 1),
                    Genotype::HomA2 => (0, 2),
                    Genotype::Missing | _ => continue,
                };
                if is_case {
                    a1_case += a1_count;
                    a2_case += a2_count;
                } else if is_ctrl {
                    a1_ctrl += a1_count;
                    a2_ctrl += a2_count;
                }
            }

            let n_case = (a1_case + a2_case) as f64;
            let n_ctrl = (a1_ctrl + a2_ctrl) as f64;
            let f_a = if n_case > 0.0 {
                a1_case as f64 / n_case
            } else {
                0.0
            };
            let f_u = if n_ctrl > 0.0 {
                a1_ctrl as f64 / n_ctrl
            } else {
                0.0
            };

            let (chisq, p) = chi_squared_2x2(a1_case, a2_case, a1_ctrl, a2_ctrl);
            let or = odds_ratio(a1_case, a2_case, a1_ctrl, a2_ctrl);

            AssocRecord {
                chrom: var.chrom.clone(),
                snp: var.id.clone(),
                bp: var.pos,
                a1: var.a1.clone(),
                f_a,
                f_u,
                a2: var.a2.clone(),
                chisq,
                p,
                or,
            }
        })
        .collect()
}

/// Pearson chi-squared test on a 2×2 table with Yates continuity correction
/// (matching plink --assoc behaviour).
///
/// Table:
///   a1_case  a2_case
///   a1_ctrl  a2_ctrl
///
/// Returns (chi_squared, p_value).
fn chi_squared_2x2(a1_case: u64, a2_case: u64, a1_ctrl: u64, a2_ctrl: u64) -> (f64, f64) {
    let a = a1_case as f64;
    let b = a2_case as f64;
    let c = a1_ctrl as f64;
    let d = a2_ctrl as f64;
    let n = a + b + c + d;
    if n == 0.0 {
        return (f64::NAN, f64::NAN);
    }
    let r1 = a + b;
    let r2 = c + d;
    let c1 = a + c;
    let c2 = b + d;
    if c1 == 0.0 || c2 == 0.0 || r1 == 0.0 || r2 == 0.0 {
        return (f64::NAN, f64::NAN);
    }
    // Yates continuity correction: subtract 0.5*n from |ad - bc|
    let ad_bc = (a * d - b * c).abs() - 0.5 * n;
    let ad_bc = ad_bc.max(0.0);
    let chisq = n * ad_bc * ad_bc / (r1 * r2 * c1 * c2);
    let p = chi2_sf(chisq, 1);
    (chisq, p)
}

/// Compute odds ratio from 2×2 table; returns NaN when indeterminate.
fn odds_ratio(a1_case: u64, a2_case: u64, a1_ctrl: u64, a2_ctrl: u64) -> f64 {
    let a = a1_case as f64;
    let b = a2_case as f64;
    let c = a1_ctrl as f64;
    let d = a2_ctrl as f64;
    let denom = b * c;
    if denom == 0.0 {
        if a * d > 0.0 { f64::INFINITY } else { f64::NAN }
    } else {
        (a * d) / denom
    }
}

/// Survival function (1 - CDF) of chi-squared distribution with df degrees of
/// freedom, evaluated at x.  Uses the regularised upper incomplete gamma
/// function: P(chi²_df > x) = Γ(df/2, x/2) / Γ(df/2).
///
/// For df=1 this simplifies to: 1 - erf(sqrt(x/2)).
pub fn chi2_sf(x: f64, df: u32) -> f64 {
    if x <= 0.0 {
        return 1.0;
    }
    if x.is_nan() || x.is_infinite() {
        return if x.is_nan() { f64::NAN } else { 0.0 };
    }
    // Regularised upper incomplete gamma: Q(a, x) = 1 - P(a, x)
    // where a = df/2, x = chi2_stat/2.
    let a = f64::from(df) / 2.0;
    let z = x / 2.0;
    gamma_q(a, z)
}

/// Regularised upper incomplete gamma function Q(a, z) = 1 - P(a, z).
/// Uses continued fraction for large z (z > a + 1), series expansion for small z.
fn gamma_q(a: f64, z: f64) -> f64 {
    if z < a + 1.0 {
        1.0 - gamma_p_series(a, z)
    } else {
        gamma_q_cf(a, z)
    }
}

/// Regularised lower incomplete gamma P(a, z) via series expansion.
fn gamma_p_series(a: f64, z: f64) -> f64 {
    if z <= 0.0 {
        return 0.0;
    }
    let mut ap = a;
    let mut sum = 1.0 / a;
    let mut del = sum;
    for _ in 0..200 {
        ap += 1.0;
        del *= z / ap;
        sum += del;
        if del.abs() < sum.abs() * 3e-15 {
            break;
        }
    }
    sum * (-z + a * z.ln() - ln_gamma(a)).exp()
}

/// Regularised upper incomplete gamma Q(a, z) via continued fraction (Lentz).
fn gamma_q_cf(a: f64, z: f64) -> f64 {
    let fpmin = f64::MIN_POSITIVE / 3e-15;
    let mut b = z + 1.0 - a;
    let mut c = 1.0 / fpmin;
    let mut d = 1.0 / b;
    let mut h = d;
    for i in 1u32..=200 {
        let an = -(f64::from(i) * (f64::from(i) - a));
        b += 2.0;
        d = an * d + b;
        if d.abs() < fpmin {
            d = fpmin;
        }
        c = b + an / c;
        if c.abs() < fpmin {
            c = fpmin;
        }
        d = 1.0 / d;
        let del = d * c;
        h *= del;
        if (del - 1.0).abs() < 3e-15 {
            break;
        }
    }
    h * (-z + a * z.ln() - ln_gamma(a)).exp()
}

/// Natural log of the gamma function using Lanczos approximation (g=7, n=9).
fn ln_gamma(x: f64) -> f64 {
    // Lanczos coefficients (Numerical Recipes 3rd ed.)
    const G: f64 = 7.0;
    const C: [f64; 9] = [
        0.999_999_999_999_809_9,
        676.520_368_121_885_1,
        -1_259.139_216_722_403,
        771.323_428_777_653_1,
        -176.615_029_162_140_6,
        12.507_343_278_686_905,
        -0.138_571_095_265_720_5,
        9.984_369_578_019_572e-6,
        1.505_632_735_149_311_6e-7,
    ];
    if x < 0.5 {
        // Reflection formula
        return std::f64::consts::PI.ln()
            - (std::f64::consts::PI * x).sin().ln()
            - ln_gamma(1.0 - x);
    }
    let x = x - 1.0;
    let t = x + G + 0.5;
    let ser: f64 = C[0]
        + C[1..]
            .iter()
            .enumerate()
            .fold(0.0, |acc, (i, &c)| acc + c / (x + i as f64 + 1.0));
    0.5 * (2.0 * std::f64::consts::PI).ln() + ser.ln() + (x + 0.5) * t.ln() - t
}

pub fn print_assoc(records: &[AssocRecord], out: &mut impl Write) {
    writeln!(out, "CHR\tSNP\tBP\tA1\tF_A\tF_U\tA2\tCHISQ\tP\tOR").unwrap();
    for r in records {
        writeln!(
            out,
            "{}\t{}\t{}\t{}\t{:.4}\t{:.4}\t{}\t{:.4}\t{:.6e}\t{:.4}",
            r.chrom, r.snp, r.bp, r.a1, r.f_a, r.f_u, r.a2, r.chisq, r.p, r.or
        )
        .unwrap();
    }
}
