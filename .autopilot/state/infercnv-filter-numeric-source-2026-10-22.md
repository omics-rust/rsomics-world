# Reference-filter numeric source inventory

Read-only inventory, not native filtering implementation or acceptance.
Primary official R archive/source pin is recorded in the locale failure ledger:
R4.6.1,SVN90187,40926649bytes,SHA256
4da6e61d2c0aac5f14a2e7e432cb5fcc269efe83da4293050ba7f03dff4e2cf4.
External source: rsomics-fixtures/sources/r-4.6.1-filter-2026-10-22/.
Official URL https://cran.r-project.org/src/base/R-4/R-4.6.1.tar.gz.

Main inspected actual archive members, not recollected generic formulas:

- base/R/mean.R19-41 dispatches finite real untrimmed input to the internal
  mean. main/summary.c482-520 accumulates using LDOUBLE, includes overflow
  handling and a correction pass, then rounds its result to double.
- stats/R/sd.R19-20 flattens a matrix to doubles and computes sqrt(var).
  stats/R/cor.R175-184 chooses everything when na.rm is false and calls C_cov.
- stats/src/cov.c221-239 computes a corrected mean, stores it as double;
  cov_na_1 lines303-334 uses sample denominator n-1 and LDOUBLE deviations/
  products/accumulation, rounds variance to double before R-level sqrt.
  Its n<=1 branch177-182 produces NA. Do not substitute population variance.
- include/Defn.h2567-2570 makes LDOUBLE conditional on HAVE_LONG_DOUBLE.
  Release version/source does not prove the captured executable's precision.
  A base-R R.version/.Machine probe is planned in the actual locale-contract
  evidence bundle, before packages; it is not yet observed.

The infercnv filter then performs double z-score arithmetic and per-row base
mean of absolute values before the hard-coded >=0.8 decision. All original
selected-gene indices, especially near-boundary genes3/6, remain mandatory
comparison targets. No tolerance/rounded threshold may repair a disagreement.
Constant/sample-size edge NaNs are part of observed R selection semantics,
not permission to swallow arbitrary nonfinite production errors.

This source inventory justifies investigating an exact private compatibility
reduction; it does not establish that ordinary Rust f64 or a generic stats
helper reproduces the binary. Keep product-specific R policy internal, not
a speculative public stats API. Original36-case witness and selected matrices
remain unchanged. The next coherent samples-workflow design should combine
group/view/workspace and native selection ownership, retaining separate tests
on frozen R-step14 and current Rust-step14; do not multiply public micro-crates.

## Bounded original-fixture diagnostic

Independent small p53/p64-significand models both reproduce12 tiny probe
routes, including the boundary's selected indices4,5,6. Their intermediate
variances/row means differ, so matching indices do not identify captured R
precision. Small script SHA256
098bf8e733b12d4bde3c2e44304a3a8be19e144f5f7b046a30f4a62c298ff90d;
verified result SHA256
b9a21750b08c688fbe5f740ca00a72845b34421f0c548fd24eb6cb669f1fcbf0.
Originals: external infercnv-filter-diagnostic-2026-10-22/
binary-extended-fraction-v1/.

The follow-up p53-only streaming model compares all52 original routes:
20 active filters and32 source-guard bypasses. Outlier and selected-index
lists match52/52, including order and NULL versus empty; no mismatch or
diagnostic error. It authenticates original ZIP SHA256
2481188b6f3d058d154bf16df9297a9eca261c6df84280e7d16569c92da28bd8,
witness26333ff388b0916d79736c7b3370b9f907fb943ca918a92ef5529a4d8bdfd304,
R archive above and210 actual ZIP member hashes. One route at a time, guards
20000genes/200cells, packed references peak2858688bytes; largest actual
matrix8508x184. Full independent rerun is byte-identical. Root inspected the
model/comparison path and separately checked all52 result flags and hashes.

External infercnv-filter-diagnostic-2026-10-22/all52-p53-stream-v1/:
script SHA256952e0ee11f846c1cd37c7aac1d2f70f4f4b00515933d2a2fe22eeed2100a1374;
result SHA256b2d89259b9ca7270cbffcaa13260b07eda1908c9931b755bde4a6bbd9e355ad4.
No epsilon/clamp/adapted threshold or numeric-result cache. This supports a
candidate reduction order only: no Rust filtering implementation, captured
precision identification, arbitrary-input equivalence, native acceptance or
performance claim follows. Actual R .Machine capture remains the separate
live reference-oracle gate; it cannot retroactively prove historical flags.

Fresh independent numerical-contract review finds no bounded model/source
error. It recommends the smallest private source-ordered f64 candidate with
fixture-scoped admission, not a precision framework. Both diagnostics omit
real_mean's overflow fallback; covariance's mean is interchangeable only on
the checked finite-sum domain. Implement those branches or explicitly define,
test and document a narrower non-public candidate domain. Do not claim all
finite inputs are compatible merely because52 routes pass. Additional broader
evidence would record actual hexadecimal mean/variance/SD/row scores and
selections for adjacent-representable boundary perturbations under pinned R
runtimes; host long double alone is not a portable contract. Design and plan:
docs/plans/2026-10-22-infercnv-native-filter-{design,plan}.md. Fresh builder
check approved Task1, incorporating precise candidate-domain/native-selected
tree/full-state binding clarifications. The reference-only native oracle
remains independent.
