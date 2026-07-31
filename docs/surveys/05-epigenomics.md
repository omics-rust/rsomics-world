# Survey: epigenomics and genome signal

Verified 2026-07-31 against deepTools 3.5.6, MACS3 documentation, SEACR 1.3,
MethylDackel, Bismark, HOMER, pyBigWig/bigtools, and the relevant Bioconductor
package documentation.

This survey records upstream workflows rather than proposing one crate per
command. Exact product contracts and historical revisions are in the
[signal](../10-products/rnaseq-qc-signal.md#rsomics-signal),
[peak](../10-products/peak.md), [methyl](../10-products/methyl.md), and
[liftover](../10-products/liftover.md) dossiers.

## deepTools 3.5.6

deepTools exposes 20 user tools. They form one coherent `rsomics-signal`
product:

| Workflow | Upstream tools | Target operations |
|---|---|---|
| track generation and arithmetic | `bamCoverage`, `bamCompare`, `bigwigCompare`, `bigwigAverage` | `track`, `compare`, `average` |
| summaries and matrices | `multiBamSummary`, `multiBigwigSummary`, `computeMatrix`, `computeMatrixOperations` | `summarize`, `matrix`, `matrix-ops` |
| alignment preparation | `alignmentSieve`, `correctGCBias` | `filter`, `gc-bias correct` |
| alignment QC | `plotFingerprint`, `bamPEFragmentSize`, `computeGCBias`, `plotCoverage`, `estimateReadFiltering`, `plotEnrichment` | named QC operations |
| matrix analysis and views | `plotCorrelation`, `plotPCA`, `plotHeatmap`, `plotProfile` | `correlate`, `pca`, `heatmap`, `profile` |

Fifteen historical Layer B candidates cover parts of this surface. They are
implementation and evidence assets, not 15 planned binaries. Important gaps
remain:

- several track operations are bedGraph-only where bigWig is the normal
  contract;
- filters, fragments, smoothing, blacklists, regions, strand, scaling, SES,
  missing data, and normalization are incomplete;
- the matrix implementation is single-sample and single-region-group;
- correlation, PCA, heatmap, profile, enrichment, matrix operations, and
  filtering estimates have no consolidated implementation;
- source-level tests and microbenchmarks do not establish a product release
  gate.

The historical `rsomics-bbi` and `rsomics-coverage-core` libraries each have
only one target product consumer after consolidation. They are internalized
into `rsomics-signal`; multiple subcommands inside one binary do not satisfy
the two-product public-foundation rule.

## Peak calling

`rsomics-peak` owns peak calling, refinement, annotation, and peak
quantification. It uses MACS3, SEACR, ChIPseeker-style annotation, and
bedtools/DiffBind-style counting as behavior sources.

MACS3 helper operations route by user workflow:

| MACS3 operation | Target |
|---|---|
| `callpeak`, `hmmratac`, `bdgpeakcall`, `bdgbroadcall`, `refinepeak`, `predictd` | `rsomics-peak` |
| `pileup`, `bdgcmp`, `bdgdiff`, `bdgopt`, `cmbreps` | public track forms in `rsomics-signal`; peak-model-private forms stay private |
| `filterdup`, `randsample` | `rsomics-bam` for public alignment transformations |
| `callvar` | variant workflow, not peak calling |

`rsomics-seacr` is a strong compatibility asset but does not make the whole
peak product complete. MACS statistical models, input formats, transactional
output sets, annotation, count matrices, and representative performance remain
required.

## Methylation

`rsomics-methyl` owns bisulfite-alignment methylation extraction and its
user-visible reports. The historical MethylDackel-like implementation is a
partial seed: non-CpG contexts, M-bias, per-read output, strand policies,
context merging, methylKit output, and bigWig interoperability remain gated.

Bismark alignment, genome preparation, deduplication, extraction, and report
generation are not represented by thin commands. Alignment remains a separate
large workflow decision; extraction overlap is deduplicated into
`rsomics-methyl`.

## Annotation, liftover, and differential analysis

- HOMER/ChIPseeker peak annotation belongs to `rsomics-peak annotate`, not
  generic interval annotation.
- Chain-based genome-coordinate conversion belongs to `rsomics-liftover`.
- DiffBind/csaw comparison belongs to an evidence-backed differential
  chromatin workflow if one is admitted later; it is not hidden in signal
  plotting.
- methylKit, bsseq, and minfi represent statistical or assay-specific
  workflows beyond the initial methylation extraction product.

## Cross-product overlap decisions

| Capability | Decision |
|---|---|
| normalized BAM coverage track | `rsomics-signal track` |
| raw peak-model pileup | private to `rsomics-peak call` unless exported as a general signal contract |
| per-base BAM depth and coverage summary | `rsomics-bam` |
| ATAC coordinate shift | `rsomics-signal filter --atac-shift` |
| peak calling and refinement | `rsomics-peak` |
| bigWig read/write and matrix access | private `rsomics-signal` modules |
| peak annotation and count matrix | `rsomics-peak` |
| methylation extraction | `rsomics-methyl` |

## Evidence state

The operation survey and product routing are complete. No epigenomics product
is declared release-ready here. Each stable slice still requires pinned-oracle
compatibility, representative CPU/memory/I/O measurements, a strict hot-path
advantage, exact-head CI on the four native platform classes, and public API
review.
