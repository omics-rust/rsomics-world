//! Gaussian mixture model fitting for BAF distributions.
//!
//! Implements Levenberg-Marquardt non-linear least squares for bounded Gaussian
//! components, mirroring bcftools/peakfit.c (MIT).  Each Gaussian is parameterised
//! as  f(x) = a² · exp(−(x − b)² / c²)  so that a² is always non-negative.
//!
//! The residual weighting factor is 1/0.01 (= 100), matching peakfit.c exactly.

const RESIDUAL_SCALE: f64 = 1.0 / 0.01;
const MAX_ITER: usize = 500;
const CONVERGE_TOL: f64 = 1e-8;

/// A single Gaussian component with optional parameter bounds.
#[derive(Clone)]
pub struct Gaussian {
    /// [a, b, c] — amplitude, centre, sigma
    pub params: [f64; 3],
    pub bounds: Option<[f64; 6]>, // [a_lo,a_hi, b_lo,b_hi, c_lo,c_hi]
}

impl Gaussian {
    pub fn eval(&self, x: f64) -> f64 {
        let [a, b, c] = self.params;
        let a2 = a * a;
        let z = (x - b) / c;
        a2 * (-z * z).exp()
    }

    fn clamp(&mut self) {
        if let Some(bnd) = &self.bounds {
            self.params[0] = self.params[0].clamp(bnd[0], bnd[1]);
            self.params[1] = self.params[1].clamp(bnd[2], bnd[3]);
            self.params[2] = self.params[2].clamp(bnd[4], bnd[5]);
        }
    }
}

/// Evaluate the sum of all Gaussians at x.
pub fn eval_mixture(gs: &[Gaussian], x: f64) -> f64 {
    gs.iter().map(|g| g.eval(x)).sum()
}

/// Goodness-of-fit: sum of squared scaled residuals, normalised by N.
pub fn gof(gs: &[Gaussian], xs: &[f64], ys: &[f64]) -> f64 {
    let n = xs.len() as f64;
    if n == 0.0 {
        return 0.0;
    }
    xs.iter()
        .zip(ys.iter())
        .map(|(&x, &y)| {
            let r = (eval_mixture(gs, x) - y) * RESIDUAL_SCALE;
            r * r
        })
        .sum::<f64>()
        / n
}

/// Gradient of gof w.r.t. each Gaussian parameter, computed analytically.
fn gradient(gs: &[Gaussian], xs: &[f64], ys: &[f64]) -> Vec<f64> {
    let n = xs.len() as f64;
    let scale2 = RESIDUAL_SCALE * RESIDUAL_SCALE;
    let mut grad = vec![0.0f64; gs.len() * 3];
    for (&x, &y) in xs.iter().zip(ys.iter()) {
        let f = eval_mixture(gs, x);
        let r = (f - y) * 2.0 * scale2 / n;
        for (gi, g) in gs.iter().enumerate() {
            let [a, b, c] = g.params;
            let z = (x - b) / c;
            let exp = (-z * z).exp();
            let a2 = a * a;
            // df/da = 2a·exp
            grad[gi * 3] += r * 2.0 * a * exp;
            // df/db = 2a²·(x-b)/c² · exp
            grad[gi * 3 + 1] += r * 2.0 * a2 * (x - b) / (c * c) * exp;
            // df/dc = 2a²·(x-b)²/c³ · exp
            grad[gi * 3 + 2] += r * 2.0 * a2 * (x - b) * (x - b) / (c * c * c) * exp;
        }
    }
    grad
}

/// Hessian diagonal approximation (Gauss-Newton J^T J diagonal).
fn hessian_diag(gs: &[Gaussian], xs: &[f64]) -> Vec<f64> {
    let n = xs.len() as f64;
    let scale2 = RESIDUAL_SCALE * RESIDUAL_SCALE;
    let mut hd = vec![0.0f64; gs.len() * 3];
    for &x in xs.iter() {
        for (gi, g) in gs.iter().enumerate() {
            let [a, b, c] = g.params;
            let z = (x - b) / c;
            let exp = (-z * z).exp();
            let a2 = a * a;
            let ja = 2.0 * a * exp;
            let jb = 2.0 * a2 * (x - b) / (c * c) * exp;
            let jc = 2.0 * a2 * (x - b) * (x - b) / (c * c * c) * exp;
            hd[gi * 3] += ja * ja * scale2 / n;
            hd[gi * 3 + 1] += jb * jb * scale2 / n;
            hd[gi * 3 + 2] += jc * jc * scale2 / n;
        }
    }
    hd
}

