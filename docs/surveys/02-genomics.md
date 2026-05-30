# Survey: genomics domain (alignment, variants, popgen-on-genotypes)

Verified 2026-05-30. Source provenance per tool in the verification notes at the
end. Status legend: ✓ canonical crate · partial (op exists, flags/inputs missing)
· gap (no crate) · adopt (FFI/Rust upstream, don't rebuild).

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

## PLINK 1.9 / 2.0 — genotype-level stats

Covered: `--r/--r2` → `rsomics-plink-ld` ✓; `--indep-pairwise` → `rsomics-plink-prune` ✓;
`--pca` (+ GRM) → `rsomics-plink-pca` ✓; `--assoc/--fisher` → `rsomics-plink-assoc` ✓;
`.bed/.pgen` read → `rsomics-pgen`/`rsomics-plink-io` ✓ (read only).

Large gap set (all `gap` unless noted): **data mgmt** — make-bed/pgen, recode/export
(15+ formats), merge/bmerge/pmerge, flip, update-{ids,map,alleles}, split-x, normalize,
set-var-ids, rm-dup. **filtering** — keep/remove, extract/exclude, chr/from-bp/to-bp,
geno, mind, maf/mac, hwe, thin, snps-only, king-cutoff. **stats/QC** — freq, missing,
het, ibc, hardy, mendel, check-sex, geno-counts, sample-counts. **LD** — indep,
indep-pairphase, blocks, show-tags, r-phased, ld, r2-phased. **popgen/distance** —
fst, cluster, mds-plot, distance, make-grm(sparse), make-king(-table), genome (IBD),
rel-cutoff, homozyg (ROH). **assoc** — model, logistic, linear, glm, mh/cmh, gxe,
tdt/qfam/dfam, adjust. **scoring** — score(-list), q-score-range, vscore, clump.

The genotype-stats layer is *one tool deep* today (LD, prune, PCA, assoc); the bulk of
PLINK is unbuilt. Highest-value next crates: `--glm` (GWAS), `--score` (PRS),
`--make-king` (kinship), `--freq/--missing/--hardy/--het` (QC battery), `--make-bed/pgen`
+ `--export vcf` (format bridge).

## Structural-variant callers — zero coverage

Manta · DELLY (call/merge/genotype/filter/cnv/classify/lr) · LUMPY — all emit VCF 4.x,
detect overlapping SV classes via split-read + discordant-pair (+ read-depth) evidence.
Partition is **by caller algorithm**, not by SV type. Entirely greenfield.

## Cross-tool dedup signals (genomics)

Same op across PLINK1 / PLINK2 / vcftools — one canonical implementation, others
depend or are input-format variants:

| op | upstreams | canonical / note |
|---|---|---|
| LD r² | plink1 --r2, plink2 --r2-unphased, vcftools --geno-r2 | `rsomics-plink-ld` (PLINK in); VCF-native r² = gap |
| LD prune | plink1+plink2 --indep-pairwise | `rsomics-plink-prune` (PLINK in); .pgen in = gap |
| allele freq / missing / het / hardy / fst | plink1+plink2+vcftools | `rsomics-vcf-popgen` (VCF in); PLINK-native in = gap |
| PCA | plink1+plink2 --pca | `rsomics-plink-pca` (PLINK in); VCF/matrix PCA = gap |
| Tajima's D | vcftools --TajimaD | `rsomics-tajima-d` (SFS) + `vcf-popgen` (VCF) — complementary inputs, not dup |
| ROH | plink1 --homozyg, vcftools --LROH, bcftools roh | `rsomics-vcf-roh` (bcftools algo, VCF); PLINK .bed in = gap |
| MarkDuplicates | gatk = picard = samtools markdup | `rsomics-bam-markdup` ✓ no gap |
| PRS score | plink1+plink2 --score | `rsomics-plink-score` = gap |
| clump | plink1+plink2 --clump | `rsomics-plink-clump` = gap |

The recurring pattern: we cover the **VCF-input** flavor of popgen stats via
`rsomics-vcf-popgen`, and the **PLINK-input** flavor only for LD/prune/PCA/assoc. A
PLINK-native stats battery (freq/missing/het/hardy/fst on .bed/.pgen) is the cleanest
high-leverage gap — same algorithms, different reader.

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
