# Coherent internal SC Leiden discovery

Build one complete raw-to-discovery feature, not public micro-crates or
separate helper acceptance campaigns. Keep the accepted samples/HMM/Bayesian
default paths unchanged. Read the SC dossier, next-workflow contract and
actual default/k5 Leiden state before exposing any new CLI mode.

## Actual source and architecture

Pinned infercnv1.28.0 source is b421d9405c97a309b081ef86d455e976df93eae4.
Actual Seurat5.6.0 source is75dfa8603befe31505e6287d32dc86c607732bd6.
The retained source package/method texts and dependency screening live at
/Volumes/KIOXIA/Developments/tmp/leiden-native-source-20261027.HejCVc.
Default run40510078823 has genuine PCA graphs but one cluster per parent;
non-default-k5 run40515941483 succeeds and its actual partition is being
retained. Neither is native acceptance. Existing matrix/filter/distance/Ward/
worker machinery remains product-local and reusable. No foundation is added.

Pipeline: selected step14 FC genes → per-parent VST/scale/PCA or simple kNN →
actual-neighbor graph semantics → CPM Leiden → size-ordered subclusters,
within-subcluster Ward and stitched parent trees → typed report/HMM input.
Expression14 and original cell/gene maps remain immutable. Preserve the
source's small/k>=N bypass distinction and global cell identities.

Source numerical policy:

- No guessed shift or NormalizeData. VST uses sample means/variances,
  quadratic Gaussian LOESS span.3/interpolate/cell.2 with no robust iteration,
  expectedSD=sqrt(10^fit), one-sided sqrt(N) clipping and legacy score
  arithmetic. Stable descending scores select2000 features.
- Scale uses sample SD and an upper10 cap only. PCA uses10 variance-weighted
  components. A cheaper cell-Gram EVD is a mathematical implementation
  change, not Seurat's feature-Gram Spectra solver or an asserted IRLBA path.
  Qualify ordered features, sign-aligned scores/subspaces, distances and graph.
- PCA graph uses exact RcppAnnoy0.0.23 Apache2 headers: float32 Euclidean,
 50 trees, default Kiss64 seed, search_k=-1. Preserve actual neighbor sets.
  SNN is shared/(2*k-shared), threshold>=1/15, including unit loops.
- Simple mode uses exact raw-expression RANN-style neighbors and binary
  unweighted adjacency. Confirm undirected conversion against actual graph,
  not an assumed Seurat SNN or UMAP policy.
- Leiden is CPM/unit vertex mass, resolution(11.98/N)^(1/1.165), beta.01,
  two outer iterations, singleton start. Native keyed streams are repeatable
  across workers, not claimed R/igraph bit streams. Retain loops and account
  for them consistently through refinement/aggregation.

## Concrete implementation choices and bounded risk

Prototype internal VST with loess-rs3.0.0 and PCA with faer0.24.0, using their
actual APIs and serial numerical kernels under parent-level workers.
loess-rs is a candidate, not accepted R LOESS equivalence: actual feature
ordering decides whether it can remain. Keep the fit/model private so it can
be replaced without locking another product into an uncertain API.

Reuse Annoy's exact permissive headers with a thin checked static C++ bridge;
do not copy the GPL Rcpp wrapper. For Leiden, adapt only needed MIT
network_partitions concepts/code from
773181488b4e5b81925e302b972b294fd51f61a4, retaining attribution, or implement
the same published algorithm independently. Use stable exact exponentials,
not its approximate exponential. A greedy library lacking beta is not a
drop-in. Reuse the existing product MT stream, not a new foundation/RNG.

## Ownership and interfaces

Root owns module/API integration, CLI/report, Cargo/build/lock resolution,
within-cluster Ward/tree stitching, attributions, snapshots and whole-feature
comparison/gates. Three isolated workers own only:

1. discovery/pca.rs and its private helper files. `pca(&CellProfiles,
   selected_genes, members, max_workspace_bytes) -> Result<PcaResult>`;
   result contains global feature indices, cell-major scores, feature-major
   loadings, standard deviations and component count. No CLI/Cargo edits.
