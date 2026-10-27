# Native DESeq2 fitted workflow

## Authority and scope

Implement the first complete workflow in docs/10-products/bulk-expression.md,
using accepted actual-source run40561801963, DESeq2 1.52.0 at
16aeab6d6158bd8cbb8e98764c2f33399b1a3fd3. The target is one product-owned
dataset and reusable fitted analysis, not another collection of numerical
micro-crates. The first real checkpoint is the unchanged14599x7 pasilla input,
design~type+condition, named forward/reverse contrasts, normalized counts and
design-aware fit-reusing VST. Passing that checkpoint is not the full release.

The candidate stays private and publish=false. Work in
`/Volumes/KIOXIA/Documents/omics-rust/rsomics-deseq/`; use external Cargo home,
target and scratch, and the required fresh storage guard before execution.
No new public foundation API is planned. Use current rsomics-common and
rsomics-help; use existing policy-free statistics where their contracts fit.
Team-owned numerical seeds may be refactored. Upstream released source is a
behavior reference with attribution, not permission to copy LGPL source into
an incorrectly licensed product.

## Dataset and design contract

Read integer nonnegative counts with unique feature/sample identifiers and
join sample metadata by identity. Preserve input feature and count-column
order. Categorical levels/reference and numeric covariates are explicit;
never infer a numeric column to be a categorical label. Keep named model
columns, factor coding and the exact finite full-rank model matrix in state.
An explicitly supplied named model matrix supports arbitrary full-rank
designs without relying on a partial R-formula interpreter. Formula support
must cover declared numeric/categorical terms, interactions and intercept
control before first publication. Reject ambiguous or unsupported syntax.
Require residual degrees of freedom for dispersion estimation.

Named contrasts must cover the intended coefficients explicitly. Retain full
coefficient covariance or sufficient weights/state to reconstruct it; storing
only per-coefficient variance cannot support a general contrast. Results and
transforms must load the existing fit, not refit it under a different design.

## Numerical migration

Port the median-of-ratios loop once. Separate normalized means/variance,
all-zero status, initial/gene/trend/raw-MAP/final dispersion, outlier and
convergence diagnostics. The real source witness includes active grid fallback
and outlier paths, so these cannot be deferred as theoretical edge cases.

Use the general-design legacy kernels only as seeds. Correct the natural-log
ridge to1e-6/log(2)^2, preserve the released augmented weighted-QR fitting
contract, actual convergence/large-coefficient behavior, raw versus floored
means, full covariance and leverage. A failed solve is an error or a declared
upstream optimizer fallback, never a zero coefficient/identity covariance or
silently successful truncated iteration. Keep the1.52 optimizer semantics
distinct from later devel changes.

Cox-Reid gene estimation, parametric trend and MAP shrinkage feed one typed
fit. Preserve source grid/convergence/outlier decisions. Approximate legacy
local smoothing and independent-filter LOWESS are not accepted substitutes
for the released behavior. Keep Cook's diagnostics/replacement and independent
filter state rather than only the final result TSV.

Parallelize independent per-gene work in one product-owned Rayon pool. Keep
gene ordering and global reductions deterministic across worker counts. Do
not add a separate public numerical execution layer for one product.

## Integrated CLI and durable state

Implement run, results and one explicit transform through rsomics-help.
Transactional versioned bundles retain the dataset/model identities,
normalization, dispersion and complete Wald fit, options, upstream version,
diagnostics and file digests. Save/load round trips must derive forward and
reverse results, normalized counts and VST without numerical fitting again.
Reject a failed or incomplete fit instead of advertising placeholder output.

## Acceptance and sequence

1. Build the private input/design/fitted-state model and selected numerical
   kernels, then run the actual complete pasilla workflow. Compare all rows,
   NA states, coefficients, fitted means, dispersion decisions, result/filter
   decisions and transformed values with the retained source tables. Keep one
   integrated comparison rather than new oracle campaigns per helper.
2. Complete the dossier's remaining first-release semantics through a small
   set of integrated cases: general numeric/interaction designs and contrasts,
   positive Cook's masking/replacement, supplied normalization and nonparametric
   trend/failure behavior. Do not silently narrow the release promise to the
   easy pasilla design or tune fixtures after seeing a mismatch.
3. After the complete workflow is correct, measure a representative full run
   against the exact upstream on one host, including startup/outputs,1/4-worker
   timing distributions, process memory and I/O. Require a strict relevant
   advantage. Perform one fresh whole-workflow/public-contract review, exact
   four-native CI, and only then consider public repository/publication.

The source witness anchors12359 nonzero converged genes,2240 all-zero rows,
filter threshold3.897806930592444,5800 padj NA and1330 discoveries at alpha0.1.
These are result/decision anchors, not a license to ignore continuous-value
differences or to infer broader biological validity. Before any native fit is
run, fix the first integrated comparison to exact identities/order, missing
states, outlier/filter decisions and discoveries, plus
abs(native-source)<=absolute+relative*abs(source) for finite values:

| Value class | Absolute | Relative |
| --- | --- | --- |
| size factors, normalized counts | 1e-10 | 1e-10 |
| dispersions | 1e-6 | 1e-4 |
| coefficients, SE, Wald statistic | 1e-4 | 1e-5 |
| p values and adjusted p values | 1e-7 | 1e-4 |
| fitted means | 1e-6 | 1e-5 |
| leverage and Cook's diagnostics | 1e-6 | 1e-4 |
| VST values | 1e-5 | 1e-6 |

Iteration counts remain diagnostic, not required to be identical when both
fits converge and every declared output/decision passes. Retain failures;
these bounds are not widened in response to a candidate mismatch.
