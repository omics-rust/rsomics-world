# Native inferCNV Distance Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Compute all107331 original captured distances exactly from their original R-selected matrices and retain exact152-tree results on all four native classes.

**Architecture:** Reuse the accepted artifact/source verifiers and deterministic fixture extractor. A validated borrowed profile view computes two-lane double reductions and shares condensed mechanics with Ward inside the product. A separate opt-in external test exercises all real vectors without a public/default API or large checked-in matrices.

**Tech Stack:** Rust1.91.0/edition2024, existing rsomics-common, Python stdlib and current native candidate workflow; no new Rust dependency.

**Spec:** docs/plans/2026-10-22-infercnv-native-distance-design.md

## Global Constraints

- No new Rust dependency or default library export is needed.
- No epsilon, distance tolerance or omitted hard case is introduced.
- Original matrices remain external.
- Rust1.91 and Cargo.lock remain unchanged.
- All local scratch/downloads/evidence stay external; no local Cargo/R/install while the Mac boot gate exceeds80%.
- Commit directly to owning main; no public repo, crate, release, deletion or automation.
- Full samples, native filtering, group/collision policy, end-to-end preprocessing trees and performance are excluded from this component's acceptance.

## Review Focus

- R selected matrices can contain zero genes while retaining cells: Task1 records the true dimensions and Task3 tests positive-zero pairs.
- A group may not contain the final global cell: Task1 derives cell count/order from actual state and matrix headers, never max(indices).
- A compiler-independent single-chain norm can differ from the captured binary: Task3 pins two-lane counterexamples and all actual107331 distance bits.
- Invalid index/shape/budget input must fail before output allocation: Task3 exercises duplicate/out-of-range dimensions, checked bytes and equality/minus-one.
- The old external selector must not start requiring a samples root: Task2 pins separate features, exact selectors and both actual missing-root gates.

---

### Task 1: Hash-bound profile metadata beside existing tree fixtures

Completed: 730ace2d; fresh task review approved, coherent full404-test run passed; c8c81dc3 exact-head Control plane39776104734 passed. Original checkbox steps below are the execution specification; this status records their completed gate.

**Files:** world scripts/{extract_infercnv_ward_fixtures.py,test_extract_infercnv_ward_fixtures.py}; product tests/fixtures/infercnv-1.28.0/ward/{profiles.tsv,README.md}.

**Interfaces:**

- Consumes: existing `_records(document)` tuples, pinned WITNESS_SHA256/EXPECTED_COUNTS, original witness-relative files and existing validator `checked_file(root, relative)` path guard.
- Produces: `extract_profiles(witness_path: Path, output_path: Path) -> dict`. Existing `extract()` and six-row tree format remain unchanged. CLI retains `--witness` and accepts exactly one of existing `--output` or new `--profiles-output` via a required mutually-exclusive group.
- Output schema is exactly `tree\tmatrix\tfeatures\tcells\tindices\n`, then152 TSV rows ordered identically to trees.tsv. Indices are comma-separated zero-based actual input indices. Return counts/output hash/bytes as for the existing extractor.

- [x] Add tests before implementation for reordered actual versus declared indices; true global count greater than max(group indices); matrix-header/state-cell order mismatch; zero features; source file hash/path rejection; output exclusivity; unchanged old tree extractor/CLI; strict152-count binding. Unit mocks may replace fixed SHA/counts only inside tests.
- [x] Run focused stdlib tests and retain actual new missing-function/behavior RED with external TMPDIR/-B.
- [x] Implement metadata extraction. Reuse `_records` ordering/validation, path guard and hash-pinned witness; hash each distinct matrix/cell metadata file once by streaming, compare first matrix header against strict state/cells TSV identities, and validate selected gene-index list/count plus actual in-range input indices. All validation precedes exclusive output creation. Do not load whole matrices or touch original evidence.
- [x] Run focused and complete controller tests/architecture/whitespace checks; generate profiles.tsv once from the original local accepted witness. Independently compare152 IDs with trees.tsv, all original maps/dimensions and output hash; update fixture README. Commit reviewed extractor/tests in world with plan/state, push, verify exact-head CI; product metadata remains pending native acceptance.

### Task 2: Add one authenticated native witness selector

Completed: c8c81dc3; fresh admission review approved and actual RED39776437129 authenticated missing condensed.rs only after source/metadata success. All157 pre/postchecks match.

**Files:** world .github/workflows/sc-cnv-core-candidate.yml; new scripts/test_infercnv_samples_candidate_workflow.py; product Cargo.toml and new tests/cnv_{samples_distance,external_samples_oracle}.rs; world immutable red-10 snapshot/state.

**Interfaces:**

- Consumes: unchanged `verify_artifact(receipt_path, run_path, artifact_path, zip_path, output_dir) -> shipped-bundle Path`, existing source guard and samples witness validator; Task1 profiles.tsv plus original trees.tsv.
- Produces: exact selector `infercnv-samples-clustering-witness-2026-09-30`; test-only feature `external-infercnv-samples-oracle = ["external-infercnv-oracle"]`; target `cnv_external_samples_oracle` with required-features exactly `["external-infercnv-samples-oracle"]`; env RSOMICS_INFER_CNV_SAMPLES_ROOT only after full witness validation.
- Add `Validate oracle selection` before existing extraction/acquisition steps. Its closed case statement writes ORACLE_FEATURES to GITHUB_ENV: empty for none, external-infercnv-oracle for old shipped, external-infercnv-samples-oracle for witness; reject everything else before network. Debug/release/Clippy consume this fixed value, retaining existing measurement-all-features behavior.

