# Historical assets for inferCNV samples/Ward.D2

Read-only inventory; no native clustering implementation or compatibility
acceptance follows. Installed-package witness run `36701645992` is separate.
All clones below remain on `/Volumes/KIOXIA/Documents/omics-rust/`.

| Asset | Verified HEAD | Worktree | Bounded classification |
| --- | --- | --- | --- |
| `rsomics-linkage` | `073b3fe829a280744a82077cd3281e418e4dd32f` | clean | refactor-then-merge candidate |
| `rsomics-sc-dendrogram` | `7a4b8e37852fda8456dd9e16bed5ef017b38a87e` | clean | fixture/leaf-order assets only |
| `rsomics-fcluster` | `b5c918fa42a0b25b1bb7dcc4efdc677fb43ea69f` | clean | fixture assets only |
| `rsomics-stats` | `bac010ed3abf1e003729ec6189845d0d50b99578` | inherited `Cargo.lock` change | no clustering code to migrate; retain foundation unchanged |

## Contracts and differences

The linkage asset has useful condensed indexing, typed merge rows, stable
height sorting and union-find relabeling. Its upper-row pair sequence matches
the ordering of the same symmetric distances in R's lower-column vector.
Its boundary does not validate dimensions or bound arithmetic/capacities;
its feature-order Euclidean accumulation has not been accepted against
parallelDist. `src/linkage.rs`, `src/core.rs` and `src/nn_chain.rs` were inspected.

`src/lance_williams.rs` and the dendrogram asset's `src/linkage.rs` both square
the current distances and take a square root at each Ward update. The
fastcluster 1.3.0 R interface instead squares initial distances once,
maintains squared distances during clustering and square-roots final heights.
Arithmetic ordering and nearest-neighbour-chain restart behavior also differ.
Strict nearest-distance comparisons alone do not establish identical tie
merges. Neither historical core is a direct replacement for this oracle.

The dendrogram wrapper aggregates category means and clusters correlation
distances. That is not individual-cell Euclidean clustering. Retain useful
leaf traversal and tests without importing another duplicate Ward core or
the scientifically different wrapper into the CNV path.

The flat-cluster asset targets general SciPy criteria. This inferCNV slice
uses partition `none`: a tree-ordered single subcluster for large groups,
and input-ordered members without a tree for groups of at most two cells.
The asset parser casts floating IDs to usize, ignores the size field and
does not establish topology or finite-height invariants. It is not an
acceptable product boundary.

The existing statistics foundation exposes hypothesis tests, FDR, HWE,
p-value combinations and chi-square survival, not Ward clustering. Its
existing consumers do not justify a new public clustering API. Keep any
accepted implementation product-local until a second distinct product's
shared contract is concrete. Do not revive the retired repositories.

## Behavior sources and reuse provenance

The preserved shipped inventory records fastcluster 1.3.0 and parallelDist
0.2.7. Primary pinned sources inspected for the comparison:

- [fastcluster core](https://raw.githubusercontent.com/cran/fastcluster/1.3.0/src/fastcluster.cpp)
- [R interface](https://raw.githubusercontent.com/cran/fastcluster/1.3.0/src/fastcluster_R.cpp)
- [R wrapper](https://raw.githubusercontent.com/cran/fastcluster/1.3.0/R/fastcluster.R)
- [license](https://raw.githubusercontent.com/cran/fastcluster/1.3.0/LICENSE)
- [package metadata](https://raw.githubusercontent.com/cran/fastcluster/1.3.0/DESCRIPTION)

The primary package license permits redistribution and modification with
copyright, conditions and disclaimer retained. If source is adapted, retain
the actual third-party notice in the owning product's distribution rather
than presuming the team's historical-source permission covers it. This does
not authorize copying infercnv's R implementation into production.

Existing compatibility fixtures target SciPy or Scanpy, not installed
infercnv. Existing benchmarks do not establish inferCNV performance. Native
design must follow the accepted actual trees, member order, filter behavior,
numerical evidence and workspace requirements, with a fresh source/API
review before implementation.
