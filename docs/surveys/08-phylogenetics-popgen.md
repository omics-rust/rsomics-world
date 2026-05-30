# Survey: phylogenetics / population genetics domain

Verified 2026-05-30 against tool docs + repo `docs/08-*` planning + live crate sources.

## Tree inference (IQ-TREE2 GPL / RAxML-NG AGPL / FastTree — clean-room)

| op | crate | status |
|---|---|---|
| Newick/NEXUS parse+emit | `rsomics-phylo-tree` (Layer A) | ✓ pure-Rust |
| NJ from distance matrix | `rsomics-nj-tree` | ✓ (TSV → Newick) |
| IQ-TREE2 ML + ModelFinder(-m MFP) + UFBoot(-B) + concordance(--gcf/scf) + topology tests | `rsomics-iqtree` (planned) | gap (canonical ML target — dominant tool) |
| RAxML-NG ML search + bootstrap | `rsomics-raxml` (planned) | gap (HPC-scale; separate crate from iqtree) |
| FastTree2 | — | skip (superseded by iqtree --fast) |
| ASTRAL/ASTER species tree | `rsomics-aster` (planned) | gap |
| UShER placement | adopt subprocess | adopt (MIT, too specialized) |
| MrBayes/BEAST2 MCMC | — | out of scope |

## MSA (MAFFT BSD / MUSCLE GPL / KAlign BSD / FAMSA GPL / trimAl)

| op | crate | status |
|---|---|---|
| trimAl -gt (gap-fraction trim) | `rsomics-msa-trim` | ✓ (only -gt mode) |
| trimAl -automated1/-strict/similarity; Gblocks | — | gap (other trim modes) |
| KAlign3 (LCS linear-time) | `rsomics-kalign` (planned P0) | gap (best pure-Rust target, ~6k LOC BSD-2) |
| MAFFT (FFT-NS / G-INS-i / L-INS-i) | `rsomics-mafft` (planned) | gap (multi-month) |
| MUSCLE5 / FAMSA (SIMD ultra-scale) | `rsomics-famsa` (planned) | gap |
| Clustal Omega | — | skip (MAFFT/FAMSA cover better) |

## Popgen stats — vcftools / bcftools / PLINK (input-format split)

VCF-input via `rsomics-vcf-popgen`: ✓ pi (--window-pi), het, hardy, missing-site/indv, freq,
singleton. PLINK-input via `rsomics-plink-io`: ✓ freq, missing, hardy. Tajima's D →
`rsomics-tajima-d` (SFS) + `rsomics-popgen-core` (Layer A `tajimas_d`/`hwe_exact`). PLINK
genotype tools: `rsomics-plink-pca` (--pca+GRM) ✓, `rsomics-plink-prune` (--indep-pairwise) ✓,
`rsomics-plink-ld` (--r2) ✓, `rsomics-plink-assoc` (--assoc chi-sq + --glm linear) ✓.
`rsomics-vcf-roh` (--LROH/bcftools roh, HMM) ✓. `rsomics-ld-matrix` (generic dosage-TSV r²) ✓.

> **⚠ Finding: FST gap + Cargo.toml mismatch.** `rsomics-vcf-popgen`'s Cargo.toml description
> mentions "Fst" but **no fst module exists in source**. FST is in vcftools (--weir-fst-pop),
> bcftools, and PLINK2 (--fst) but unimplemented anywhere in rsomics. → add `weir_cockerham_fst`
> to `rsomics-popgen-core`, surface as a vcf-popgen subcommand + a plink-fst path. (Doc/code
> mismatch, not a wrong-output bug — fix during build-out, not a halt.)

Other popgen gaps: **relatedness/kinship missing entirely** (vcftools --relatedness, PLINK2
--make-king, PLINK --genome IBD) → `rsomics-plink-king` would cover all three. PLINK het
(per-sample), --ibc, logistic --glm, PLINK2 .pgen input all gap. VCF-native LD r² gap.

## ADMIXTURE / STRUCTURE → all gap
ADMIXTURE (block-relaxation EM, clean-room) + fastSTRUCTURE (VB) → `rsomics-admixture` (planned).
STRUCTURE (MCMC) out of scope (ADMIXTURE supersedes).

## R packages (clean-room; deep-dive 10-r-bioconductor.md)
ape (read/write.tree, dist.dna, nj/bionj, ace) — Newick IO + NJ covered; ace/comparative gap.
phangorn (pml ML, parsimony) — overlaps rsomics-iqtree. pegas (tajima.test, Fu.Li.D, Fs,
nucdiv) — overlaps tajima-d/vcf-popgen. adegenet (dapc, find.clusters, pca) — pca overlaps
plink-pca; dapc gap.

## Cross-tool dedup summary
| stat | vcftools | bcftools | PLINK | canonical |
|---|---|---|---|---|
| FST | --weir-fst-pop | stats | --fst | **GAP everywhere** → add to popgen-core |
| pi / Tajima D | --window-pi / --TajimaD | — | — | vcf-popgen / tajima-d ✓ |
| HWE / het | --hardy / --het | +HWE | --hardy / --het | split by input fmt (PLINK het gap) |
| LD r² / prune / PCA | — | — | --r2 / --indep-pairwise / --pca | plink-ld / -prune / -pca ✓ (VCF LD gap) |
| ROH | --LROH | roh | --homozyg | vcf-roh ✓ |
| relatedness | --relatedness | — | --make-king/--genome | **GAP everywhere** → plink-king |
| NJ tree | — | — | — | nj-tree ✓ (ape::nj is R-only) |

## Verification notes
crate inventory from live FS + REGISTRY.md. Upstream ops from maintained docs +
verified `docs/08-*` planning files. FST-absence confirmed by direct source inspection of
vcf-popgen and all plink crates. Tree-inference / MSA / ADMIXTURE marked planned-gap per
`docs/08` — multi-month efforts, not started.
