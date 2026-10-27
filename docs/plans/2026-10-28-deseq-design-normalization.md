# DESeq general designs and supplied normalization

## Scope and source contract

Extend the real private fitted workflow, not its public crate boundary. Keep
the frozen schema1/green1 checkpoint and its source evidence unchanged. The
new product-owned schema records formula versus explicit-matrix provenance,
typed factor levels/reference, numeric covariates, model columns and supplied
normalization. No new foundation API or product dependency is warranted.

Authority is the already retained DESeq2 1.52.0 source: AllClasses.R:192-290,
core.R:378-396,1359-1405,2645-2716, methods.R:1-28,264-282,344-406,
fitNbinomGLMs.R:38-65 and results.R:551-607,1122-1130. R model-matrix
term/coding semantics are also described by the official
[model.matrix](https://stat.ethz.ch/R-manual/R-patched/library/stats/html/model.matrix.html)
and [terms object](https://stat.ethz.ch/R-manual/R-patched/library/stats/html/terms.object.html)
contracts. Do not copy GPL implementation bodies into Rust or infer matrix
semantics solely from old rsomics additive-categorical code.

## Product interface

`run` accepts exactly one of the existing `--design MATRIX` and a new
`--formula FORMULA`. Formula columns are explicitly declared with repeated
`--numeric NAME` and `--factor NAME=LEVEL1,LEVEL2,...`; the first factor level
is the reference. Metadata values are never guessed to be numeric or factors.
Support additive numeric/categorical terms, `:` and `*` interactions,
parentheses, `~1`, and intercept removal by0/-1. Preserve source term ordering,
dummy versus treatment coding and coefficient identity. Reject unsupported
R expressions rather than treating them as ordinary variable names. Explicit
matrices remain the route for designs beyond the declared formula grammar.

Add optional `--size-factors TSV` (sample/factor rows) and
`--normalization-factors TSV` (gene by sample positive numeric matrix).
Join both by identity. Supplied values are not recentered. If both exist,
gene/sample normalization factors take priority for normalized counts and
every fit offset; retain the supplied SF separately for provenance and the
later local-VST scale contract. With neither, estimate existing ratio SF once.
Store typed normalization state and input digests, not unrelated vectors that
allow one stage to silently use another normalization policy.

`--no-replace-outliers` is an implemented opt-out equivalent to upstream
minReplicatesForReplace=Inf. Default replacement remains a separate incomplete
private path until its actual refit lifecycle is implemented. Record options
in fitted state. Formula provenance must survive reload because the single
two-level-factor Cook low-count exemption does not apply to an equivalent
explicit matrix. Preserve original-count-based Cook/result semantics.

Keep complete named contrasts, common/help CLI/error/atomic-output contracts,
one Rayon pool, ordered deterministic reductions and fit-reusing results/VST.
Read and validate normalization at the boundary; avoid repeated invariant
checks inside per-gene hot loops. Keep comments rare and explanatory only.

## Integrated evidence, fixed before candidate comparison

Use one source driver and a frozen deterministic18-sample count/metadata set:
three-level group, two-level batch and three numeric x values per cell. Whole
workflows cover `~batch*group + group*x`, supplied SF, NF-only, differing SF+NF,
no-intercept `~0+group+batch+x`, and intercept-only `~1`. Include a matched
explicit matrix. Record actual R formula/contrasts settings, all named model
columns, count/sample IDs, normalization, complete fit diagnostics, forward
and mixed contrasts, reload, normalized counts and fitted VST.

The seven general case IDs are interaction-ratio-formula,
interaction-ratio-matrix, interaction-sf-formula, interaction-nf-formula,
interaction-sf-nf-formula, nointercept-sf-formula and intercept-sf-formula.
Use1,200 generated NB genes, group levelsA/B/C, batchb0/b1 and three x values
(-.75,.125,1) per group/batch cell. Record generator and RNG state and seal
all inputs/case inventory before the first source fit. Eighteen is the sample
count, not a request for eighteen redundant general-design fits.

Retain the existing unfiltered pasilla14599x7 regression unchanged. General
case matrices/identities and missing/decision states compare exactly; existing
fixed numerical bounds apply to finite outputs. Do not widen them after a
mismatch. The independent source driver must use actual public calls, not a
translation of candidate logic. Preserve failed source/native runs.

The same driver also records a14-sample7+7 and10-sample7+3 Cook family for the
next coherent refit implementation: default versus disabled replacement,
formula versus matrix, eligible/ineligible outliers and replacement-to-zero.
An asserted branch that does not occur is uncovered, not an accepted witness.
Do not infer positive replacement acceptance from ordinary pasilla or general
design cases. Source-only branch qualification may precede implementation;
it must not be tuned against native answers.

Use whole-workflow comparisons at1/4 workers, one fresh integrated review and
four-native acceptance at the next stable checkpoint. Do not create exhaustive
helper RED/review loops. Nonparametric trends, low-df prior simulation and
formal whole-path performance remain later release work. Keep publish=false.

All source/scratch/build/data paths remain on the authorized external disks;
recheck storage/APFS/resolved Cargo paths before every local build/test or
download. Hosted source runs use runner-temp, the existing exact R4.6.1/
Bioc3.23/DESeq1.52 source pin and retained provenance. No local R installation.
