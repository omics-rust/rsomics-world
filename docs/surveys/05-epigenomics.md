# Survey: epigenomics domain (ChIP/ATAC/CUT&RUN/methylation)

Verified 2026-05-30 against deeptools.readthedocs.io, MACS3 source+docs, MethylDackel main.c,
SEACR README, Bismark/HOMER docs, Bioconductor pages, pyBigWig/CrossMap. Status from live FS.

> **⚠ Build-out integrity flag:** none of the ~15 epigenomics crates is published to crates.io
> and **none has a `perf-*.md` record** — all are pre-release on KIOXIA, most are *partial*
> implementations. Per the DONE gate (perfgate >1.0× + compat + CI + publish), this whole
> domain is NOT DONE despite the crates existing. This is the #24 build-out backlog, surfaced.

## deepTools (20 tools) → 8 crates, all partial

| tool | crate | status |
|---|---|---|
| bamCoverage | rsomics-bam-signal | partial — bedGraph only (no bigWig); missing extendReads/smoothLength/region/blacklist/filterRNAstrand |
| bamCompare | rsomics-bam-compare | partial — bedGraph only; no SES scaling/normalizeUsing |
| bamPEFragmentSize | rsomics-fragment-size | partial — full-scan (matches samtools/picard, NOT deeptools sampling) |
| alignmentSieve | rsomics-atac-shift | partial — only --ATACshift mode; general MAPQ/flag/fraglen/BEDPE filtering = gap |
| plotFingerprint | rsomics-bam-fingerprint | partial — no Jensen-Shannon/CHANCE/JSDsample |
| multiBamSummary | rsomics-multibam-summary | largely complete (bins+BED) |
| multiBigwigSummary | rsomics-multibigwig-summary | largely complete (bins+BED) |
| bigwigCompare | rsomics-bigwig-compare | partial — bedGraph out only |
| computeMatrix | rsomics-compute-matrix | partial — single-bigWig/single-BED only |
| computeGCBias / correctGCBias | — | gap |
| estimateReadFiltering / plotEnrichment / plotCoverage | — | gap |
| bigwigAverage | — | gap |
| computeMatrixOperations | — | gap |
| plotHeatmap/Profile/Correlation/PCA | — | gap (visualization; numerical export could be a flag) |

## MACS3 (14 subcommands) → 0 implemented; `rsomics-macs` is P0

callpeak·hmmratac·bdgpeakcall·bdgbroadcall·refinepeak (peak calling); pileup·bdgcmp·bdgopt·
bdgdiff·cmbreps (bedGraph ops); filterdup·predictd·randsample·callvar (pre/post). All gap.
MACS3 pileup is a distinct "raw, no-normalization, BED-input" coverage niche vs bam-signal.

## SEACR → ✓ COMPLETE

`rsomics-seacr` v0.1.0 — all 6 mode combos (stringent/relaxed × norm/non) byte-identical to
SEACR_1.3.sh, compat verified. (Still needs perfgate+publish to be fully DONE.)

## MethylDackel (4 subcommands) → 1 partial

extract → `rsomics-methyldackel` (CpG bedGraph only; CHG/CHH, bigWig, --MBias, --perRead,
strand filtering OT/OB/CTOT/CTOB, --mergeContext, --methylKit all gap). mbias/mergeContext/
perRead = gap.

## Bismark (GPL — clean-room) → all gap

genome_preparation·bismark(aligner)·methylation_extractor·deduplicate·bismark2bedGraph·
coverage2cytosine·bismark2report. The **extraction** step overlaps `rsomics-methyldackel`
(any bisulfite BAM → CpG); the **alignment** step (bismark proper, needs rsomics-align) is the
missing piece.

## HOMER → all gap (MEDIUM confidence, SSL errors on docs)

findMotifsGenome (motif enrichment — NOT the same as `rsomics-motif-scan` IUPAC scanning),
annotatePeaks (gene/context annotation — NOT `rsomics-bed-annotate` which is bedtools-annotate
overlap-fractions), makeTagDirectory, findPeaks, hicFindPeaks.

## R targets (deep-dive in 10-r-bioconductor.md)

ChIPseeker (annotatePeak), DiffBind (dba.count/contrast/analyze → DESeq2/edgeR backend),
csaw (windowCounts sliding-window), methylKit (calculateDiffMeth DMR), bsseq (BSmooth
smoothing), minfi (Illumina array — array-specific, lower priority than bsseq/methylKit).

## Python: pyBigWig (read = `rsomics-bbi` ✓ / write = gap, blocked) · CrossMap (liftover = gap → future `rsomics-liftover`, domain-agnostic).

## Cross-tool dedup signals (epigenomics)

- **coverage/signal** (NOT duplicates — distinct outputs): deeptools bamCoverage
  (`bam-signal`, normalized binned) vs MACS3 pileup (raw, gap) vs bedtools genomecov
  (`bed-genomecov`) vs samtools depth (`bam-depth`, per-base).
- **fragment size**: deeptools bamPEFragmentSize (sampling) vs samtools/picard (full-scan).
  `rsomics-fragment-size` matches the full-scan oracle — a deliberate compat divergence from
  deeptools.
- **peak annotation**: HOMER annotatePeaks (CLI, gap) vs ChIPseeker (R, gap) — equivalent,
  different ecosystem; a `rsomics-peak-annotate` would cover the CLI side. ≠ `rsomics-bed-annotate`.
- **dedup**: MACS3 filterdup (BAM→BED/BEDPE, distinct output) vs samtools/picard markdup
  (BAM→BAM, `rsomics-bam-markdup`) — not true duplicates.
- **methylation extraction**: MethylDackel extract (`rsomics-methyldackel`) vs Bismark
  extractor (gap) vs methylKit processBismarkAln (R) — equivalent purpose, different output
  formats; pipelines pin one.

## High-value new crates (prioritized)
`rsomics-macs` (P0, all 14 subcommands — pure algorithm) · `rsomics-compute-gc-bias` ·
`rsomics-bigwig-average` · `rsomics-peak-annotate` (HOMER/ChIPseeker CLI) ·
`rsomics-liftover` (CrossMap) · `rsomics-alignment-sieve` (extend atac-shift).
Plus: complete the 4 partial crates (bam-signal, methyldackel, compute-matrix, fingerprint).

## Verification notes
HIGH: deeptools (all 20, live per-tool docs), MACS3 (subcommands from source), SEACR/MethylDackel
(source-level), Bioconductor pages, pyBigWig, CrossMap. MEDIUM: Bismark (docs nav, GPL no-source),
HOMER (SSL errors — cached+partial), MACS3 bdgcmp exact mode names, deeptools computeMatrixOperations
sub-op behaviour. Unconfirmed: rsomics-bbi bigBed read scope (only bigWig verified).
