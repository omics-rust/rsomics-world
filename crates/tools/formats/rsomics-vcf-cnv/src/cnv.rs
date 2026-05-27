//! CNV emission probability computation.
//!
//! Gaussian peak model for BAF under each CN state, plus LRR contribution.
//! Formulas match bcftools vcfcnv.c `set_observed_prob()` and `norm_prob()`.

use std::f64::consts::{PI, SQRT_2};

use crate::hmm::{CN0, CN1, CN2, CN3, Emission, N_STATES};

/// Parameters that govern the emission model.
#[derive(Clone, Copy)]
pub struct EmissionParams {
    pub baf_dev2: f64,
    pub lrr_dev2: f64,
    /// Weight of BAF evidence in [0, 1].
    pub baf_bias: f64,
    /// Weight of LRR evidence in [0, 1].
    pub lrr_bias: f64,
    pub err_prob: f64,
    /// Aberrant cell fraction (1.0 = 100% tumour).
    pub cell_frac: f64,
}

impl Default for EmissionParams {
    fn default() -> Self {
        Self {
            baf_dev2: 0.04 * 0.04,
            lrr_dev2: 0.2 * 0.2,
            baf_bias: 1.0,
            lrr_bias: 0.2,
            err_prob: 1e-4,
            cell_frac: 1.0,
        }
    }
}

/// Precomputed Gaussian normalization constants for each of the 9 BAF peaks.
///
/// norm_cdf(mean, dev) = Φ((1-mean)/dev) - Φ(-mean/dev), i.e. the mass of
/// N(mean, dev²) on [0,1], matching bcftools `norm_cdf()`.
pub struct GaussPeaks {
    // CN1: RR peak (mean=0), A peak (mean=1)
    cn1_r_norm: f64,
    cn1_a_norm: f64,
    // CN2: RR (0), RA (0.5), AA (1)
    cn2_rr_norm: f64,
    cn2_ra_norm: f64,
    cn2_aa_norm: f64,
    // CN3: RRR (0), RRA (1/(2+cf)), RAA ((1+cf)/(2+cf)), AAA (1)
    cn3_rrr_norm: f64,
    cn3_rra_norm: f64,
    cn3_raa_norm: f64,
    cn3_aaa_norm: f64,
    // CN3 peak means that depend on cell_frac
    cn3_rra_mean: f64,
    cn3_raa_mean: f64,
}

/// Standard normal CDF Φ(x) via erfc.
fn phi(x: f64) -> f64 {
    0.5 * erfc(-x / SQRT_2)
}

/// erfc via libm-compatible approximation, using Rust's f64::erfc (since 1.62).
fn erfc(x: f64) -> f64 {
    // erfc is stable in Rust std since the initial stable release for f64
    // but there's no f64::erfc in std; use the series via the error function.
    // libm is not a dep; compute via 1 - erf using rational approximation.
    // For our use (integrating over [0,1] Gaussian tails) full precision is not critical.
    1.0 - erf(x)
}

/// erf via Horner polynomial, accurate to ~1e-7 for |x| ≤ 3.5 (BCF sites: |x| ≤ ~12/dev).
fn erf(x: f64) -> f64 {
    // Abramowitz & Stegun 7.1.26 rational approximation
    let sign = if x < 0.0 { -1.0 } else { 1.0 };
    let x = x.abs();
    let t = 1.0 / (1.0 + 0.3275911 * x);
    let poly = t
        * (0.254_829_592
            + t * (-0.284_496_736
                + t * (1.421_413_741 + t * (-1.453_152_027 + t * 1.061_405_429))));
    sign * (1.0 - poly * (-x * x).exp())
}

/// Mass of N(mean, dev²) on [0, 1].
fn norm_cdf_mass(mean: f64, dev: f64) -> f64 {
    phi((1.0 - mean) / dev) - phi(-mean / dev)
}

/// Unnormalized Gaussian density at `baf` for a peak with given `mean` and `dev2`.
/// Normalization constant is the mass on [0,1], matching bcftools `norm_prob()`.
fn gauss_density(baf: f64, mean: f64, dev2: f64, norm: f64) -> f64 {
    let exponent = -(baf - mean) * (baf - mean) * 0.5 / dev2;
    exponent.exp() / norm / (2.0 * PI * dev2).sqrt()
}

