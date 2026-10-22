# Native inferCNV Ward.D2 Kernel Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Reproduce all152 accepted actual Ward.D2 trees from their original captured distance vectors in a private rsomics-sc kernel.

**Architecture:** A strict deterministic controller extracts derivative fixtures from the hash-pinned original witness. The product's private integration test compiles the kernel file directly without exposing an unused default API. Existing immutable snapshots and four-native CI provide RED/GREEN evidence without local compilation.

**Tech Stack:** Rust1.91.0/edition2024, existing rsomics-common, Python stdlib, existing GitHub Actions candidate workflow.

**Spec:** docs/plans/2026-10-22-infercnv-native-ward-design.md

## Global Constraints

- No default product API, CLI, package, dependency or foundation is added.
- Retain Rust1.91.0 and the existing Cargo.lock exactly.
- No native tolerance, epsilon, omitted tree or summary oracle is allowed at this gate.
- All scratch/downloads/evidence stay on the specified external disks.
- Do not install, compile or run Cargo/R on this Mac: APFS occupancy is above80%.
- Commit one concern directly to main in each owning repo; preserve inherited VCF/scratch changes.
- Product is still private/publish=false with no remote.
- World pushes require exact-head Control plane CI.

## Review Focus

- A missing or malformed oracle route must not silently reduce the152-tree comparison; Task1 pins source hash and complete counts.
- R's signed merge encoding must not reorder children numerically; Task2 compares every actual merge and local leaf order exactly.
- Initial squaring or Ward updates can overflow despite finite input; Task2 rejects them before a result is returned.
- A tiny workspace budget must fail before a large allocation, including output/sorting/traversal buffers; Task2 pins its exact requested-size boundary.
- Equal rounded square-root heights must preserve original merge order; Task2 includes dynamic-range/chain probes and all actual ties.

---

### Task1: Deterministic authenticated Ward fixtures

**Files:**

- Create world scripts/extract_infercnv_ward_fixtures.py
- Create world scripts/test_extract_infercnv_ward_fixtures.py
- Create product tests/fixtures/infercnv-1.28.0/ward/trees.tsv (generated)
- Create product tests/fixtures/infercnv-1.28.0/ward/README.md

**Interfaces:**

- Consumes: original pinned witness at the spec's external path.
- Produces: `extract(witness_path: Path, output_path: Path) -> dict`, an argparse CLI accepting `--witness`/`--output`, and the exact six-row TSV format in the spec. Return counts/hash metadata for a durable ledger; errors exit nonzero. Refuse existing destination and validate all content before exclusive creation.
- Private `_records(document: dict) -> list[tuple[str, int, list[str], list[list[int]], list[str], list[int]]]` validates each tree and returns ordered records; `extract` additionally fixes the original SHA and global36/52/160/152/107331/142 counts.

- [x] Write small-document tests for literal scalar preservation, source case/route/group ordering and R leaf orientation; malformed dimensions, finite scalars, topology, permutation and omitted routes must fail. Full extraction tests use temporary hash-patched fixture bytes only inside unittest mocks, never a configurable production pin. Assert old output remains unchanged on rejection.
- [x] Run focused stdlib tests with no implementation and retain RED; expected import/behavior failure, not network/data absence.
- [x] Implement the two interfaces and CLI. Require both workflow routes, only isolated for probes, and success status. Retain no-tree groups in global counts but produce no fake tree. Require the exact tree-presence flags and empty tree/distance fields for legitimate small groups. Preserve all original scalar strings.
- [x] Run focused tests, architecture validator and all existing385 controller tests with external TMPDIR and -B; no local Cargo/R. Generate the fixture once from the actual pinned witness and independently reparse its152 records/107331 pairs. Write README with original witness/receipt/source hashes and generated fixture SHA, index conventions, scope/exclusions and the explicit non-performance/non-release boundary.
- [ ] Commit only owned world extractor/tests after review, push and verify exact-head CI. Product fixture stays pending with Task2 until native verification; never stage inherited work.

### Task2: Private native tree kernel and all original tree comparisons

**Files:**

