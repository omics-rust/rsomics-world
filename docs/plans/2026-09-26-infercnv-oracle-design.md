# inferCNV priority and oracle design

## Intent and boundary

The user prioritized inferCNV on 2026-09-26 and requested parallel unattended
work. The objective is a native, scientifically traceable single-cell CNV
workflow, not revival of the deleted `rsomics-infercnv` micro-crate.

Keep the accepted boundary: `rsomics-sc cnv` owns expression-derived CNV;
the released `rsomics-cnv` owns VCF/BCF BAF/LRR workflows. This prioritization
does not require moving either boundary or finishing all counts-to-clusters
operations first. The CNV implementation must ultimately consume and produce
identity-aligned annotated state. Internal development slices are not releases.

Three approaches were considered:

1. Merge the old five-step approximation: rejected because the behavior and
   evidence are not inferCNV-compatible.
2. Add expression data to `rsomics-cnv`: rejected because its data model,
   statistical assumptions, and public contract are different.
3. Build the expression-CNV workflow inside `rsomics-sc`, starting from pinned
   upstream intermediate results: selected. It preserves the product map and
   makes each numerical stage independently falsifiable.

No new Layer A crate or item is required for the oracle. Future product work
uses `rsomics-common` and `rsomics-help`; expression matrices, gene ordering,
reference policy, and expression-specific HMM remain product-local until two
concrete consumers justify an API. No Layer B-to-B dependency is introduced.

## Verified starting point

- Retired source `rsomics-infercnv` is clean at
  `393882653ea27e65f38ec6997f8f26e91d04927b`; its published 0.1.0 tag predates
  that head. Retain parser/fixture ideas, not scientific outputs as goldens.
- The old core silently turns bad coordinates and expression values into zero,
  smooths across chromosome boundaries, omits depth normalization and reference
  groups, and accepts an empty reference selection. Its compatibility test
  checks only qualitative direction; its benchmark uses a nonexistent flag.
- `rsomics-sc` has neither a local directory nor a GitHub repository at this
  audit. Do not create a repository merely to reserve its name.
- The existing BAF/LRR product is clean at
  `d0c4ba873d72e388f7f375d12c3b89fa0747dc7a`.

## Authoritative behavior

Use Bioconductor 3.23 `infercnv` 1.28.0 at commit
`b421d9405c97a309b081ef86d455e976df93eae4`, not Broad's older master branch.

