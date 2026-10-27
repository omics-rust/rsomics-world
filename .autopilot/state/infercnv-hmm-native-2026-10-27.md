# Native HMM calibration work in progress

Scope remains the full portfolio and full inferCNV. This is implementation
state, not native acceptance, an accepted public API or a release. Follow the integrated
HMM plan and infercnv-hmm-workflow-2026-10-27.md for real-package execution.

Owning product HEADc5db7c74 is the accepted denoise/docs baseline. The current
uncommitted candidate now contains the complete first raw-to-grouped-i6
workflow under src/cnv/hmm/, the private shared preprocessing tail, CLI/report
wiring, a critical CLI configuration test and updated notices. These are
owned work, not historical debris. Do not stage unrelated changes. Cargo
dependencies and lockfile are unchanged. The new candidate is not accepted
or published; see the latest integration entry below.

Implemented candidate source:

- normal.rs: MIT AS241 ordinary-probability inverse normal, selected from
  John Burkardt's licensed primary implementation. Original algorithm is
  Wichura1988. No log(p)-then-exp round trip or Box-Muller replacement.
- rng.rs: R-aligned state initialization, MT twist/tempering, endpoint fixup,
  inversion normals and rejection indices. The MT machinery reuses the
  team-owned downsample asset, not its incompatible Numba seed routine.
- spline.rs: independently implemented rounded-x pooling, weighted cubic
  basis/derivatives, banded Gram/roughness, Cholesky, selected inverse, GCV
  search and linear extrapolation. Preserves historical0.3330, restricted
  trace ratio, endpoint leverage displacement and last-trial fit retention.

Two RNG stream tests and two critical spline tests were written before their
implementations. Literal expectations were obtained from live R4.6.1 on4090:
mixed runif/rnorm/sample stream, zero-SD/singleton draw consumption across the
MT boundary, full cars GCV prediction/extrapolation and roughness323.676.
Rust1.91 standalone formatting/parser checks pass. The initial three-file
candidate was not wired, compiled, executed, Clippy-checked or reviewed.
Do not claim their tests pass. Do not create another per-helper CI campaign;
wire the functioning calibration/state/region operation and validate it as
the integrated feature. All project files remain external; no local build.

The original MT BSD notice and AS241 MIT attribution/permission are included
in THIRD_PARTY_NOTICES. R source is a mathematical/behavior reference, not
mechanically translated GPL code. The exact tagged R4.6.1 low-level spline
rules were independently checked against official SVN, not assumed from trunk.

## Next integration

Use the existing run_with_observer Stage::DepthNormalized boundary to build
the spike from the real filtered/depth-normalized state. Share the existing
post-depth preprocessing tail with the spike through one private pipeline
helper; do not duplicate transformations or add a generic public foundation.
Keep original expression and later states/denoise as distinct results.

For ordinary references, fit mean/variance and dropout tables over observation
and reference groups. The reference-free source creates a temporary single
observation group over all cells before fitting those tables; do not preserve
the original per-annotation groups in that proxy fit. Aggregated normals pool
the actual reference cells when computing sampled means. Precompute each
simulated gene's mean/SD once, then keep serial cell-major normal draws and
subsequent gene-major dropout draws. This avoids repeated spline prediction
without changing the RNG stream. The unused optional nls logistic fit is not
needed for the actual spline-based dropout values.

Pool transformed spike observations in chromosome-block order, then cells,
then genes within each block, matching c(matrix). Neutral blocks concatenate
in that order; do not gather all neutral genes within each cell instead.
CNV-level insertion/fit order is neutral,0.01,0.5,1.5,2,3 (state indices
2,0,1,3,4,5), not sorted state order. For ncells=1, consume all100 samples
before excluding its NA SD from regression. For ncells>1, preserve rowMeans
across the100 replicate columns. Then fit the99 log-SDs against log(ncells),
predict group SDs, and use their median in adjusted Viterbi.

After native calibration, wire chromosome/group decoding and actual region
tables, with existing rsomics-help/common UX. Runtime worker consistency,
real upstream comparisons, whole-path timing/RSS, four-native debug/release,
strict Clippy and one feature review remain open. No native HMM acceptance
or complete inferCNV claim is justified yet.

Current syntax-formatted candidate fingerprints (uncompiled):

- normal.rs dcbf87c4c89dfa687d0b4d1bc0d64dea483e4a6e49cec8d18219009346ff0473
- rng.rs 938e334a5cb6da0a2c470b312d3da61aa92c42b358d28157f5839ac1113d0bed
- spline.rs e92286dd7388bc653dae947294d542780f40be370bd2e2f64d6ff9bf132d5d4c

That initial three-file candidate contained771 source lines and four critical
tests. The newer integration below supersedes its wiring status, not its lack
of runtime acceptance. No native compilation/test/Clippy success is claimed.

Ruling: use two isolated implementation workers for decoder and region
emission while root implements simulation/calibration and wiring. The user
explicitly requested parallel progress; their consolidated-testing/one-review
direction overrides per-helper review/five-round rituals. No parallel worker
shares files or stages/commits. Direct main and existing .autopilot ledgers
override generic worktree/workspace/deletion rituals. Cost: integration and
the one whole-feature review must catch cross-file mismatches. Neither
worker's code/report is accepted runtime evidence on its own.

Interfaces checked: decoder owns Model::new/decode over six calibrated
means/SDs and one sequence, consumed by root group/chromosome prediction;
regions owns CnvRegion/regions over validated genes and states plus a shared
counter, consumed after root's deterministic report-group ordering. Their
owned files do not overlap root or each other. Source semantics are in the
two bounded brief files; main owns the sole combined runtime/review gate.

## Integrated candidate

The full first-slice source is now connected: simulate.rs builds depth-
normalized calibration cells using the shared spline/RNG/moments helpers;
calibrate.rs preserves upstream block pooling and rowMeans bootstrap order;
mod.rs runs real and spike preprocessing, calibration, sample clustering,
group/chromosome-parallel state inference and deterministic region emission.
The decoder and region workers are complete, syntax-formatted, still unrun.
The Cephes/SciPy notice is retained alongside AS241, MT and infercnv notices.

CLI --hmm explicitly requires --cluster-by-groups. Seed, transition and
aggregation options remain product-local. The optional JSON HMM result keeps
calibration, group state paths and region ranges separate from expression14
and denoise22. All new report fields are omitted when HMM is disabled. No
new foundation, dependency or persisted calibration format is introduced.

Boot APFS currently has18142384128 free of245107195904 bytes (92.60% occupied).
No local Cargo/R compilation, package installation or dependency download.
Use the integrated four-native candidate gate with one real-R comparison,
not independent per-helper campaigns. Its HMM selector uses the successful
real R run40477088139; original archive collection is still in progress.
One bounded worker owns only the comparison script and its critical tests;
root owns native code and CI wiring. One whole-feature review remains after
the actual integrated runtime results. The full portfolio goal remains active.

Frozen green24 contains180 exact source files. Archive SHA
3dd6fbe54a9a1e1cfc082db8ab054b3d534ebf7df9fcbfccf1b1f209fcd1e03b;
source manifest77bcebb2232345a970a4880677c2fbde1fe354d993e41e3791dbe3f4077ac242.
The existing snapshot guard verifies/extracts all180 files externally. The
complete controller suite passes450 tests in51.202 seconds, two existing
native-only skips; architecture validation passes. This establishes controller
and archive correctness, not native HMM acceptance. The native candidate
workflow now accepts an explicit HMM selector, verifies the pinned real-R
artifact, and runs the complete 1/4-worker comparison on all four platforms.
No new R installation/workflow rerun is needed for these comparisons.
