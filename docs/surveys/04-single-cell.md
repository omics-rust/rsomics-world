# Survey: single-cell domain

Verified 2026-05-30 against scanpy API, Seurat reference, Bioconductor (scran/scater/scuttle/
DropletUtils), STARsolo.md, alevin-fry main.rs, Scrublet/DoubletFinder/CellBender GitHub,
CellRanger algorithm docs. **Domain is largely greenfield** — 5 shallow crates exist, all
operate on pre-computed TSV not h5ad/AnnData.

> **The clearest cross-ecosystem dedup in the whole project:** nearly every analysis op exists
> in BOTH scanpy (Py) and Seurat (R), most also in scran/scuttle. → **one canonical Rust crate
> per op serves both.** Don't build scanpy-clone + Seurat-clone.

## Preprocessing layer (FASTQ → count matrix) → ADOPT

CellRanger (restricted license — not adoptable, but its ops define the standard: barcode
correction posterior>0.975, UMI Hamming-1 dedup, OrdMag+EmptyDrops cell calling). **Adopt**:
alevin-fry ① (generate-permit-list/collate/quant/infer + atac), simpleaf ① orchestrator,
anndata-rs (h5ad — project decision). STARsolo (MIT, STAR mode) is the open CellRanger-count
equivalent. **Velocyto spliced/unspliced output (for RNA velocity) is a gap.**

## Analysis layer — shared scanpy↔Seurat↔scran op set → canonical crates

| op | scanpy | Seurat | scran/scuttle | canonical crate |
|---|---|---|---|---|
| library normalize + log1p | normalize_total+log1p | NormalizeData | logNormCounts | `rsomics-sc-normalize` (P0) |
| per-cell QC | calculate_qc_metrics | PercentageFeatureSet | perCellQCMetrics | `rsomics-sc-qc` (P0) |
| cell/gene filter | filter_cells/genes | subset | quickPerCellQC | `rsomics-cell-filter` (exists, partial) |
| HVG | highly_variable_genes | FindVariableFeatures | modelGeneVar+getTopHVGs | `rsomics-sc-hvg` (P0) |
| scale | scale | ScaleData | — | `rsomics-sc-scale` (P0) |
| regress-out | regress_out | ScaleData(vars) | fitLinearModel | `rsomics-sc-regress` (P1) |
| PCA | pp.pca | RunPCA | denoisePCA | `rsomics-sc-pca` / adopt linfa-reduction |
| kNN graph | neighbors | FindNeighbors | buildSNNGraph | `rsomics-sc-neighbors` / adopt hnsw_rs |
| leiden/louvain | leiden/louvain | FindClusters | clusterSNNGraph | `rsomics-leiden` / adopt leiden-rs |
| UMAP / tSNE | umap/tsne | RunUMAP/TSNE | runUMAP/TSNE | adopt annembed / bhtsne |
| marker DE | rank_genes_groups | FindMarkers | findMarkers | `rsomics-sc-markers` (P1) |
| gene-set score | score_genes | AddModuleScore | — | fold into sc-markers |
| cell cycle | — | CellCycleScoring | cyclone | `rsomics-sc-cellcycle` (P2) |
| doublet | scrublet | (DoubletFinder ✗) | doubletCells | `rsomics-sc-doublet` (P1) |
| barcode rank | — | CalculateBarcodeInflections | barcodeRanks | `rsomics-barcode-rank` (exists) |
| empty-droplet cell call | — | SubsetByBarcodeInflections | emptyDrops | `rsomics-sc-emptydrops` (**P0 gap — converts raw→filtered matrix**) |
| pseudobulk | — | PseudobulkExpression | aggregateAcrossCells | `rsomics-sc-pseudobulk` (P1) |
| ComBat batch | combat | — | — | `rsomics-sc-combat` (P1) |
| ambient RNA | — | — | — | `rsomics-sc-ambient` (CellBender, P1) |

**PCA / Leiden / UMAP / Wilcoxon are NOT sc-specific** — shared with PLINK PCA, ATAC,
bulk RNA, general ML. Build them as **domain-agnostic Layer-A primitives** consumed by sc-* tools.

## Adopt list (no build) → alevin-fry ①, simpleaf ①, anndata-rs, leiden-rs ① (audit+fork), annembed ①, bhtsne ①, linfa-reduction ①, hnsw_rs ①.

## Existing crates (shallow)
`rsomics-barcode-rank` (knee from counts TSV; needs alevin-fry-output format), `rsomics-cell-filter`
(min-genes/umis/mito from stats TSV; needs h5ad input), `rsomics-count-matrix` (**misnamed for
sc — merges bulk featureCounts, not MEX/h5**), `rsomics-fastq-umi` (UMI extract), `rsomics-infercnv`
(CNV from expression — implemented-vs-stub unconfirmed).

## Key findings
1. Existing sc crates are shallow + TSV-bound (not h5ad) — need anndata-rs integration.
2. `rsomics-count-matrix` is misnamed for sc (bulk merge; MEX/h5 assembly is a distinct op).
3. **scran is deprecated** → scuttle (QC) + scrapper (normalization) are the new upstreams; cite those.
4. **DoubletFinder CC BY-NC 4.0 — BLOCKED** (no Rust derivative). Use scrublet (MIT) / scran
   doubletCells (GPL clean-room) / scDblFinder (GPL clean-room).
5. emptyDrops cell-calling is a **P0 gap** — nothing converts raw barcode matrix → filtered cells.
6. anndata-rs not standalone on crates.io (bundled in SnapATAC2) — adopt via path/fork until published.

## Verification notes
HIGH: scanpy.pp/tl lists + algorithm details, Seurat reference, scran/scater/scuttle/DropletUtils
(rdrr.io), STARsolo.md, alevin-fry main.rs, CellRanger algorithm page, Scrublet/DoubletFinder/
CellBender GitHub, scran-deprecation quote. MEDIUM: leiden-rs/annembed rayon claims (project docs,
not re-fetched), pp.neighbors/combat/scrublet generated pages (404 — from index summaries).
Follow-up: survey `scrapper` (scran's replacement).