- [Release metadata](https://bioconductor.org/packages/3.23/bioc/html/infercnv.html)
- [Pinned source](https://github.com/bioconductor-source/infercnv/tree/b421d9405c97a309b081ef86d455e976df93eae4)
- [Object creation and ordering](https://github.com/bioconductor-source/infercnv/blob/b421d9405c97a309b081ef86d455e976df93eae4/R/inferCNV.R)
- [Pipeline and numerical operations](https://github.com/bioconductor-source/infercnv/blob/b421d9405c97a309b081ef86d455e976df93eae4/R/inferCNV_ops.R)
- [Workflow documentation](https://github.com/broadinstitute/infercnv/wiki/Running-InferCNV)
- [Outputs](https://github.com/broadinstitute/infercnv/wiki/Output-Files)

The package license is BSD-3-Clause plus its LICENSE file. Preserve license
and source attribution with any reused code or data. Broad's current README
says inferCNV is no longer supported; compatibility with a pinned established
workflow remains a useful engineering target, but is not an endorsement of
its biological conclusions or a claim of continuing upstream support.

Source-backed hazards include `log2(x + 1)` rather than natural logarithms,
gene filtering over all retained cells despite reference-only wording in the
documentation, separate reference-group bounds rather than one pooled mean,
and actual `analysis_mode` defaulting to subclusters. Ordering, filtering,
smoothing endpoints, medians, and inverse transforms need stage-level oracles.

## Current deliverable: source-pinned pre-clustering oracle

This plan builds verification infrastructure in the control plane, not a Rust
product or a user-facing partial CLI. Run the real installed upstream package
through step 14 with explicit parameters. Step 15 performs clustering even in
sample mode and is outside this bounded oracle.

The oracle runs four profiles: one reference group; two reference groups with
mean bounds; two groups with mean-of-group-means subtraction; and an explicit
no-reference baseline. All use `cutoff=0.1`, `min_cells_per_gene=3`,
`window_length=101`, `smooth_method="pyramidinal"`,
`max_centered_threshold=3`, `scale_data=FALSE`, `HMM=FALSE`, `denoise=FALSE`,
`analysis_mode="samples"`, `num_threads=1`, `resume_mode=FALSE`,
`plot_steps=FALSE`, `no_plot=TRUE`, `no_prelim_plot=TRUE`, `save_rds=TRUE`,
`remove_genes_at_chr_ends=FALSE`, `prune_outliers=FALSE`,
`mask_nonDE_genes=FALSE`, `num_ref_groups=NULL`, `cluster_by_groups=TRUE`,
`cluster_references=TRUE`, and `up_to_step=14`. Object creation uses
`min_max_counts_per_cell=c(1, Inf)`, `chr_exclude=c("chrX", "chrY", "chrM")`,
and `max_cells_per_group=NULL`. Unselected behavior is not declared compatible.

| Profile | `ref_group_names` | `ref_subtract_use_mean_bounds` |
|---|---|---|
| `single_reference` | `c("normal_a")` | `TRUE` |
| `grouped_bounds` | `c("normal_a", "normal_b")` | `TRUE` |
| `grouped_mean` | `c("normal_a", "normal_b")` | `FALSE` |
| `no_reference` | `NULL` | `TRUE` |

The no-reference profile uses all observation cells as one proxy reference at
both subtraction stages; it does not use a zero baseline. Step 11 uses each
cell's median over all retained genes, not a separate median per chromosome.

Capture exactly checkpoints 1, 2, 3, 4, 8, 9, 10, 11, 12, and 14. These are
sparse upstream step numbers, not ten consecutive steps. Preserve actual
checkpoint names, expression matrices, gene order, and reference/observation
indices. Export finite numeric values at round-trip precision with exact row
and column identifiers. Capture installed dependency versions, R/platform,
seed, input hashes, source commit/archive hash, explicit arguments, and raw log.

The synthetic fixture has at least twelve cells, two unequal reference groups,
two observation groups, chromosomes longer than 101 genes, shorter than 101
genes, and a single-gene chromosome. It includes shuffled input order,
low-expression and sparsely detected genes, and a gene absent from genomic
ordering. Signals must exercise gains, losses, and distinct reference baselines;
shape-only tests are insufficient. Inputs are reproducible integer counts,
cell annotation TSV, and the upstream four-column gene-order table, not GTF.

A neutral background must contain more genes than the altered chromosomes
combined. Predeclare a gain chromosome and a loss chromosome for one observation
group. Under `single_reference`, their median stage-14 signals must respectively
exceed and fall below 1, with corresponding positive/negative stage-12 signals.
Depth-normalized column totals must agree at stage 3; stage 4 must be
`log2(stage3 + 1)` and stage 14 must be `2^stage12`. These checks do not substitute
for the saved upstream results. A single-gene chromosome must remain unchanged
during smoothing; a blanket requirement that every stage change is invalid.

The result bundle contains `oracle.json`, profile/checkpoint TSVs, input files,
RDS checkpoints, session information, and source provenance. A standard-library
Python checker rejects missing profiles/stages, stale package version/commit,
path escapes, missing files, duplicate or misaligned identities, ragged/nonfinite
matrices, empty results, and absent declared signal contrasts. Validated bundles receive
a sorted SHA-256 manifest. A checksum certifies bytes, not scientific agreement.

CI installs the pinned package and its real dependencies, including JAGS, on a
GitHub-hosted Linux runner. Missing dependencies are failures, never skipped
compatibility. Always upload available raw evidence on failure. Only terminal
exact-head success and inspected artifacts count as an accepted oracle run.
This first oracle is source-pinned and environment-recorded: installed dependency
versions are retained, but an independently restored dependency lock has not yet
been verified. Do not claim byte-identical future environment reproduction.
This is one-platform oracle preparation, not the four-native-platform release
gate required for the eventual Rust product.

## Subsequent implementation gates

1. Import a strictly validated raw-count matrix, cell/group annotations, and
   gene order into product-local identity-aligned state. Preserve raw counts.
2. Implement the ten captured stages with checkpoint differentials, failure
   tests, allocation budgeting, and chromosome-separated execution. No hidden
   conversion of malformed numbers or unknown references into valid data.
3. Add sample clustering and the preliminary report, then denoising and
   subclustering with separately pinned profiles and independent evidence.
4. Add expression-specific i3/i6 models, spike calibration, Bayesian filtering,
   reports, and resumability. The BAF/LRR HMM is not a compatible substitute.
5. Measure equivalent upstream/native work on public real data with pinned
   provenance, repeated wall/CPU/peak-RSS measurements, and identical outputs.
6. Review API/hot paths, finish unified CLI/transactional reports, verify all
   four native platform classes, and only then consider publication.

## Execution constraints

The 2026-09-26 boot APFS check found 5,357,383,680 free bytes of
245,107,195,904 bytes, approximately 97.81% occupied. Local compilation,
dependency installation, and product execution stay stopped. Source editing,
read-only inspection, and small control-plane Python checks with `-B` and
external `TMPDIR` are allowed; CI handles dependency installation and R runs.
Do not delete files or clean disks. Keep existing VCF work and failed evidence.

Project files, caches, and scratch remain on the two authorized external disks.
Commit only this concern on control-plane `main`; push after review and inspect
the exact-head control-plane and dispatched oracle runs. Routine design and
execution choices are delegated by the user; the AGENTS stop conditions still
apply. Durable progress belongs in `.autopilot/state`.
