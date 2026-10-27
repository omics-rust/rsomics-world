# Integrated inferCNV HMM workflow

Continue the full portfolio objective after integrated denoise; this is not
a revised definition of inferCNV completion. Execute inline on main. Use
bounded independent source work, critical integrated tests and one feature
review, not a separate campaign for each helper. The product stays unpublished.

## Source and architecture

Authoritative source: infercnv1.28.0, commit
b421d9405c97a309b081ef86d455e976df93eae4, retained archive SHA
b2a1b6f8cc09dc04562513e3cd877b3c97a6efcd5ff92cb5e0763660905e3207.
Root inspected the actual HMM, hidden-spike, mean/variance simulation and
pipeline sources; the independent read-only inventory agrees.

- i6 calibration starts at step3 after depth normalization, before log2.
  The hidden spike follows reference subtraction, smoothing, recentering and
  inverse log2; it cannot be reconstructed from a denoised step22 matrix.
- `meanvar` uses all observation/reference groups to fit mean/variance and
  dropout splines; each modeled reference group supplies gene means and
  produces100 normal plus100 spike cells. Without references, observations
  are the proxy. See R/inferCNV_hidden_spike.R:3-164 and
  R/inferCNV_meanVarSim.R:1-54,105-158,178-210.
- Emission means/SDs and cell-count regressions come from transformed spikes.
  The actual bootstrap uses rowMeans of an ncells-by100 matrix, not the usual
  bootstrap mean across ncells. Do not silently correct that axis.
  See R/inferCNV_HMM.R:45-97,154-208,586-613.
- The adjusted Viterbi decoder replaces SDs by their median and uses inverse
  negative-log upper-tail-normal scores, not normal densities. Integer state3
  is neutral; six-state transition diagonals are1-5t, other entries t.
  Singleton chromosomes return neutral. See R/inferCNV_HMM.R:230-264,1101-1175.
- Step17 HMM states are separate from expression. The run return at that
  checkpoint is the expression object; state/region outputs must be captured
  from the actual HMM result and its reports. Bayesian filtering is steps18/19,
  proxy visualization20, denoise22. See R/inferCNV_ops.R:1235-1384.

The first integrated native HMM operation is raw-to-grouped-sample i6 states
and region reports, explicitly without Bayesian filtering. Subclusters,
per-cell/i3 modes, Bayesian reassignment, plots and complete upstream-default
behavior remain required later work, not implied by this slice. Existing
samples grouping and preprocessing remain the consuming workflow.

Keep source expression, calibrated model, state paths and region tables
distinct inside rsomics-sc. No public HMM/RNG/spline foundation is justified
by this single consumer. Use disjoint chromosome/group work for parallelism
where it preserves calibration reproducibility. No unfinished public flags.

Historical assets: rsomics-infercnv39388265 supplies input fixtures only;
discard its five-step approximation. rsomics-hmm6bac6d99 supplies a reviewed
log-space recurrence/first-max pattern, not emissions or calibration.
rsomics-hmm-decodecc870858 has permissive parsing and no real oracle; discard
its CLI. rsomics-vcf-cnv's four-state distance-dependent model is not an i6
implementation. Preserve provenance/attribution when adapting selected code.

## Execution

- [ ] Capture real installed-package raw HMM workflows using the existing
  pinned R4.6.1 environment. Cover the real shipped full input with grouped
  references, no-reference proxy and aggregated simulated normals. Retain the
  synthetic grouped input as the verified upstream spline-failure case, not
  a fabricated successful calibration. Retain expression/state identities,
  transformed-spike calibration, fitted coefficients and actual step17 tables.
- [ ] Characterize the actual pooled-HMM branch separately: its source uses
  c(all_observations=unlist(obs),refs), unlike step15's list-wrapped group.
  Record actual errors/group membership rather than assume equivalence.
- [ ] Record the seeded calibration/RNG contract from that evidence before
  exposing a public model format. Do not claim cross-language seed parity
  merely because both implementations accept a seed.
- [ ] Implement calibration plus mirrored preprocessing, adjusted decoding
  and regions as one usable product operation, keeping HMM states separate
  from unmodified expression and later denoise output.
- [ ] Compare actual raw native workflows to the retained real package,
  exercise critical group/chromosome/state/report boundaries and worker
  consistency, then run four-native acceptance and relevant whole-path
  timing/RSS. Review once and commit the owning files.
- [ ] Continue subclusters, i3/per-cell behavior, Bayesian filtering,
  persisted state/reporting and other product families; no phase-boundary stop.

Mac boot APFS currently exceeds80%. All files, fixtures, Cargo/TMP paths and
scratch remain external. No local compile, package installation or dependency
download; reuse native hosted runners. Preserve existing scratch/evidence.

Ruling after run40468601684: the synthetic preprocessing fixture is not a
successful i6 calibration fixture. Actual infercnv fails in
smooth.spline(logv~logm), before decoding, with "smoothing parameter value too
small". The same actual stage3 object and source mean/variance-table function
reproduce the error in R4.6.1 on4090 (1784 rows,16 unique log means). Preserve
that exact error/call as a negative case; unrelated errors still fail the
workflow. Move positive proxy/aggregation coverage to the real shipped counts,
without changing meanvar, spline settings, decoding, source pins or numerical
criteria. Cost: the hand-designed synthetic input does not supply successful
HMM state goldens; its existing creation/preprocessing/denoise evidence remains
unchanged. Full native HMM and all remaining products are still required.
