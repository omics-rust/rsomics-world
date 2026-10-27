# Native HMM calibration work in progress

Scope remains the full portfolio and full inferCNV. This is implementation
state, not native acceptance, a public API or a release. Follow the integrated
HMM plan and infercnv-hmm-workflow-2026-10-27.md for real-package execution.

Owning product HEADc5db7c74 is the accepted denoise/docs baseline. Current
owned uncommitted files are THIRD_PARTY_NOTICES and three new files under
src/cnv/hmm/: normal.rs, rng.rs, spline.rs. They are not historical debris.
Do not overwrite them or stage unrelated work. No Cargo dependency/lockfile,
public module, production CLI or accepted preprocessing code has changed.

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

Two RNG stream tests and two critical spline tests are written before their
implementations. Literal expectations were obtained from live R4.6.1 on4090:
mixed runif/rnorm/sample stream, zero-SD/singleton draw consumption across the
MT boundary, full cars GCV prediction/extrapolation and roughness323.676.
Rust1.91 standalone formatting/parser checks pass. These new files are NOT
yet wired into a Rust module, compiled, executed, Clippy-checked or reviewed.
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
