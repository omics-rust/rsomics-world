# Isolated native i6 decoder task

Implement only src/cnv/hmm/decode.rs and, if useful, a private sibling
src/cnv/hmm/log_normal.rs in rsomics-sc. Do not edit normal.rs, rng.rs,
spline.rs, any mod.rs, CLI, Cargo metadata, notices or other files.
Main owns integration and the single whole-feature review.

Interfaces:

```rust
pub(super) struct Model { /* private fields */ }
impl Model {
    pub(super) fn new(means: [f64; 6], sds: [f64; 6], transition: f64)
        -> rsomics_common::Result<Self>;
    pub(super) fn decode(&self, observations: &[f64])
        -> rsomics_common::Result<Vec<u8>>;
}
```

The enclosing HMM config validates finite transition in0..=0.2. Calibrated
means/SDs must be finite; SDs nonnegative, their median strictly positive.
Use their median as one shared SD. Six states are one-based1..6, neutral3.
Transition diagonal1-5t, every other entry t. Initial probabilities likewise
favor neutral3 with1-5t and t for others. Zero probabilities map to -Inf.

Read actual pinned R/inferCNV_HMM.R:230-264,1101-1175 before implementation.
Archive: /Volumes/Zane's HDD/rsomics-fixtures/evidence/infercnv-ingestion-2026-09-30/run-36707344893/artifact/infercnv-source.tar.gz;
root infercnv-b421d9405c97a309b081ef86d455e976df93eae4/.
This is NOT Gaussian-density Viterbi. For each observation/state compute
log upper-tail normal probability at abs(x-mean)/median_sd, invert its
negative, normalize the six weights, then log. Preserve normalization,
first-maximum ties, recurrence and traceback. Singleton sequence returns3.
Empty input should error. Preserve upstream unreachable-final-score failure;
propagate numerical/allocation errors rather than fabricate states.

Use rolling six-state scores plus compact u8 predecessor rows, not full
emission and score matrices. Caller budgets overall chromosome/group work;
fallible reserve is appropriate. The native model is built once per group.
Inputs come from validated immutable CnvState; avoid repeated shape/ID checks.

Stable log survival must not underflow for biologically possible large
deviations. Reuse/adapt compatible math from the historical team-owned
rsomics-normaltest/src/ndtr.rs where appropriate, computing logs directly
for tails. Do not mechanically copy GPL R code or add a dependency/foundation.
Report exact attribution/notice requirements to main; do not edit notices.
Keep source comments rare, only non-obvious numerical invariants/reasons.

Write a few meaningful model/path tests, including neutral/singleton,
transition/tie behavior and a reference finite tail case. No exhaustive grids,
mock oracle or separate per-helper CI/review. Real installed-R whole-workflow
goldens and four-native validation will be integrated by main. New code is
not accepted merely because it is written or formatted.

Product=/Volumes/KIOXIA/Documents/omics-rust/rsomics-sc.
World=/Volumes/Zane's HDD/Documents/rsomics-world.
Cargo home=/Volumes/KIOXIA/Developments/cargo-home;
target=/Volumes/KIOXIA/Developments/cargo-target;
scratch=/Volumes/KIOXIA/Developments/tmp.
Boot APFS92%: no local builds, tests that compile Rust, installs or downloads.
Only existing external Rust1.91 rustfmt is allowed for syntax/format checking.
Do not delete, stage, commit, publish, dispatch CI or spawn subagents.

Write a concise implementation/verification-limits report to
.autopilot/state/hmm-decoder-report-2026-10-27.md in world. Return only status,
files, concerns and report path. Explicitly say uncompiled if not compiled.
