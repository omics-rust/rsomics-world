# inferCNV oracle harness implementation, 2026-09-26

## Scope and state

Created only `scripts/infercnv_oracle.R`, `scripts/validate_infercnv_oracle.py`,
`scripts/test_infercnv_oracle.py`, and `.github/workflows/infercnv-oracle.yml`
for the task, plus this report. No stage goldens, installed infercnv run, release,
or scientific compatibility result is claimed. The workflow must be dispatched
and its exact-run evidence inspected before acceptance.

The R driver generates deterministic integer-count inputs (13 cells; 3
`normal_a`, 4 `normal_b`, 3 `gain_obs`, 3 `loss_obs`; 300 neutral, 128 gain,
17 loss and 1 single-gene chromosome, plus low/sparse genes and an absent-order
gene). Gene and cell inputs are shuffled before object construction. It calls
installed `infercnv::CreateInfercnvObject` and `infercnv::run` with explicit
selected parameters and `up_to_step=14`. Original checkpoint file names come
from pinned package `.get_relevant_args_list`, not a local formula copy. The
driver reads those real RDS objects, exports finite 17-significant-digit TSVs,
and checks identity, stage-3 column sums, stage-4 `log2(x+1)`, stage-14 `2^x`,
single-reference final gain/loss direction, and non-collapsed reference modes.
These are execution gates, not observed results yet.

## Bundle contract

The checker module docstring is the user-facing schema reference. In brief,
`oracle.json` schema 1 contains pinned package/version/commit/archive SHA-256,
seed, explicit shared run settings, profile settings, bundle-relative paths
for three input TSVs and three metadata files, input SHA-256 values, and exactly
four profiles with ten required stages each. Every stage references expression,
gene-order, cell-group TSVs and the untouched upstream RDS checkpoint. The
checker requires complete stage/profile presence, finite rectangular matrices,
unique IDs, aligned gene/cell order, valid roles and reference settings,
contained existing paths, input hashes, and nontrivial stage-3/4 changes. Its
CLI can emit a sorted `sha256-manifest.tsv` after validation. A manifest only
certifies the bytes exported, not scientific correctness.

## Verification receipt

- TDD initial red: external-`TMPDIR` `python3 -B -m unittest discover -s scripts -p test_infercnv_oracle.py` failed because the checker module was absent (1 import error). With a minimal checker interface, 14 tests errored at `NotImplementedError`.
- First green: focused 14/14 tests passed. A second red cycle added input-hash and profile-role/reference cases: 3 expected assertion failures among 17 tests. A third red cycle caught malformed package metadata raising `AttributeError` instead of `ValueError`. Final focused run: 18/18 passed.
- Full suite: `TMPDIR=/Volumes/KIOXIA/Developments/tmp PYTHONDONTWRITEBYTECODE=1 python3 -B -m unittest discover -s scripts -p 'test_*.py'` passed 25/25.
- Syntax only: streamed R source to remote R 4.6.1 `parse(text=readLines("stdin"))`, which returned `syntax-ok`; no R driver or infercnv execution was performed. Ruby YAML parse found 11 workflow steps.

### Independent-review fix round 1

The reviewer found that the initial checker could accept shape-correct but
scientifically empty or identity-drifted exports. Added independent tests
first: the old two-cell fixture produced 7 expected failures among 25 tests;
after replacement with an input-backed nine-cell, gain/loss/singleton fixture,
10 new rejection tests failed among 28. The checker now binds stage gene
coordinates and cell annotations to raw inputs, requires every requested
reference group, verifies common and creation settings, checks stage-3 totals,
stage-4 log2, stage-14 inverse log2, singleton stage-9-to-10 preservation,
gain-and-loss signs in `gain_obs`, and step-8 profile differences. The R
fixture now gives `gain_obs` both chrGain elevation and chrLoss depression.
Final focused checker run passed 28/28; full control-plane Python suite passed
35/35. Remote R 4.6.1 syntax-only parse again returned `syntax-ok`. The new
numeric gates remain unverified against real infercnv execution.

### Independent-review fix round 2

The reviewer identified a two-cell median counterexample: stage-12 gain
values `[-2,1]` have a negative median while their stage-14 inverse-log2
values `[0.25,2]` have a median above one. Two tests were added first and
failed as expected among 30 focused tests. The checker now independently
requires stage-12 gain median above zero and loss median below zero, as well
as the existing stage-14 signs. The R driver asserts the same stage-12 signs.
The two additional explicit run arguments (`tumor_subcluster_partition_method`
and `BayesMaxPNormal`) now appear in shared JSON settings and are passed only
once to `infercnv::run` and `.get_relevant_args_list`. Final full Python suite:
37/37 passed. Remote R 4.6.1 syntax-only parse: `syntax-ok`. No infercnv run.

## Remote-run risks and next gate

The workflow pins Ubuntu 24.04, R 4.6.1/Bioconductor 3.23, JAGS 4 and the
specified source commit. It saves the downloaded source archive and its
SHA-256, installs real DESCRIPTION imports, asserts installed infercnv version,
and uploads raw logs/partial bundle even on failure. Dependency resolution
(especially `Seurat`, `rjags`, and native system libraries), R/Bioconductor
availability, package checkpoint behavior, and numerical fixture survival are
unverified until the GitHub run. Installed package versions and `sessionInfo()`
are recorded as environment evidence, not a complete reproducibility lock.
If any dependency, checkpoint, arithmetic, or gain/loss gate fails, retain
failure artifacts and diagnose; do not substitute approximated outputs.
Ubuntu Noble lists JAGS as a single `jags` package with headers; the nonexistent
`libjags4` package was removed from the workflow before execution.
