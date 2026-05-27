//! 2-state Hidden Markov Model for runs-of-homozygosity detection.
//!
//! States: 0=HW (Hardy-Weinberg), 1=AZ (autozygous/ROH).
//! Emission probabilities are computed by the caller per site.
//! Viterbi decoding finds the most likely state sequence.
//! Forward-backward computes posterior state probabilities for quality scores.
//!
//! ## Forward array layout (matching bcftools HMM.c)
//!
//! bcftools allocates `fwd[n+1]` and stores the initial prior at index 0.
//! After the forward pass, `fwd[i+1]` = scaled P(obs[0..=i], state_i=s).
//! The backward pass multiplies `fwd[i+1]` by `bwd` in-place so that after
//! the full backward pass `fwd[i+1]` = posterior P(state_i=s | all obs).
//! vcfroh.c then reads `fwd[i*2 + state]` — note the index is i*2, not
//! (i+1)*2 — which means it reads the posterior of the PREVIOUS site
//! (or the initial prior for i=0). This one-site offset is the reason the
//! quality of the first site is always `phred_score(0.5) ≈ 3.0`.

/// Per-site emission probabilities: [P(data|HW), P(data|AZ)].
pub type Emission = [f64; 2];

/// Transition parameters for the 2-state HMM.
#[derive(Clone, Copy)]
pub struct TransParams {
    /// P(AZ|HW): probability of entering autozygous from HW at each site.
    pub t2az: f64,
    /// P(HW|AZ): probability of leaving autozygous to HW at each site.
    pub t2hw: f64,
}

/// One HMM site: 0-based genomic position + emission probabilities.
#[derive(Clone)]
pub struct Site {
    /// 0-based position on the current contig.
    pub pos: u32,
    /// [eprob_HW, eprob_AZ]
    pub eprob: Emission,
}

/// Result of running Viterbi + forward-backward on a sequence of sites.
pub struct HmmResult {
    /// Viterbi path: 0=HW, 1=AZ for each site.
    pub vpath: Vec<u8>,
    /// Posterior probability of the Viterbi state, using the bcftools one-site
    /// offset convention: quality[i] = phred_score(1 - fwd[i+1][vpath[i]]).
    /// For i=0 this equals phred_score(0.5) ≈ 3.0 (the initial prior).
    pub posterior: Vec<f64>,
}