/// Levenberg-Marquardt step on one set of components.
///
/// Returns the final GOF value.
pub fn fit_lm(gs: &mut Vec<Gaussian>, xs: &[f64], ys: &[f64]) -> f64 {
    if xs.is_empty() {
        return 0.0;
    }
    let mut lambda = 1e-3_f64;
    let mut cost = gof(gs, xs, ys);

    for _ in 0..MAX_ITER {
        let grad = gradient(gs, xs, ys);
        let hd = hessian_diag(gs, xs);

        // LM update: Δp = -(H + λI)⁻¹ · g
        let mut delta = vec![0.0f64; grad.len()];
        for i in 0..grad.len() {
            let denom = hd[i] + lambda;
            if denom.abs() > 1e-30 {
                delta[i] = -grad[i] / denom;
            }
        }

        // Proposed step.
        let mut proposed = gs.clone();
        for (pi, g) in proposed.iter_mut().enumerate() {
            g.params[0] += delta[pi * 3];
            g.params[1] += delta[pi * 3 + 1];
            g.params[2] += delta[pi * 3 + 2];
            g.clamp();
        }
        let new_cost = gof(&proposed, xs, ys);

        if new_cost < cost {
            *gs = proposed;
            cost = new_cost;
            lambda /= 10.0;
        } else {
            lambda *= 10.0;
        }

        // Convergence: gradient magnitude below tolerance.
        let gnorm = grad.iter().map(|g| g * g).sum::<f64>().sqrt();
        if gnorm < CONVERGE_TOL {
            break;
        }
    }
    cost
}

/// Monte Carlo restarts: run LM from `nmc` random initialisations within
/// the supplied parameter bounds; return the best (lowest GOF) result.
///
/// Mirrors `peakfit_set_mc` with nmc=50.
pub fn fit_mc(
    gs_init: &[Gaussian],
    xs: &[f64],
    ys: &[f64],
    nmc: usize,
    rng: &mut u64,
) -> (Vec<Gaussian>, f64) {
    let (mut best_gs, mut best_cost) = {
        let mut tmp = gs_init.to_vec();
        let cost = fit_lm(&mut tmp, xs, ys);
        (tmp, cost)
    };

    for _ in 0..nmc {
        let mut trial = gs_init.to_vec();
        for g in trial.iter_mut() {
            if let Some(bnd) = &g.bounds {
                g.params[0] = bnd[0] + lcg(rng) * (bnd[1] - bnd[0]);
                g.params[1] = bnd[2] + lcg(rng) * (bnd[3] - bnd[2]);
                g.params[2] = bnd[4] + lcg(rng) * (bnd[5] - bnd[4]);
            }
        }
        let cost = fit_lm(&mut trial, xs, ys);
        if cost < best_cost {
            best_cost = cost;
            best_gs = trial;
        }
    }
    (best_gs, best_cost)
}

/// Simple LCG PRNG — deterministic, no external deps.
/// Returns a float in [0, 1).
fn lcg(state: &mut u64) -> f64 {
    *state = state
        .wrapping_mul(6_364_136_223_846_793_005)
        .wrapping_add(1_442_695_040_888_963_407);
    ((*state >> 11) as f64) / (1u64 << 53) as f64
}

/// Format a Gaussian mixture as a function string (mirrors peakfit_sprint_func).
pub fn sprint_func(gs: &[Gaussian]) -> String {
    gs.iter()
        .map(|g| {
            let [a, b, c] = g.params;
            format!("{}**2 * exp(-(x-{})**2/{}**2)", a, b, c)
        })
        .collect::<Vec<_>>()
        .join(" + ")
}
