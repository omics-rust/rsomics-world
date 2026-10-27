# Next integrated inferCNV workflow contract

Read-only source preparation, not current-candidate acceptance or a revised
definition of complete inferCNV. No builds, installs, CI or product changes.
Source is infercnv1.28.0 commit b421d9405c97a309b081ef86d455e976df93eae4,
the already retained source archive with SHA
b2a1b6f8cc09dc04562513e3cd877b3c97a6efcd5ff92cb5e0763660905e3207.
All line references below are to files inside that pinned archive.

## Recommended next usable slice

After the grouped-i6 candidate passes, add raw counts through real Ward
subcluster discovery (`pheight` and `qnorm`), i6 predictions for those
subclusters, and actual state/region outputs. This reuses the existing
distance/Ward/filter machinery and current calibration; it introduces a
biologically meaningful finer grouping rather than another standalone helper.
It is explicitly a non-default subclustering profile, not default inferCNV.

In parallel, capture a real stage18/19 Bayesian witness from the retained real
stage17 objects and installed package/JAGS environment. This is oracle work
for the next meaningful post-HMM operation, not a separate primitive campaign.
Then implement the native Bayesian mixture/filtering workflow before claiming
the ordinary HMM-enabled run is complete. Native default Leiden/PCA follows
its actual graph/preprocessing oracle, not an approximation of these profiles.

## Subcluster contract and default-path gap

`R/inferCNV_ops.R:267-297` defaults to i6, `analysis_mode='subclusters'`,
Leiden/PCA/CPM, `k_nn=20`, auto resolution, `BayesMaxPNormal=0.5`,
`reassignCNVs=TRUE`; HMM itself is opt-in. Step15 discovery calls are at
1066-1143; step17 dispatch independently selects samples/subclusters/cells at
1251-1304. Current grouped-samples inference is therefore not the default
subcluster-discovery path.

`R/inferCNV_tumor_subclusters.R:34-81,181-267` uses existing temporary gene
selection and Ward.D2. For groups with more than two cells: pheight cuts at
`p_val * max(height)`; qnorm cuts at the `(1-p_val)` normal quantile fitted
using the mean and sample SD of Ward merge heights. Small groups remain whole.
Names, cut-height inclusion, traversal and actual memberships need one real
raw workflow oracle. In particular, lines242-257 form successive traversal
chunks using lengths from sorted split labels, not a direct per-label gather;
do not silently assume equivalence. qgamma additionally requires gamma MLE
and gamma quantiles; shc is listed but its implementation branch is commented
out and errors. Random trees are a separate early-stage path (ops.R:716), not
a variant to fake by cutting the current step14 tree.

The true default is materially larger (`tumor_subclusters.R:569-640,699-740`):
groups under3 cells are retained; `k_nn >= group size` keeps a single group;
auto resolution is `(11.98/ncells)^(1/1.165)`. PCA mode creates a Seurat
object, runs FindVariableFeatures, ScaleData over all genes on counts,
RunPCA(npcs=10), FindNeighbors, then igraph Leiden on its weighted SNN graph.
A FindVariableFeatures warning returns a non-Seurat value and falls back to
the simple route. Capture the actual route, Seurat/igraph versions, features,
scaling, PCA, graph and membership from the real workflow before reproducing
it. The simple route is RANN nn2 then binary neighbor adjacency and undirected
igraph Leiden; despite its helper name, it does not compute SNN Jaccard weights.
Subclusters are ordered by decreasing size, followed by within-subcluster
Ward trees and stitched display-tree geometry. Optional per-chromosome
subclusters add a distinct graph/membership route, not just repeated reporting.

## Bayesian post-HMM operation

`ops.R:1362-1384,1405-1452` runs the actual mixture when HMM is on,
BayesMaxPNormal>0 and the source state-condition passes. It uses expression
data, HMM state assignments and the original region/group tables, then
filters/reassigns states and adjusts reports. It does not denoise expression.