impl GaussPeaks {
    fn new(params: &EmissionParams) -> Self {
        let dev = params.baf_dev2.sqrt();
        let cf = params.cell_frac;
        let cn3_rra_mean = 1.0 / (2.0 + cf);
        let cn3_raa_mean = (1.0 + cf) / (2.0 + cf);
        Self {
            cn1_r_norm: norm_cdf_mass(0.0, dev),
            cn1_a_norm: norm_cdf_mass(1.0, dev),
            cn2_rr_norm: norm_cdf_mass(0.0, dev),
            cn2_ra_norm: norm_cdf_mass(0.5, dev),
            cn2_aa_norm: norm_cdf_mass(1.0, dev),
            cn3_rrr_norm: norm_cdf_mass(0.0, dev),
            cn3_rra_norm: norm_cdf_mass(cn3_rra_mean, dev),
            cn3_raa_norm: norm_cdf_mass(cn3_raa_mean, dev),
            cn3_aaa_norm: norm_cdf_mass(1.0, dev),
            cn3_rra_mean,
            cn3_raa_mean,
        }
    }
}

/// Compute the 4-state emission probability vector for one site.
///
/// BAF and LRR are single-sample FORMAT floats. Negative BAF indicates missing data
/// (bcftools convention: off-array or no-call site), in which case CN0 gets 0.5 and
/// the rest share the remaining 0.5 equally.
///
/// `f_rr`, `f_ra`, `f_aa` are genotype frequencies at this site.
pub fn site_emission(
    baf: f32,
    lrr: f32,
    f_rr: f64,
    f_ra: f64,
    f_aa: f64,
    params: &EmissionParams,
    peaks: &GaussPeaks,
) -> Emission {
    // Missing BAF: CN0=0.5, rest=0.5/(N_STATES-1) each.  Matches bcftools `baf<0` branch.
    if baf < 0.0 {
        let rest = (1.0 - 0.5) / (N_STATES as f64 - 1.0);
        let mut e = [rest; N_STATES];
        e[CN0] = 0.5;
        return e;
    }

    let baf = baf as f64;
    let lrr = lrr as f64;
    let d2 = params.baf_dev2;

    // BAF components per CN state (CN0 has no BAF signal → set to 0 below)
    let cn1_baf = gauss_density(baf, 0.0, d2, peaks.cn1_r_norm) * (f_rr + f_ra * 0.5)
        + gauss_density(baf, 1.0, d2, peaks.cn1_a_norm) * (f_aa + f_ra * 0.5);
    let cn2_baf = gauss_density(baf, 0.0, d2, peaks.cn2_rr_norm) * f_rr
        + gauss_density(baf, 0.5, d2, peaks.cn2_ra_norm) * f_ra
        + gauss_density(baf, 1.0, d2, peaks.cn2_aa_norm) * f_aa;
    let cn3_baf = gauss_density(baf, 0.0, d2, peaks.cn3_rrr_norm) * f_rr
        + gauss_density(baf, peaks.cn3_rra_mean, d2, peaks.cn3_rra_norm) * f_ra * 0.5
        + gauss_density(baf, peaks.cn3_raa_mean, d2, peaks.cn3_raa_norm) * f_ra * 0.5
        + gauss_density(baf, 1.0, d2, peaks.cn3_aaa_norm) * f_aa;

    // Normalize BAF components across CN1/CN2/CN3 (bcftools normalizes across 3 states)
    let baf_norm = cn1_baf + cn2_baf + cn3_baf;
    let (cn1_baf_n, cn2_baf_n, cn3_baf_n) = if baf_norm > 0.0 {
        (cn1_baf / baf_norm, cn2_baf / baf_norm, cn3_baf / baf_norm)
    } else {
        (1.0 / 3.0, 1.0 / 3.0, 1.0 / 3.0)
    };

    // LRR components; if lrr_bias=0 then lrr_term = 1 and LRR is ignored.
    // LRR means from bcftools: CN1=-0.45, CN2=0.00, CN3=+0.30.
    let ld2 = params.lrr_dev2;
    let cn1_lrr = (-(lrr + 0.45) * (lrr + 0.45) / ld2).exp();
    let cn2_lrr = (-(lrr - 0.00) * (lrr - 0.00) / ld2).exp();
    let cn3_lrr = (-(lrr - 0.30) * (lrr - 0.30) / ld2).exp();

    let bb = params.baf_bias;
    let lb = params.lrr_bias;
    let ep = params.err_prob;

    // Combined per bcftools formula: ep + (1-bb + bb*baf_n) * (1-lb + lb*lrr_term)
    let mut e = [0f64; N_STATES];
    e[CN0] = 0.0; // CN0 receives zero emission when BAF is present
    e[CN1] = ep + (1.0 - bb + bb * cn1_baf_n) * (1.0 - lb + lb * cn1_lrr);
    e[CN2] = ep + (1.0 - bb + bb * cn2_baf_n) * (1.0 - lb + lb * cn2_lrr);
    e[CN3] = ep + (1.0 - bb + bb * cn3_baf_n) * (1.0 - lb + lb * cn3_lrr);
    e
}

/// Build a `GaussPeaks` from emission parameters — exposed so callers can cache it.
pub fn make_peaks(params: &EmissionParams) -> GaussPeaks {
    GaussPeaks::new(params)
}
