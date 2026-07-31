# Genotype and population-genetics product dossiers

Status: joint boundary, upstream-operation, and historical-source audit
complete. Neither target repository exists yet.

## Portfolio decision

Retain two products:

- `rsomics-plink` for PLINK-style genotype data management, QC, relatedness,
  structure preparation, association, and scoring;
- `rsomics-popgen` for neutral variation, population differentiation,
  admixture statistics, haplotype diversity, and selection scans.

The routed source pools contain 42 PLINK/genotype-analysis candidates and 14
population-genetics candidates.

The input format does not determine the product. A VCF-native Hardy-Weinberg
test or LD-pruning implementation belongs to the genotype-analysis workflow,
while a VCF-native F-statistic or iHS implementation belongs to the
population-genetics workflow. `rsomics-vcf` owns VCF/BCF representation,
inspection, transformation, filtering, and indexing; it does not own every
analysis whose input happens to be VCF.

The current upstream anchors reviewed on 2026-07-31 are:

| Contract | Reviewed source | Role |
|---|---|---|
| PLINK 2 | [official documentation dated 4 May 2026](https://www.cog-genomics.org/plink/2.0/); each differential must pin an exact binary build | default behavior oracle for `rsomics-plink` |
| PLINK 1.9 | [official documentation](https://www.cog-genomics.org/plink/1.9/); each retained legacy profile must pin an exact 1.90 build | explicit legacy report and algorithm profiles only |
| scikit-allel | [1.3.13](https://pypi.org/project/scikit-allel/1.3.13/) documentation, source, and generated goldens | array-level oracle for diversity, F-statistics, LD, and selection scans |
| vcftools | 0.1.17 binary, manual, and captured outputs | compatibility oracle for named legacy VCF reports |
| Specifications and papers | VCF 4.x, the PLINK 2 format specification, and the cited statistical methods | semantic authority where a tool exposes only one policy choice |

PLINK 2 remains a moving target and does not generally produce PLINK 1
drop-in output. Every compatibility result therefore names the oracle build
and profile instead of calling an output generically “PLINK-compatible.”

Live GitHub inspection on 2026-07-31 found no `omics-rust/rsomics-plink` or
`omics-rust/rsomics-popgen` repository. The retained names are planning
allowlist entries, not published implementations.

## Boundary and overlap map

| Capability | Canonical product | Reason |
|---|---|---|
| PGEN/BED/VCF genotype import, filtering, conversion, and metadata | `rsomics-plink` | shared genotype dataset lifecycle |
| allele/genotype counts, missingness, HWE, heterozygosity, sex and Mendel QC | `rsomics-plink` | dataset QC rather than VCF-format manipulation |
| LD pairs/matrices, pruning, haplotype blocks | `rsomics-plink` | genotype preparation and association workflow |
| GRM, KING, IBD summaries, PCA, clustering | `rsomics-plink` | sample relationship and structure workflow |
| linear/logistic/Firth association, model tests, TDT, scoring | `rsomics-plink` | phenotype-aware genotype analysis |
| nucleotide diversity, Watterson theta, Tajima's D, Dxy, fixed differences | `rsomics-popgen` | neutral variation and divergence |
| SFS, F-statistics, PBS, D/f2/f3/f4 | `rsomics-popgen` | population differentiation and admixture inference |
| EHH, iHS, nSL, XP-EHH, XP-nSL, Garud statistics | `rsomics-popgen` | haplotype-based selection analysis |
| windowed LD as a genome-scan summary | `rsomics-popgen` | population-level spatial statistic, not pruning |
| generic VCF view/filter/norm/query/stats | `rsomics-vcf` | format-centered operations |

No Layer B product depends on another Layer B product. An operation that needs
the same policy-free numerical kernel in both products may promote that kernel
to `rsomics-stats` only after both consumer call sites and tests exist.

## `rsomics-plink`

### Boundary

`rsomics-plink` is one genotype-analysis product, not 31 binaries named after
individual PLINK flags. It owns a typed dataset containing variants, samples,
pedigree fields, phenotypes, covariates, allele orientation, ploidy, phase,
dosage, and hard-call state.

The default compatibility target is current PLINK 2. PLINK 1 reports and
algorithms remain named legacy profiles when their behavior is scientifically
or operationally useful. The CLI uses product subcommands and shared options
rather than reproducing PLINK's order-sensitive flag parser:

```text
rsomics-plink make
rsomics-plink stats
rsomics-plink ld
rsomics-plink relatedness
rsomics-plink pca
rsomics-plink glm
rsomics-plink score
rsomics-plink family
```

Each invocation records its input profile, filters, allele convention,
founder policy, missingness policy, output schema, oracle profile, and thread
count. `rsomics-help` supplies the common help, version, diagnostics, output,
and completion contract.

### Input and execution model

- PLINK 2 `.pgen/.pvar/.psam` is a first-class input, including reference
  status, phased calls, dosage, multiallelic variants, named phenotypes, and
  categorical covariates declared by the format.
- PLINK 1 `.bed/.bim/.fam` remains a first-class legacy input. Its A1/A2
  orientation is not silently reinterpreted as REF/ALT.
- VCF/BCF import retains only fields represented by the genotype dataset and
  reports discarded fields. Sample identity and order are checked.
- Filters are a typed plan applied in one documented order. A report and a
  derived fileset from the same invocation consume the same filtered state.
- X, Y, PAR, mitochondrial, haploid, unknown-sex, founder, and pedigree rules
  are part of the dataset model rather than special cases reimplemented by
  each report.
- Multi-file outputs are transactional. A late failure cannot leave a
  complete-looking `.pgen/.pvar/.psam` or report set.

The historical `rsomics-pgen` does not implement PGEN. It is a PLINK 1
`.bed/.bim/.fam` reader whose public types are named `Pgen` and `PgenMmap`.
Its useful parser and mmap code is internalized under an accurate product-local
name. A real PGEN reader/writer is implemented against the current format
specification and exact PLINK/pgenlib differentials. `rsomics-pgen` is not
retained as a public foundation because it has only one product consumer.

### Operation map

| Upstream operation family | Target surface | Release decision |
|---|---|---|
| `.pgen/.pvar/.psam`, `.bed/.bim/.fam`, VCF/BCF and supported import | `make` and shared input adapters | PGEN, BED, and VCF first; additional formats only with complete schema tests |
| sample/variant selection, chromosome/range, missingness, frequency, HWE and pedigree filters | shared filter plan | filters used by a stable operation ship with that operation; no inert flags |
| `--make-pgen`, `--make-bed`, `--export`, metadata updates and merge | `make` | transactional PGEN/BED output first; broad export and merge later |
| `--freq`, `--geno-counts`, `--sample-counts`, `--missing`, `--hardy`, `--het` | `stats` | first-release QC slice |
| `--mendel`, `--check-sex`, `--impute-sex`, `--fst` | `stats` and `family` | later after chromosome, pedigree, and shared FST contracts pass |
| `--indep*`, `--r[2]-*`, `--ld`, legacy `--blocks` | `ld` | unphased pairwise LD and pruning first; phased LD and blocks later |
| `--make-rel`, GCTA GRM, `--make-king*`, legacy `--genome` | `relatedness` | later complete relationship slice |
| `--pca` and projection | `pca` | later; exact normalization, missingness, and projection metadata required |
| `--glm` linear/logistic/Firth, covariates, interactions, conditions and permutations | `glm` | later complete regression slice; old single-predictor binaries do not qualify |
| PLINK 1 `--assoc`, `--model`, `--linear`, `--logistic`, `--epistasis` | named legacy modes under `glm` | only behavior still useful and oracle-tested |
| `--score[-list]`, `--variant-score`, score ranges | `score` | later complete scoring slice |
| `--tdt`, missingness tests, Mendel reports | `family` and `stats` | later pedigree-aware slice |
| legacy `--homozyg`, `--cluster`, `--flip-scan` | `roh`, `relatedness`, and `stats` | later named compatibility profiles |
| clumping, distributed parts, every PLINK 1 niche operation | later or explicit exclusion | no placeholder flags |

The first release is one complete genotype-QC and LD-preparation slice:
PGEN/BED/VCF input, shared sample and variant filters, allele and genotype
counts, sample and variant missingness, biallelic HWE, sample heterozygosity,
unphased pairwise LD, and LD pruning. It includes transactional PGEN output,
one durable filtered-dataset state, machine-readable reports, and PLINK 2
exact-build compatibility evidence. Association, relationship, PCA, pedigree,
and scoring operations remain undocumented or feature-gated until their
complete slices pass.

### Historical asset disposition

| Historical asset | Audited revision | Disposition |
|---|---|---|
| `rsomics-kinship` | `ce5f879d59fd8277f54aed396e2cc6e8c66e46d8` | refactor then merge into `relatedness`; retain KING bitplane/SIMD and PLINK 2 golden assets |
| `rsomics-plink-assoc` | `d1555fb80e2a350bef63365e9f801f77401392f0`; untracked `Cargo.lock` only | test and PLINK 1 report asset; its chi-square, trend, and linear paths overlap later typed implementations |
| `rsomics-plink-blocks` | archive-only source tree, no Git revision | algorithm and fixture asset for later legacy blocks mode |
| `rsomics-plink-check-sex` | `6a7bf2ba812b824307b1581e434dbe38ad9aa6de` | refactor then merge after PAR, ploidy, frequency, and sex-state review |
| `rsomics-plink-cluster` | `5e206573a985b22e10ea2032746fee4ffb90fa00` | algorithm and fixture asset for a later PLINK 1 clustering profile |
| `rsomics-plink-epistasis` | `68a197ce6cb20f67355840b1f7dd554aa9e8af54` | refactor then merge only with a complete interaction-test contract |
| `rsomics-plink-flip-scan` | `5a149f5a287cd1ed7dba12de4870b9c5bd7dddef` | refactor then merge as later strand-QC mode |
| `rsomics-plink-freq` | `2b234d50d7e89c07c695b36dd7f4b4758f851c3c` | refactor then merge as a first-release counting hot-path seed; replace its private fileset reader |
| `rsomics-plink-freqx` | `846b745ae48a2af569a2261142b945a684397b7a` | refactor then merge into genotype counts; preserve PLINK 1 output fixture only as a named profile |
| `rsomics-plink-genome` | archive-only source tree, no Git revision | algorithm and fixture asset for later legacy IBD/IBS summaries |
| `rsomics-plink-grm` | `bff8f61891ab59b4f943ba246a777e09e330efd5` | refactor then merge into `relatedness`; validate normalization and matrix formats |
| `rsomics-plink-hardy` | archive-only source tree, no Git revision | refactor then merge into first-release HWE; replace duplicated input and formatting code |
| `rsomics-plink-het` | `24f1664edfd2878931c9e3e0b270afcbe76d01fb` | refactor then merge into first-release sample QC |
| `rsomics-plink-homozyg` | `fa7c781e4e63450187652dcbde96aa8dfe63be00` | refactor then merge into a later ROH slice |
| `rsomics-plink-ibc` | `302c9d92ddb5f7a1e3ae44eb91fce15e465235a8` | refactor then merge as a later legacy inbreeding report |
| `rsomics-plink-io` | `5298bd0acf0d5100f9550c19fdeb10c33025f47c` | refactor then merge for BED fixtures and export behavior; do not retain its multi-operation binary |
| `rsomics-plink-ld` | `fbe04835ea2df60c1483dca06077aaf3d2e4739d` | formula and fixture asset for first-release LD; replace whole-matrix and private-reader policy |
| `rsomics-plink-linear` | `5820975d2de61d436d23296ac2208f4c820d48ce` | refactor then merge as a later quantitative-trait kernel and PLINK 1 golden asset |
| `rsomics-plink-logistic` | `162b76d549d612cdc1a316fb66c7031d8c609f67` | refactor then merge as a later binary-trait kernel; add covariates, Firth state, and error codes |
| `rsomics-plink-mendel` | `238e18b393e6f149563592be8345e055d4252fec` | refactor then merge after general chromosome and multiallelic inheritance review |
| `rsomics-plink-missing` | `5c2a030b23020bf12d1aa95dae705ebcca63bc12` | refactor then merge into first-release QC |
| `rsomics-plink-model` | `5526445ad65d5f72cd1dedc55cb65100ba576939`; inherited `Cargo.lock` diff only | refactor then merge only as a named PLINK 1 model profile |
| `rsomics-plink-pca` | `56f68a546b23f382c839d18a22ce435b46ebeaff`; inherited `Cargo.lock` diff only | refactor then merge after normalization, missingness, projection, and large-matrix review |
| `rsomics-plink-prune` | `d7ab7d45749118598a1583e67b270ecbeb0645e0` | refactor then merge as first-release PLINK pruning seed |
| `rsomics-plink-recode` | `2625bfae6c6dd70e172e367cfc19171cb1506e42` | refactor then merge into later export; retain additive-dosage fixtures |
| `rsomics-plink-score` | `69f882855ac1f7b28b40455b1cebab31565fe46f` | refactor then merge into a later multi-score slice |
| `rsomics-plink-tdt` | `e7487918d8891672cf615c3d34701baa160f12a2` | refactor then merge into later pedigree association |
| `rsomics-plink-test-missing` | `b8b6d2ccbe4fce3afa3d2a0cf24a19a276ee2ffc` | refactor then merge into later phenotype-stratified QC |
| `rsomics-ld-matrix` | `e1b691e400e03a04d0c5560457f4b60af9b2534d` | refactor then merge as a matrix input adapter and oracle asset for `ld` |
| `rsomics-vcf-af-dist` | `5d3587b3f87fad72f15ab6131076218201bb3b65` | refactor then merge as a bcftools-profile genotype-probability report under `stats`; update the 1.24 per-sample mode |
| `rsomics-vcf-contrast` | `c029a24cf4792dd4640247c031973f1eaafc2874` | refactor then merge into later case/control `glm`; preserve Fisher and novel-allele fixtures |
| `rsomics-vcf-freq-table` | `0acb37c728e0e075b132ac6a31255bd6a4fd1680` | refactor then merge into first-release allele/genotype counts as a vcftools output profile |
| `rsomics-vcf-geno-r2` | local non-Git source tree | formula, fixture, and vcftools report asset for first-release `ld`; remove its inherited build output before migration |
| `rsomics-vcf-gtcheck` | `1c324f170cf613fc2808f1c8d8b46d7f4a3b7b7e` | refactor then merge into `relatedness` and sample-concordance QC |
| `rsomics-vcf-hardy` | `7c2a75f2c7cd1876fd3d2933c06b5c285af35871` | test, formatting, and exact-test asset for VCF-input HWE; invalid allele indices must fail |
| `rsomics-vcf-indv-stats` | `01f3c4c453e6636f59f202a94e5e24e0e6d6eb09` | split then merge Ts/Tv, singleton, and depth profiles into first-release sample QC |
| `rsomics-vcf-ld-prune` | `527ac349609d649872ddf6c3363c00bf2d7553d9` | refactor then merge as the scikit-allel pruning profile over the shared VCF input |
| `rsomics-vcf-missing-stats` | `dfe256b41fa8708baac839d8ac6bd64ec32fdeb4` | refactor then merge into first-release sample and variant missingness |
| `rsomics-vcf-roh` | `61314b504884dfacdab17066c26fb3b7c5f1bfa0` | refactor then merge as a later genotype-likelihood HMM profile under `roh`; correct the 1.24 quality offset |
| `rsomics-vcf-site-depth` | `be0404d7ae5eb83b2d77b0ccdbccef4fb50ab99d` | refactor then merge into first-release depth QC with checked wide accumulators |
| `rsomics-vcf-smpl-stats` | `613065f0c67c4788459ef12392ee202612e23f33` | refactor then merge into first-release per-sample genotype statistics |
| `rsomics-vcf-trio-stats` | `88a1dd5304482203734beae25588084dafe881f3` | refactor then merge into later pedigree-aware `family` reports |

`rsomics-vcf-popgen` is primarily routed to `rsomics-popgen`; its `het` and
haplotype-LD code and fixtures are secondary source assets for `rsomics-plink`.
That split is recorded in the popgen table below instead of counting the
repository twice.

### Existing implementation gaps

- The shared crate named `rsomics-pgen` only parses PLINK 1 binary files. None
  of the historical products reads or writes real PLINK 2 PGEN.
- At least six products contain another BED reader or bitplane layout. The
  linear, logistic, and missingness-test products contain near-identical
  streaming readers, while other hot paths use private mmap readers.
- Most operations implement PLINK 1 behavior. Current PLINK 2 supports
  multiallelic variants, dosage, phase, flexible columns, founder policy,
  chromosome-specific ploidy, multiple phenotypes/covariates, and different
  defaults that the old binaries do not model.
- The old linear and logistic products fit a single additive predictor. They
  do not implement the current general `--glm` contract, Firth fallback,
  multiallelic columns, local covariates, conditioning, interactions,
  permutations, or structured error codes.
- Several “compatibility” tests return success when PLINK is absent. Others
  compare only variant-ID sets or tiny committed PLINK 1 reports. These remain
  useful fixtures but are not current compatibility gates.
- Criterion benches commonly use the tiny golden fileset. A microbenchmark
  over a few variants is not evidence of a throughput or memory advantage.
- Every historical PLINK repository has Ubuntu-only CI. None meets the four
  native platform release gate.
- Only a small minority use `rsomics-help`; command surfaces, status output,
  errors, and output creation are inconsistent.

### Compatibility and performance gates

The first release must pass:

1. exact-build PLINK 2 differentials for every declared PGEN/BED/VCF input,
   filter, report, and pruning behavior;
2. a separate pinned PLINK 1 profile for retained legacy reports;
3. adversarial fixtures covering multiallelic sites, dosage, phase, missing
   calls, founder/nonfounder state, pedigrees, sex chromosomes, PAR boundaries,
   unknown sex, duplicate IDs, allele orientation, unsorted metadata, and
   truncated files;
4. round-trip and cross-reader checks against PLINK 2 and pgenlib for every
   declared PGEN encoding;
5. representative cohort benchmarks that report wall time, CPU time, peak RSS,
   input/output bytes, versions, machine, flags, warmups, repetitions, and
   output equivalence;
6. strict advantage on at least one first-release hot path without regression
   on the remaining declared operations;
7. native Linux and macOS CI on both `x86_64` and `aarch64`.

## `rsomics-popgen`

### Boundary

`rsomics-popgen` is a population-genetics analysis product over one typed
sample, population, variant, allele-count, genotype, and haplotype model. It is
not a VCF utility suite and not one crate per statistic.

Its operations are grouped by scientific workflow:

```text
rsomics-popgen diversity
rsomics-popgen sfs
rsomics-popgen fst
rsomics-popgen admixture
rsomics-popgen selection
rsomics-popgen ld
```

Every run records sample-to-population assignments, ploidy, ancestral-allele
policy, site accessibility, missingness, filtering, window coordinates,
normalization, standardization bins, genetic-map interpolation, oracle
profile, and thread count. Coordinate windows and variant-count windows are
different typed plans and cannot be exchanged silently.

### Operation map

| Scientific contract | Target surface | Release decision |
|---|---|---|
| nucleotide diversity, Watterson theta, Tajima's D | `diversity` | first-release per-site, region, and bp-window forms |
| Dxy and fixed differences between populations | `diversity` | first release with explicit population and accessibility contracts |
| folded/unfolded 1D and joint 2D SFS | `sfs` | first release; ancestral state and projection are explicit |
| Hudson and Weir-Cockerham FST | `fst` | first release; site components, region aggregation, and uncertainty are distinct outputs |
| PBS/PBSn1 | `fst --pbs` | later differentiation/selection slice |
| Patterson D, f2, f3, f4 and block jackknife | `admixture` | later complete admixture-statistics slice |
| haplotype diversity and Garud H1/H12/H123/H2/H1 | `selection haplotype` | later phased-haplotype slice |
| EHH decay, iHS, nSL | `selection single` | later single-population selection slice |
| XP-EHH and XP-nSL | `selection cross` | later cross-population selection slice |
| windowed Rogers-Huff LD summaries | `ld scan` | later spatial LD slice |

The first release is a complete neutral-variation and differentiation slice:
VCF input plus checked sample/population metadata, site filtering and
accessibility, π, Watterson theta, Tajima's D, Dxy, fixed differences, folded
and unfolded SFS, and Hudson/Weir-Cockerham FST. Region and window aggregation,
uncertainty, undefined results, output schemas, and provenance are included.
Selection and admixture operations remain undocumented or feature-gated until
their full contracts pass.

### Historical asset disposition

| Historical asset | Audited revision | Disposition |
|---|---|---|
| `rsomics-ehh-decay` | `8ae6abac6b8eff6f6ffbefb3f70e0f2645c2c227`; inherited `Cargo.lock` diff only | refactor then merge into later selection; deduplicate phased-VCF and EHH state |
| `rsomics-haplotype-diversity` | `2a5fb064678854045444b3f82d9ee3705ae82c3d` | refactor then merge into later haplotype selection |
| `rsomics-pbs` | `7d91faeec7100503ec16b48d6dff944e0588ffef` | refactor then merge after the shared FST contract |
| `rsomics-popgen-fstats` | `c979ff884ad9c66be4cf4adc0d7785e6f3de8a5d` | refactor then merge into later admixture; retain jackknife and 1.3.13 goldens |
| `rsomics-popgen-garudh` | `8b76f3f775aed89bc532739a680a27be141fa8b2` | refactor then merge into later haplotype selection |
| `rsomics-popgen-ihs` | `4ba35b8d17294b49184552f7a9f42f949874091d` | refactor then merge as single-population selection seed |
| `rsomics-popgen-windowed` | `9e555d458d929f7dae7f59945ae13ad18fd66cee`; inherited `Cargo.lock` diff and untracked external-disk `target/` | refactor then merge as first-release diversity seed |
| `rsomics-popgen-xpehh` | `3c4acaafcb27e466b2bef3e4d797e1dc1641a85c` | refactor then merge as cross-population selection seed |
| `rsomics-tajima-d` | `9f4c8c4a05dcfa1088aae6e8d49f253cd22c649b` | formula and SFS-input fixture asset; do not retain a standalone binary |
| `rsomics-vcf-popgen` | `acebc80d2cd7a949063093228de572abfbe3483d` | split then merge: FST and Dxy into first-release popgen; heterozygosity and haplotype LD become PLINK source/test assets |
| `rsomics-vcf-sfs` | `b604e0e56ef2e3c4728916737315234eb18ed2a0` | refactor then merge into first-release SFS |
| `rsomics-vcf-tajima-d` | `9208e83a621c0cca11942939a599c57f67436a01` | vcftools output-profile and fixture asset; use the shared diversity implementation |
| `rsomics-vcf-window-pi` | `49ce79956ccd43d8837093d3a64a67ed836a2a18` | vcftools output-profile and performance asset; reject its permissive invalid-allele behavior |
| `rsomics-windowed-ld` | `45c6ed45f8f179d494b3489ffb61abeea46e4f76` | refactor then merge into later `ld scan`; deduplicate dosage, VCF, and window code |

`rsomics-popgen-core` at
`640cfbc4f11e799e25f16aaddcf78f336472d920` has only this product as a
consumer. Its π, Watterson theta, Tajima's D, HWE, and LD functions are
algorithm assets, not a retained public foundation. Product-specific items are
internalized. A policy-free kernel moves to `rsomics-stats` only when another
named product exercises the same contract.

### Existing implementation gaps

- The historical products repeat VCF parsing, genotype dosage conversion,
  sample/population joins, window generation, allele counting, float
  formatting, and CLI setup. `rsomics-vcf-ld-prune` and
  `rsomics-windowed-ld` even contain an identical dosage module.
- Window conventions are inconsistent: inclusive bp windows, vcftools
  zero-based bins, variant-count windows, moving windows, and dropped trailing
  windows are encoded independently rather than represented in types.
- Accessibility masks, callable denominators, ploidy, multiallelic behavior,
  ancestral-state uncertainty, spanning deletions, contig boundaries, and
  missing calls vary by operation and are not captured in one provenance
  model.
- Some compatibility tests use strong committed 1.3.13 goldens, while others
  preserve only tiny examples. Oracle generation is not pinned and replayed
  as a release gate across the whole product.
- Several benchmarks skip successfully when a private environment variable is
  absent. Published README timings use different input sizes and do not form a
  comparable product-level performance record.
- Invalid allele indices must be parse failures. Retaining deterministic
  output for malformed VCF because vcftools sometimes crashes violates the
  fail-loud contract.
- Historical CI is overwhelmingly Ubuntu `x86_64`; two VCF reports also ran a
  generic macOS job, but no repository covers all four native target classes.
- Most binaries use `rsomics-common` flags directly and do not use the required
  `rsomics-help` command layer.

### Shared foundation decisions

- `rsomics-common` owns process errors, execution context, output transactions,
  provenance envelopes, and common runtime policy used by both products.
- `rsomics-help` owns the unified command/help/version/completion experience
  for both binaries.
- `rsomics-stats` may receive Hudson and Weir-Cockerham component kernels after
  `rsomics-plink` and `rsomics-popgen` both have consumer-side tests. Input
  parsing, founder selection, population assignment, windowing, aggregation,
  and output policy stay product-local.
- HWE and Rogers-Huff LD are not promoted merely because old repositories
  contain copies. PLINK and scikit-allel differ in founder, multiallelic,
  precision, pruning, and missingness policy; a shared API requires two
  genuinely identical consumer contracts.
- `rsomics-pgen` and `rsomics-popgen-core` are internalized, not evolved as
  speculative public foundations.

### Compatibility and performance gates

The first popgen release must pass:

1. regenerated and committed scikit-allel 1.3.13 goldens with a recorded
   environment lock and deterministic generator;
2. vcftools 0.1.17 differentials only for explicitly named output profiles;
3. independent formula tests from the cited papers for every estimator;
4. fixtures spanning multiple contigs, empty and partial windows, inaccessible
   bases, missing calls, unequal population sizes, ploidy, multiallelic sites,
   ancestral-state errors, spanning deletions, monomorphic sites, and
   undefined denominators;
5. property checks that region aggregation agrees with site components and
   that window partitioning preserves the declared denominator;
6. representative whole-contig benchmarks against the matching upstream path,
   with output equivalence, wall/CPU time, peak RSS, bytes, versions, machine,
   flags, warmups, and repetitions;
7. strict performance or resource advantage on a first-release hot path;
8. native Linux and macOS CI on both `x86_64` and `aarch64`.

## Explicit exclusions

- Variant calling, generic VCF transformation, normalization, querying, and
  indexing stay in `rsomics-vcf`.
- ADMIXTURE/fastSTRUCTURE ancestry models, ANGSD genotype-likelihood analysis,
  RFMix local ancestry, Hail distribution, and IBD-segment callers are not
  silently folded into either first release. They require separate product
  evidence and boundaries.
- The first `rsomics-plink` release does not claim complete PLINK 1.9 or PLINK
  2 parity.
- The first `rsomics-popgen` release does not advertise selection or admixture
  subcommands merely because historical code exists.
- No public `rsomics-pgen`, `rsomics-popgen-core`, LD, HWE, EHH, or window
  micro-crate is created.
