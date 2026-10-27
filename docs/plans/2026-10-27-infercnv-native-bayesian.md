# Native grouped-i6 Bayesian workflow implementation plan

> Use executing-plans inline for integration and one isolated sampler worker.
> Run one whole-feature review after the actual operation and R comparison,
> not a separate native campaign for each numerical helper.

**Goal:** Extend the working raw grouped-i6 workflow through actual Bayesian
region filtering/reassignment while retaining original HMM predictions.

**Architecture:** Keep the model in rsomics-sc. Its likelihood uses existing
transformed-spike means/raw SDs, one categorical state per cell, and one shared
Dirichlet(1,...,1) mixture per region. Integrate out theta during Gibbs updates,
then estimate its posterior mean from conditional counts; this targets the
same BUGS model without inventing a gamma implementation or copying JAGS.
Expose public configuration only after actual R posterior/filter evidence.

**Tech stack:** Rust1.91, existing private MT generator, typed HMM regions,
existing chunk workers; real infercnv1.28.0/R4.6.1/JAGS oracle.

**Spec:** AGENTS.md, the next-workflow contract and pinned
R/inferCNV_BayesNet.R plus inst/BUGS_Mixture_Model. Source archive and HMM
accepted identities remain those in infercnv-hmm-accepted-2026-10-27.md.

## Constraints and review focus

- No new foundation, dependency, empty CLI operation or publication.
- External paths only; boot APFS>80% forbids local compilation/installation.
- Preserve expression14, original HMM states17 and filtered states separately.
- Raw spike SDs, not group-fitted median SDs, enter the normal likelihood.
- Posterior mixture probabilities differ from per-cell state frequencies.
- Filtering is strictly P(normal)>threshold; retained normal maxima may be
  reassigned to state3 without dropping the original region ID.
- Retain original region IDs/ranges; do not regenerate/merge adjacent runs.
- Native keyed streams must reproduce across worker counts. Do not claim
  JAGS bit streams or bitwise stochastic compatibility.
- Statistical agreement and convergence/near-threshold behavior are checked
  against actual retained coda samples before public acceptance.

## Task 1: Collapsed categorical/Dirichlet sampler

Worker owns only product src/cnv/hmm/bayes/sampler.rs, not module wiring.

Interfaces:

    pub(super) struct Posterior {
        pub probabilities: [f64; 6],
        pub cell_probabilities: Vec<[f64; 6]>,
        pub chain_probabilities: [[f64; 6]; 6],
    }
    pub(super) fn sample(log_likelihoods: &[[f64; 6]], seed: u32)
        -> rsomics_common::Result<Posterior>;
    pub(super) fn stream_seed(seed: u32, stream: usize) -> u32;

Use six chains initialized in each respective state;700 burn-in sweeps then
1000 kept sweeps per chain. A cell update weights state s by
(other-cell-count[s]+1)*likelihood[cell][s]. Accumulate cell frequencies and
E(theta[s]|epsilon)=(count[s]+1)/(cells+6), without storing sample histories.
Expose CHAINS/BURN_IN/SAMPLES constants to the parent as pub(super).

- [ ] Write two critical tests before implementation: concentrated likelihoods
  at1/4 cells distinguish mixture priors from epsilon (2/7 and1/2 for normal);
  one-cell likelihood weights1..6 give epsilon probabilities1..6 divided21
  and mixture probabilities22..27 divided147 within declared MC tolerance.
- [ ] Implement stable log-weight updates, fallible allocation and keyed
  region/chain streams using existing private CalibrationRng. Format only
  locally; execute these tests in the complete native workflow gate.

## Task 2: Likelihood, filtering and coherent workflow integration

Root owns product src/cnv/hmm/bayes/mod.rs, HMM wrapper, CLI/report wiring,
notices and final comparison controller. No overlap with sampler ownership.

Consume state plus validated original regions from the existing HMM workflow.
Compute each cell/region's normal log likelihood from a stable centered sum
of squares, once per state, not once per MCMC sweep. Parallelize independent
regions with deterministic indexed output order. Keep missing-CNV results a
real no-op, not an invented posterior. Preserve original rectangle identities
through filtering and reassignment.

- [ ] Pin exact pipeline rectangles/strict threshold with a critical workflow
  test, implement actual posteriors and final states, then wire one usable
  opt-in CLI operation. No advertised flag until the operation exists.
- [ ] Compare posterior means with actual R repetitions/coda uncertainty,
  compare filtering/reporting where the oracle is stable, and verify native
  1/4-worker identity. Do not relax a failed criterion without diagnosing it.
- [ ] Run four-native fmt/strict Clippy/tests and actual workflow comparisons,
  retain timing/RSS, review once, commit the owning files and continue the
  remaining inferCNV/portfolio work. Publication requires the broader gates.

## Execution ruling

The explicit BUGS model and public filtering source already establish the
private mathematical contract. Develop the private sampler concurrently with
the installed witness, but hold public Bayesian API/acceptance on its actual
posterior contract. This refines the earlier oracle-first ruling without
committing other products to an uncertain abstraction. Cost if interpretation
is wrong: refactor the unpublished private candidate. Ward membership policy
remains on hold pending actual source outputs and any required user decision.

## Predeclared whole-workflow comparison

Before executing the native candidate, compare each region/state mixture mean
to each of the two actual JAGS repetitions using
`0.002 + 6*sqrt(R_batch_MCSE^2 + native_chain_SE^2)`. R MCSE uses120
nonoverlapping50-draw batch means over the six1000-draw chains; native SE is
the sample SD of six chain means divided by sqrt(6). These are descriptive
Monte Carlo diagnostics, not proofs of convergence. Preserve the original
samples and require their reduced means to agree with the recorded actual
posterior tables. Cell-state frequencies use a fixed absolute0.05 tolerance.

Require exact removals and original rectangle IDs, memberships, coordinates
and report order against both repetitions. Reassigned states, filtered paths
and adjusted-table state columns are exact where the actual source resolves
one maximum. Source-indeterminate maxima are classified from R alone using
interval radius0.002+6*R_batch_MCSE: a state is admissible only when its upper
bound reaches the largest lower bound. Require the native posterior's actual
maximum to lie in that set, explicitly label the region and both choices,
and do not claim exact report equality there. All unchanged posterior/error
criteria still apply. The source-only evidence/ruling is retained in
infercnv-bayesian-statistical-contract-2026-10-27.md before any native comparison.

Also compare unmodified expression/HMM outputs using the accepted grouped
witness. Native1/4-worker reports must be identical apart from their recorded
thread setting. These descriptive intervals do not establish convergence,
JAGS/native bit streams, arbitrary-input parity or biological validation.
