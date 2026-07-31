# Survey: R / Bioconductor + CRAN package landscape (cross-cutting)

Status: ecosystem survey. Current product boundaries and source decisions
supersede the original crate suggestions; see
[`../10-products/bulk-expression.md`](../10-products/bulk-expression.md).

Verified 2026-05-30 from Bioconductor package pages + download stats (bioconductor.org/
packages/stats/bioc/<name>, as of 2026-05-29) + CRAN pages. **This is the priority
rewrite frontier** — much of the field's analysis layer is decade-old, single-threaded,
memory-hungry R. Download ranks judge real-world usage. (R-from-Rust integration comes
LATER, after the crates exist.)

> Strategy per package = one of: **rebuild** (tractable 1:1) · **build primitives** (port the
> hot numerical kernel as Layer-A, the full stat engine is too coupled) · **adopt** (a Rust/Py
> equivalent exists). Same-op-across-ecosystems (scanpy↔Seurat) → one canonical Rust crate.

## Ranges / annotation infrastructure — the most-downloaded packages

These are *infrastructure* (containers + interval algebra), not analysis. We rebuild our own
equivalents rather than FFI-wrap.

| pkg | 2025 dl | role | our crate |
|---|---|---|---|
| IRanges | 2.61M (#4) | integer range algebra (400+ deps) | `rsomics-intervals` ✓ |
| Biostrings | 2.18M (#9) | DNA/RNA/AA containers + pattern match | `rsomics-seqio`+`rsomics-kmer` (partial) |
| GenomicRanges | 2.08M (#10) | GRanges (interval+strand+meta) | gap (intervals covers primitives) |
| AnnotationDbi | 1.67M (#16) | SQLite gene-ID/GO/pathway lookup | gap (value = the prebuilt org.* DBs, not the query layer) |
| GenomicAlignments | 0.93M (#35) | BAM read counting / summarizeOverlaps | `rsomics-bamio`+`featurecounts`+`peak-count` (partial) |
| rtracklayer | 0.87M (#40) | BED/GFF/bigWig/WIG/2bit IO | `rsomics-bed`, `rsomics-annotation`, and private `rsomics-signal` BBI modules |
| GenomicFeatures | 0.65M (#47) | TxDb (GTF/GFF→transcript model) | **gap — prime target** (every DE pipeline uses it) |
| BSgenome | 0.49M (#57) | reference-genome 2bit infra | gap (getfasta covers main use) |

## Bulk RNA-seq differential expression

| pkg | 2025 dl | core | strategy | our crate |
|---|---|---|---|---|
| limma | 1.27M (#18) | lmFit/eBayes/voom (empirical Bayes) | stateful product reconstruction | 16 historical assets → `rsomics-limma` |
| DESeq2 | 1.05M (#21) | DESeq/results/lfcShrink (NB-GLM) | stateful product reconstruction | 12 historical assets → `rsomics-deseq` |
| edgeR | 0.85M (#26) | estimateDisp/glmQLFit (NB QL-F) | stateful product reconstruction | 17 historical assets → `rsomics-edger` |
| fgsea | 0.78M (#24) | fast GSEA permutation | **REBUILD (MIT, Tier-1 target)** | gap |
| clusterProfiler | 0.64M (#31) | enrichGO/GSEA (hypergeometric ORA) | primitives + adopt data layer | gap |
| apeglm | 0.15M | Bayesian LFC shrinkage | primitives | gap |
| tximport | 0.14M | transcript→gene aggregation | **REBUILD (LGPL, I/O+aggregation)** | gap; `rsomics-count normalize` covers only declared gene-count arithmetic |
| DEXSeq | 0.08M | per-exon differential usage (NB-GLM) | primitives (GPL-3) | gap |

DESeq2, edgeR, and limma are not scalar-kernel ports. Each reconstruction must
preserve its fitted state, design and contrast semantics, convergence,
diagnostics, and versioned results. Policy-free numerical kernels move to
`rsomics-stats` only after two product consumers prove the same contract; no
new `rsomics-glm-nb` package is planned.

## Single-cell — scanpy(Py) ↔ Seurat(R), the clearest cross-ecosystem dedup

Nearly every op exists in BOTH scanpy and Seurat (normalize, HVG, PCA, neighbors, leiden,
UMAP, marker DE) → **one canonical Rust crate per op serves both**.

| pkg | 2025 dl | core | our crate |
|---|---|---|---|
| Seurat (CRAN) | dominant | Normalize/HVG/ScaleData/RunPCA/FindNeighbors/FindClusters/RunUMAP/FindMarkers | adopt KNN/graph primitive (hnsw-rs ①); don't 1:1 port |
| SingleCellExperiment | 0.65M (#44) | SCE container | **adopt anndata-rs** (project decision) |
| scran | 0.21M | computeSumFactors (pooling deconvolution), modelGeneVar HVG | primitives (GPL-3) |
| scater | ~0.1M | perCellQCMetrics, runPCA/UMAP | `rsomics-cell-filter`+`barcode-rank` (partial) |

Shared op set (both ecosystems) → canonical crates: normalize, HVG, PCA, neighbors-graph,
leiden, UMAP, marker-DE. (PCA/UMAP/leiden also overlap general ML.) See 04-single-cell.md.

## ChIP/ATAC + methylation R

DiffBind (0.054M) dba.count→DESeq2/edgeR — count layer `rsomics-peak-count` ✓, DE=primitives.
ChIPseeker (0.091M) annotatePeak = interval overlap → `rsomics-bed-annotate` covers core; needs
TxDb + distance-to-TSS. csaw sliding-window counting = rebuild count layer. methylKit (0.046M)
calculateDiffMeth = per-CpG logistic regression (parallelisable; `rsomics-methyldackel` does
calling). bsseq (0.061M) BSmooth = local-likelihood smoothing over sorted CpGs (**Tier-1
rebuild — embarrassingly parallel**). minfi (0.11M) = Illumina array (IDAT) — lower priority
(declining vs WGBS).

## Phylo / popgen / microbiome R
ape/phangorn/pegas/adegenet → see 08-phylogenetics-popgen.md. phyloseq/vegan/microbiome (vegan
HIGH-conf, ~80 diversity/ordination/PERMANOVA fns) → see 06-metagenomics.md (`rsomics-diversity`/
`ordinate`/`permanova`).

## Stats / multiple testing (used everywhere)
qvalue (Bioc #36, 0.57M) Storey FDR π₀ → add π₀ to `rsomics-pvalue-adjust`. base R p.adjust
(BH/BY/Holm/Hochberg/Hommel/Bonferroni) → `rsomics-pvalue-adjust` v0.2.0 ✓ **DONE**.

## Ranked top R rewrite targets

**Tier-1 (direct rebuild, high impact, feasible now):**
1. **fgsea** — MIT, permutation GSEA, SIMD ranked-sum; no crate.
2. **tximport/tximeta** — LGPL, TSV parsing plus length-weighted transcript-to-gene aggregation; keep separate from simple count-matrix normalization until a coherent consumer exists.
3. **bsseq::BSmooth** — Artistic-2.0, parallel sliding-window methylation smoothing.
4. **ChIPseeker peak annotation** — `rsomics-bed-annotate` close; add TxDb + dist-to-TSS.
5. **GenomicFeatures/TxDb** — no crate; GTF→transcript-model DB; every DE pipeline needs it.
6. **methylKit differential methylation** — per-CpG logistic regression, parallelisable.

**Tier-2 (stateful product reconstruction):**
7. DESeq2/edgeR/limma → complete `rsomics-deseq`, `rsomics-edger`, and
   `rsomics-limma` workflows, with shared kernels only after consumer evidence.
8. clusterProfiler → hypergeometric ORA CLI (needs GO/KEGG DB access — the hard part).
9. scran computeSumFactors → pooling-deconvolution linear solve (needs SCE container first).
10. phangorn ML → Felsenstein pruning + BFGS kernel (under rsomics-phylo).

**Tier-3 (adopt/defer):** Seurat (too entangled — adopt KNN primitive), SingleCellExperiment
(anndata-rs), AnnotationDbi/BSgenome (value=the data, not the code), minfi (array, declining),
phyloseq (AGPL container+ordination), dada2 full port (intricate — piecemeal), ape/pegas
(NJ done, ML lower-priority).

## Verification notes
HIGH (Bioc page + dl stats fetched): all Bioconductor packages above. MEDIUM (CRAN, no Bioc
dl rank): Seurat, ape, phangorn, pegas, adegenet — "widely used" from citations/GitHub stars.
**Caveat**: every package's dl roughly *doubled* 2024→2025 — likely a Bioconductor counting
methodology change, not real usage doubling; relative ranks within 2025 still reliable. limma
"21yr"/edgeR "17yr" ages from package-page text, not cross-checked vs release archives.
