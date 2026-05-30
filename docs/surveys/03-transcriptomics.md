# Survey: transcriptomics domain (RNA-seq QC, quant, counting)

Verified 2026-05-30 against official docs (rseqc.sourceforge.net, subread.sourceforge.net,
broadinstitute.github.io/picard, salmon/kallisto/STAR/HISAT2 manuals). Crate status from
live filesystem + `.autopilot/state/perf-*.md`. This is the **best-covered** domain after
formats — RSeQC is essentially complete.

## RSeQC (34 modules) → mostly ✓ DONE with perfgates

29 of 34 modules have a perfgated crate (PASS recorded). Highlights:

| module | crate | perf |
|---|---|---|
| tin.py | rsomics-tin | 33.4× |
| infer_experiment.py | rsomics-bam-strandedness | 447× |
| read_distribution.py | rsomics-bam-read-dist | PASS |
| junction_annotation.py | rsomics-bam-junctions | PASS |
| RPKM_saturation.py | rsomics-rpkm-saturation | 175× |
| mismatch_profile.py | rsomics-mismatch-profile | 72.8× (19× lower RSS) |
| deletion_profile.py | rsomics-deletion-profile | 10.5× (27.6× lower RSS) |
| geneBody_coverage / inner_distance / insertion·clipping·junction-sat / read_dup·GC·NVC·quality | (each its own crate) | PASS |
| FPKM_count.py | rsomics-fpkm-count | PASS |

**RSeQC gaps (no crate):** FPKM-UQ.py (TCGA UQ-norm, GTF input — distinct from FPKM_count),
RNA_fragment_size.py (per-transcript stats — distinct from inner_distance distribution AND
from ATAC `rsomics-fragment-size`), read_hexamer.py (primer bias), bam_stat.py (RSeQC-flavored
vs samtools flagstat), split_bam.py (exonic/non-exonic/junk by gene list — NOT chromosomal
`rsomics-bam-split`), split_paired_bam.py (mate-separation), divide_bam.py (N-equal subsets),
bam2wig/geneBody_coverage2/normalize_bigwig/overlay_bigwig (**all blocked by bigWig-write gap**),
sc_* (4 single-cell tools — separate domain).

## featureCounts (Subread 2.1.1) → ✓ DONE

Single operation (count reads over features). `rsomics-featurecounts` v0.1.1, **23.7×** vs
featureCounts 2.1.1, compat DONE. Covers -a/-t/-g/-s/-p/-O/-M/-T/-Q/-B/-d/-D/-C.
Gap (PDF-only flags, MEDIUM confidence): --fraction, --fracOverlap, --minOverlap, -f
(exon-level mode) — verify coverage.

## Picard (RNA-seq tools)

| tool | crate | status |
|---|---|---|
| CollectRnaSeqMetrics | rsomics-rnaseq-metrics | ✓ 21.4× vs Picard 3.4.0 |
| CollectBaseDistributionByCycle | rsomics-read-nvc | ✓ (= RSeQC read_NVC) |
| MeanQualityByCycle | rsomics-read-quality | ✓ (= RSeQC read_quality) |
| CollectInsertSizeMetrics | rsomics-inner-distance | partial (no FR/RF/TANDEM orientation, no histogram PDF) |
| MarkDuplicates | rsomics-bam-markdup | overlap (no optical-dup / complexity report) |
| CollectAlignmentSummaryMetrics | rsomics-bam-flagstat/stats | partial (no per-RG breakdown) |
| EstimateLibraryComplexity | — | gap (pre-dedup complexity, distinct from markdup) |
| CollectGcBiasMetrics | — | gap (GC-vs-coverage bias, distinct from `rsomics-gc-windows`) |
| CollectMultipleMetrics | — | gap (meta-tool wrapper) |

## Quantifiers → ADOPT (per project decision)

salmon (index, quant mapping+alignment; alevin REMOVED → alevin-fry) and kallisto (index,
quant, quant-tcc, bus, h5dump, inspect) are **adopt** — oarfish ① + piscem ② already chosen.
No rsomics-salmon/kallisto planned. `rsomics-tpm` (count-matrix → TPM) is complementary, not
a replacement. salmon `quantmerge` unverified in v1.11.4 docs.

## Spliced aligners → gap/defer (P2+)

STAR (genomeGenerate, alignReads, --quantMode GeneCounts/TranscriptomeSAM, 2-pass) and HISAT2
(build, align --dta/--dta-cufflinks, inspect) — no crate; deferred (30GB index, pthreads
architecture). StringTie, rMATS/SUPPA/LeafCutter (alt-splicing) also P2+ gaps.

## Cross-tool dedup signals (transcriptomics)

| op | upstreams | canonical / note |
|---|---|---|
| strandedness | RSeQC infer_experiment + salmon auto | `rsomics-bam-strandedness` (standalone CLI); salmon's is internal |
| insert/inner distance | RSeQC inner_distance + Picard CollectInsertSize | `rsomics-inner-distance`; Picard adds orientation (gap) |
| read duplication | RSeQC read_duplication + Picard/samtools markdup | distinct: read_dup *reports rate*, markdup *marks/removes* — both kept |
| per-cycle NVC | RSeQC read_NVC + Picard BaseDistByCycle | `rsomics-read-nvc` (one canonical) |
| per-cycle quality | RSeQC read_quality + Picard MeanQualByCycle | `rsomics-read-quality` (one canonical) |
| gene counts | featureCounts + HTSeq + STAR --quantMode | `rsomics-featurecounts` (standalone); STAR's is internal |
| transcript TPM | salmon + kallisto + `rsomics-tpm` | distinct: salmon/kallisto align+quant; tpm normalizes existing matrix |

## Verification notes
- **HIGH**: all 34 RSeQC modules (sourceforge per-tool pages), Picard tool existence +
  CollectRnaSeqMetrics/InsertSize args, kallisto (8 subcommands), HISAT2 (manual), salmon
  index/quant.
- **MEDIUM**: featureCounts PDF-only flags (binary PDF unreadable); Picard MarkDuplicates /
  EstimateLibraryComplexity arg lists (HTML truncated); STAR (PDF partial).
- **count-matrix / tpm / deseq-prep / de-volcano** are perf-exempt (no canonical CLI upstream;
  gated vs Python reference) per `.autopilot/state/perf-exempt.md`.
- key distinctions confirmed: `rsomics-fragment-size` (ATAC, deeptools) ≠ RSeQC
  RNA_fragment_size (per-transcript); `rsomics-bam-split` (chromosomal) ≠ RSeQC split_bam
  (by gene list) ≠ split_paired_bam (by mate).
