# Integrated inferCNV denoise plan

> Use superpowers:executing-plans inline. The user delegates routine choices,
> rejects per-helper RED/review campaigns, and requires functioning integrated
> work, meaningful compatibility/performance checks and one overall review.

Goal: add the complete non-HMM denoise operation to the existing samples CLI,
with explicit step-22 output and real installed-package comparison. This is
not a smaller definition of inferCNV or the whole portfolio objective.

Architecture: keep the original step-14 state and samples tree immutable.
Produce a separate column-major denoised value buffer, reuse cnv::parallel,
and expose one product-local typed library operation. No dependency or public
foundation is added. rsomics-stats has no suitable public moments interface;
its private Welch reductions do not establish a second consumer for a new
R-specific public reduction API.

Spec: docs/10-products/sc.md and
.autopilot/state/infercnv-next-operations-2026-10-26.md; authoritative source
is infercnv1.28.0 commit b421d9405c97a309b081ef86d455e976df93eae4,
R/inferCNV_ops.R:1538-1622,2232-2350 and
R/inferCNV_heatmap.R:2783 onward, including .logistic.

## Contracts

- Rust1.91, direct main commits, external storage only, no local build while
  APFS boot occupancy exceeds80%; no path dependencies or new public crate.
- DenoiseConfig contains NoiseFilter::{Fixed(f64),ReferenceSd(f64)}, logistic,
  positive resolved threads and max_numeric_bytes for its value buffer.
  DenoiseConfig::new(filter,logistic,threads,max_numeric_bytes) validates these
  once and retains private fields; CLI constructs it before reading inputs.
- denoise(&CnvState,&CreatedInput,&DenoiseConfig) -> Result<Denoised>.
  Denoised has immutable values() and optional bounds(); NoiseBounds exposes
  center and threshold. Input gene/cell order and samples results do not change.
- Reference cells are pooled by cell, not equally weighted by group. With no
  references, use all cells. The center is one global reference-value mean.
- ReferenceSd width is amplifier times the mean of per-cell sample SDs across
  genes. A sample SD requires at least two genes; amplifier is finite >=0.
- Fixed nonpositive widths bypass transformation, including logistic, as the
  run pipeline does. Nonfinite parameters fail. SD-derived zero width still
  invokes the selected transform, matching the actual upstream distinction.
- Hard filtering replaces only strictly interior values with the global center.
  Logistic filtering uses the source's default slope20 and absolute deviation.
  Verify the exact .logistic formula from source before implementing it.
- Aggregation is serial and deterministic; independent value columns transform
  in parallel. There is one additional value buffer, no copied gene/cell maps.
- CLI adds --denoise, --noise-filter, --sd-amplifier and --noise-logistic.
  The last three require --denoise; explicit filter and explicit amplifier
  conflict. Default SD amplifier is1.5, hard filtering. This interface is our
  choice within the approved product, not a new user-confirmed design.
- --include-expression continues to mean the untouched step-14 values.
  Enabling denoise always persists the step-22 matrix in a separate optional
  denoised report member; no flag whose result is discarded. Optional new
  settings/results are omitted when disabled, preserving default reports.
- New numerical/IO/configuration errors propagate before transactional output.
  No per-element commentary, speculative generic API or unfinished CLI flag.

## Task 1: working library and CLI

Files: src/cnv/denoise.rs, src/cnv/mod.rs, src/cli.rs, src/cli/report.rs,
tests/cnv_cli.rs and the product README/THIRD_PARTY_NOTICES when verified.
Move report-only serialization into the CLI child module as part of this
feature; it can read private arguments without exposing them publicly.

- [x] Write the integrated CLI test first. Observe its missing --denoise
  behavior on the retained accepted Linux binary on4090; do not create a
  separate per-helper RED/native campaign. No production edit before this.
- [x] Add critical numeric tests with independent literal expectations:
  pooled global center/open bounds, SD axis, logistic midpoint/tails,
  fixed bypass versus SD-zero logistic, deterministic workers and real
  configuration/budget failures. Trivial getters earn no separate tests.
- [x] Implement the typed operation and CLI report wiring. Preserve step14
  values and trees; persist step22 values even without --include-expression.
- [x] Freeze green-22 and run existing Linux correctness/benchmark mode,
  confirming unchanged default report fingerprint and actual new CLI behavior.

## Task 2: installed-package workflow evidence

Files: scripts/infercnv_denoise_workflow.R,
scripts/compare_infercnv_denoise_workflow.py, focused behavior tests and
.github/workflows/infercnv-oracle.yml; one immutable native candidate receipt.

- [x] Reuse the existing exact-source R installation. An explicit opt-in
  denoise mode runs raw-input samples workflow comparisons for off, fixed,
  SD and logistic configurations; skip unrelated old ingestion probes.
- [x] Compare actual native CLI outputs to actual installed infercnv results,
  with exact IDs/order/selections/merges and existing numerical criteria for
  expression/heights. Keep actual outputs, timings, errors and source/config
  provenance. Do not replace the namespace with copied equations or mocks.
- [x] Include a real full input, grouped references and no-reference data.
  Retain small resulting goldens for four-native default tests where useful;
  large fixtures stay external. Do not infer success from source inspection.

## Task 3: integrated acceptance

- [x] One fresh whole-feature review, formatting/strict Clippy, all existing
  tests and final four-native debug/release checks. No old oracle weakening.
- [ ] Verify output/memory/throughput in proportion to this value transform;
  commit only the owning product files and scoped control-plane evidence.
- [ ] Keep publish=false until complete release gates. Continue equivalent
  upstream performance, HMM/subclusters/Bayesian/reporting and other products.

Review focus: pooled versus group-balanced moments; per-cell SD axis; exact
open bounds; fixed bypass versus zero derived width; immutable pre-denoise
expression/tree identities; default report compatibility and retained output;
finite results, bounded allocation and worker errors. Critical tests cover
these integrated contracts; do not invent exhaustive component campaigns.
