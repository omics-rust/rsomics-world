# Private inferCNV samples filter implementation plan

Use executing-plans and subagent-driven-development: one bounded implementation
agent at a time, fresh review per task, root owns admission and durable state.
User authorizes routine decisions and direct main commits without PRs.

Spec: 2026-10-22-infercnv-native-filter-design.md.
Status: fresh builder check approved Task1 with no Critical/Important blocker.
Its native-test composition, active <=1 rejection, true-constant-only zero SD
and four-state-path/streaming clarifications are incorporated. No implementation
has begun on the Rust candidate; Task1 is complete at90ba71bd and product
fixture4f1cafc after independent review, original regeneration and432-test
coherent controller success. Detailed provenance is in
`.autopilot/state/infercnv-native-filter-2026-10-22.md`.

## Global gates

- Owning product: /Volumes/KIOXIA/Documents/omics-rust/rsomics-sc.
- Control plane: /Volumes/Zane's HDD/Documents/rsomics-world.
- Cargo home/target/tmp: /Volumes/KIOXIA/Developments/{cargo-home,cargo-target,tmp}.
- Fixtures/evidence: /Volumes/Zane's HDD/rsomics-fixtures/.
- APFS boot remains over80%: no local R/Cargo/build/install. Bounded Python
  -B tests use explicit external paths; genuine Rust RED/GREEN uses native CI.
- No new dependency, public API, foundation, feature selector, repository,
  publication, deletion or automation. Preserve inherited VCF/scratch changes.
- Existing original witness, receipts, source pins, trees.tsv and profiles.tsv
  remain unchanged. Existing samples selector/verifiers remain authoritative.

## Task 1 — Original route fixture extraction

Owned files: scripts/extract_infercnv_ward_fixtures.py and
scripts/test_extract_infercnv_ward_fixtures.py only. Root owns generated
product route TSV, plan/state and admission records.

- [x] First add focused failures for route extraction and its strict schema.
- [x] Add extract_filter_routes and a mutually exclusive --filter-output
  beside the existing --output/--profiles-output modes. Use existing accepted
  witness counts and file-binding mechanisms, no inferred numeric results.
- [x] Record every52 route, all four original expression/genes/cells/maps paths,
  filtered-matrix path, ordered references,
  argument and exact literal selection, authenticating all consumed files.
  Validate actual filtered row names/order/value bits and null/empty cases.
- [x] Cover wrong hash/path/symlink, malformed source maps/indices/dimensions,
  row/value drift, boundary/no/all qualifiers and inactive/no-reference guards.
- [x] Focused/controller/architecture/whitespace tests; owned-file commit,
  self-review and fresh independent task review. No push by implementer.
- [x] Root regenerates all three outputs externally, proves old two outputs
  unchanged, and copies only the new small route TSV into product fixtures.

## Task 2 — Freeze native tests and authenticate RED

Owned product tests/cnv_samples_filter.rs and tests/cnv_external_samples_oracle.rs
only; the generated route fixture is already frozen. No filter implementation,
CellProfiles accessor change, Cargo dependency/feature or workflow edit yet.
Use private filter::select_genes and filter::required_workspace as specified
in the design. Root owns native snapshot, exact-head CI, dispatch and admission.

- [ ] Extend the existing external samples test to consume all52 actual
  unfiltered matrices/maps, compare exact selection/null/empty behavior and
  filtered values/order, and retain all152 computed-tree comparisons.
  Every tree must use native-selected rows, not the old R-filtered input path.
- [ ] Retain CellProfiles shape/finiteness boundary coverage; add ordinary
  filter unit contracts for reference range and
  duplicates, guard values, constant/sample-size behavior, explicit arithmetic
  domain rejections and exact/minus-one requested workspace budgets.
  Active reference elements <=1 explicitly reject; zero SD is allowed only for
  actual constants, not nonconstant variance underflow. Other nonfinite
  accumulation/correction/variance/z-score arithmetic rejects.
  Exercise both R-index validation scratch and active/inactive output-buffer
  budget peaks, including exact/minus-one. Do not assert allocator capacity or
  process memory. Native external tests emit separate complete route/tree
  markers and52/152 totals, proving original-input-to-native-filter execution.
- [ ] Preserve current distance/Ward tests and expectations. Capture missing
  filter.rs as a real native failure after authenticated source/target setup;
  parser, source-fetch or unrelated failure is not the intended RED.
- [ ] Fresh review of test coverage and actual RED before implementation.

## Task 3 — Private candidate and four-native admission

Owned product src/cnv/samples/filter.rs and accessor-only changes in
distance.rs: read-only feature_count/cell_count plus crate-private unchanged
column. Existing validated CellProfiles is the concrete typed consumer
boundary; no distance arithmetic or public export changes.

- [ ] Implement the private source-ordered f64 candidate and checked workspace
  contract. Reproduce the specified finite domain and deliberate edge semantics;
  reject unsupported arithmetic clearly. No new epsilon or public contract.
  Contract: select_genes(&CellProfiles, &[usize], f64, usize) returns private
  GeneSelection{outliers:Option<Vec<usize>>, selected:Vec<usize>}, all zero-based.
  Fallible R-index copy uses allocation-free sort_unstable and drops before
  either output Vec; requested peak is
  max(R*sizeof(usize),(active?2:1)*G*sizeof(usize)). Each output reserves G,
  including empty logical lists. Test both competing-peak budget boundaries.
- [ ] Keep native test/fixture bytes unchanged from authenticated RED.
- [ ] Fresh source/API/hot-path review; formatting, strict Clippy and all tests
  in debug/release on Linux/macOS x86_64/aarch64 using the existing candidate
  workflow and exact original artifact verifier.
- [ ] Independently authenticate original run/API/logs/artifacts/source snapshots,
  all52 exact selections/filtered matrices and152 exact computed trees per
  profile/target. Preserve failures; no inferred or repaired success markers.
- [ ] Root records the bounded result or concrete failure and updates current
  product/durable status. No performance, full-workflow or release claim.

## Task 4 — Continue coherent samples ownership

- [ ] Use independently admitted reference-only outcomes before choosing its
  empty/NULL mapping. Honor explicit pooled collision rejection.
- [ ] Write/review the private group-owner plan against actual CellGroup and
  CnvState consumers, ordered maps, <=2 bypass validation and aggregate memory.
- [ ] Keep frozen-R and current-Rust preprocessing checks distinct; do not
  change existing tolerances in response to a tree/selection mismatch.
- [ ] Continue to group integration and measured hot-path performance before
  considering a complete inferCNV/public CLI/release claim. No phase-boundary
  approval stop is required.
