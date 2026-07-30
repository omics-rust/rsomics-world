# Population genetics

Verified 2026-07-31. The implementation contract is
[`genotype-popgen.md`](../10-products/genotype-popgen.md).

## Retained product boundaries

| Product | Scope |
|---|---|
| `rsomics-plink` | genotype filesets, filters, QC, LD, relationships, PCA, association, pedigree tests, and scoring |
| `rsomics-popgen` | diversity, SFS, population differentiation, admixture statistics, haplotype summaries, and selection scans |
| `rsomics-vcf` | VCF/BCF representation, inspection, transformation, filtering, and indexing |

VCF, PGEN, and BED are input representations. They do not create separate
products for the same scientific operation. In particular:

- VCF HWE and LD pruning route to `rsomics-plink`;
- VCF π, Tajima's D, SFS, FST, Dxy, and selection statistics route to
  `rsomics-popgen`;
- generic VCF statistics and transformations stay in `rsomics-vcf`.

The historical source pool now maps 31 assets primarily to `rsomics-plink` and
14 to `rsomics-popgen`. These are migration inputs, not subcommand counts.

## `rsomics-plink`

The behavior anchor is the current
[PLINK 2 documentation](https://www.cog-genomics.org/plink/2.0/), reviewed at
the 4 May 2026 documentation revision. PLINK 1.9 remains a separately named
legacy profile where its output or algorithm is still useful.

The recognizable upstream workflow covers:

- PGEN/BED/VCF and other genotype inputs, filters, conversion, and metadata;
- allele/genotype counts, missingness, HWE, heterozygosity, Mendel and sex QC;
- LD calculation and pruning;
- relationship matrices, KING, PCA, and structure preparation;
- linear/logistic/Firth association, conditioning, interactions, and
  permutations;
- clumping, scoring, family tests, and selected legacy analyses.

The first release is intentionally narrower: complete PGEN/BED/VCF input,
shared filters, core QC reports, unphased LD, and LD pruning. It does not
expose placeholder regression, relationship, or scoring commands.

### Critical historical finding

`rsomics-pgen` 0.3.1 is not a PGEN implementation. It reads PLINK 1
`.bed/.bim/.fam`, while naming its types `Pgen` and `PgenMmap`. It also has
only one target-product consumer. Its useful code is internalized in
`rsomics-plink` under accurate names; a real PGEN reader/writer is implemented
against the current format specification and PLINK/pgenlib differentials.

The 28 original PLINK micro-crates also repeat BED readers, metadata parsers,
formatters, and statistical helpers. The product reconstruction uses one
typed genotype dataset and operation modules.

## `rsomics-popgen`

The main array-level oracle is
[scikit-allel 1.3.13](https://pypi.org/project/scikit-allel/1.3.13/), with
vcftools 0.1.17 retained only for explicitly named report profiles.

The operation families are:

- neutral diversity and divergence: π, Watterson theta, Tajima's D, Dxy, and
  fixed differences;
- folded and unfolded one- and two-population SFS;
- Hudson and Weir-Cockerham FST and PBS;
- Patterson D, f2, f3, and f4 with block jackknife;
- haplotype diversity and Garud statistics;
- EHH, iHS, nSL, XP-EHH, and XP-nSL;
- windowed LD genome scans.

The first release covers neutral diversity, SFS, and FST with one checked
sample/population model, explicit window and accessibility semantics, and
replayable oracle fixtures. Admixture and selection scans remain gated until
their complete slices pass.

`rsomics-popgen-core` has only one product consumer and is internalized. A
policy-free FST kernel may move to `rsomics-stats` after both
`rsomics-popgen` and `rsomics-plink` exercise the same contract.

## Other upstream families

These are real population-genetics tools, but the current source pool does not
justify adding them to the public allowlist:

| Upstream family | Distinct capability | Current decision |
|---|---|---|
| ADMIXTURE / fastSTRUCTURE | global ancestry-proportion models | excluded from the two first releases; requires its own product dossier and evidence |
| EIGENSOFT | smartPCA and ancient-DNA statistics | PCA overlaps PLINK; qp-style operations need a concrete workflow review |
| ANGSD | genotype-likelihood analysis for low-coverage data | distinct future product candidate, no placeholder crate |
| RFMix | local ancestry inference | distinct future product candidate |
| IBDseq / hap-IBD | IBD segment detection | distinct from PLINK pair summaries; requires its own review |
| sgkit | array/Zarr population-genetics framework | interoperability target, not another CLI product by default |
| Hail | distributed JVM/Spark analysis | no current Rust product case |

No `rsomics-admixture`, `rsomics-eigensoft`, `rsomics-angsd`,
`rsomics-rfmix`, or `rsomics-ibdseq` repository is created from this survey.

## Release evidence

Both retained products require:

- exact upstream versions and replayable oracle generation;
- adversarial format and statistical fixtures;
- a representative benchmark with output equivalence, wall/CPU time, peak
  RSS, bytes, machine, flags, warmups, and repetitions;
- a strict performance or resource advantage on a declared hot path;
- common `rsomics-help` CLI behavior;
- native Linux and macOS CI on `x86_64` and `aarch64`.

Historical Ubuntu-only CI and tiny Criterion fixtures are source evidence, not
release gates.
