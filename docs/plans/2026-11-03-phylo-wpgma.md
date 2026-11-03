# Phylo WPGMA implementation plan

> Use superpowers:executing-plans inline, with one fresh whole-feature review.

**Goal:** Extend existing `infer` with a complete WPGMA method, not another
command or crate.

**Architecture:** Reuse the private nearest-neighbor-chain agglomeration and
rooted tree reconstruction already used by UPGMA. Select the distance-update
rule by typed inference method; keep parsing, validation, transactional output
and shared help unchanged. No new dependency or foundation item.

**Tech stack:** Rust1.91, current checked DistanceMatrix and phylo-tree builder,
existing scikit-bio0.7.3 oracle and SciPy weighted linkage.

**Spec:** `docs/10-products/phylo.md`, later inference slice; pinned source
https://github.com/scikit-bio/scikit-bio/blob/0.7.3/skbio/tree/_upgma.py
and SciPy linkage documentation. WPGMA updates distance to another cluster as
the mean of the two merged clusters' distances, irrespective of their sizes.
The result is rooted, with branch lengths derived from half merge distances.

## Constraints and review focus

- Direct main in the owning external repository; preserve unrelated changes.
- No speculative modules, helper-only scaffolding or per-helper test ladder.
- Keep NJ and UPGMA behavior and defaults unchanged.
- Unequal cluster sizes must distinguish WPGMA from UPGMA.
- Existing boundary rejects nonfinite/negative input; do not duplicate checks.
- Tie/zero/minimum-size cases must retain deterministic source-compatible
  topology/lengths, not just a valid Newick string.
- Use existing shared CLI output/error behavior; do not add a thread flag for
  sequential linkage or imply a parallel speedup.
- Do not publish before fresh method-specific compatibility/performance and
  four-native exact-head CI gates; registry credential hold remains separate.

## Task 1: One complete inference-method extension

Files: owner `src/inference.rs`, `src/cli.rs`, `tests/cli.rs`,
`tests/oracle.py`, `benches/inference.rs`, README when qualified.
Interface: add `InferenceMethod::Wpgma` and CLI `--method wpgma`; consume the
existing `infer_tree(&DistanceMatrix, InferenceMethod, NegativeBranchPolicy)`.

- [x] Add one integrated CLI case for unequal cluster sizes, exact expected
  rooted tree, UPGMA distinction, and shared named-output failure preservation.
- [x] Run it once before implementation; expect unknown wpgma method.
- [x] Generalize the existing linkage update and expose the typed/CLI method.
- [x] Run the integrated test and full debug/release suites, fmt, strict Clippy.
- [x] Extend existing source oracle with WPGMA and deterministic tie/zero/two-tip
  cases; run against pinned real source, with full results retained externally.
- [x] Obtain one fresh whole-feature review and at most one necessary fix pass.
- [x] Measure representative WPGMA workflow timing/RSS with full topology and
  branch-length checks; retain losses and do not relabel old measurements.
- [x] Commit/push owning changes when correct; wait for exact-head native4 CI.
- [x] Record actual release decision; no placeholder/publication claims.
