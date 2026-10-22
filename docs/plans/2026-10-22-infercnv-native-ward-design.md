# Product-local inferCNV Ward.D2 kernel

## Intent and scope

The next inferCNV workflow gate is samples clustering, not another public
linkage crate. Its first independently testable component is the tree kernel:
given the exact Euclidean distance vector captured by the accepted installed
package, reproduce its Ward.D2 merges, heights and leaf order.

This is an agent-selected implementation sequence under the user's existing
unattended authorization, not a new user-approved public interface. No default
product API, CLI, package, dependency or foundation is added. The kernel lives
under rsomics-sc/src/cnv/samples/ and is initially compiled directly by a
private integration test. A later samples module will call the same file.
No unused public API or permissive dead-code lint is needed for this gate.

Acceptance covers all152 real trees in the existing36-case/52-route witness,
including positive-filter cases **only after their actual distances have been
provided**. It does not implement their gene selection or distance calculation.
The full samples gate must subsequently test both frozen R step14 and current
Rust step14 matrices, ordered groups, zero selected genes and small groups.
Positive-filter boundary genes3/6 remain unresolved; no tolerance is inherited
from the artifact checker's1e-12. Default Leiden and later HMM/report/CLI work
remain excluded. Kernel correctness is not a performance or release decision.

## Primary evidence and provenance

Accepted witness:
`/Volumes/Zane's HDD/rsomics-fixtures/evidence/infercnv-ingestion-2026-09-30/run-36707344893/artifact/samples-clustering/witness.json`,
SHA25626333ff388b0916d79736c7b3370b9f907fb943ca918a92ef5529a4d8bdfd304.
It contains152 trees,107331 pair distances and at most142 leaves per tree.
All routes are retained, even when their numeric payloads are duplicates.
The original installed-package source is infercnv1.28.0 commit
b421d9405c97a309b081ef86d455e976df93eae4; its actual samples path calls
parallelDist0.2.7 Euclidean then fastcluster1.3.0 hclust(method="ward.D2").

Fresh primary source archives are retained externally under
`/Volumes/Zane's HDD/rsomics-fixtures/sources/infercnv-ward-2026-10-22/`:

| Package/tag | CRAN Git mirror commit | Archive SHA256 |
| --- | --- | --- |
| fastcluster1.3.0 | 347ef70fe3c5257fecab7dcfcd71c0620f451171 | 9f3e6e55769eeb09b14417f03b08f781f30601af3926ed2a35603edd95c5f841 |
| parallelDist0.2.7 | 10db7bebaedcaf345731a3c7ad383ca5f68fe087 | 12f841a44366cb0dceab550e86983cf1e7ebf8b7749bcf4743f82f7f165839f3 |

The relevant original fastcluster files are src/fastcluster.cpp
(NN_chain_core, f_ward, linked list and union-find), src/fastcluster_R.cpp
(initial squaring, final square roots, stable height sorting, R merge encoding
and leaf traversal), R/fastcluster.R, LICENSE and DESCRIPTION. Read source is
pinned by commit rather than an unversioned web page. The package's actual
two-condition BSD notice must be reproduced verbatim in the product's
THIRD_PARTY_NOTICES when adapting this implementation. The two source headers
have the same Daniel Muellner/Google copyright and no separate GPL section.

parallelDist's GPL source is a behavior reference only; no production code is
copied from it or infercnv. Its Euclidean expression delegates reduction to
Armadillo; installed RcppArmadillo15.6.0-1 arithmetic requires its own later
investigation. This kernel consumes distances and makes no promise about that
reduction. Historical team-owned rsomics-linkage commit073b3fe829a280744a82077cd3281e418e4dd32f
is refactor-then-merge material, not an oracle. Its per-update square roots,
chain restart and SciPy output policy are not imported. Condensed indexing,
typed temporary merges and union-find mechanics may be reused with provenance.

## Kernel contract

In src/cnv/samples/ward.rs:

```rust
pub(crate) struct Dendrogram {
    pub(crate) merge: Vec<[i32; 2]>,
    pub(crate) height: Vec<f64>,
    pub(crate) order: Vec<usize>,
}

pub(crate) fn ward_d2(
    distances: &[f64],
    leaves: usize,
    max_workspace_bytes: usize,
) -> rsomics_common::Result<Dendrogram>;

pub(crate) fn required_workspace(leaves: usize) -> rsomics_common::Result<usize>;
```

