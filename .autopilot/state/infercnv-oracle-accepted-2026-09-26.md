# Accepted inferCNV pre-clustering oracle

Run `36216764707` completed successfully at exact world head
`38625af42b55b089678a387b79adf4cc8d4e8a28`. Its matching Control plane run
`36216747712` also passed. The acceptance below follows artifact inspection,
not the green job status alone.

## Preserved evidence

Directory:
`/Volumes/Zane's HDD/rsomics-fixtures/evidence/infercnv-2026-09-26/run-36216764707/`.
It contains terminal run/jobs/artifact API records, the original artifact ZIP,
full job-logs ZIP, extracted source and results, and the independent numerical
inspection script. Nothing from the failed earlier runs was overwritten.

| Evidence | SHA-256 |
|---|---|
| Original artifact ZIP | `9152bc2438db96635be7c17116151ff0b3e64fbabc4a4e79050f1a2f3e80ff6d` |
| Full job logs ZIP | `1cee86b9e4cb4f7f2c10f76308ba18f007bf2e737510f8ec81ba5e30fbcaf9a8` |
| Pinned source archive | `b2a1b6f8cc09dc04562513e3cd877b3c97a6efcd5ff92cb5e0763660905e3207` |
| `bundle/oracle.json` | `a2fd7989553a4457bed3556b37798b999a31b8fa4c8b6e04eb83f32764039171` |
| `bundle/sha256-manifest.tsv` | `9211aa334c75ac014eca18f8888483c5b881d62b468b8341f9a7baf298e50f57` |
| `independent-numeric-review.py` | `b9027e2a0be40b900d31e3ab9915df8bd951f380462918ef02066dbfef278076` |

The controller verified both ZIP CRCs; artifact SHA-256 against GitHub's API
digest; all 297 extracted files against their ZIP members; all 167 sorted
bundle-manifest entries; all 40 distinct nonempty original RDS checkpoints;
and the source archive hash against the oracle record. The committed checker
also passed on the downloaded bundle without rewriting its manifest.

Installed runtime: infercnv 1.28.0 from source commit
`b421d9405c97a309b081ef86d455e976df93eae4`, R 4.6.1, Bioconductor 3.23,
Ubuntu 24.04.5 x86_64, OpenBLAS 0.3.26. The artifact records package versions,
library paths, locale, explicit arguments, source identity, and input hashes.
This is source-pinned and environment-recorded, not a restored dependency lock.

## Numerical inspection

Four profiles export ten checkpoints each: single reference, unequal grouped
references with bounds, equally weighted reference-group means, and no-reference
proxy. Input dimensions are 449 genes by 13 cells; creation produces 448 rows,
and low/detection filtering produces 446 rows. The post-filter depth target is
10,006. The exact retained chromosome sizes are 300, 128, 17, and 1.

A separate reviewer directly read the TSVs and reconciled formulas with pinned
source without importing the production validator. It recomputed 185,536
entries across stages 3, 4, 8, 9, 10, 11, 12, and 14. Maximum absolute
discrepancy was `8.881784197001252e-16`. No gene, coordinate, cell, annotation,
reference-role, or ordering discrepancy was found. Singleton smoothing is
exactly unchanged; reference-mode outputs are meaningfully distinct.

Single-reference gain-observation medians are `2.259085575933253` on the gain
chromosome and `0.2370792346661384` on the loss chromosome. Corresponding
centered medians are `1.1757389221856736` and `-2.076558789537417`.
Full per-stage values and method are in
`infercnv-oracle-numeric-review-2026-09-26.md`.

## Accepted scope and remaining gates

Accept these bytes as a finite, synthetic, pre-clustering numerical oracle
for native core development. They do not establish Rust compatibility,
performance, biological validity, downstream parity, or publication readiness.

- Stage-9 clipping changed no entries. Add explicit positive/negative and
  equality-at-threshold tests; do not claim saturation coverage from this run.
- Chromosome lengths 2, 100, 101, and 102 remain source-derived edge cases,
  not observed cases in this artifact.
- No NA path, zero-depth-after-filter case, real-data workflow, step-15+
  clustering, denoising, i3/i6, JAGS, or Bayesian filtering was executed.
- The observed error and the review's `1e-9` stop threshold are not universal
  tolerances. Native comparisons must declare their own bounded tolerances and
  cover all four native platforms.
- The complete workflow remains `rsomics-sc cnv`; neither an approximate
  micro-crate revival nor a new public matrix/HMM foundation is warranted.
