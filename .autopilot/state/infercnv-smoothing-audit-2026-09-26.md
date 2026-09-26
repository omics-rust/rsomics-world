# Pinned inferCNV pyramidinal smoothing: read-only source audit

Scope: `bioconductor-source/infercnv` commit
`b421d9405c97a309b081ef86d455e976df93eae4`, selected
`smooth_method="pyramidinal"`, `window_length=101`. This is source-derived
behavior and an optimization hypothesis, not an accepted oracle run, native
implementation, measured speedup, or public API decision.

## Exact stage-10 behavior

[`smooth_by_chromosome`, lines 2406–2434](https://github.com/bioconductor-source/infercnv/blob/b421d9405c97a309b081ef86d455e976df93eae4/R/inferCNV_ops.R#L2406-L2434)
selects current gene-order rows per chromosome, smooths each cell separately,
and writes each chromosome back to its own indices. It skips a chromosome
with exactly one retained gene, leaving that value unchanged. There is no
cross-chromosome window. `smooth_ends` is accepted and recursively passed to
an hspike, but does not control the call to `.smooth_window` in this function.
[`apply(data, 2, .smooth_helper)` and name restoration, lines 2440–2465](https://github.com/bioconductor-source/infercnv/blob/b421d9405c97a309b081ef86d455e976df93eae4/R/inferCNV_ops.R#L2440-L2465)
make the smoothing per column; there is no pooling across cells.

For a 101-gene window, let `h=50` and local chromosome index `i` be 1-based.
The full triangular weights are `1,2,...,50,51,50,...,2,1`, with denominator
`2601`. For an interior index `51 <= i <= n-50`, the source's
[`stats::filter(..., sides=2)` call, lines 2640–2660](https://github.com/bioconductor-source/infercnv/blob/b421d9405c97a309b081ef86d455e976df93eae4/R/inferCNV_ops.R#L2640-L2660)
uses these weights divided by 2601. R's documented default is centered,
non-circular convolution; incomplete external windows become NA. The center
helper copies only non-NA filtered results back into the original vector, so
the endpoint loop replaces retained raw endpoint values, not NA inputs.
[R's filter documentation](https://stat.ethz.ch/R-manual/R-devel/library/stats/html/filter.html)
confirms this alignment and boundary behavior.

For each endpoint or any chromosome shorter than the window, the source
truncates the same triangular kernel at the chromosome boundary and divides
by the sum of the retained weights. In 1-based notation its intended finite
value is

`y_i = sum_{j=max(1,i-50)}^{min(n,i+50)} (51-|i-j|) x_j / sum_{j=max(1,i-50)}^{min(n,i+50)} (51-|i-j|)`.

This follows from the explicit `d_left`, `d_right`, triangular missing-weight
denominator, and left/right weighted chunks in
[lines 2483–2532](https://github.com/bioconductor-source/infercnv/blob/b421d9405c97a309b081ef86d455e976df93eae4/R/inferCNV_ops.R#L2483-L2532).
The actual operation is index-sensitive: for `n>101`, exactly the first and
last 50 indices are overwritten by endpoint sums; for `n=101`, the loop runs
51 iterations and overwrites the center too; for `2<=n<101`,
`ceiling(n/2)` left/right iterations cover all genes; for `n=1`, the outer
function performs no smoothing. Odd short chromosomes calculate the midpoint
from both sides in the same loop; the second assignment wins. The source
removes NA positions per cell before calculating local indices, then restores
NA slots, so its effective neighborhood for missing data is a compacted
per-cell sequence, not original genomic-distance indexing. The current
oracle fixture requires finite values; NA compatibility is not thereby proved.

## Performance and numeric risk

The center is direct convolution, with one per-output coefficient loop in
[R 4.6.0 `stats` `filter.c`, lines 35–66](https://svn.r-project.org/R/tags/R-4-6-0/src/library/stats/src/filter.c).
For `n` genes, `c` cells, and window `w`, that suggests `O(c*n*w)` arithmetic
for long chromosomes, plus endpoint work bounded by `O(c*w^2)`; for short
chromosomes the endpoint path can be `O(c*n^2)`. With fixed `w=101`, both
long-path formulas are linear in `n` but differ materially in constant work.
An algebraically equivalent per-cell method could keep prefix sums of `x_j`
and `j*x_j` (or a rolling triangular sum) and evaluate truncated weighted
numerators/denominators in `O(c*n)` time. This is an opportunity to test, not
a measured win; memory traffic, indexing, allocation, and the existing C loop
could dominate on realistic matrices.

Bitwise identity is not expected from a prefix/rolling formulation: the
upstream center multiplies by pre-divided coefficients and sums in C's
coefficient order, whereas endpoints sum weighted products before dividing;
an optimized path changes operation order and may involve cancellation on
centered positive/negative values. The `n=101` center overwrite and odd-short
midpoint assignment are especially easy to approximate incorrectly. A future
comparison should use actual stage-9 and stage-10 oracle rows (including
`n=1,2,17,100,101,102` and long chromosomes), establish explicit absolute
and relative tolerances from observed double-precision deltas, and examine
whether stage-11 centering, stage-12 reference subtraction or stage-14 inverse
log transform amplifies differences. No numeric tolerance or performance
claim is accepted before that oracle evidence is inspected.

An independent read-only review confirmed the source indices, the two-gene
witness, and the bounded optimization hypothesis. No numerical run or speedup
was inferred from that review.
