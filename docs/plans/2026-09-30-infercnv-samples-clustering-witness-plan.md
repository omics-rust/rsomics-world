# inferCNV samples-clustering witness implementation plan

> Execution: use executing-plans with isolated exporter/checker ownership and a fresh whole-change review. Routine decisions and direct-main delivery are covered by the user's unattended authorization.

**Goal:** obtain inspectable installed-infercnv evidence for the non-HMM
samples/Ward.D2 path without claiming native clustering or default Leiden.

**Architecture:** extend the existing pinned oracle workflow. One R exporter
executes the real namespace function and complete workflow; an independent
Python checker verifies immutable exports, structural invariants and route
agreement. Keep source probes separate from end-to-end workflow cases.

**Tech:** R 4.6.1, infercnv 1.28.0, fastcluster, parallelDist, Python standard
library and existing oracle IO/artifact verifiers.

**Spec:** `2026-09-30-infercnv-samples-clustering-witness-design.md`.

## Global constraints

- Source commit `b421d9405c97a309b081ef86d455e976df93eae4`, archive SHA-256
  `b2a1b6f8cc09dc04562513e3cd877b3c97a6efcd5ff92cb5e0763660905e3207`.
- No local R/Cargo build or dependency download while APFS exceeds 80%.
  Controller test scratch is `/Volumes/KIOXIA/Developments/tmp`.
- HMM, denoise, pruning, masking, scaling, chromosome-end removal and plotting
  are off. `num_ref_groups=NULL`, seed 260930, one requested distance thread.
- Calls use `analysis_mode="samples"`, `hclust_method="ward.D2"`,
  `partition_method="none"`; defaults are not inferred from this witness.
- Do not copy upstream clustering code into a native implementation. Metadata
  records the actual package versions, seed, locale and complete arguments.
- No public foundation/API, product CLI, publication or performance claim.

## Review focus

1. Small groups may lack a tree-list entry: compare by group name, not position.
2. Pooled observations concatenate group maps rather than matrix-column order.
3. Positive z-score settings use the fixed 0.8 source threshold; empty/constant
   references require observed outcomes, not a repaired expected result.
4. Ward ties require actual fastcluster merge/order/labels, not just membership.
5. Generated pooled names can collide with references; record actual outcomes
   separately instead of declaring a native policy.

## File and data contract

Create `scripts/infercnv_samples_clustering_witness.R`,
`scripts/validate_infercnv_samples_clustering_witness.py` and
`scripts/test_infercnv_samples_clustering_witness.py`. The controller owns
workflow integration, original artifact acquisition, receipt and dossier.

Exporter CLI:

```text
Rscript scripts/infercnv_samples_clustering_witness.R SYNTHETIC_BUNDLE SHIPPED_BUNDLE NEW_OUTPUT_DIR
```

Checker CLI and callable:

```text
python3 -B scripts/validate_infercnv_samples_clustering_witness.py SYNTHETIC_BUNDLE SHIPPED_BUNDLE WITNESS_DIR
validate_witness(synthetic: Path, shipped: Path, witness: Path) -> dict
```

Top-level `witness.json` records `schema_version=1`, pinned `package`,
`runtime` (R, locale, seed, package versions, requested threads), `cases`, and
SHA-256 `files`. Each case declares `kind` (`workflow` or `probe`), explicit
arguments and input/checkpoint hashes. A workflow has both `isolated` and
`full_run` output records. A probe has `isolated` only. Each output is either
`status="success"` with state/tree exports and hashes, or `status="error"`
with nonempty error message, class list and call. Workflow errors fail the
exporter/checker; probe errors remain observed characterization, not accepted
compatibility. Paths are relative, unique and confined to the output root.

Successful exports include expression/genes/cells and ordered creation maps;
ordered group names/indices; per-group optional `tree` (merge pairs, 17-digit
heights, order, labels, method), named ordered subclusters and index names;
selected one-based gene indices, filtered expression and lower-column-major
Euclidean distance vectors. Keep all actual RDS results and preliminary files.
Empty versus missing list entries and NULL versus empty index names must be
represented deliberately, never converted through automatic scalar unboxing.

## Task 1: independent checker and regression contracts

**Owner:** checker worker; owns only the two Python files.

- [ ] Add failing tests before implementation. Run the focused suite and record
  the missing-module failure using external scratch.

```python
spec = importlib.util.spec_from_file_location("samples_checker", CHECKER)
self.assertTrue(CHECKER.exists(), "samples clustering checker is absent")
module = importlib.util.module_from_spec(spec)
spec.loader.exec_module(module)
```

