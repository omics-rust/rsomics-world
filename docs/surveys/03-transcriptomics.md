# Survey: transcriptomics

Verified 2026-07-31 against the RSeQC 5.0.5 wheel, RSeQC documentation,
featureCounts 2.1.1, Picard 3.4.0, and the public Salmon, kallisto, STAR, and
HISAT2 documentation.

This survey records upstream workflow coverage. Historical crates are source
assets, not the target public layout. Product contracts and exact asset
revisions are in the
[RNA-seq QC dossier](../10-products/rnaseq-qc-signal.md#rsomics-rnaseq-qc) and
[count dossier](../10-products/count.md).

## RSeQC 5.0.5

The current wheel ships 33 scripts. They partition into five coherent product
workflows:

| Upstream operations | Target | Current source evidence |
|---|---|---:|
| mapping, strandedness, distribution, coverage, junction, fragment, bias, duplication, saturation, and TIN metrics | `rsomics-rnaseq-qc` | 21 candidates for 20 RSeQC/Picard metric operations |
| `FPKM_count`, `FPKM-UQ` | `rsomics-count` | two normalization/counting assets |
| `bam2fq`, `divide_bam`, `split_bam`, `split_paired_bam` | `rsomics-bam` | three split assets plus existing FASTQ conversion |
| `bam2wig`, `normalize_bigwig`, `overlay_bigwig` | `rsomics-signal` | signal-generation and arithmetic assets |
| four `sc_*` scripts | `rsomics-sc` | single-cell source pool |

`geneBody_coverage2` is the multi-input form of gene-body coverage, not a
separate product operation. Two historical implementations target
`read_distribution`; they must be reconciled against 5.0.5 rather than both
merged.

The prior survey treated each script as a nearly finished product. That status
was incorrect. The historical implementations do preserve useful algorithms,
fixtures, compatibility tests, and benchmark seeds, but they repeat BAM and
annotation plumbing, expose inconsistent filters, write non-transactional
output sets, and lack four-native-target release evidence.

## Picard RNA-seq metrics

Picard 3.4.0 remains a second oracle where it provides a stronger or
complementary contract.

| Picard operation | Target decision |
|---|---|
| `CollectRnaSeqMetrics` | `rsomics-rnaseq-qc mapping`, `distribution`, and `coverage` |
| `CollectBaseDistributionByCycle` | deduplicate with RNA-seq QC nucleotide-cycle bias |
| `MeanQualityByCycle` | deduplicate with RNA-seq QC quality-cycle bias |
| `CollectInsertSizeMetrics` | library fragment evidence; RNA-aware distance remains explicit |
| `CollectAlignmentSummaryMetrics` | BAM format statistics plus RNA-seq report fields |
| `EstimateLibraryComplexity` | later RNA-seq QC duplication slice |
| `CollectGcBiasMetrics` | later RNA-seq QC bias slice, distinct from reference GC windows |
| `CollectMultipleMetrics` | report planning behavior, not another public operation |

## Counting and count normalization

featureCounts is the primary alignment-based feature-counting oracle.
`rsomics-count` owns the multi-input assignment matrix, summary, optional
assignment evidence, and length-aware TPM/FPKM/FPKM-UQ normalization of
existing gene-count matrices.

This does not make `rsomics-count` a Salmon, kallisto, or RSEM replacement.
Those tools infer transcript abundance using compatibility classes,
effective-length corrections, and model-specific assumptions that are absent
from count-matrix arithmetic.

## Quantifiers and aligners

Salmon, kallisto, oarfish, and piscem remain adoption or future integration
decisions rather than reasons to publish thin wrappers. STAR and HISAT2 remain
deferred because their index construction, spliced-alignment behavior, memory
model, and performance gate require a dedicated product decision.

Alternative-splicing workflows such as StringTie, rMATS, SUPPA, and LeafCutter
are not hidden inside RNA-seq QC. They require their own demonstrated coherent
workflow before entering the public allowlist.

## Cross-product overlap decisions

| Capability | Decision |
|---|---|
| strandedness inference | RNA-seq QC report operation; quantifier auto-detection remains internal to the quantifier |
| inner or insert distance | RNA-seq QC keeps RNA-aware and library-level modes with explicit semantics |
| duplication | reporting belongs to RNA-seq QC; marking or removing alignments belongs to BAM |
| gene counts | `rsomics-count`; STAR or quantifier counts remain outputs of those products |
| TPM/FPKM/FPKM-UQ from counts | `rsomics-count normalize` |
| BAM partition or mate split | `rsomics-bam split` |
| BAM to bigWig and bigWig arithmetic | `rsomics-signal` |
| single-cell read QC | `rsomics-sc` |

## Evidence state

The survey is complete at the operation and routing level. No transcriptomics
product is declared release-ready by this document. Correctness, representative
performance and memory, exact-head four-native-platform CI, public API review,
and coherent product integration remain release gates.
