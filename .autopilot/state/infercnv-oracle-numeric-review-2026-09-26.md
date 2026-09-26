# Independent inferCNV oracle numeric review, 2026-09-26

## Outcome and scope

PASS for the actual four-profile, finite synthetic pre-clustering bundle.
The greatest absolute entrywise discrepancy among independently recomputed
stages 3, 4, 8, 9, 10, 11, 12, and 14 is
`8.881784197001252e-16` (no-reference stage 8), below the controller's
predeclared `1e-9` stop threshold. There were no identity surprises.
This is numerical inspection of an installed-package oracle artifact, not
evidence of Rust compatibility, biological validity, downstream HMM parity,
or performance superiority.

Run: `36216764707`; controller-reported successful exact head:
`38625af42b55b089678a387b79adf4cc8d4e8a28`.
Bundle:
`/Volumes/Zane's HDD/rsomics-fixtures/evidence/infercnv-2026-09-26/run-36216764707/artifact/bundle`.
Archive, manifest, and package/session provenance checks are separately owned
by the controller; this reviewer did not repeat or independently certify them.

## Independent method and reproducibility

A new Python standard-library script reads TSVs directly. It does not import,
call, or copy the production bundle validator and does not execute R, install
dependencies, or build any product. Checkpoint formulas were reconciled with
the archived source for commit
`b421d9405c97a309b081ef86d455e976df93eae4`, particularly
`R/inferCNV_ops.R:1681-1787` (reference means/bounds),
`:2406-2532,2640-2660` (chromosome-local smoothing), and
`:2074-2105` (global column median).

Scratch script:
`/Volumes/KIOXIA/Developments/tmp/infercnv-independent-numeric-review-2026-09-26.py`

Identical durable copy retained by the controller:
`/Volumes/Zane's HDD/rsomics-fixtures/evidence/infercnv-2026-09-26/run-36216764707/independent-numeric-review.py`.

Script SHA-256: `b9027e2a0be40b900d31e3ab9915df8bd951f380462918ef02066dbfef278076`

Executed successfully with:

```sh
TMPDIR=/Volumes/KIOXIA/Developments/tmp PYTHONDONTWRITEBYTECODE=1 python3 -B /Volumes/KIOXIA/Developments/tmp/infercnv-independent-numeric-review-2026-09-26.py
```

Every stage is reconstructed from the relevant immediately preceding observed
checkpoint, so differences isolate that stage rather than accumulating
earlier errors. Each requested stage compares all 5,798 entries per profile:
185,536 entrywise comparisons across eight stages and four profiles.
Python `math.fsum` is used for sums; no rounding, rescaling, fitting,
or correction of exported values is performed.

- Stage 3: each retained count divided by its column total, multiplied by
  the median of the post-filter column totals.
- Stage 4: `log2(stage3 + 1)`.
- Stages 8 and 12: means within each named reference group. Bounds mode
  subtracts the lower/upper group-mean boundary outside that interval and
  returns zero inside. Grouped-mean mode subtracts the equally weighted
  mean of the group means, not a cell-pooled mean. No-reference mode uses
  all 13 cells as one proxy group.
- Stage 9: pointwise clamp to `[-3, 3]`.
- Stage 10: independent direct triangular weighted sum within each chromosome,
  weight `51 - abs(i-j)`, radius 50, divided by retained weight sum.
  A single-gene chromosome is copied exactly, matching the upstream skip.
- Stage 11: subtract each cell's median over all 446 retained genes,
  not a separate median by chromosome.
- Stage 14: `2 ** stage12`.

## Identity and fixture observations

All profiles share the same exact ordered cell IDs and retained gene IDs.
Each stage's gene coordinates and cell annotations were checked against
the raw input tables; reference/observation roles were checked against the
specified profile. Within-chromosome coordinates are sorted.

Raw input is 449 genes by 13 cells. Creation removes only
`ABSENT_FROM_ORDER`, yielding 448 by 13 at stage 1.
Stage 2 removes `LOW` and `SPARSE`, yielding 446 by 13 at all remaining
captured stages. The retained subset and untouched counts were independently
checked against mean-count cutoff 0.1 and detection in at least three cells.

Groups contain 3 `normal_a`, 4 `normal_b`, 3 `gain_obs`, and 3
`loss_obs` cells. Single-reference mode selects only the first group;
grouped modes select both unequal-sized reference groups. No-reference
mode assigns all cells observation roles.