- [ ] Add `validate_tree(tree, indices, cell_names)` to check finite nonnegative
  Ward heights, exactly n-1 merges, negative leaf range and positive references
  only to earlier merge rows, unique full leaf coverage, DFS leaf order,
  labels corresponding to input indices and member order matching that tree.
  A missing tree is valid only for group size <=2. Test at least:

```python
tree = {"merge": [[-1, -2], [-3, 1]], "height": [0.0, 2.0],
        "order": [3, 1, 2], "labels": ["a", "b", "c"], "method": "ward.D2"}
validate_tree(tree, [1, 2, 3], ["a", "b", "c"])
# Separate tests replace a child with its own positive merge row, duplicate a
# leaf, permute order, change a label, set a negative/NaN height, and add a tree
# to a one-cell group. Each must fail at the changed contract.
```

- [ ] Verify cell-map coverage, group order, pooled concatenation, tree/subcluster
  names and named ordered membership. Test a shuffled global index list and a
  two-cell group without a tree. Bind filtered genes/distances to retained
  inputs and verify finite distance shape n*(n-1)/2. Validate selection using
  the literal source filter, but do not run a replacement Ward oracle.
- [ ] Validate hashes, package/runtime pins and complete argument identities;
  reject traversal/symlinks, missing files, reordered groups, conflicting
  routes, missing preliminary checkpoint and mutated matrix values. Probe
  failures must not satisfy a workflow success contract.
- [ ] Run focused and whole controller tests with external TMPDIR. Record real
  counts; neither hand-built regression fixtures nor checker green is an
  installed-package acceptance.

## Task 2: real-package exporter

**Owner:** exporter worker; owns only the R file.

- [ ] Read source functions and existing shipped/synthetic scripts; validate
  every source/env/runtime pin and input checksum before loading checkpoints.
  Reuse `infercnv_oracle_io.R`; fail if the output directory already exists.
- [ ] Set the real package thread environment and obtain the actual function:

```r
ns <- asNamespace("infercnv")
cluster <- get("define_signif_tumor_subclusters", envir=ns)
context <- get("infercnv.env", envir=ns)
context$GLOBAL_NUM_THREADS <- 1L
set.seed(260930L)
isolated <- cluster(stage14, hclust_method="ward.D2", partition_method="none",
                    cluster_by_groups=grouped, z_score_filter=z_setting)
```

- [ ] Use shipped subset/full plus synthetic grouped_bounds/no_reference, each
  grouped and pooled, with filter 0 and 0.8. For each workflow case use a fresh
  creation object and complete explicit run settings matching its source
  profile, except `up_to_step=15L` and the declared clustering options.
  Resolve actual filenames with `.get_relevant_args_list`; retain stage14,
  stage15 and preliminary RDS. Compare isolated/full-run expression, maps,
  trees and subclusters exactly; both routes must succeed.
- [ ] Export actual tree fields by subcluster group names. Bind diagnostic
  distance/tree computations to the actual returned package trees rather than
  using generated expectations. Matrix/group identity must remain unchanged.
- [ ] Add controlled function-only probes for 1/2/3-cell groups, shuffled
  indices, equal profiles/equidistant ties, multiple reference order,
  z-score 0/0.2/0.8, constant/no/all-outlier references and pooled
  `all_observations` collision. Save the pre-call valid object and all actual
  output/warning/error fields; do not force degenerate calls to succeed.
- [ ] Export complete provenance, original script identities, session, actual
  fastcluster/parallelDist versions and sorted output hashes. R syntax parsing
  is allowed on 4090 with `/dev/shm` scratch; do not assume infercnv is installed
  there or classify parsing as execution.

## Task 3: integration, review and actual acceptance

**Owner:** controller; owns workflow, receipts and control-plane records.

- [ ] Add an explicit opt-in `samples_clustering` dispatch input, default false;
  require shipped dataset for this witness. Run focused checker tests, exporter
  and checker after existing verified shipped/creation bundles. Include script
  hashes and all files in the original uploaded artifact.
- [ ] Run complete controller tests and R syntax checks. Fresh independent
  review covers source/API pins, export shapes, route independence, literal
  edge coverage and whole-artifact provenance. Resolve substantive findings.
- [ ] Commit only owned files, push main, wait exact-head control CI; dispatch
  shipped with `samples_clustering=true`. Confirm created run head before
  assigning evidence status.
- [ ] Acquire original run/jobs/artifact metadata, ZIP and logs externally.
  Verify API digest/size, CRCs, safe unique paths, input/output/script hashes
  and source manifests before extraction. Run independent checker and review
  raw logs, warnings/errors and both routes, not only JSON summaries.
- [ ] Record accepted successful cases and observed failing probes separately.
  Update the dossier only with verified scope. Preserve failures unchanged.
  Native clustering design/implementation and representative measurement
  remain future gates; continue with the evidence-backed next action.