2. discovery/graph.rs, cpp/annoy_shim.cpp and exact vendor/annoy assets.
   `Graph {vertices, edges: Vec<(usize,usize,f64)>, neighbors:Vec<Vec<usize>>}`;
   canonical undirected edges include loops. `pca_graph(scores,cells,dims,k)`
   and `simple_graph(profiles,selected_genes,members,k)` return Result<Graph>.
3. discovery/leiden.rs and private helper files. `partition(&Graph,
   resolution,seed:u32) -> Result<Vec<usize>>` returns contiguous one-based
   membership in vertex order. Root exposes the existing CalibrationRng
   crate-internally for its uniform/index draws without changing old streams.

All use rsomics_common::Result and narrow typed boundaries; no swallowed
allocation, numeric, I/O or invariant failure. Keep comments rare, except
required license/safety/invariant reasons. No worker stages, publishes,
deletes, installs, builds, launches campaigns or edits another owner's files.

## Whole-feature gate

Write a few strong internal algorithm/model tests, then wire the entire
usable workflow and compare actual k20/k5 PCA/simple data/graphs/partitions/
trees/report identities and1/4-worker outputs in one four-native gate.
Do not advertise an unfinished flag or always-one-cluster approximation.
Inspect any real stochastic or numerical discrepancy before changing a
criterion; deterministic inputs, format and structural identities stay exact.
One fresh whole-feature review and one fix pass follow the working operation.
Retain full process/CPU/RSS/flags/input/version evidence; keep representative
discovery performance and general biological claims separate.

Mac boot APFS>80% forbids local compile/install/dependency download. Use
existing source/read-only tools and external rustfmt only; resolve/build/test
dependencies on remote/hosted storage. Full30-family goal remains active;
the separate Ward source-corruption choice does not block this feature.

## Predeclared stochastic and numerical comparison

Before any native discovery comparison: Seurat feature order, global cell/
gene identities, preprocessing/filter selection and actual kNN/SNN graph
structure/weights are deterministic contracts. Keep them exact, apart from
existing expression14 numeric tolerance. PCA axes have arbitrary sign and
may rotate within an eigenspace; qualify standard deviations and pairwise
score distances at abs1e-6+rel1e-6, recording sign-aligned axis differences
as diagnostics rather than insisting on a particular eigenvector basis.

Leiden's keyed native random stream intentionally differs from igraph/R.
Do not require or claim identical source community numbers or every random
partition. Require complete positive labels, exact native membership/report
joins, connected communities, objective/parameters matching CPM beta.01/two
iterations, reproducible native1/4-worker output, and native unnormalized
CPM quality no lower than the actual source partition on the same graph
(absolute numeric tolerance1e-9). Record source/native cluster counts,
canonical membership agreement and ARI rather than hiding disagreements.
If quality is lower, diagnose before accepting or changing the contract.
This is graph/model/quality compatibility, not exact R RNG, biological
equivalence or a claim that matching objective alone proves the algorithm.
The fresh whole-feature code review and internal model tests also remain
required. Where memberships agree modulo labels, compare source parent-tree
clades/heights by global cell identity; otherwise validate the native tree
against its own complete partition and the retained Ward/kernel contract.

## First complete execution and optimization

Linux bootstrap40537142945 preserves all existing lock pins/checksums and
passes debug/release tests plus strict Clippy. All four raw k20 processes
pass. Retained k5 PCA output independently matches every parent's ordered
2000 features, PCA geometry and exact graph. Two observed issues prevent
acceptance: singleton stitching fails to carry ape's consumed root-edge
state, and one native two-iteration local optimum has CPM61.047143529770906
versus source61.066181137916452. Neither a single-node move nor a whole-
community merge improves that local optimum; this is not a graph mismatch.

Retain the original failed artifact, numerical tolerances and quality gate.
Repair the exact tree-state semantics. Before the next native execution,
use eight deterministic independent starts, each retaining the same two
outer iterations/beta.01, and choose the highest original-graph CPM quality.
The first start remains the previous seed; other starts use a fixed derived
seed schedule. This is a general objective-guided search improvement, not a
fixture-specific partition, source-label injection or chosen passing seed.
Report restarts=8 explicitly, distinguish it from upstream's one start,
retain deterministic worker identity and measure its eventual whole-path cost.
The source/public model and exact RNG streams are still not claimed identical.
