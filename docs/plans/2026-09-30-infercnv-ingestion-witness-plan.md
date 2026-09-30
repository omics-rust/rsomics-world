# inferCNV Raw-Input Witness Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Obtain inspected real infercnv 1.28.0 creation-stage evidence for canonical plain and gzip inputs and a focused decimal/order case.

**Architecture:** Extend the existing installed-package workflow, not a second R installer. A creation-only R driver exports matched state and ordered maps; a standard-library Python checker validates the preserved bundle independently before any native parser claim.

**Tech Stack:** R 4.6.1, infercnv 1.28.0 from `b421d9405c97a309b081ef86d455e976df93eae4`, Python 3 standard library, GitHub Actions Ubuntu 24.04.

**Spec:** `docs/plans/2026-09-30-infercnv-ingestion-witness-design.md`.

## Global constraints

- Work in `rsomics-world`; do not edit or publish `rsomics-sc` in this plan.
- No project build, download or scratch on the Mac mini boot disk; the current APFS occupancy exceeds 80%.
- Keep existing synthetic and shipped oracle validators and accepted receipts unchanged.
- Preserve actual failures and original artifacts; do not overwrite an evidence directory.
- The original shipped `.counts.matrix.gz` is a separate format probe, not canonical plain/gzip equivalence evidence.

## Review focus

- Canonical TSV gzip decompresses to exactly the source bytes, not merely an equivalent table.
- Plain/gzip reference and observation list order and one-based indices agree, not only cell labels.
- The small witness confirms `0.076439` as hexadecimal f64 evidence without asserting universal R-decimal behavior.
- The validator rejects path traversal, missing source files and changed input hashes; existing source-bundle validators retain duplicate-path checks.
- Missing or changed stage-1 RDS fails the R comparison and the independent file/hash checks.

---

### Task 1: Creation-only R exporter

**Files:** Create `scripts/infercnv_ingestion_witness.R`; reuse `scripts/infercnv_oracle_io.R`.

**Interfaces:** Consumes validated `synthetic-bundle` and optional `shipped-bundle` directories. Produces a fresh `ingestion-witness` directory with `witness.json`, source/gzip input hashes, and per-case `plain`/`gzip` expression, gene, cell and ordered group-map exports.

- [ ] Write an R smoke contract that constructs a two-gene, three-cell fixture including `0.076439` and `7.6439e-2`, calls installed `infercnv::CreateInfercnvObject`, and asserts the expected ordered gene/cell IDs. Retain the fixture input bytes.
- [x] Pipe the script to 4090 and run `TMPDIR=/dev/shm Rscript --vanilla -e 'invisible(parse(file="stdin"))'`; it checks syntax only, not package semantics.
- [ ] Implement shared case logic: verify package/source identity; read the bundle manifest and stage-1 RDS; gzip the canonical counts with deterministic content; call `CreateInfercnvObject` on plain and `.gz` with identical explicit arguments; require `identical` expression, count, gene, cells and both ordered maps against each other and stage 1; export with existing `write_expression`/`write_identity` plus a typed map table; record inputs, arguments, runtime identity, SHA-256 and `%a`/17-digit decimal witness values.
- [x] Run the R syntax check again; ensure neither its output nor any generated bundle is staged from the controller.
- [ ] Commit only the exporter and its tests with `test(sc): add infercnv creation witness exporter`.

### Task 2: Independent checker

**Files:** Create `scripts/validate_infercnv_ingestion_witness.py` and `scripts/test_infercnv_ingestion_witness.py`.

**Interfaces:** `validate_bundle(witness_root, synthetic_root, shipped_root=None) -> dict` checks the R-exported schema and returns exact dimensions, hashes and map summaries. CLI accepts these paths and exits nonzero on any mismatch.

- [x] Write failing standard-library unit tests for valid synthetic fixture, changed input digest, mismatched gzip/plain output, changed ordered map, changed stage-1 coordinates, nonfinite output, adjacent decimal and path traversal.
- [x] Run `PYTHONDONTWRITEBYTECODE=1 python3 -B -m unittest discover -s scripts -p test_infercnv_ingestion_witness.py` with `TMPDIR=/Volumes/KIOXIA/Developments/tmp`; record the expected failure.
- [ ] Implement safe bundle-relative file resolution, exact SHA-256 verification, plain/gzip byte equality, row/column/value identity checks against stage-1 exports, ordered map coverage and case-specific input/argument constraints. Do not infer RDS content in Python.
- [x] Rerun the targeted unit tests and the complete control-plane suite with bytecode disabled and external scratch; require green. The import-based tests also compile the Python source.
- [ ] Commit only checker/tests with `test(sc): validate infercnv ingestion witness bundle`.

### Task 3: Remote execution and acceptance

**Files:** Modify `.github/workflows/infercnv-oracle.yml`; create `.autopilot/state/infercnv-ingestion-witness-2026-09-30.md` after evidence audit.

**Interfaces:** Existing `dataset=synthetic|shipped` dispatch remains; the R exporter and checker run after their respective existing validators, writing only under `$RUNNER_TEMP/infercnv-oracle`. The existing artifact upload preserves the new directory and logs.

- [ ] Add a workflow step to run the Python unit tests; after synthetic/shipped oracle validation, run `Rscript --vanilla scripts/infercnv_ingestion_witness.R "$ORACLE_OUTPUT" "$EVIDENCE_DIR/ingestion-witness" "$SHIPPED_OUTPUT"` for shipped mode (omit the third argument otherwise), then run the independent checker with the same bundle selection. Keep `set -euo pipefail` and `tee` logs.
- [ ] Review the exact workflow diff, run local Python unit tests without boot-disk scratch, then commit only workflow changes with `ci: collect infercnv creation witnesses`.
- [ ] Push the exact world head, wait for exact-head Control plane CI, dispatch the installed-package workflow in shipped mode, wait for terminal status, and acquire its original artifact and logs under `/Volumes/Zane's HDD/rsomics-fixtures/evidence/` without deleting prior evidence.
- [ ] Independently audit hashes, package/source identity, RDS/source-to-export equality reported by R, Python checker output, dimensions, decimal hex strings and all warnings. Record both successes and failures in the state note; accept only if every required check passes.
- [ ] Advance to product-local raw ingestion design only after the witness scope is accepted. Do not claim full inferCNV or arbitrary fractional raw-input parity.