/// Run Viterbi + forward-backward on a chromosome's site list.
///
/// Implements the bcftools HMM.c algorithm exactly:
/// - Forward array has n+1 entries; index 0 = initial prior = (0.5, 0.5).
/// - After forward pass, fwd[i+1] = P(obs[0..=i], state_i=s) (scaled).
/// - Backward pass multiplies fwd in-place: fwd[i+1] becomes posterior.
/// - vcfroh uses fwd[i*2 + state] → this is the posterior of site i-1 (or prior for i=0).
pub fn run(sites: &[Site], params: TransParams) -> HmmResult {
    let n = sites.len();

    // Emission shorthand
    let eprob = |i: usize| sites[i].eprob;

    // --- Transition matrix components (constant — no genetic map in this path) ---
    let stay_hw = 1.0 - params.t2az; // P(HW|HW)
    let enter_az = params.t2az; // P(AZ|HW)
    let stay_az = 1.0 - params.t2hw; // P(AZ|AZ)
    let enter_hw = params.t2hw; // P(HW|AZ)

    // --- Viterbi ---
    // Use log-space to avoid underflow on long chromosomes.
    let log_stay_hw = stay_hw.max(f64::MIN_POSITIVE).ln();
    let log_enter_az = enter_az.max(f64::MIN_POSITIVE).ln();
    let log_stay_az = stay_az.max(f64::MIN_POSITIVE).ln();
    let log_enter_hw = enter_hw.max(f64::MIN_POSITIVE).ln();

    // traceback[i][j] = which previous state leads to state j at site i
    let mut traceback: Vec<[u8; 2]> = Vec::with_capacity(n);

    let mut v = [
        0.5f64.ln() + eprob(0)[0].max(f64::MIN_POSITIVE).ln(),
        0.5f64.ln() + eprob(0)[1].max(f64::MIN_POSITIVE).ln(),
    ];
    traceback.push([0, 1]);

    for i in 1..n {
        let e0 = eprob(i)[0].max(f64::MIN_POSITIVE).ln();
        let e1 = eprob(i)[1].max(f64::MIN_POSITIVE).ln();

        let hw_from_hw = v[0] + log_stay_hw;
        let hw_from_az = v[1] + log_enter_hw;
        let (new_v0, prev0) = if hw_from_hw >= hw_from_az {
            (hw_from_hw + e0, 0u8)
        } else {
            (hw_from_az + e0, 1u8)
        };

        let az_from_hw = v[0] + log_enter_az;
        let az_from_az = v[1] + log_stay_az;
        let (new_v1, prev1) = if az_from_az >= az_from_hw {
            (az_from_az + e1, 1u8)
        } else {
            (az_from_hw + e1, 0u8)
        };

        v[0] = new_v0;
        v[1] = new_v1;
        traceback.push([prev0, prev1]);
    }

    // Traceback
    let mut vpath = vec![0u8; n];
    vpath[n - 1] = if v[1] > v[0] { 1 } else { 0 };
    for i in (0..n - 1).rev() {
        vpath[i] = traceback[i + 1][vpath[i + 1] as usize];
    }

    // --- Forward-backward (matching bcftools HMM.c layout) ---
    //
    // fwd has n+1 entries. fwd[0] = initial prior = (0.5, 0.5).
    // fwd[i+1] = forward probability for site i (scaled).
    // After backward pass, fwd[i+1] holds the posterior for site i.
    // vcfroh reads fwd[i*2 + state], so quality[i] uses fwd[i] = posterior of site i-1.
    let mut fwd = vec![[0f64; 2]; n + 1];
    fwd[0] = [0.5, 0.5]; // initial prior

    // Forward pass
    let mut scale = vec![1f64; n + 1];
    for i in 0..n {
        let f = fwd[i];
        let e = eprob(i);
        fwd[i + 1][0] = (f[0] * stay_hw + f[1] * enter_hw) * e[0];
        fwd[i + 1][1] = (f[0] * enter_az + f[1] * stay_az) * e[1];
        let s = fwd[i + 1][0] + fwd[i + 1][1];
        if s > 0.0 {
            scale[i + 1] = s;
            fwd[i + 1][0] /= s;
            fwd[i + 1][1] /= s;
        }
    }

    // Backward pass: multiply fwd[i+1] by bwd in-place (matching bcftools bwd loop).
    // bcftools backward loop: for i in 0..n, operates on fwd[(n-i)*nstates] (i.e. fwd[n], fwd[n-1], ..., fwd[1])
    // and bwd is computed from the previous (rightward) bwd step.
    let mut bwd = [1f64; 2]; // bwd starts at n-th site = all ones

    for i in 0..n {
        let site_idx = n - i; // fwd array index: n, n-1, ..., 1

        // Combine fwd * bwd at this position, normalize
        let p0 = fwd[site_idx][0] * bwd[0];
        let p1 = fwd[site_idx][1] * bwd[1];
        let norm = p0 + p1;
        if norm > 0.0 {
            fwd[site_idx][0] = p0 / norm;
            fwd[site_idx][1] = p1 / norm;
        }

        // Propagate bwd one step left (towards site site_idx-2):
        // bwd[j] = sum_k(bwd[k] * eprob[site_idx-1][k] * T[k,j]) / norm
        if i < n - 1 {
            let e = eprob(site_idx - 1); // emission at the site to the left
            let bwd_norm_val = {
                let b0 = bwd[0] * e[0] * stay_hw + bwd[1] * e[1] * enter_az;
                let b1 = bwd[0] * e[0] * enter_hw + bwd[1] * e[1] * stay_az;
                let s = b0 + b1;
                if s > 0.0 {
                    bwd = [b0 / s, b1 / s];
                }
                s
            };
            let _ = bwd_norm_val;
        }
    }

    // Now fwd[i+1] = posterior for site i. vcfroh uses fwd[i*2] (0-indexed from 0),
    // so quality for site i reads fwd[i][vpath[i]], which is the posterior of site i-1
    // (or the prior 0.5 for i=0).
    let mut posterior = vec![0f64; n];
    for i in 0..n {
        // bcftools: fwd[i*2 + vpath[i]] (0-indexed), which maps to fwd[i] in our array.
        // fwd[0] = prior = (0.5, 0.5); fwd[i] = posterior of site i-1 for i >= 1.
        posterior[i] = fwd[i][vpath[i] as usize];
    }

    HmmResult { vpath, posterior }
}

/// Convert posterior probability to Phred quality score, matching bcftools `phred_score()`.
///
/// bcftools: `score = -4.343 * log(1.0 - prob)`, clamped to [0, 99].
pub fn phred_score(posterior: f64) -> f64 {
    let complement = 1.0 - posterior;
    if complement <= 0.0 {
        99.0
    } else {
        (-4.343_f64 * complement.ln()).clamp(0.0, 99.0)
    }
}