The depth target is 10,006 in every profile. Recomputed stage-3 column
totals match that target with zero observed absolute error using
`math.fsum`.

## Maximum absolute entrywise discrepancies

Zero means exact equality of the parsed floating-point values and this
independent calculation, not a claim of universal bitwise reproducibility.

| Stage | single_reference | grouped_bounds | grouped_mean | no_reference |
|---|---:|---:|---:|---:|
| 3 | 0 | 0 | 0 | 0 |
| 4 | 0 | 0 | 0 | 0 |
| 8 | 0 | 0 | 0 | 8.88178419700125232e-16 |
| 9 | 0 | 0 | 0 | 0 |
| 10 | 5.55111512312578270e-16 | 1.66533453693773481e-16 | 4.44089209850062616e-16 | 1.66533453693773481e-16 |
| 11 | 0 | 0 | 0 | 0 |
| 12 | 0 | 6.93889390390722838e-18 | 2.77555756156289135e-17 | 0 |
| 14 | 2.22044604925031308e-16 | 1.11022302462515654e-16 | 0 | 1.11022302462515654e-16 |

### Stage-10 chromosome-specific discrepancies

| Chromosome | Genes | single_reference | grouped_bounds | grouped_mean | no_reference |
|---|---:|---:|---:|---:|---:|
| chrNeutral | 300 | 1.66533453693773481e-16 | 1.66533453693773481e-16 | 3.33066907387546962e-16 | 1.11022302462515654e-16 |
| chrGain | 128 | 5.55111512312578270e-16 | 1.11022302462515654e-16 | 4.44089209850062616e-16 | 1.66533453693773481e-16 |
| chrLoss | 17 | 0 | 0 | 0 | 0 |
| chrSolo | 1 | 0 | 0 | 0 | 0 |

The 17-gene chromosome uses truncated, renormalized weights. Its independently
recomputed outputs match exactly here. The single-gene row is unchanged
exactly between stages 9 and 10 in every profile.

## Observed gain/loss signals

Medians below are over each declared chromosome and the same three
`gain_obs` cells.

| Profile | Stage 12 gain | Stage 12 loss | Stage 14 gain | Stage 14 loss |
|---|---:|---:|---:|---:|
| single_reference | 1.1757389221856736 | -2.076558789537417 | 2.259085575933253 | 0.2370792346661384 |
| grouped_bounds | 0.8225803925172749 | -2.0774618457885596 | 1.7685664087646362 | 0.23693088114301758 |
| grouped_mean | 1.0144420213680179 | -2.077010379101224 | 2.020121437040203 | 0.23700503620379046 |
| no_reference | 0.8052054356446958 | -1.159717833753503 | 1.747394598107883 | 0.44760006975087363 |

All four profiles exhibit the expected direction on this fixture. The
predeclared single-reference sign assertions pass independently.

Stage-8 maximum absolute profile separations:

- Single reference versus grouped bounds: `0.22269053374661585`.
- Grouped bounds versus grouped mean: `0.11134526687330837`.
- Single reference versus no reference: `1.0388656285949605`.

These differences show that the exported reference modes did not collapse to
identical matrices on this fixture.

## Coverage and interpretation limits

- Stage 9 changes **zero entries in every profile**. This run validates the
  in-range path, not positive/negative saturation or equality-at-boundary
  behavior. Separate witnesses are needed before claiming that coverage.
- Observed chromosome lengths are 300, 128, 17, and 1. This artifact does
  not execute the 2-, 100-, 101-, or 102-gene edge cases, including the
  special 101-gene center overwrite. Earlier hand-derived witnesses remain
  source-derived, not newly executed upstream goldens.
- All reviewed expressions are finite. NA compaction, zero-depth-after-filter
  failure, and additional creation/filtering policies are not validated here.
- The `1e-9` threshold was a predeclared review stop condition. Observed
  discrepancies are at most `8.881784197001252e-16`; neither value is
  established as a universal tolerance for future datasets, platforms,
  prefix-sum optimizations, or a native implementation.
- No stage 15+ clustering, denoising, i3/i6, JAGS, Bayesian filtering, clinical
  interpretation, real-data benchmark, or native-platform release claim follows.