- [x] Write actual-YAML/Bash regressions before editing: exact three choices/default; invalid/path selectors fail before acquisition; none causes no external features/acquisition; legacy selector retains its roots/validators; witness runs original ZIP and all validators before exporting samples root; matching feature use; real missing-root commands require explicit env text and `test result: FAILED`; four-native matrix/RED/postcheck/upload behavior unchanged.
- [x] Run regressions to capture RED. Add only the closed selector/features wiring; reuse original receipt and downloader/ZIP verifier unchanged. Preserve existing step names. Source validation keeps original synthetic/shipped checks, then for witness invokes `validate_infercnv_samples_clustering_witness.py "$verified/bundle" "$verified/shipped-bundle" "$verified/samples-clustering"` before exporting the new env.
- [x] In Cargo.toml add only feature and opt-in target. No dependency/lock update. Write new Rust tests against Task3 interfaces while the new source files remain absent. Existing Ward test additionally declares the new sibling condensed module; no old assertion changes. Default test covers small profiles and resource/index/math failures. External test checks explicit env/root/files first, reads both strict metadata fixtures, caches one route matrix and asserts all152 IDs/all107331 distances plus real Ward outputs, without tolerance/skips. Every identity produces its real comparison marker; totals are asserted before success.
- [x] Verify controller suites and actual script syntax/YAML. Freeze red-10 from owning source using the unchanged exact logical-file guard, with new distance/condensed modules absent. Commit/push controller/snapshot changes and wait exact-head CI; dispatch `snapshot=red-10 expected_red=true oracle_receipt=none`. Authenticate original actual missing-module failure after successful metadata/source/path steps; no network or unrelated compiler failure counts as RED.

### Task 3: Private native distance kernel and complete original-vector execution

Complete: native39777465667 passes four classes/debug-release, independent original-artifact audit and root verification agree; private product15e731128baa8fe54577cf8dec7d6559333f1076 records the verified implementation. Exact receipt and bounded acceptance are tracked in .autopilot. Final controller acceptance-record CI follows the owning documentation commit; no product release follows.

**Files:** product src/cnv/samples/{condensed.rs,distance.rs,ward.rs}; existing/new private Rust test targets and fixture metadata; world immutable green-16 snapshot/state/result receipt.

**Interfaces:**

- Produces: `pair_count(leaves: usize) -> Result<usize>` and `condensed_index(leaves: usize, a: usize, b: usize) -> usize` in condensed.rs; exact existing pair count/index contracts, no new policy.
- Produces: `CellProfiles<'a>::new(values: &'a [f64], features: usize, cells: usize) -> Result<Self>` and `distances(&self, indices: &[usize], max_numeric_bytes: usize) -> Result<Vec<f64>>`, exactly as the spec. No exported fields or default library module declaration.
- Consumes: existing Ward signature/model/workspace tests unchanged after importing shared mechanics; new tests from Task2.

- [x] Implement the exact factored pair-count/index functions and borrowed column view. Check finite input once and shape overflow; return column-ordered condensed distances using separate even/odd sums and explicit multiplication, final add/sqrt. Reject nonfinite result; preserve square underflow and zero features. Output allocation is fallible/exact and budgeted pairs*8 before reservation; reject repeated/out-of-range indices without an extra numeric buffer.
- [x] Keep all old Ward assertions and bytes of its mathematical update/chain/order unchanged except imports/mechanical helper moves. Use both shared mechanics from both kernels; no public foundation, generic speculative API, dead-code suppression or hidden scratch allocation.
- [x] Standalone external rustfmt only; confirm all other source/dependency/lock changes are scoped, generate and independently guard green-16. New default tests pin two-lane even/odd and positive-zero outputs, permuted indices, no input mutations, nonfinite/overflow/underflow, exact budget and shared condensed order. Fresh independent implementation/spec review checks actual math, fixture authentication, source/index identity, API and hot path before admission.
- [x] Commit/push candidate world snapshot and require exact-head Control plane CI. Dispatch `snapshot=green-16 expected_red=false oracle_receipt=infercnv-samples-clustering-witness-2026-09-30` on the existing four-native matrix. Retain original acquisition API/digest/manifest/witness verification, actual source and debug/release results plus missing-root negative test. Do not upload four copies of1.57GB data; native artifacts retain authenticated bindings/logs/index/source.
- [x] Independently inspect original four artifact/run/jobs/log/source bytes, exact hashes/hosts/Rust1.91, complete old/new test outcomes, all107331 bit comparisons/all152 real computed-vector trees in each profile, formatting/Clippy and postchecks. A failed bit/case remains failed; no tolerance/dropped data or repaired evidence. Diagnose without changing acceptance when needed.
- [x] Commit verified product directly to main, record exact scope/failures/source/platform proof, commit/push world acceptance and verify exact-head CI. No public package/CLI/performance or samples-workflow claim. Continue group/step14 integration with its unresolved boundaries recorded separately.

## Plan self-review

All spec sections map to metadata, authenticated selection/tests and native
kernel tasks. Feature names, selector, roots, TSV columns and signatures match
across tasks. Each Review Focus condition has an explicit owning test. The
existing source/ZIP guards remain unchanged; two-lane source inference is not
confused with binary proof. No production code precedes actual tests. User
routine-approval opt-out remains in force. At20:08 CST the user chose explicit
failure for the pooled all_observations reference-name collision as a known
incompatibility. Original distance witnesses remain unchanged.
