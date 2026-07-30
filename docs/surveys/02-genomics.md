# Survey: genomics domain (alignment, variants, popgen-on-genotypes)

Initial survey verified 2026-05-30; the PLINK and cross-tool routing sections
were reverified 2026-07-31. Source provenance per tool is in the verification
notes at the end.

## Aligners — bwa / bowtie2 / minimap2

| op | tool | our crate | status |
|---|---|---|---|
| index + mem / aln·samse·sampe / bwasw | bwa | — | gap (GPL clean-room; P2+) |
| build + align (e2e/local, presets, PE) + inspect | bowtie2 | — | gap (P2+) |
| index (-d) + align (all `-x` presets are modes of one binary) + paftools | minimap2 | `rsomics-minimap2` | ✓ **adopt ② FFI** (all presets covered; paftools gap) |

Aligners are the heaviest GPL clean-room rebuilds — deferred to a later phase.
minimap2 is adopted as an FFI wrapper (Quadrant ②); bwa/bowtie2 remain gaps. The
three index builders (bwa index / bowtie2-build / minimap2 -d) are **distinct
algorithms**, not dedup candidates — one crate each if/when built.

## GATK4 — commonly-used tools → mostly gap

| op | our crate | status |
|---|---|---|
| MarkDuplicates | `rsomics-bam-markdup` | ✓ (= picard/samtools markdup, one canonical) |
| AddOrReplaceReadGroups | `rsomics-bam-addreplacerg` | ✓ |
| CreateSequenceDictionary | `rsomics-bam-dict` | ✓ |
| ValidateSamFile | `rsomics-bam-quickcheck` | partial |
| VariantFiltration / SelectVariants | `rsomics-vcf-filter` / `rsomics-vcf-view`+`vcf-sample` | partial |
| FastqToSam·SamToFastq·MergeBamAlignment·MarkIlluminaAdapters | — | gap (uBAM pre-processing) |
| BaseRecalibrator·ApplyBQSR·AnalyzeCovariates | — | gap (BQSR) |
| HaplotypeCaller·CombineGVCFs·GenomicsDBImport·GenotypeGVCFs·SplitNCigarReads | — | gap (germline calling) |
| Mutect2·FilterMutectCalls·GetPileupSummaries·CalculateContamination | — | gap (somatic) |
| VariantRecalibrator·ApplyVQSR·CNNScoreVariants·Funcotator | — | gap (VQSR / annotation) |

GATK germline/somatic calling is a large greenfield area; only the picard-derived
BAM-manipulation tools are covered. BQSR + HaplotypeCaller + Mutect2 are the high-value
gaps. (GATK docs return 403; tool existence cross-checked vs GitHub javadoc + NIH-HPC
+ bioinformatics-workbook — see notes.)

## PLINK 1.9 / 2.0 — genotype-analysis product

The 2026-07-31 joint dossier maps 31 historical implementations into one
`rsomics-plink` product. The source pool covers BED parsing, counts, missingness,
HWE, heterozygosity, sex and Mendel QC, LD/pruning/blocks, relationship
matrices, KING, PCA, PLINK 1 association modes, TDT, ROH, scoring, and selected
legacy reports.

This is implementation coverage, not product readiness. The historical crate
named `rsomics-pgen` reads PLINK 1 `.bed/.bim/.fam`; it does not read or write
PLINK 2 `.pgen/.pvar/.psam`. Readers, formatters, CLI policy, and numerical
helpers are repeated across the micro-crates, and most operations implement a
small PLINK 1 subset.

The default target is current PLINK 2, with explicitly named PLINK 1 profiles.
The first complete slice is PGEN/BED/VCF input, shared filters, core QC,
unphased LD, and LD pruning. General `--glm`, relationship/PCA, pedigree,
scoring, broad conversion, and legacy analyses follow only as complete
feature-gated slices. See
[`genotype-popgen.md`](../10-products/genotype-popgen.md#rsomics-plink).

## Structural-variant callers — zero coverage

Manta · DELLY (call/merge/genotype/filter/cnv/classify/lr) · LUMPY — all emit VCF 4.x,
detect overlapping SV classes via split-read + discordant-pair (+ read-depth) evidence.
Partition is **by caller algorithm**, not by SV type. Entirely greenfield.

## Cross-tool dedup signals (genomics)

The input format does not own an analysis. PLINK, vcftools, and scikit-allel
overlaps route to one product:

| op | upstreams | canonical / note |
|---|---|---|
| LD r² and pruning | PLINK 1/2, vcftools, scikit-allel | `rsomics-plink ld`; VCF and matrix are input adapters |
| allele/genotype counts, missingness, HWE, heterozygosity | PLINK 1/2, vcftools | `rsomics-plink stats` |
| PCA, relationship and genotype association | PLINK 1/2 | `rsomics-plink` product modules |
| FST | PLINK 2, vcftools, scikit-allel | `rsomics-popgen fst`; PLINK-compatible report consumes a shared kernel only after two consumer tests |
| π, Dxy, SFS, Tajima's D | vcftools, scikit-allel | `rsomics-popgen diversity` and `sfs` |
| ROH | PLINK 1, vcftools, bcftools | genotype workflow in `rsomics-plink`; the HMM/source assets are reviewed before choosing the retained method |
| MarkDuplicates | gatk = picard = samtools markdup | `rsomics-bam-markdup` ✓ no gap |
| PRS score and clump | PLINK 1/2 | later `rsomics-plink score` / association postprocessing slices |

`rsomics-vcf-popgen`, `rsomics-vcf-hardy`, `rsomics-vcf-ld-prune`, and the
PLINK micro-crates remain source assets. Their repositories are not revived.

## Verification notes
- **VERY HIGH** (primary man page / docs read directly): PLINK1.9 (cog-genomics
  basic_stats/ld/assoc/data/filter/ibd/strat/cnv/family), PLINK2.0 (basic_stats/ld/
  assoc/distance/score/input/formats), bwa (`bwa.1`), bowtie2 (manual.shtml+README),
  minimap2 (`minimap2.1`), vcftools (`vcftools.1` full man page), Manta/DELLY/LUMPY READMEs.
- **MEDIUM-HIGH** (GitHub source + cross-ref; GATK docs 403): GATK4 tools — existence
  confirmed via javadoc + NIH-HPC + bioinformatics-workbook; flag-level detail for
  FilterMutectCalls/GetPileupSummaries/CalculateContamination/SplitNCigarReads needs
  re-verify before a crate spec.
- **Excluded as unverified**: bwa `fastmap`, plink2 `--admix`/`--pca-approx`/`--pca-biplot`
  (not found in fetched pages) — do not implement without re-verification.
