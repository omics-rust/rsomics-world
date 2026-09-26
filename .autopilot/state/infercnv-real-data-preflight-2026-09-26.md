# inferCNV shipped-data preflight

Read-only follow-on to the accepted synthetic oracle and the native core plan.
No real-data R execution, native result, or performance measurement is claimed.

Source: infercnv 1.28.0 commit
`b421d9405c97a309b081ef86d455e976df93eae4`, archive SHA-256
`b2a1b6f8cc09dc04562513e3cd877b3c97a6efcd5ff92cb5e0763660905e3207`,
preserved in accepted oracle run `36216764707`.

## Input and provenance

Use these shipped `inst/extdata` files together:

- `oligodendroglioma_expression_downsampled.counts.matrix.gz`
- `oligodendroglioma_annotations_downsampled.txt`
- `gencode_downsampled.EXAMPLE_ONLY_DONT_REUSE.txt`

The source preflight found 10,338 genes by 184 cells with matching gene/cell
identities and six groups. Upstream `example/run.R` selects references
`Microglia/Macrophage` (19 cells) and `Oligodendrocytes (non-malignant)`
(23 cells); the four other groups contain 35, 33, 34, and 40 cells.
Coordinates span chr1–22, X, Y. `R/data.R` describes GRCh37; `example/README.txt`
explicitly forbids reusing this abridged coordinate file for unrelated data.
The matrix contains fractional values: do not describe it as integer UMI
counts. Full biological provenance is not established by these shipped files;
this is an upstream example-derived engineering fixture, not biological
validation. Do not substitute `data/infercnv_data_example.rda`, which is
documented as generated data.

## Next correctness gate

Run a separate deterministic chr1+chr19+chr21 subset with all 184 cells, then
the full shipped example. A slice of a full-run output is not a valid subset
golden because depth normalization and global centering change.

Source-only predictions, to be confirmed from real R checkpoint exports:

| Input | Stage 1 genes | Stage 2 genes |
|---|---:|---:|
| chr1+chr19+chr21 subset | 1,775 | 1,487 |
| All autosomes | 9,939 | 8,508 |

The predicted subset includes 90 filtered chr21 genes, below the smoothing
window. Full post-exclusion cell depths are predicted above 72,000.
Preparation uses `min_max_counts_per_cell=c(1,Inf)`, no random cell cap, and
`chr_exclude=c("chrX","chrY","chrM")`.

Use both reference groups, bounds mode, cutoff 1, detection minimum 3,
window 101, pyramidinal smoothing, clamp 3, one thread, no reference regrouping,
no HMM/denoise/scaling/outlier pruning/masking/end trimming, no plots/resume,
and stop at step 14. Correctness runs save the ten checkpoints and export
ordered gene/coordinate and cell/group identities with 17-digit expressions,
arguments, source/runtime identity, and hashes. Check actual dimensions and
near-threshold genes rather than promoting the preflight predictions to truth.

Compare identities exactly and report absolute/relative deltas per stage.
The accepted synthetic tolerance is not an assumed universal real-data bound;
any disagreement requires investigation before changing a criterion.

## Matched benchmark boundary after correctness

On one native machine, compare installed R `infercnv::run(stage1_object, ...,
up_to_step=14)` against release Rust `run(&PreparedCounts, ...)` on the full
example. Both timers exclude input reading and prepared-state construction;
report those costs separately. Both disable checkpoint/export/plot work.
Set R/BLAS/OpenMP threads explicitly to one. Record machine, CPU, OS, compiler,
R/package/BLAS versions, flags, input hashes and final-output checks.

Use at least seven fresh-process trials per implementation, each with an
untimed warm-up and a measured execution from unchanged prepared counts.
Record wall/CPU time per trial and distributions, plus whole-process peak RSS
and the pre-run baseline. Peak RSS includes runtime, preparation and warm-up;
do not subtract baselines and label the difference core-only peak memory.
Hash outputs outside the timed region. Full stage comparisons belong in
separate correctness runs, not a timed snapshotting observer.

The full stage-1 dense matrix has 1,828,776 values (~14.6 MB); predicted
filtered matrices have 1,565,472 values (~12.5 MB). Ten full text checkpoint
exports may occupy 0.25–0.35 GB uncompressed; measure actual sizes and keep
them on external fixtures, separate from timing artifacts. This is a bounded
real-example benchmark, not representative large-cohort performance proof.

Source pointers: `example/run.R`, `example/README.txt`, `R/data.R:16-20`,
`R/inferCNV.R:133-180,200-328,352-427`,
`R/inferCNV_ops.R:242-345,538-599,2114-2198`.
