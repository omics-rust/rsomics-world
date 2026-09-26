# inferCNV Oracle Implementation Plan

> **For agentic workers:** Use superpowers:subagent-driven-development for the
> isolated harness task and its independent review. The controller owns Git
> commits, pushes, CI dispatch, and durable evidence records.

**Goal:** Produce inspected, source-pinned stage-level results from real infercnv
1.28.0 so a native implementation cannot pass on qualitative smoke tests alone.

**Architecture:** A control-plane R driver generates a deterministic input and
invokes the real package. A Python checker validates its exported artifact.
GitHub Actions supplies the runtime without using the Mac boot disk.

**Tech Stack:** R 4.6, Bioconductor 3.23, infercnv 1.28.0, JAGS 4, Python standard
library, GitHub Actions.

**Spec:** `docs/plans/2026-09-26-infercnv-oracle-design.md`.

## Global Constraints

- Use source commit `b421d9405c97a309b081ef86d455e976df93eae4` and assert version `1.28.0`.
- Do not build, install dependencies, or run product binaries locally while boot APFS exceeds 80%.
- Local scratch uses `/Volumes/KIOXIA/Developments/tmp`; use Python `-B` and external `TMPDIR`.
- No deleted repository revival, new public API, Layer B dependency, or publication.
- No changes to inherited VCF work or unrelated untracked files.
- Source comments remain rare; errors must propagate.
- Main owns commits and pushes; agents do not stage, commit, publish, or delete.

## Review Focus

- A missing or mismatched upstream package must fail before creating accepted evidence.
- A fixture whose gain disappears under normalization must not pass on shape alone.
- Lost gene/cell order must fail even when numeric matrix dimensions still match.
- Missing intermediate checkpoints must not silently yield a partial green oracle.
- A path escaping the result bundle or a stale source identity must fail validation.

## Task 1: Real-package oracle harness and artifact validation

**Files:**
- Create `scripts/infercnv_oracle.R` for deterministic inputs, real upstream invocation, and stage export.
- Create `scripts/validate_infercnv_oracle.py` for artifact validation and optional manifest generation.
- Create `scripts/test_infercnv_oracle.py` for controlled artifact validation failures.
- Create `.github/workflows/infercnv-oracle.yml` for pinned remote execution and evidence upload.

**Interfaces:**
- R entry: `Rscript --vanilla scripts/infercnv_oracle.R OUTPUT_DIRECTORY`.
- Checker entry: `python3 -B scripts/validate_infercnv_oracle.py OUTPUT_DIRECTORY`.
- Python API: `validate_bundle(root: pathlib.Path) -> dict` returns counts after validation and raises `ValueError` for rejected bundles.
- `oracle.json` records schema version 1, package/version/source commit, explicit profile settings, and each exported stage's relative paths. The implementer must document its exact artifact schema in the script's user-facing module docstring; generator and checker share this contract, not implementation code.
- Profile names: `single_reference`, `grouped_bounds`, `grouped_mean`, `no_reference`.
- Each profile exports stages `[1, 2, 3, 4, 8, 9, 10, 11, 12, 14]` with expression, gene-order, and cell-group identity information.

- [x] Write behavior tests before the checker. Tests construct tiny independent valid bundles, then mutate one property per rejection case. For example:

```python
def test_rejects_nonfinite_expression(self):
    root = self.make_valid_bundle()
    matrix = root / "single_reference/04.tsv"
    matrix.write_text("gene\tcell_1\nG1\tNaN\n")
    with self.assertRaisesRegex(ValueError, "finite"):
        validate_bundle(root)
```

  Also cover wrong version/commit, missing profile/stage, duplicate IDs, gene or cell order mismatch, ragged rows, path traversal, absent referenced files, and a valid bundle. Fixture helpers are test-only and never imported by production scripts.
- [x] Run `python3 -B -m unittest discover -s scripts -p test_infercnv_oracle.py` with external `TMPDIR`; retain the expected pre-implementation failure.
- [x] Implement the checker, run focused tests, then `python3 -B -m unittest discover -s scripts -p 'test_*.py'`.
- [x] Implement the R driver using actual upstream checkpoint RDS files, not a hand-written replica of upstream formulas. Create inputs with the coverage in the spec, set every selected parameter explicitly, validate stage identity/finiteness, and export numeric values using 17-digit formatting. Record `sessionInfo()`, installed packages, seed, input and checkpoint artifacts. The actual R oracle must decide numerical goldens.
- [x] Add a manually dispatched Linux workflow. Install R 4.6, JAGS and dependency requirements, obtain exact pinned source and record its SHA-256, install dependency packages without silently upgrading the oracle, assert `packageVersion("infercnv")`, and run driver plus checker. Evidence upload uses `if: always()` and includes install/run logs, source identity, input data, intermediate exports/RDS and package/session metadata. Use explicit runner-temp paths for all caches/scratch and no secrets.
- [x] Self-review and provide TDD evidence, changed files, exact schema, and unresolved runtime risks in `.autopilot/state/infercnv-oracle-implementation-2026-09-26.md`. No claim of upstream execution until CI actually runs.
- [x] Controller obtains an independent spec/quality review, fixes defects through the implementer, commits only these files, pushes, and verifies exact-head control-plane CI.

## Task 2: Execute and inspect the pinned oracle

**Files:**
- Update `.autopilot/state/infercnv-priority-2026-09-26.md` with run identity and evidence.
- Retain evidence beneath `/Volumes/Zane's HDD/rsomics-fixtures/evidence/infercnv-2026-09-26/`.

**Interfaces:** Consumes the Task 1 workflow and bundle checker. Produces an
immutable run receipt with exact head, conclusion, archive hash, source hash,
profile/checkpoint counts and scoped interpretation.

- [x] Dispatch `.github/workflows/infercnv-oracle.yml` at the pushed head and inspect the terminal job result.
- [x] If installation or execution fails, retain the raw failure and diagnose before editing the harness; never replace a missing oracle with local approximations.
- [x] Retrieve the exact-run artifact to external storage, verify archive integrity, validate exported identities/numerics, and independently inspect depth normalization, log2/inverse relation, reference-mode differences and chromosome-boundary cases.
- [x] Record what is proved and what is not: this establishes deterministic pre-clustering upstream evidence, not Rust compatibility, native-platform CI, performance, or release readiness.
- [ ] Commit the evidence receipt, push, and verify exact-head control-plane CI. Proceed to the native numeric-core plan only after the inspected oracle is valid.
