//! 4-state Hidden Markov Model for copy-number variation detection.
//!
//! States: CN0 (complete loss), CN1 (single-copy loss), CN2 (normal), CN3 (single-copy gain).
//! Viterbi decoding finds the most likely CN state sequence.
//! Forward-backward gives posterior probabilities used for region quality scores.
//!
//! The forward array has n+1 entries; index 0 holds the initial uniform prior (0.25 each).
//! After the forward pass fwd[i+1] = scaled P(obs[0..=i], state_i=s).
//! The backward pass multiplies fwd in-place so fwd[i+1] becomes the posterior.
//! Per-region quality is computed from the mean posterior of the dominant state,
//! converted to a Phred score as bcftools does: -4.343 * ln(1 - mean_posterior).

pub const N_STATES: usize = 4;

/// Indices into the 4-state vector.
pub const CN0: usize = 0;
pub const CN1: usize = 1;
pub const CN2: usize = 2;
pub const CN3: usize = 3;

/// Per-site emission probabilities: [P(obs|CN0), P(obs|CN1), P(obs|CN2), P(obs|CN3)].
pub type Emission = [f64; N_STATES];

/// Result of running Viterbi + forward-backward on one chromosome.
pub struct HmmResult {
    /// Viterbi path: 0=CN0, 1=CN1, 2=CN2, 3=CN3 per site.
    pub vpath: Vec<u8>,
    /// Posterior probability of the Viterbi state at each site (forward-backward).
    pub posterior: Vec<f64>,
}

/// Run Viterbi + forward-backward on a slice of per-site emissions.
///
/// `ij_prob` is the off-diagonal transition probability P(j|i) for i≠j.
/// Diagonal = 1 - ij_prob*(N_STATES-1).  Matches bcftools vcfcnv.c transition matrix.
#[allow(clippy::needless_range_loop)] // index-based loops are load-bearing: state index ↔ transition-matrix diagonal
pub fn run(emissions: &[Emission], ij_prob: f64) -> HmmResult {
    let n = emissions.len();
    if n == 0 {
        return HmmResult {
            vpath: vec![],
            posterior: vec![],
        };
    }

    let stay = 1.0 - ij_prob * (N_STATES as f64 - 1.0);
    let log_stay = stay.max(f64::MIN_POSITIVE).ln();
    let log_jump = ij_prob.max(f64::MIN_POSITIVE).ln();

    // --- Viterbi (log-space) ---
    let log_prior = (1.0 / N_STATES as f64).ln();

    let mut v = [0f64; N_STATES];
    for s in 0..N_STATES {
        v[s] = log_prior + emissions[0][s].max(f64::MIN_POSITIVE).ln();
    }

    let mut traceback: Vec<[u8; N_STATES]> = Vec::with_capacity(n);
    {
        let mut init_tb = [0u8; N_STATES];
        for s in 0..N_STATES {
            init_tb[s] = s as u8;
        }
        traceback.push(init_tb);
    }

    for i in 1..n {
        let mut new_v = [0f64; N_STATES];
        let mut tb = [0u8; N_STATES];
        for dst in 0..N_STATES {
            let e = emissions[i][dst].max(f64::MIN_POSITIVE).ln();
            let mut best_val = f64::NEG_INFINITY;
            let mut best_src = 0u8;
            for src in 0..N_STATES {
                let trans = if src == dst { log_stay } else { log_jump };
                let val = v[src] + trans;
                if val > best_val {
                    best_val = val;
                    best_src = src as u8;
                }
            }
            new_v[dst] = best_val + e;
            tb[dst] = best_src;
        }
        v = new_v;
        traceback.push(tb);
    }

    // Traceback
    let mut vpath = vec![0u8; n];
    vpath[n - 1] = v
        .iter()
        .enumerate()
        .max_by(|a, b| a.1.partial_cmp(b.1).unwrap())
        .map(|(s, _)| s as u8)
        .unwrap();
    for i in (0..n - 1).rev() {
        vpath[i] = traceback[i + 1][vpath[i + 1] as usize];
    }

    // --- Forward-backward (scaled, matching bcftools HMM.c layout) ---
    // fwd has n+1 entries; fwd[0] = uniform prior.
    let prior = 1.0 / N_STATES as f64;
    let mut fwd = vec![[0f64; N_STATES]; n + 1];
    fwd[0] = [prior; N_STATES];

    // Forward pass
    for i in 0..n {
        let f = fwd[i];
        let e = emissions[i];
        let mut sum = 0.0;
        for dst in 0..N_STATES {
            let mut acc = 0.0;
            for src in 0..N_STATES {
                let trans = if src == dst { stay } else { ij_prob };
                acc += f[src] * trans;
            }
            fwd[i + 1][dst] = acc * e[dst];
            sum += fwd[i + 1][dst];
        }
        if sum > 0.0 {
            for dst in 0..N_STATES {
                fwd[i + 1][dst] /= sum;
            }
        }
    }

    // Backward pass: multiply fwd[i+1] by bwd in-place.
    let mut bwd = [1f64; N_STATES];

    for i in 0..n {
        let site_idx = n - i; // walks n, n-1, …, 1

        // Combine fwd * bwd and normalise into posterior
        let mut norm = 0.0;
        for s in 0..N_STATES {
            fwd[site_idx][s] *= bwd[s];
            norm += fwd[site_idx][s];
        }
        if norm > 0.0 {
            for s in 0..N_STATES {
                fwd[site_idx][s] /= norm;
            }
        }

        // Propagate bwd one step left
        if i < n - 1 {
            let e = emissions[site_idx - 1];
            let mut new_bwd = [0f64; N_STATES];
            let mut bwd_sum = 0.0;
            for src in 0..N_STATES {
                let mut acc = 0.0;
                for dst in 0..N_STATES {
                    let trans = if src == dst { stay } else { ij_prob };
                    acc += bwd[dst] * e[dst] * trans;
                }
                new_bwd[src] = acc;
                bwd_sum += acc;
            }
            if bwd_sum > 0.0 {
                for s in 0..N_STATES {
                    new_bwd[s] /= bwd_sum;
                }
            }
            bwd = new_bwd;
        }
    }

    // Collect posteriors; fwd[i+1] = posterior of site i.
    let mut posterior = vec![0f64; n];
    for i in 0..n {
        posterior[i] = fwd[i + 1][vpath[i] as usize];
    }

    HmmResult { vpath, posterior }
}

/// Convert posterior probability to Phred quality, matching bcftools `phred_score()`.
///
/// score = -4.343 * ln(1 - prob), clamped to [0, 99].
pub fn phred_score(posterior: f64) -> f64 {
    let complement = 1.0 - posterior;
    if complement <= 0.0 {
        99.0
    } else {
        (-4.343_f64 * complement.ln()).clamp(0.0, 99.0)
    }
}
