# Private native Euclidean distances for inferCNV samples

Decision update, 2026-10-22 20:08 CST: the user chose explicit failure for the pooled `all_observations` reference-name collision, recorded as a known upstream incompatibility. Historical pending-policy wording below describes the design-time boundary; group implementation will enforce this choice. Original distance witnesses remain unchanged.

## Intent and bounds

After the accepted supplied-distance Ward kernel, compute the actual pair
distances from the original R-selected matrices. This is the second component
of samples clustering, not another public crate, product API or CLI. Keep it
inside rsomics-sc/src/cnv/samples; a later workflow will call it and Ward.
No new Rust dependency or default library export is needed. Direct private
integration tests compile the real source files, as for the accepted kernel.

The agent chooses this implementation sequence under the user's unattended
authorization. Acceptance requires all152 captured tree vectors/107331
distances, not only the747 small-case Python diagnostic. Positive-filter
cases consume their actual already-filtered R matrices; this does not
implement or validate native filtering. End-to-end R-step14/current-Rust-step14
trees, small-group bypass, group membership and performance remain later
gates. No epsilon, distance tolerance or omitted hard case is introduced.

The observed pooled-reference-name collision is retained in oracle metadata,
but no product-interface policy is chosen here. A nonblocking question asks
the user to choose explicit failure versus reproducing that upstream anomaly.
Distance tests consume the recorded actual input vectors, not declared group
indices or inferred roles. This mathematical gate does not decide collision
policy or endorse the upstream anomaly.

## Evidence and arithmetic

Use the accepted original36707344893 artifact11092988090,1572353133bytes,
SHA2562481188b6f3d058d154bf16df9297a9eca261c6df84280e7d16569c92da28bd8.
Its samples witness hash is
26333ff388b0916d79736c7b3370b9f907fb943ca918a92ef5529a4d8bdfd304;
the accepted tree fixture hash remains
33617f00dc77608c3dab25623cf2143f4fa49ae4aa073940d0df5fff546b7236.
Retain every original tree ID as case/route/group-index and all original
numeric strings. Do not merge identically named groups or deduplicate routes.

Primary source packages are external under
`/Volumes/Zane's HDD/rsomics-fixtures/sources/infercnv-ward-2026-10-22/`:

- parallelDist0.2.7 commit10db7bebaedcaf345731a3c7ad383ca5f68fe087;
  src/DistanceDist.h:111 computes the Euclidean norm of a difference.
- RcppArmadillo15.6.0-1 commit53d3f54dccfd1ef706adb999349637191da77bd7,
  Armadillo15.6.0. Archive1027536bytes, SHA
  a4afb3031e8541240446a612482e160f4b403cc43bcf90766f4d97f1e4413a36.
  inst/include/armadillo_bits/operator_minus.hpp:148-166, fn_elem.hpp:588-599,
  Proxy.hpp:155-180,343-420, eGlue_bones.hpp/eOp_bones.hpp and
  op_accu_meat.hpp:26-69,230-252 establish the expression path.

For the non-fast-math expression reduction, each double difference is
squared by multiplication; even and odd feature positions accumulate into
separate double sums, an odd final feature belongs to the even sum, then the
two sums are added and square-rooted. Do not use a single sequential sum,
long-double/compensated/hypot reduction, FMA or algebraic reassociation.
Zero features yield positive-zero distances for every pair, not an error.

This is an independently implemented mathematical reduction; no parallelDist,
infercnv or RcppArmadillo source is copied into the product. Original Posit
installation logs identify binary packages, not their build-time header or
compiler flags. The source model and747 exact small-case diagnostic values
therefore justify a candidate, not universal binary compatibility. The
actual107331 provided distances are the admission comparison. If any bits
disagree, retain the failure and diagnose; do not choose a tolerance afterward.

## Product-local interfaces

Factor existing pair-count/index mechanics into
src/cnv/samples/condensed.rs, used by both ward.rs and distance.rs:

```rust
pub(crate) fn pair_count(leaves: usize) -> rsomics_common::Result<usize>;
pub(crate) fn condensed_index(leaves: usize, a: usize, b: usize) -> usize;
```

Preserve pair_count's exact2..=i32::MAX contract and checked multiplication.
Indexing is upper-row order, equivalent to R's symmetric lower-column order;
callers enforce distinct in-range indices. Move mechanics, not policy or a
public foundation. Ward's signature, arithmetic, requested workspace and all
152-tree behavior remain unchanged. Both private integration targets declare
the sibling condensed module so the actual product source is compiled.

In src/cnv/samples/distance.rs:

```rust
pub(crate) struct CellProfiles<'a> { /* private validated borrowed columns */ }

impl<'a> CellProfiles<'a> {
    pub(crate) fn new(values: &'a [f64], features: usize, cells: usize)
        -> rsomics_common::Result<Self>;
    pub(crate) fn distances(&self, indices: &[usize], max_numeric_bytes: usize)
        -> rsomics_common::Result<Vec<f64>>;
}
```

