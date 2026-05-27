//! 4-state Hidden Markov Model for copy-number variation detection.
//!
//! States: CN0 (complete loss), CN1 (single-copy loss), CN2 (normal), CN3 (single-copy gain).
//! Viterbi decoding finds the most likely CN state sequence.
//! Forward-backward gives posterior probabilities used for region quality scores.
//!
//! Transition probabilities are distance-dependent: T^d where d is the number of
//! intervening genomic positions between consecutive SNP sites.  For the symmetric
//! 4-state matrix this has the closed-form:
//!
//!   T^d[i,i] = 1/N + (N-1)/N * λ^d       (diagonal)
//!   T^d[i,j] = 1/N - 1/N      * λ^d       (off-diagonal)
//!
//! where λ = stay - jump = 1 - N*ij_prob and N = N_STATES.
//!
//! This matches bcftools HMM.c which pre-computes T^1 … T^10000 and
//! applies them according to the gap between consecutive VCF sites.

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

/// Compute the distance-dependent transition probabilities for `pos_diff` skipped positions.
///
/// Returns `(diag, off)` = (T^d[i,i], T^d[i,j] for i≠j).
/// When `pos_diff == 0` (consecutive sites), d=1, i.e. one transition step.
#[inline]
fn tprob(pos_diff: u32, ij_prob: f64) -> (f64, f64) {
    // λ = eigenvalue of (T - 1/N * J) where J is the all-ones matrix.
    // For symmetric N-state matrix: λ = 1 - N * ij_prob.
    let n = N_STATES as f64;
    let lambda = 1.0 - n * ij_prob;
    // d = number of transition steps = 1 (consecutive) + number of skipped positions
    let d = (pos_diff + 1) as f64;
    let lambda_d = lambda.powf(d);
    let diag = 1.0 / n + (n - 1.0) / n * lambda_d;
    let off = 1.0 / n - 1.0 / n * lambda_d;
    (diag, off)
}

/// Run Viterbi + forward-backward on a slice of per-site emissions.
///
/// `positions` are 0-based genomic positions (one per site).  The number of
/// skipped positions between sites[i] and sites[i+1] is `sites[i+1] - sites[i] - 1`,
/// which determines the transition matrix power used for that step.
///
/// `ij_prob` is the base off-diagonal transition probability P(j|i, 1 step) for i≠j.
/// Diagonal (1 step) = 1 - ij_prob*(N_STATES-1).  Matches bcftools vcfcnv.c defaults.
#[allow(clippy::needless_range_loop)] // state index is load-bearing for diagonal/off-diagonal selection
pub fn run(emissions: &[Emission], positions: &[u32], ij_prob: f64) -> HmmResult {
    let n = emissions.len();
    assert_eq!(
        n,
        positions.len(),
        "emissions and positions must have the same length"
    );
    if n == 0 {
        return HmmResult {
            vpath: vec![],
            posterior: vec![],
        };
    }

    let prior = 1.0 / N_STATES as f64;

    // --- Viterbi (probability space, normalized at each step to prevent underflow) ---
    // This matches bcftools hmm_run_viterbi() which also works in probability space.
    let mut v = [prior; N_STATES];
    for s in 0..N_STATES {
        v[s] *= emissions[0][s].max(f64::MIN_POSITIVE);
    }
    // Normalize initial step
    let vsum: f64 = v.iter().sum();
    if vsum > 0.0 {
        for s in 0..N_STATES {
            v[s] /= vsum;
        }
    }

    // traceback[i][dst] = best source state for arriving at dst at step i
    let mut traceback: Vec<[u8; N_STATES]> = Vec::with_capacity(n);
    {
        let mut init_tb = [0u8; N_STATES];
        for s in 0..N_STATES {
            init_tb[s] = s as u8;
        }
        traceback.push(init_tb);
    }

    for i in 1..n {
        let pos_diff = positions[i]
            .saturating_sub(positions[i - 1])
            .saturating_sub(1);
        let (stay, jump) = tprob(pos_diff, ij_prob);

        let mut new_v = [0f64; N_STATES];
        let mut tb = [0u8; N_STATES];
        for dst in 0..N_STATES {
            let e = emissions[i][dst].max(f64::MIN_POSITIVE);
            let mut best_val = 0.0_f64;
            let mut best_src = 0u8;
            for src in 0..N_STATES {
                let trans = if src == dst { stay } else { jump };
                let val = v[src] * trans;
                if val > best_val {
                    best_val = val;
                    best_src = src as u8;
                }
            }
            new_v[dst] = best_val * e;
            tb[dst] = best_src;
        }
        // Normalize to prevent underflow
        let vnorm: f64 = new_v.iter().sum();
        if vnorm > 0.0 {
            for s in 0..N_STATES {
                new_v[s] /= vnorm;
            }
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
    let mut fwd = vec![[0f64; N_STATES]; n + 1];
    fwd[0] = [prior; N_STATES];

    // Forward pass
    for i in 0..n {
        let pos_diff = if i == 0 {
            0
        } else {
            positions[i]
                .saturating_sub(positions[i - 1])
                .saturating_sub(1)
        };
        let (stay, jump) = tprob(pos_diff, ij_prob);

        let f = fwd[i];
        let e = emissions[i];
        let mut sum = 0.0;
        for dst in 0..N_STATES {
            let mut acc = 0.0;
            for src in 0..N_STATES {
                let trans = if src == dst { stay } else { jump };
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

        // Propagate bwd one step left (using distance-adjusted transitions)
        if i < n - 1 {
            let pos_diff = positions[site_idx - 1]
                .saturating_sub(positions[site_idx - 2])
                .saturating_sub(1);
            let (stay, jump) = tprob(pos_diff, ij_prob);

            let e = emissions[site_idx - 1];
            let mut new_bwd = [0f64; N_STATES];
            let mut bwd_sum = 0.0;
            for src in 0..N_STATES {
                let mut acc = 0.0;
                for dst in 0..N_STATES {
                    let trans = if src == dst { stay } else { jump };
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