`R/inferCNV_BayesNet.R:247-264,294-336` joins each unique reported CNV ID
to genes/cells using the actual pred_cnv_genes and cell_groupings tables.
For i6, lines150-161 take means and raw SDs from the transformed spike and
convert SDs to precisions `1/sd^2`; these are not cell-count-fitted HMM SDs.
`inst/BUGS_Mixture_Model:1-57` defines one shared theta~Dirichlet(1,...,1)
per region, one epsilon~Categorical(theta) per cell, and independent normal
gene observations with fixed state means/precisions. The extra sigma~Gamma
node is unused in the likelihood. i3 uses the analogous three-state model.

The actual JAGS protocol (`BayesNet.R:1054-1106`) initializes6/i6 or3/i3
chains with all cell states set to each corresponding state, adapts500,
updates200, then samples1000 iterations of theta and epsilon. Region-level
probabilities are posterior mean theta; cell probabilities are epsilon
frequencies after chain concatenation (364-385,1137-1149). `CORES>1` parallelizes
regions with mclapply (409-427). Inits contain no explicit JAGS RNG seed or
algorithm, so R set.seed alone does not establish reproducible JAGS draws.

Filtering uses strictly P(normal)>threshold and sets the entire region/cell
rectangle to neutral, dropping its posterior/region entry (565-607). Afterwards,
reassignCNVs replaces retained regions with their posterior-mean maximum state
(491-525,1394-1420); the comment about preserving neutral maxima is not active.
Ties and the all-regions-removed shape need actual outcome evidence, not a
silent invented policy. Preserve original and filtered predictions separately.

A native Gibbs implementation can use the model's categorical and Dirichlet
conditionals, with gene sums/squared sums avoiding repeated likelihood scans;
this is a mathematical implementation opportunity, not tested code. Parallelize
independent regions with reproducible native streams. Validate actual posterior
and filter behavior with declared Monte Carlo uncertainty, not bitwise JAGS
stream equivalence or a threshold applied to adjusted-Viterbi emission scores.
The existing oracle workflow already installs JAGS/system headers; retain its
JAGS/rjags versions and actual posterior/filter output for the new witness.

## i3, per-cell modes and reuse

`R/inferCNV_i3HMM.R:17-78,99-150,435-491` pools reference expression (or all
observation groups without references) for mu/sample SD. Alternate means are
mu +/- abs(qnorm(p,0,sigma)); optional KS calibration simulates two samples at
11 offsets,100 repetitions each, then regresses offsets against mean KS pvals.
It computes KS calibration even when its result is not selected. The run
signature defaults HMM_i3_use_KS=FALSE despite contrary nearby prose and helper
defaults. The actual i3 transition diagonal is1-5t, not1-2t; capture installed
behavior before normalizing or correcting that unusual three-state matrix.

Per-cell i6 (`HMM.R:284-317`) uses the raw spike SD model, not the group-size
regression evaluated at one. Subcluster i6 (345-398) averages expression over
members and uses cell-count-fitted emissions. Reporting granularity is separate
from inference mode: consensus/subcluster/cell reports use state voting, with
smallest-state tie behavior (`HMM.R:706-758,977-984`). Implement both mode and
report semantics before advertising per-cell output.

Reuse current SC distances/Ward/filter, calibrated means/raw SDs, decoder,
parallel execution and region models internally. Historical rsomics-sc-pca
68fa73ae and the non-Git sc-neighbors/sc-neighbors-exact source pools contain
faer PCA/exact kNN assets already routed to SC. Refactor then merge selected
numeric kernels, not their parsers/CLIs or Scanpy graph policy: the neighbor
assets use f32 distance/Scanpy neighbor-width and UMAP-connectivity semantics,
not RANN/Seurat SNN. No inspected asset implements the Seurat feature/scaling/
irlba or igraph Leiden contract. The separate rsomics-pca52417ce7 routes to
ecology; it is not a Layer B dependency to add to SC. No named Gibbs/Dirichlet
asset was found in the routing ledger. Do not revive micro-crates or promote
RNG, PCA, graph, mixture or partition APIs to Layer A without a second concrete
consumer.
