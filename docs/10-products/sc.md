# Single-cell product dossier

Status: upstream-operation and historical-asset audit complete. The target
repository has not been created, and no product release has been published.

## Boundary

`rsomics-sc` is one stateful single-cell expression-analysis product. It owns
the coherent path from a feature-by-cell count matrix through quality control,
normalization, feature selection, reduction, neighborhoods, clustering,
embedding, marker analysis, trajectory analysis, aggregation, and selected
expression-derived analyses.

The public unit is the persisted annotated dataset and its workflow, not one
mathematical function. Operations update or derive named parts of the same
cell, feature, matrix, graph, embedding, and metadata state.

The primary behavior sources are:

- [Scanpy 1.12.3](https://github.com/scverse/scanpy/releases/tag/1.12.3) and
  its [stable API](https://scanpy.readthedocs.io/en/stable/api/index.html);
- [Seurat 5.5.1](https://github.com/satijalab/seurat/releases/tag/v5.5.1)
  for independently established overlapping workflow semantics;
- [AnnData](https://anndata.readthedocs.io/) and its on-disk encoding
  specification;
- 10x Matrix Market and feature-barcode matrix contracts;
- Bioconductor 3.23 `infercnv` 1.28.0 for the optional expression-derived CNV
  workflow;
- published methods for PCA, UMAP, Leiden, diffusion maps and pseudotime,
  PAGA, ComBat, Scrublet, analytic Pearson residuals, and marker tests.

Cell Ranger, STARsolo, alevin-fry, and simpleaf define upstream
FASTQ-to-count-matrix workflows. They are not reimplemented inside this
analysis product. `rsomics-sc` starts from a matrix with stable feature and
cell identities.

## State contract

The product uses an AnnData-like state model:

| State | Contract |
|---|---|
| `X` and layers | raw counts, normalized values, residuals, or scaled values with a declared role; raw counts are never overwritten implicitly |
| observations | stable cell identifiers, sample/batch labels, QC fields, doublet evidence, clusters, pseudotime, and derived scores |
| variables | stable feature identifiers, names, types, genomic annotations, QC fields, and HVG decisions |
| observation matrices | PCA, UMAP, diffusion, and other cell embeddings with method metadata |
| variable matrices | loadings and feature-level derived matrices |
| pairwise matrices | distances, connectivities, and other cell graphs with matching observation identity |
| unstructured metadata | parameters, versions, seeds, category order, provenance, and result schemas |

Every operation validates its required predecessor state and records the keys
it reads and writes. Cell or feature filtering updates all aligned state
atomically. A command cannot leave `X`, observation rows, embeddings, and
graphs with different identities or dimensions.

10x MEX is an import/export format, not the working state. TSV and Matrix
Market outputs remain available for interoperability, but users do not
manually shuttle unrelated files between algorithms.

## Operation map

### Product workflow

| Target subcommand | Upstream operations | Contract |
|---|---|---|
| `import` | Scanpy readers; Seurat `Read10X`; AnnData I/O | 10x MEX, h5ad, and declared matrix/metadata inputs into checked state |
| `export` | AnnData writers and stable table exports | h5ad or Zarr state plus requested MEX, tables, embeddings, graphs, and reports |
| `qc` | Scanpy `calculate_qc_metrics`, `filter_cells`, `filter_genes`, `scrublet`; Seurat and DropletUtils equivalents | barcode rank, per-cell/per-feature metrics, doublet evidence, explicit filter plan, and before/after report |
| `normalize` | `normalize_total`, `log1p`, analytic Pearson residuals; Seurat `NormalizeData` overlap | layer-aware transformations with exact denominator, target sum, offset, and clipping contracts |
| `features` | `highly_variable_genes`, `score_genes`, `score_genes_cell_cycle`; Seurat variable-feature and module-score overlap | HVG flavors, gene-set scores, and cell-cycle scores without losing the source layer |
| `correct` | `regress_out`, `scale`, `combat` | covariate regression, centering/scaling, clipping, and declared batch correction |
| `reduce` | `pca`, `diffmap`, `umap`, `tsne` | named embeddings and loadings with solver, feature mask, dimensions, seed, and graph provenance |
| `neighbors` | Scanpy `neighbors`; Seurat `FindNeighbors` | exact or approximate kNN, distances, UMAP connectivities, metric, representation, and deterministic tie policy |
| `cluster` | Scanpy `leiden`/`louvain`; Seurat `FindClusters` | graph clustering with resolution, objective, weights, seed, and stable category labels |
| `markers` | `rank_genes_groups`, `filter_rank_genes_groups`, `marker_gene_overlap`; Seurat `FindMarkers` overlap | group contrasts, marker tests, multiple testing, expression fractions, fold changes, filters, and reference-set overlap |
| `trajectory` | `diffmap`, `dpt`, `paga` | root selection, diffusion state, pseudotime, cluster abstraction, and disconnected-component policy |
| `aggregate` | Scanpy `aggregate`; Seurat pseudobulk; Bioconductor aggregation | pseudobulk sums or means by sample and biological group with complete metadata |
| `spatial` | Moran's I, Geary's C, embedding density | graph/spatial feature autocorrelation and embedding-density evidence |
| `cnv` | `infercnv` | optional complete expression-derived CNV workflow with references, genomic ordering, denoising, HMM policy, and reports |
| `run` | Scanpy/Seurat counts-to-clusters practice | declarative end-to-end plan composed only from stable operations |

`run` expands to a printed, reviewable plan. The execution record stores input
hashes, state keys, parameters, versions, seeds, stage durations, peak memory,
and output hashes. It does not hide defaults in an opaque recipe.

### Cross-ecosystem deduplication

Scanpy, Seurat, and Bioconductor often expose the same scientific operation
under different object systems. `rsomics-sc` implements one typed operation
with explicit compatibility profiles where semantics differ.

| Capability | Canonical decision |
|---|---|
| library normalization and log transform | one `normalize` operation; Scanpy and Seurat target sums and storage choices are profiles |
| variable features | one `features --hvg` operation with `seurat`, `cell-ranger`, `seurat-v3`, and other completed flavors |
| PCA | one `reduce --method pca`; solver and sparse/dense behavior are explicit |
| neighborhoods | one `neighbors`; exact and approximate search are execution modes |
| Leiden/Louvain | one `cluster` operation with named objective and implementation |
| UMAP/t-SNE | `reduce` modes sharing the selected representation and neighborhood provenance |
| marker analysis | one `markers` operation with named tests and contrast model |
| cell cycle and module scores | `features --score` modes sharing the same binned-control contract |
| pseudobulk | one `aggregate` operation; differential analysis belongs to the bulk DE products |

The product does not create separate Scanpy-compatible and Seurat-compatible
binaries.

## Data and execution model

- Sparse count matrices remain sparse through operations that preserve
  sparsity. Densifying transformations estimate their memory requirement and
  fail before allocation when the selected execution mode cannot fit.
- Cell and feature identifiers are unique, non-empty, and stable. Feature ID,
  display name, and feature type are separate fields.
- Matrix orientation is checked at import and never inferred later from shape.
- Integer count requirements are enforced for downsampling, doublet
  simulation, and count-model operations. Normalized matrices cannot silently
  enter raw-count methods.
- Layers and derived state carry a provenance record. An operation refuses a
  stale graph or embedding after its source representation or identities
  change.
- Category ordering, reference groups, contrasts, batch keys, gene sets,
  mitochondrial feature rules, and genomic gene order are declared inputs.
- Randomized operations accept and record a seed. Reproducibility covers the
  algorithm and RNG family, not merely a top-level integer.
- Approximate-neighbor recall is measured against an exact reference on a
  bounded sample. Exact search is not used by default where its quadratic cost
  makes a realistic dataset impractical.
- Statistical results include method, hypothesis, effect, denominator,
  multiple-testing family, exclusions, and failure counts.
- State and report outputs are staged and published together. In-place updates
  use a recoverable replacement and never truncate the only valid dataset.

## Target structure

```text
src/
├── cli.rs
├── state/
│   ├── model.rs
│   ├── keys.rs
│   ├── provenance.rs
│   └── transaction.rs
├── io/
│   ├── anndata.rs
│   ├── mex.rs
│   └── tables.rs
├── qc/
│   ├── metrics.rs
│   ├── barcodes.rs
│   ├── doublets.rs
│   └── filter.rs
├── preprocess/
│   ├── normalize.rs
│   ├── residuals.rs
│   ├── features.rs
│   ├── regress.rs
│   ├── scale.rs
│   └── combat.rs
├── graph/
│   ├── neighbors.rs
│   ├── connectivities.rs
│   └── cluster.rs
├── reduce/
│   ├── pca.rs
│   ├── diffusion.rs
│   └── umap.rs
├── analysis/
│   ├── markers.rs
│   ├── trajectory.rs
│   ├── aggregate.rs
│   ├── spatial.rs
│   └── cnv.rs
└── workflow/
    ├── plan.rs
    └── run.rs
```

The internal layout follows shared state and computation, not the names of the
29 historical repositories.

## Historical asset disposition

Twenty-seven candidates are clean Git worktrees at the listed revisions. The
two neighbors directories are unversioned source snapshots and cannot be
treated as authoritative provenance.

| Asset and source snapshot | Disposition |
|---|---|
| `rsomics-barcode-rank` `7b2aa87ed3914f65b75e41c7491e4fb5ceae4c0f` | Refactor statistics and fixtures into `qc`; replace counts-only TSV input |
| `rsomics-cell-filter` `97e3acd119ca343791792547f742f4a692c145a9` | Test and CLI-policy asset; deduplicate with `rsomics-sc-filter` |
| `rsomics-infercnv` `393882653ea27e65f38ec6997f8f26e91d04927b` | Fixture and parser asset only; discard the five-step approximation as an inferCNV implementation |
| `rsomics-sc-cell-cycle` `69ec53b7f9c3c9247db9695fda126112b8c26867` | Refactor then merge into `features --cell-cycle` |
| `rsomics-sc-combat` `2d6e58a3634be02e6f2b5e152ab9fb5617cffb1d` | Refactor batch-only parametric path into `correct --method combat`; add covariate and current 1.12.3 behavior |
| `rsomics-sc-dendrogram` `7a4b8e37852fda8456dd9e16bed5ef017b38a87e` | Merge useful group aggregation, linkage tests, and leaf-order fixtures into analysis presentation |
| `rsomics-sc-diffmap` `41166d039fb517056e759a30c108c03bff5b06bd` | Refactor sparse eigensolver and fixtures into `reduce --method diffmap` |
| `rsomics-sc-downsample` `22593584c22aee24a3b36a245b5407efe4b716ba` | Refactor then merge into `qc --downsample-counts`; retain RNG fixtures |
| `rsomics-sc-dpt` `24b0077781725facfe164eea1e078ca456b9af62` | Refactor then merge into `trajectory --method dpt`; add branching and disconnected-state policy |
| `rsomics-sc-embedding-density` `9de0ba24921923ff49841bb01b7fee3cae001c36` | Refactor numerical core into `spatial --embedding-density` |
| `rsomics-sc-filter` `c855fff8665ce0116756ed90a9d3e5cf91cb60cd` | Primary matrix-filter seed for `qc`; make all aligned state changes atomic |
| `rsomics-sc-filter-rank-genes` `91dcb383886e4336fb074d13337652a28bc3c4b8` | Merge predicates and golden tables into `markers --filter` |
| `rsomics-sc-hvg` `9dc853b75b6e53ad39106189c2b63b96fb005cfd` | Refactor then merge the dispersion-based flavor into `features --hvg` |
| `rsomics-sc-hvg-cellranger` `f8c8304f2bf67af524949ee28bfdb7e63493dd8d` | Merge the Cell Ranger flavor and fixtures into the same HVG module |
| `rsomics-sc-marker-overlap` `a7c6c93271fd6e80488c985385516996a7592c73` | Refactor into `markers --reference`; add representative benchmark |
| `rsomics-sc-neighbors` content manifest `d4bd0925b138cd517b1a78c89e3cf00275c78ddc3e122ab9f6ea884452d0de74` | Unversioned algorithm and test asset; archive before selection |
| `rsomics-sc-neighbors-exact` content manifest `ced063f209a4234d4b92e8b310766b559ce1a7565a6de282762e3972e49689cd` | Duplicate unversioned exact-neighbor asset; diff against the other snapshot, retain one test lineage |
| `rsomics-sc-normalize` `7d30a6b26d240fdfb13747d8074011671da9c4d5` | Refactor then merge into `normalize`; preserve sparse arithmetic fixtures |
| `rsomics-sc-paga` `4c5afe3dfb5f77e82ea0ea7e365c0303afc249ef` | Refactor then merge into `trajectory --method paga` |
| `rsomics-sc-pca` `68fa73aeb1a257e79e2ffa32add1d94a1f5dd861` | Refactor solver, outputs, sign-aware tests, and benchmarks into `reduce --method pca` |
| `rsomics-sc-pearson-residuals` `9e98590460363f52b6783d18e81b6ac9df4f084b` | Refactor then merge into `normalize --method pearson-residuals` |
| `rsomics-sc-pseudobulk` `fd2c52e4a17ea6d710110c4a9824b83994a456e6` | Refactor then merge into `aggregate`; require sample-aware biological grouping |
| `rsomics-sc-qc-metrics` `5ee05ac392cde05d756854e8c61bfe5de35a6d99` | Primary metric seed for `qc`; integrate feature metadata and state |
| `rsomics-sc-rank-genes` `1c6de9626ed7959e120c83bcbd36f58550d59532` | Refactor Welch and Wilcoxon paths into `markers`; add all declared Scanpy methods only when complete |
| `rsomics-sc-regress-out` `fa510fa49a2781c2eb3ea1706e0db7f6192bbdfa` | Refactor then merge into `correct --method regress`; retain singular-design failure cases |
| `rsomics-sc-scale` `1af0793cfbd9714313e390c38d5c7bb62a7b9363` | Refactor then merge into `correct --method scale`; preserve sparse uncentered mode before advertising it |
| `rsomics-sc-score-genes` `d820e79547faf74ff4cbf8eaae29d4cf94dec736` | Refactor then merge into `features --score`; retain RNG and binning fixtures |
| `rsomics-sc-spatial-autocorr` `58081d2bcb566afd9b7588b73fdc70947cc16bac` | Refactor Moran and Geary primitives into `spatial`; review reuse through `rsomics-stats` only with another product consumer |
| `rsomics-sc-subsample` `2287feb8588fa2b1ae0c9d43ea167e2843de4846` | Refactor then merge into `qc --sample-cells`; current Scanpy uses `sample`, with `subsample` deprecated |

Content-manifest hashes cover all files outside `target` in each unversioned
neighbors directory, with sorted relative paths and SHA-256 file hashes. They
identify the audited source but do not replace Git provenance.

## Existing implementation problems

- The assets are standalone file converters. Most read a 10x directory,
  dense TSV, or graph triplets and write another disconnected matrix or table.
  They do not maintain AnnData state or provenance.
- `rsomics-cell-filter` and `rsomics-sc-filter` overlap. `rsomics-sc-hvg` and
  `rsomics-sc-hvg-cellranger` are flavors of one operation. The two neighbors
  directories describe the same exact-search operation and lack Git history.
- Essential counts-to-clusters stages are absent: persistent h5ad/Zarr I/O,
  Scrublet, approximate neighbors, Leiden clustering, UMAP, coherent plotting,
  and atomic state filtering.
- Several transformations emit a dense Matrix Market file without forecasting
  memory or preserving which input layer and feature mask produced it.
- Individual assets hard-code mixed Scanpy 1.11.5, 1.12.1, NumPy, SciPy, and
  umap-learn behavior. The current oracle is Scanpy 1.12.3.
- Exact internal RNG and float replication is valuable evidence, but it is not
  a substitute for a stable scientific result contract when upstream solver,
  ordering, or implementation details change.
- The historical inferCNV asset only log-normalizes, smooths, and subtracts a
  reference mean. It omits the upstream denoising, reference handling,
  subclustering, HMM, Bayesian filtering, reporting, and checkpoint semantics,
  while permissively converting malformed coordinates and expression values
  to zero.
- Marker ranking offers only Welch and Wilcoxon; cell-cycle and gene-set
  scoring assume feature IDs without a complete identifier-resolution policy.
- No single benchmark measures import, sparse state, preprocessing, graph
  construction, clustering, embedding, marker analysis, serialization, and
  peak RSS as one workflow.
- `rsomics-sc-filter` and `rsomics-sc-marker-overlap` have no benchmark
  harness. The remaining harnesses do not establish representative product
  performance.
- CI and compatibility evidence do not cover the four native platform
  classes at one exact target head.
- READMEs and source contain extensive algorithm narration and version-history
  commentary that should become concise user contracts, tests, or provenance
  records rather than production comments.

## Missing operations required for a real product

The historical pool does not define the product scope. The first release
requires several new implementations or reviewed dependencies:

- h5ad/Zarr read, write, slicing, categorical metadata, sparse arrays, and
  backed or streaming behavior;
- 10x MEX import with feature types and gzip support;
- doublet simulation, scoring, threshold evidence, and batch handling;
- scalable approximate neighbors with measured recall and deterministic
  metadata;
- Leiden clustering with weighted-graph and resolution semantics;
- UMAP optimization and stable embedding output;
- coherent report and plot generation over the persisted results.

Empty-droplet cell calling, ambient RNA correction, sample demultiplexing,
modern integration, RNA velocity, multiome, and spatial-assay I/O are later
workflow decisions. They are not placeholders in the first release.

## Foundations and external dependencies

- `rsomics-common` owns errors, exit mapping, execution reports, seeds,
  resource estimates, progress policy, path collision checks, and
  transactional state publication.
- `rsomics-help` is mandatory. It owns the state-aware subcommand hierarchy,
  shared dataset/layer/key options, plan display, examples, output summaries,
  and terminal presentation.
- `rsomics-stats` may provide reviewed policy-free linear algebra,
  distributions, multiple testing, correlations, and spatial statistics.
  `rsomics-sc`, ecology, PLINK, DE, and signal are named product consumers, but
  each public item still requires consumer-side contract tests.
- PCA, graph, Leiden, UMAP, approximate-neighbor, HDF5, Zarr, and AnnData
  dependencies receive a license, maintenance, unsafe-code, determinism,
  threading, memory, and hot-path review before adoption.

State schema, single-cell policies, feature/cell metadata, preprocessing
recipes, graphs, and embeddings remain internal to `rsomics-sc`. No public
`rsomics-anndata`, `rsomics-graph`, `rsomics-pca`, `rsomics-umap`, or
`rsomics-leiden` foundation is created speculatively.

Layer B products do not depend on `rsomics-sc`. A future cross-product
exchange uses stable files, or a public foundation only after a second
consumer demonstrates the exact same policy-free contract.

## Compatibility gates

The compatibility matrix uses Scanpy 1.12.3 as the primary oracle and Seurat
5.5.1 or Bioconductor where it supplies an independent contract.

- Import/export gates cover dense and sparse h5ad, Zarr, categorical
  observation data, layers, embeddings, pairwise matrices, Unicode IDs,
  empty dimensions, gzip MEX, and corrupted state.
- QC gates cover zero-count cells and genes, mitochondrial feature selection,
  multiple feature types, batches, doublets, thresholds, and identity-aligned
  filtering.
- Preprocessing gates cover sparse and dense inputs, zero denominators, all
  HVG flavors advertised in help, batch keys, masks, clipping, covariates,
  singular designs, and layer selection.
- PCA comparisons are sign- and subspace-aware while preserving variance and
  loadings. Neighbor gates compare exact distances, connectivity construction,
  approximate recall, ties, metrics, and graph symmetry.
- Leiden gates pin objective, resolution, iterations, weights, seed, category
  ordering, disconnected nodes, and multigraph handling.
- UMAP gates pin graph, initialization, distance parameters, negative
  sampling, epochs, seed, and output metadata. Numerical comparison uses
  neighborhood preservation and aligned embedding evidence where exact
  coordinates are not a stable contract.
- Marker gates cover group-versus-rest and explicit references, ties, small
  groups, zero expression, effect definitions, adjusted p-values, filters,
  and complete feature identity.
- Workflow differentials compare state keys, shapes, identifiers, numerical
  outputs, categories, and declared plots after every stage, not only the final
  UMAP.

Committed golden fixtures always run. Live oracle tests never silently turn a
missing Scanpy, Seurat, or inferCNV environment into a compatibility pass.

## Performance gates

Performance is measured per stage and end to end:

- a small adversarial fixture for correctness and failure paths;
- PBMC 3k-class data for exact workflow comparison;
- at least a 50,000-cell sparse dataset for normal performance and memory;
- a larger dataset that exercises backed or streaming I/O and approximate
  neighbors without requiring dense materialization.

Each record includes versions, commits, binary hashes, machine, native target,
input provenance and hashes, state/layer keys, commands, seeds, threads,
warmups, timing distribution, total CPU, peak RSS, I/O, output hashes, and
compatibility results.

Comparisons include Python/R startup and serialization only when the rsomics
command performs the same work. Algorithm kernels are also measured directly.
Thread scaling records total CPU and memory. The stable replacement hot path
must show a strict throughput or resource-use advantage; a faster isolated
kernel does not compensate for slower or memory-heavier state round trips.

## Release slices

### Slice 1: counts to clusters

The first publishable product is a complete path:

1. import 10x MEX or h5ad;
2. compute QC and doublet evidence, then apply an explicit filter;
3. normalize/log-transform;
4. select highly variable features;
5. scale the selected representation;
6. compute PCA;
7. build a scalable neighborhood graph;
8. cluster with Leiden;
9. compute UMAP;
10. rank and filter marker genes;
11. write the annotated state, tables, execution record, and report.

Every step is available independently and through `run`. No advertised step is
a placeholder, and a failed run does not publish a partial state.

### Later slices

- Pearson residuals, regression, ComBat, module scores, and cell cycle;
- diffusion map, DPT, PAGA, and dendrograms;
- pseudobulk and spatial statistics;
- complete inferCNV;
- additional integration, demultiplexing, ambient RNA, multiome, spatial, and
  velocity workflows only after separate scope and evidence review.

Publication requires strict formatting and Clippy, tests, compatibility,
representative performance and memory, public API and hot-path review, and
exact-head native CI on Linux and macOS for both `x86_64` and `aarch64`.

## Explicit exclusions

- No revival of the 29 operation-sized repositories as public products.
- No TSV-only chain presented as a stateful single-cell workflow.
- No silent overwrite of raw counts, identities, layers, graphs, or
  embeddings.
- No dense conversion without a checked resource plan.
- No generic statistics, graph, PCA, UMAP, Leiden, or AnnData public crate
  without two concrete product consumers and contract tests.
- No raw FASTQ alignment, barcode correction, or UMI quantification in
  `rsomics-sc`.
- No dependency on another Layer B product.
- No inferCNV name or compatibility claim for the historical five-step
  approximation.
- No incomplete clustering, embedding, doublet, I/O, or plotting command in
  help or release documentation.
