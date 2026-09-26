# Shipped input parser bridge: observed R 4.6.1 conversion

Oracle run `36220439929`, exact world head
`68dcf7c4525a4e87581743f96e0e09dfb6d453d3`, completed the synthetic regression
and both real R cases through stage 14, then failed the independent checker's
original-to-canonical exact floating-point comparison. This failed run is
preserved, not relabeled successful or accepted as a complete oracle.

External evidence: `evidence/infercnv-shipped-2026-09-26/run-36220439929/`.
The controller verified terminal identity and failed step, all 390 ZIP entries
and extracted bytes, CRCs, and artifact digest against GitHub's API.

| Evidence | SHA-256 |
|---|---|
| Original artifact ZIP | `c16e72cb8a8d1595cb55d151a7fcd57958d931fd905c3636864328bda550fd49` |
| Full logs ZIP | `66e99d78b96713be794a9080db29cff5e797e1859c882b76a4161d0f50bd8a8c` |
| Canonical full TSV | `539ea675832047cc77dc550c648dd421f2f5370aa2e7b52e0907b7dc690237c5` |
| Canonical subset TSV | `c7da7dd19ef7cb7fce5ddf41d7624db58a482bc7bce5e011f402c8b90e0e2ca8` |

## Independent entrywise diagnosis

The controller and independent reviewer `/root/infercnv_oracle_review` each
compared all 1,902,192 original/full-canonical values. Exactly five differ.
Each is the original decimal literal `0.076439`, in cell `MGH53_P12_D12`,
at genes CCNF, SPATA18, SRSF8, ZNF273 and ZZZ3.

| Representation | Value |
|---|---|
| Python's original-literal conversion | `0x1.391819d2391d5p-4` |
| Observed R numeric value | `0x1.391819d2391d6p-4` |
| R 17-digit canonical text | `0.076439000000000007` |
| Difference | +1 adjacent representable value; `1.3877787807814457e-17` |

On `4090`, R 4.6.1 independently reproduced the upper value using
`scan(text="0.076439", quiet=TRUE)`, `as.numeric("0.076439")`, and
`read.table(text="0.076439")[[1L]]`. Reading the canonical 17-digit text
returned the same upper value. The tiny diagnostic used `TMPDIR=/dev/shm`;
no package installation, input-file creation or local R execution occurred.

Canonical subset values exactly equal canonical full by identity. Each case's
stage-1 export equals its R-canonical input exactly; retained stage-2 values
equal stage 1 exactly. Actual stage-1/2 ordered identities and dimensions
match the independent preflight. This is an input-parser bridge, not a native
algorithm discrepancy. No downstream numerical tolerance was changed.

## Corrected bounded contract

Ruling: define the canonical numeric input as the pinned R runtime's
interpretation of the pinned original bytes, exported at 17 digits — that is
what the existing generator actually produces — the contract is specific to
these inputs/runtime, not a portable claim about every R parser.

Pin both reviewed canonical hashes in trusted checker configuration, outside
generated JSON. At the original-to-canonical boundary only, require exact
identities/order/shape and finite nonnegative values, and audit every numeric
difference as equality or representable-value adjacency. Do not use a broad
absolute/relative tolerance. Hash changes require a renewed input review;
an adjacency check alone must not authorize arbitrary one-ULP mutations.

After this boundary, preserve exact full-canonical-to-subset,
case-canonical-to-stage-1, and stage-1-to-retained-stage-2 comparisons. Native
stage tolerances and the existing independent arithmetic checks stay unchanged.
Do not implement an unneeded R decimal parser or gene/value-specific exceptions.

The code fix, regression tests, independent review and successful fresh remote
oracle run are still required. R 4.6.1 parser source was not inspected; this
audit does not claim universally correctly rounded parsing, a universal
one-ULP bound, or identical behavior on future runtimes/platforms.
