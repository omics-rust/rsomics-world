# Survey: single-cell expression analysis

Verified 2026-07-31 against Scanpy 1.12.3, Seurat 5.5.1, AnnData, current
Bioconductor single-cell packages, infercnv 1.28.0, the 10x feature-barcode
matrix contract, and public Cell Ranger, STARsolo, alevin-fry, and simpleaf
documentation.

This survey records the upstream workflow. The public architecture and exact
historical source dispositions are in the
[`rsomics-sc` dossier](../10-products/sc.md).

## Product boundary

Single-cell analysis is stateful. Normalization, features, reductions, graphs,
clusters, embeddings, markers, and metadata all refer to the same cells and
features. The target is therefore one `rsomics-sc` product over an annotated
dataset, not one crate per Scanpy or Seurat function.

Scanpy and Seurat expose many equivalent scientific operations through
different object systems. They are compatibility sources for one set of
product modules:

| Workflow stage | Scanpy | Seurat/Bioconductor overlap | Target |
|---|---|---|---|
| import and state | readers, AnnData | `Read10X`, SingleCellExperiment | `import`, `export` |
| QC and filtering | QC metrics, cell/gene filters, Scrublet | Seurat QC, scuttle, DropletUtils | `qc` |
| normalization | total/log1p, Pearson residuals | `NormalizeData`, scrapper/scran lineage | `normalize` |
| feature selection and scores | HVG, score genes, cell cycle | variable features, module scores, cyclone | `features` |
| correction and scaling | regress, scale, ComBat | `ScaleData`, batch methods | `correct` |
| reduction and graph | PCA, neighbors, UMAP, diffusion | PCA, neighbors, UMAP | `reduce`, `neighbors` |
| clustering | Leiden/Louvain | `FindClusters`, graph clustering | `cluster` |
| markers | rank/filter/overlap | `FindMarkers`, Bioconductor marker tests | `markers` |
| trajectory | diffusion pseudotime, PAGA | trajectory ecosystems | `trajectory` |
| aggregation | `aggregate` | pseudobulk aggregation | `aggregate` |
| derived analysis | spatial metrics, inferCNV | spatial statistics, infercnv | `spatial`, later `cnv` |

## Historical source pool

Twenty-nine candidates route to `rsomics-sc`. They cover useful parts of QC,
normalization, HVG selection, scaling, regression, ComBat, PCA, exact
neighbors, diffusion, PAGA, DPT, marker testing, scores, pseudobulk, spatial
statistics, and an inferCNV approximation.

They are not a working product:

- most exchange 10x directories, dense TSV, Matrix Market, or graph triplets
  instead of maintaining annotated state;
- filters, HVG flavors, and exact neighbors are duplicated;
- two neighbors snapshots have no Git history;
- h5ad/Zarr I/O, Scrublet, scalable approximate neighbors, Leiden, UMAP, and a
  coherent report are missing;
- dense transformations do not define a product-level memory plan;
- compatibility targets mix older Scanpy, NumPy, SciPy, and umap-learn
  versions;
- the inferCNV approximation omits most of the real infercnv workflow.

The implementations remain valuable algorithm, fixture, RNG, compatibility,
and benchmark assets. Their existence does not make an operation stable or
release-ready.

## Matrix generation

`rsomics-sc` starts from a feature-by-cell matrix with stable identifiers.
FASTQ alignment, barcode correction, permit-list generation, UMI
deduplication, and transcript/gene quantification remain upstream workflows.

Open implementations such as STARsolo, alevin-fry, and simpleaf can produce
the matrix. Cell Ranger behavior is an interoperability source, not a reason
to embed a restricted upstream implementation.

Empty-droplet cell calling sits at the boundary between raw and filtered
matrices. It is a later `qc` capability after the initial counts-to-clusters
slice; it is not a separate public crate.

## Required first workflow

The first release must complete:

1. 10x MEX or h5ad import;
2. QC metrics, doublet evidence, and explicit filtering;
3. normalization and log transform;
4. highly variable feature selection;
5. scaling and PCA;
6. scalable neighbors;
7. Leiden clustering;
8. UMAP;
9. marker analysis;
10. annotated state, tables, and report export.

Operations may also run independently, but all read and write the same checked
state contract. A collection of fast TSV programs without this path is not a
single-cell product.

## Later workflow decisions

- Pearson-residual normalization, regression, ComBat, scores, and cell cycle;
- diffusion maps, DPT, PAGA, and dendrograms;
- pseudobulk and spatial statistics;
- complete inferCNV;
- empty droplets, ambient RNA, demultiplexing, and additional integration;
- RNA velocity, multiome, and spatial-assay I/O.

Deep generative methods such as scVI are adoption/integration decisions with
their own model and accelerator requirements, not small Rust port targets.

## Shared components

`rsomics-common` and `rsomics-help` provide the common runtime and CLI
experience. Policy-free numerical items may evolve in `rsomics-stats` through
single-cell, ecology, PLINK, DE, and signal consumers.

AnnData state, cell/feature metadata, preprocessing recipes, neighbor graphs,
and embeddings remain product-internal. Similar algorithms in multiple
single-cell subcommands still represent one product consumer and do not
justify public `rsomics-anndata`, graph, PCA, UMAP, or Leiden foundations.

## Evidence state

The upstream survey and source routing are complete. The product is not
release-ready. It still needs the complete first workflow, current-oracle
compatibility, representative sparse and backed-data CPU/memory/I/O evidence,
a strict hot-path advantage, exact-head four-native-platform CI, and public
API review.
