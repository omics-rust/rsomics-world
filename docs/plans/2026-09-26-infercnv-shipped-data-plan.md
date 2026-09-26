# inferCNV shipped-data implementation plan

Use superpowers:subagent-driven-development. The controller owns Git, remote
execution, immutable snapshots and evidence; one fresh implementer owns each
bounded code task. Read the full binding spec
`2026-09-26-infercnv-shipped-data-design.md` first.

Goal: accept independently verified real-example upstream checkpoints and
demonstrate the unpublished native core matches them on four native platforms.
Do not claim the full inferCNV workflow, performance superiority or a release.

## Global constraints

- Control plane: `/Volumes/Zane's HDD/Documents/rsomics-world`.
- Product: `/Volumes/KIOXIA/Documents/omics-rust/rsomics-sc`.
- Scratch: `/Volumes/KIOXIA/Developments/tmp`; fixtures/evidence:
  `/Volumes/Zane's HDD/rsomics-fixtures/`.
- No local Cargo, product builds/tests, dependency installation or R execution
  while boot APFS exceeds 80%. Small Python standard-library checker tests may
  run with `PYTHONDONTWRITEBYTECODE=1`, `-B`, and external TMPDIR.
- Native dependency lock, common 0.12.3, unpublished/no-binary policy, exact
  identity and `1e-12 + 1e-12*abs(expected)` numerical gate remain unchanged.
- Existing synthetic schema, generator settings, scenario assertions and
  negative tests must remain intact after shared-exporter extraction.
- Use the spec's source and three input hashes verbatim. Never select a latest
  run/artifact or accept the generator's self-reported digest as its own oracle.
- No public crate/API/foundation/CLI, Layer B dependency or biological claim.
- Preserve inherited VCF/untracked files and every earlier evidence snapshot.
- Controller alone stages/commits/pushes on main. No PRs, Co-Authored-By,
  publication, deletion, automations or implementation-spawned subagents.
- Rare comments; propagate malformed data and I/O failures; no fallback data.

## Task 1: Real-package shipped-data oracle and bounded checker

Files in the control plane:

- `scripts/infercnv_oracle_io.R`: shared 17-digit expression and identity
  exporters extracted unchanged from the current synthetic script.
- `scripts/infercnv_oracle.R`: source that helper relative to the script path;
  keep its settings, schema, fixtures and scenario-specific checks unchanged.
- `scripts/infercnv_shipped_oracle.R`: pinned shipped-input verification,
  canonical conversion, independently prepared subset/full cases, installed-R
  runs, original RDS and TSV exports, metadata and provenance.
- `scripts/validate_infercnv_shipped_oracle.py`: strict independent bundle
  validation and sorted manifest production using bounded memory.
- `scripts/test_infercnv_shipped_oracle.py`: parser, schema, identity, checksum,
  path, numeric and missing-artifact negative tests.
- `.github/workflows/infercnv-oracle.yml`: dataset choice (default synthetic),
  pinned source-byte check and conditional shipped run/checker, using existing
  installation plumbing. Shipped mode also executes the original synthetic
  generator and checker after exporter extraction.

Read the existing generator/checker and pinned package's creation/run source.
Write an explicit schema in the new checker's module contract before coding
against it: root source/package/runtime/argument/input identities, `subset`
and `full` case records, canonical/raw input mapping, the ten stage paths,
actual dimensions and checkpoint identities. Each required artifact is an
owned relative path. Keep source hash constants outside generated metadata.

- [x] Add checker tests first and demonstrate a real red before implementation.
  Tiny focused fixtures may test parser/validator components; they are unit
  fixtures, not substitutes for the required installed-package real-data run.
  Include malformed original 184/185-column dialect, duplicate/missing IDs,
  wrong source/input hash, unsafe/missing paths, changed count/coordinate/group,
  wrong subset membership, nonfinite data, changed stage-1 raw value, changed
  stage-2 retained value, and wrong arithmetic relationships.
- [x] Implement the generator and checker. Join by gene/cell identity, verify
  exact stage-1 values against canonical pinned inputs, and independently derive
  stage-2 membership and unchanged values. Validate stages pairwise or with
  compact numeric buffers. Save all ten original RDS checkpoints for each case.
  Record every relevant creation/run parameter and package/session information.
- [x] Extract only the two common R export helpers. Keep existing synthetic
  behavioral tests unchanged. Run all Python tests and control-plane validation.
  Controller supplies remote R syntax checking; no local R execution.
- [x] Independently review the harness and its diff. Fix material findings,
  then controller commits/pushes, verifies exact-head Control plane CI and
  dispatches the shipped-data oracle.
- [x] Inspect the actual upstream result and preserve original API metadata,
  logs, ZIP, source/raw input bytes, RDS, exports and sorted hashes externally.
  Re-run strict bundle checks from preserved data. Verify predicted dimensions
  against actual checkpoints and document discrepancies without substituting
  values. Commit a small immutable oracle receipt with exact run/head/artifact
  identity, ZIP digest and bundle-manifest digest. No native claim yet.

## Task 2: Explicit external-oracle native conformance

Product files: `tests/support/mod.rs`, `tests/cnv_oracle.rs`, new
`tests/cnv_external_oracle.rs`, and fixture-provenance README as needed.
Controller files: candidate CI workflow, immutable receipt/acquisition checker,
source snapshots and durable execution ledger. Production CNV code changes
only if actual evidence requires a separately diagnosed fix.

- [ ] Generalize the test-only reader to an explicit root while preserving
  all checked-in synthetic behavior. Add malformed-input tests before the
  generalization and verify real failures remotely.
- [ ] Add an explicitly selected external-oracle test path that fails on
  missing configuration/data. Default small-fixture tests remain self-contained;
  opting into the large test must not silently skip. Use a dedicated test
  feature or equivalent explicit target selection, recorded in CI and README.
- [ ] Check all ten stages from one native execution per case, exact
  identities/dimensions/returned state, unchanged raw input and fixed numerical
  tolerance. Report per-stage maximum absolute and scaled errors.
- [ ] Extend CI to acquire only the pinned receipt's artifact into runner-temp,
  verify its exact head/status/digests and safe paths, then validate the manifest
  before invoking tests. Preserve acquisition and numerical logs. No large
  fixtures or R package code enters Git or the product package.
- [ ] Freeze and validate source on Linux/macOS x86_64/aarch64 in debug/release,
  including all old tests, formatting and strict Clippy. Independently review
  source and evidence; diagnose disagreements without tolerance widening.
- [ ] Commit verified product changes locally on main; commit/push control-plane
  receipts and verify exact-head CI. Record the actual correctness scope and
  proceed to matched performance measurement and downstream workflow work.