- Create product src/cnv/samples/ward.rs
- Create product tests/cnv_samples_ward.rs
- Create product THIRD_PARTY_NOTICES
- Modify product README.md/tests/README.md only to state the bounded verified status after acceptance.
- Create world .autopilot/snapshots/sc-cnv-core-2026-09-26/red-9/{README.md,files.sha256,source.tar.gz,checksums.sha256}
- Create world .autopilot/snapshots/sc-cnv-core-2026-09-26/green-15/{README.md,files.sha256,source.tar.gz,checksums.sha256}
- Create world .autopilot/state/infercnv-native-ward-2026-10-22.md

**Interfaces:**

- Consumes: Task1 exact TSV fixture; existing rsomics_common::{Result, RsomicsError}.
- Produces: `ward_d2(distances: &[f64], leaves: usize, max_workspace_bytes: usize) -> Result<Dendrogram>`, `required_workspace(leaves: usize) -> Result<usize>` and the exact crate-private Dendrogram fields defined in the spec. Test imports it via `#[path = "../src/cnv/samples/ward.rs"] mod ward;`; do not yet declare/export it from the product library.

- [x] Write actual tests: all152 records require literal merge vectors, bitwise binary64 height equality and source one-based order converted to zero-based. Pin total152/107331 and max142. Pin n=2 `[3.0] -> merge[[-1,-2]],height[3.0],order[0,1]`; `[1,.5,.5]` gives `[[-1,-3],[-2,1]]`, heights[.5,.8660254037844386], order[1,0,2]. Include all-zero/equal ties, repeated merge heights, immutable input and dynamic-range squared-distance/sqrt ties. Reject n=0/1/>i32::MAX, wrong condensed length, arithmetic overflow, NaN/Inf/negative distances, square overflow and insufficient budget.
- [ ] Freeze absent-kernel red-9 after standalone external rustfmt on the test (skip_children=true). Source manifest/archive generation uses the existing exact logical-file guard; existing source/lock hashes must be unchanged apart from new tests/fixtures. Commit RED snapshot/plan/state in world, push, wait exact-head CI, dispatch `snapshot=red-9 expected_red=true oracle_receipt=none`, and authenticate original run/jobs/logs/artifact. Require the actual absent ward.rs diagnostic, not another failure or an assumed compiler error code.
- [ ] Implement the kernel exactly as the spec's pinned nearest-neighbour chain, squared Ward expression, post-sqrt ordinal-stable sorting, union-find R relabel and iterative order traversal. Use narrow typed internal helpers and almost no source comments. Implement checked workspace-size calculation and fallible exact allocations. Add original BSD notice verbatim with provenance to THIRD_PARTY_NOTICES; no parallelDist/infercnv source copying.
- [ ] Test exact workspace equality/minus-one using the actual size_of of raw temporary merges and named buffer counts. Retain dynamic-range and long-chain edge tests; no hardcoded oracle answers in the kernel, no epsilon or special case for fixture IDs.
- [ ] Freeze green-15 with existing lock, README and new notice. Use standalone external rustfmt only. Commit frozen candidate in world, push and verify exact-head CI; dispatch `snapshot=green-15 expected_red=false oracle_receipt=none`. Default all-targets executes the private tree test on all four native classes without new workflow inputs or APIs.
- [ ] Authenticate four original artifacts/run/jobs/logs: exact snapshot bytes/host/architecture/Rust version, all152-tree debug/release comparisons, format/strict Clippy, existing tests and source postchecks. One fresh implementation/spec review, fix actual findings and reverify affected gates; do not loop unbounded whole reviews.
- [ ] Commit verified product files directly to local main (`feat(sc): add private Ward.D2 tree kernel`), update durable world result/remaining distance/group gates, commit/push and verify exact-head CI. No product remote/release/public API is created. Continue the next unblocked samples integration work afterward.

## Plan self-review

The fixture task owns complete oracle selection and its mutation tests. The
kernel task owns numeric ordering, memory/shape rejection, native tests and
source provenance. All spec gates map to these tasks; fixture ordering and
index conversion match the declared interfaces. No distance/group/filter
implementation is smuggled into tree-kernel acceptance. The five Review Focus
conditions each have explicit task tests. The user has delegated routine
implementation decisions and waived repeated approvals; execute inline with
bounded parallel work/review instead of another approval checkpoint.
