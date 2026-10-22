# Private inferCNV samples filter

## Decision

Implement the next product-local candidate, not a general statistics API:
derive the temporary clustering gene selection from the original unfiltered
profiles and ordered reference-cell indices. Reuse the accepted distance and
Ward kernels afterward. No new public crate, dependency, CLI or release.

The source-ordered ordinary-f64 model matches52/52 original route selections,
but only20 routes exercise filtering. The small p53/p64 models agree on
selection despite differing intermediate values. This supports a candidate,
not arbitrary-input or cross-runtime R precision equivalence. Source pins,
diagnostics and fresh independent review are recorded in
`.autopilot/state/infercnv-filter-numeric-source-2026-10-22.md`.

## Source contract

Pinned infercnv b421d9405c97a309b081ef86d455e976df93eae4,
R/inferCNV_tumor_subclusters.R45-71,118-123: the guard is positive
z_score_filter plus at least one reference group. Once active, the decision
threshold is the literal0.8, not the supplied positive argument's magnitude.
Use all references in their actual ordered maps, flattening column-major.
Compute global reference mean and sample SD, then per-gene mean absolute
reference z-score. Exclude which(score >=0.8). R's negative integer(0)
subsetting selects zero genes when no outlier qualifies. Inactive guards
retain all genes and have NULL outliers. Constant-reference NaN row means
compare false; do not convert this to an arbitrary nonfinite-input allowance.
Selection is a temporary view; original state, values and maps stay unchanged.

R4.6.1 mean uses corrected accumulation and has a separate overflow fallback;
sample variance has its own corrected mean, n-1 denominator and final
double-rounding before sqrt. Follow actual source, not one interchangeable
mean helper for every branch. The first private f64 candidate must explicitly
reject arithmetic outside its checked finite-sum/finite-variance domain,
except deliberate constant/sample-size semantics established by evidence.
For this candidate, active reference element count <=1 is explicitly rejected;
that branch is absent from the accepted witness. Zero SD is allowed only for
truly constant references, not nonconstant variance underflow. Reject other
nonfinite accumulation, correction, variance or z-score arithmetic. Test and
document those rejections. This is a narrower candidate contract,
not acceptance of all finite R inputs. It cannot ship as a general replacement
until that domain and any demonstrated precision differences are resolved.
No epsilon, clamp, adaptive threshold, omitted boundary or fixture-specific
correction is permitted.

## Architecture

Keep filtering in src/cnv/samples/filter.rs, beside distance.rs and ward.rs.
Reuse the validated borrowed CellProfiles rather than repeat its dimension
and finite-value checks. Add read-only feature_count/cell_count accessors and
make its unchanged column accessor crate-private; distance arithmetic stays
unchanged. The private contract is select_genes(&CellProfiles, &[usize], f64,
usize) -> Result<GeneSelection>, with outliers: Option<Vec<usize>> and
selected: Vec<usize>, both zero-based and ordered. Inactive NULL differs from
active empty outliers. Validate reference range/distinctness even on bypass,
and require a finite filter argument. Zero selected features is valid.

Use fallible reservations and checked requested-byte arithmetic for owned
numeric/index buffers. The budget describes this component's additional
requested buffers, not caller-owned profiles, process RSS or allocator
capacity. Duplicate validation uses a fallible R-index copy and allocation-free
sort_unstable without
changing original reference order, then explicitly dropped before output
buffers. Reserve G indices for selected and, only when active, G for outliers,
even if logical lengths are empty. Required requested-byte peak is
max(R*sizeof(usize), (active ?2:1)*G*sizeof(usize)); private
required_workspace(features, reference_count, active) returns Result<usize>
for the concrete future group owner. Reductions need only scalars
and repeatable iterators, not copied reference values. The future group owner
must aggregate live selection, filtered
profile, distance, Ward and retained-output allocations; passing the same
full budget separately is not an aggregate guarantee. No speculative public
constructor, clustering enum or shared stats foundation is introduced.

Retain current private/test-only admission until a complete samples owner
can consume this result. Do not expose an unfinished product operation.
Reference-only grouped/pooled shape acquisition remains separate and does
not block this filter's original52-route evidence. The user-selected pooled
reference-name collision rejection belongs to the group owner, not filtering.

## Evidence contract

Extend the existing hash-pinned fixture extractor, preserving trees.tsv and
profiles.tsv byte-for-byte. A new deterministic route TSV records all52
route identities, all four original expression/genes/cells/maps paths,
filtered-matrix path,
dimensions, ordered reference indices, filter argument, exact outlier NULL
versus empty distinction and selected indices. Indices are explicitly
zero-based; empty lists and NULL use distinct representations. No large
matrix is copied into Git and no native statistics are calculated by the
extractor to create expectations.

Probe input is explicit pre_call_state; workflow full_run input is explicit
stage14_state. Workflow isolated state is usable only with its unchanged
exporter contract and equality to the explicit stage14 file hashes. Stream
authenticate every consumed original file against the immutable witness.
Bind reference maps, cell identities/order, source gene identities and actual
filtered rows, including zero-row matrices. Reject malformed indices, wrong
selected row/value/order, source drift and incomplete route coverage.

Native admission requires all52 exact outlier/selected lists, unchanged
selected values/order against R's filtered matrices and all152 exact
computed-vector trees from original matrix to native selection to native
distance to Ward; the old R-filtered input path alone is insufficient.
Frozen R-step14 evidence is separate from future
current-Rust-step14 integration; the accepted preprocessing tolerance is not
a tree or selection tolerance. All four native classes run debug/release.
Capture a genuine missing-implementation RED before implementing the kernel.
Preserve original artifacts, test bytes, hashes and failed launches.

## Exclusions and next owner

This component does not accept full step15, group/map orchestration,
reference-only behavior, current-Rust-step14-to-tree agreement, performance,
arbitrary-input R parity, clinical validity or publication. After its bounded
gate, continue directly with the coherent samples group owner using the
actual reference-only oracle and the existing memory/map contract inventory.
There are51 ordinary original group routes/158 groups/150 trees; the remaining
collision route/two upstream trees must instead yield the user's explicit
error, not silently disappear from coverage.