new requires cells>0, exact checked features*cells column-major shape and
finite values, including all columns; features=0 with an empty value slice is
valid. It borrows the caller's immutable values without copying or allocating.
distances requires2..=i32::MAX distinct in-range cell indices, retaining their
order. It returns exactly n*(n-1)/2 values in condensed order, using the stated
two-lane reduction and no temporary term vector. Reject nonfinite arithmetic;
allow ordinary binary64 square underflow to zero. There is no silent clamp or
replacement of a computed distance.

max_numeric_bytes bounds requested initialized **output** double elements,
exactly pair_count(n)*size_of::<f64>(), not caller input, allocator overhead,
headers or process memory. Check shape/budget/indices before fallible exact
reservation. Validate duplicate indices without additional numeric allocation;
O(n^2) index validation is bounded by the output's own pair count. Validate a
profile once, not its entire matrix separately for every group. Source comments
remain rare stable explanations, not phases/history/narration.

## Authenticated external test inputs

Extend the existing deterministic fixture extractor rather than another
artifact downloader or a Rust JSON dependency. Produce a separate compact
profiles.tsv index from the pinned witness. Each152 tree row contains its
literal tree ID, actual relative filtered-expression path, selected gene
count, global cell count, and **actual input_indices** converted from R's
one-based indices to zero-based integers. Determine global cell count from
the route's hash-checked state/cells TSV, not the maximum group index.
Hash-check the source matrix as a stream; read only its header for identity
and shape binding during extraction, not a giant in-memory expression table.
Validate expected counts and paths before exclusively creating the output.

Index columns are exactly:
`tree\tmatrix\tfeatures\tcells\tindices` with tab-separated rows; indices are
comma-separated zero-based integers. Sort identically to trees.tsv. Include
all152 rows, even identical payloads. Metadata-only changes cannot repair a
bad matrix or invent an expected vector. Keep generated metadata small and
checked in beside trees.tsv, with original/generated hashes and conventions
in its README. Original matrices remain external.

Default cnv_samples_distance tests use small hand-verifiable profiles and
source-derived two-lane counterexamples, invalid dimensions/nonfinite data,
ordered/permuted indices, duplicate/out-of-range rejection, even/odd/zero
features, immutable input, square/accumulation overflow, underflow and exact
output-budget equality/minus-one. The existing Ward target remains unchanged
apart from importing the factored shared mechanics, retaining every old test.

Add external-infercnv-samples-oracle as a test-only feature depending on
external-infercnv-oracle, and cnv_external_samples_oracle with that exact
required feature. It must fail loudly if RSOMICS_INFER_CNV_SAMPLES_ROOT or any
index/matrix is absent. Read profiles.tsv and unchanged trees.tsv, cache at
most one current route's strict column-major filtered matrix, construct one
validated CellProfiles per route, and compare **all107331 distance bits**
exactly across all152 identities. Also pass each computed vector to the real
Ward kernel and retain the existing exact merge/order/height assertions.
No missing environment/data skip, adaptive case count, tolerance or fixture
reconstruction is allowed. The test source and metadata are frozen; large
expression matrices are never injected into the source snapshot.

## Workflow and preservation

Use the existing sc-cnv-core-candidate.yml source/ZIP verification unchanged.
Add only selector infercnv-samples-clustering-witness-2026-09-30 alongside
none and infercnv-shipped-2026-09-26; reject unknown/path-like selectors before
network access. The new accepted receipt still has bundle_path=shipped-bundle,
so the existing verify_infercnv_oracle_artifact.py already authenticates full
ZIP/API/CRC/safe sibling extraction. Do not weaken its schema or guard.

Keep existing synthetic/shipped validators and
RSOMICS_INFER_CNV_ORACLE_ROOT=$verified/shipped-bundle. Only the new selector
also runs the existing complete samples witness validator on bundle,
shipped-bundle and samples-clustering, then exports
RSOMICS_INFER_CNV_SAMPLES_ROOT=$verified/samples-clustering. Old selector
behavior remains unchanged. Debug/release/Clippy select matching features.
Preserve old missing-root checks; new selector independently proves that
missing samples root fails the actual external test with the explicit variable
message and `test result: FAILED`, not a compiler failure. Four-native matrix,
Linux-only RED, source postchecks and upload step names remain unchanged.

Do not upload four duplicate1.57GB original archives or large extracted
matrices. Preserve acquisition APIs/verification logs, source/test/index,
matrix/input hash bindings and actual conformance logs/results in the native
artifacts. Source immutable snapshots retain original receipt pinning. All
local scratch/downloads/evidence stay external; no local Cargo/R/install
while the Mac boot gate exceeds80%. Rust1.91 and Cargo.lock remain unchanged.

First RED captures the absent new distance/shared mechanics modules, with
actual source/metadata gates succeeding. GREEN runs the existing old tests,
new portable distance tests and actual external tests in debug/release on
all four native classes, plus Linux formatting/strict Clippy. Original API/
artifact/log/source validation and fresh independent implementation review
precede acceptance. Commit directly to owning main; no public repo, crate,
release, deletion or automation. Continue workflow integration afterward;
this gate alone does not establish samples, full inferCNV or performance.
