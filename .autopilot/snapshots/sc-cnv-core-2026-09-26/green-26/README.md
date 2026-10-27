# Grouped i6 Bayesian workflow candidate

Unpublished integrated candidate based on product7853e922: raw grouped-i6
inference, shared Dirichlet/categorical cell-mixture posteriors, strict normal
filtering, original rectangle identities and separate final reports. Sampling
uses six700-burn/1000-kept collapsed-Gibbs chains with deterministic indexed
streams. Likelihood scaling is computed once, outside the sampling hot loop.

Four critical analytic/rectangle/no-op tests and CLI contract checks accompany
the complete opt-in operation. They have not run natively at freeze time.
Standalone Rust1.91 formatting/parser checks are not compilation or acceptance.
The first real R Bayesian call completed but its collector failed on unnamed
calibration vectors; retain that original failed artifact and repair only the
collector. The completed Ward profiles remain separate evidence, with actual
membership inconsistencies awaiting a user compatibility decision.

Run this as one complete Bayesian feature gate on all four native platforms,
using two completed real JAGS repetitions and the predeclared posterior/state/
report comparison. Do not rerun per-helper or accepted grouped-HMM campaigns.
No native Bayesian, full-inferCNV, performance-advantage or publication claim
is justified by this snapshot. The product remains publish=false.
