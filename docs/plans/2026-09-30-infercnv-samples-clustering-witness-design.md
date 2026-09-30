# inferCNV samples-clustering witness

## Boundary and source

Obtain installed-package evidence for non-HMM step 15 with
`analysis_mode="samples"`, `hclust_method="ward.D2"`, and grouped or pooled
observations before implementing native clustering. This design does not
accept step-15 compatibility, a product API, default Leiden, plotting, HMM,
or a release. The existing prepared-input and raw-reader receipts remain
separate gates.

Pin infercnv 1.28.0 at commit
`b421d9405c97a309b081ef86d455e976df93eae4`. Inspected source is preserved at
`/Volumes/Zane's HDD/rsomics-fixtures/evidence/infercnv-shipped-2026-09-26/run-36221554790/artifact/infercnv-b421d9405c97a309b081ef86d455e976df93eae4/`.
References below are relative to this archive; see also
`../../.autopilot/state/infercnv-downstream-audit-2026-09-26.md`.

## Source contracts to witness

`R/inferCNV_ops.R:1129–1143` routes samples/cells to
`define_signif_tumor_subclusters(..., partition_method="none")`.
`R/inferCNV_tumor_subclusters.R:36–40` orders observation groups before
references; pooled observations concatenate the ordered observation maps,
not global matrix-column order. References remain separate in either mode.

For groups larger than two cells, the source computes Euclidean distances,
an hclust tree and one subcluster whose members follow tree leaf order
(`tumor_subclusters.R:189–259`). Smaller groups have no tree and retain input
index order (`:262–264`). `NAMESPACE:57,90` binds these calls to
**fastcluster::hclust** and **parallelDist::parallelDist**. Treat merge
encoding, tree order, labels and ordered members as oracle data rather than
assuming another Ward implementation has equivalent tie behavior. Export
tree presence per group: assigning NULL at `tumor_subclusters.R:118` can
remove an hclust-list entry.

The reference z-score filter also applies to samples. The source uses
`z_score_filter > 0` as a guard but a literal `>= 0.8` gene threshold
(`tumor_subclusters.R:45–57`). It changes the clustering matrix, not the
original expression object. Observe settings 0, 0.2 and 0.8 separately. An
empty `which()` result still enters the non-NULL branch before negative
subsetting; constant reference values also raise a zero-SD question. Preserve
actual package outputs, warnings or errors for both cases before specifying
native behavior. Do not silently repair either branch or infer that positive
settings implement an adjustable threshold.

## Execution and evidence

Extend the existing pinned installed-package oracle workflow; use its runner
temporary storage and preserve original artifacts externally. Do not install,
build or execute R locally while the Mac boot-disk gate is exceeded.

For each shipped/synthetic workflow case, run two independent routes with the same explicit
arguments, package versions, locale, thread count and seed:

1. Load the validated step-14 RDS, verify its hash and absence of `.hspike`,
   and invoke the actual package clustering function obtained from its
   namespace. Set the package's distance-thread context explicitly. Preserve
   both elements of the returned list.
2. Start from the original creation object and execute `infercnv::run` through
   step 15 in a fresh directory with `resume_mode=FALSE`, HMM off,
   `num_ref_groups=NULL`, no optional masking/denoising/pruning, and plotting
   disabled. Never run the preprocessing pipeline again on a step-14 object.
   Resolve checkpoint names with the real filename builder
   (`ops.R:3407–3417`) and retain `preliminary.infercnv_obj` (`:1153–1156`).

Require exact agreement between the routes for expression values and
gene/cell identities, reference/observation maps, tree presence, merges,
labels, order, heights and named ordered subcluster members. Verify the
original expression matrix is unchanged from step 14. Export actual R index
values and names, with explicit one-based indexing; do not reconstruct names
to conceal source differences.

Retain the clustering gene selection, filtered matrix, distance vector and
its ordering, tree fields and memberships. Bind isolated distance/tree
exports to the package-produced trees, rather than treating a second call
with self-generated expected output as validation. Preserve 17-digit numeric
exports alongside RDS, complete arguments, source/input/checkpoint hashes,
R session and fastcluster/parallelDist versions, logs and actual file
inventory. An independent checker must validate file hashes, shape, index
ranges, partition coverage, tree structure and agreement with upstream
exports. Later native acceptance requires exact structural ordering and a
separately justified numerical comparison for distances/heights.

## Required cases and acceptance

- Shipped subset/full and accepted synthetic inputs, grouped and pooled.
- One-, two- and three-cell groups; no references; multiple references in
  requested order; observation-map concatenation differing from column order.
- Equal profiles, equal distances and permuted input indices to expose tree
  ties and ordered membership.
- Filter off and positive guard values; constant references, no qualifying
  genes and all qualifying genes. Include values around the literal 0.8
  boundary without asserting their output in advance.
- A reference named `all_observations` in pooled mode: inspect the generated
  name collision (`tumor_subclusters.R:40,79`) before choosing a native policy.

Workflow cases require both routes to succeed and agree. Controlled tie and
filter probes may replace the expression matrix in a valid creation object
before invoking the actual clustering function. They characterize that function
only: rerunning preprocessing would change their deliberately chosen values,
so they are not advertised as end-to-end workflow comparisons. Degenerate probes
record branch-specific observed outcomes; they are not forced into successful
goldens. Unexpected or contradictory behavior blocks only the affected claim
and must be recorded explicitly. Accept this witness only after inspecting
the original artifact and every warning/error, with an exact-source CI
receipt. Native implementation, four-platform verification and clustering
performance are subsequent gates, not consequences of oracle collection.

## Exclusions and subsequent model work

Default `analysis_mode="subclusters"` uses Leiden, PCA/Seurat or the simple
RANN/igraph path, then cluster-tree assembly through ape
(`ops.R:285–297`; `tumor_subclusters.R:569–740`). It needs separate witnesses,
including fallback and small-group naming; samples/Ward does not validate it.
HMM i6 creates a hidden spike at step 3 (`ops.R:588–590`) and mirrors its
clustering (`tumor_subclusters.R:155–160`); it cannot be appended to the
current non-HMM step-14 matrix.

The native model currently stores expression state and creation groups but
no tree or ordered subclusters (`rsomics-sc/src/cnv/model.rs`, `input.rs`,
`pipeline.rs`). After observing this witness, design a product-local result
for group identities, global indices, optional trees, ordered memberships
and clustering gene selection, preserving the original matrix and explicit
reference identity. Budget distance/tree workspace. Do not add a public
foundation or lock that API before the evidence resolves its contracts.