Input pair order is R lower-column-major, equivalent for a symmetric matrix
to upper-row-major: (0,1),(0,2),...,(1,2),... . Require2<=leaves<=i32::MAX,
exact checked leaves*(leaves-1)/2 length, finite nonnegative distances and
finite squares. Reject nonfinite/negative intermediate updates; never clamp
or replace a distance. The input is borrowed and remains bitwise unchanged.

The output uses literal R merge encoding: singleton leaves -1 through -leaves,
internal nodes1 throughleaves-1. Children are ordered by their underlying nonnegative
cluster identifier before conversion, **not by the signed R integer**.
order contains zero-based local leaf indices, not global cell indices.

Square each initial distance once. In the nearest-neighbour chain use the
actual fastcluster restart condition tip<=3, restart at the smallest active
slot, strict '<' comparisons and predecessor-biased ties. Remove the smaller
slot and retain the larger slot. Update squared distance with exactly:
`((v+s)*a - v*c + (v+t)*b) / (s+t+v)`, where s/t are removed/retained sizes.
No fused multiply-add or algebraic reassociation is requested. Square-root
each raw merge height **before** sorting, since rounded square roots can tie.
Order by height then original merge ordinal, using allocation-free unstable
sorting to reproduce a stable height sort without hidden numeric scratch.
Relabel sorted raw endpoints via union-find; traverse the final tree
iteratively left before right. No recursive per-leaf stack or O(n^3) scan.

max_workspace_bytes bounds the sum of requested initialized numeric buffer
sizes at their overlapping peak, including the copied distances, active-list
links, chain, member sizes, raw merges, relabel parents, output merge/height/
order and traversal stack. It excludes Vec headers, allocator overhead,
sort's call stack and caller-owned input; it is not a process-memory bound.
Compute all sizes with checked arithmetic before any workspace allocation.
Use fallible exact reservations; allocation errors propagate. Document the
exact buffer-size formula in tests and test its equality boundary.

## Fixture and tests

A deterministic stdlib-only controller extractor accepts the pinned original
witness and produces an LF TSV with one six-row record per actual tree:
tree(case/route/group-index, leaves), distances, merge pairs, heights, order,
end. Fields are tab-separated; merge pairs are comma-separated signed
integers within their field. The order row preserves R's one-based indices;
the Rust fixture reader converts them to zero-based indices.
Preserve original scalar strings and all152 records; order source cases
lexicographically, routes isolated/full_run, groups in recorded order.
Validate the pinned witness SHA before creating the destination exclusively.
Validate the expected record/pair/max-leaf counts and per-tree lengths,
finite nonnegative numbers, R topology and leaf permutations. Reject absent
routes/trees or malformed scalar strings rather than silently dropping them.
Fixture generation is derivative data, not reconstruction of missing evidence.

The rsomics-sc private integration test imports the actual ward.rs by path,
parses the checked-in generated fixture with no new dependency, and compares
every merge/order exactly and every parsed binary64 height bitwise. No native
tolerance, epsilon, omitted tree or summary oracle is allowed at this gate.
If exact arithmetic does not reproduce a real tree, retain the failing case
and diagnose before changing the kernel or acceptance criterion.

Additional tests pin: unequal/equal/zero ties, n=2, invalid n, malformed shape,
overflow dimensions, NaN/infinities/negative input, square overflow, immutable
input, exact workspace budget and budget-minus-one rejection. A high-dynamic-
range tie and a multi-merge chain test must prevent restart/height-sort
regressions even if small standard examples pass.

## Native execution and delivery

Use the existing sc-cnv-core-candidate workflow and immutable snapshots.
First freeze tests with the kernel absent as red-9: Linux x86_64 must reach
Cargo compilation and fail at that missing module, with other source/metadata
gates checked. Then implement and freeze green-15. Retain Rust1.91.0 and the
existing Cargo.lock exactly. Native debug/release run on Linux/macOS
x86_64/aarch64; Linux strict Clippy and formatting plus unchanged-source checks
must pass. Existing controller and product tests must remain unchanged/passing.
Do not install, compile or run Cargo/R on this Mac: APFS occupancy is above80%.
All scratch/downloads/evidence stay on the specified external disks.

Review original run/jobs/logs/ZIP bytes and all four target outcomes before
claiming this bounded kernel accepted. Commit one concern directly to main
in each owning repo; preserve inherited VCF/scratch changes. Product is still
private/publish=false with no remote. World pushes require exact-head Control
plane CI. Advance to distance/group integration afterward without implying
whole-step15, performance or release acceptance.
