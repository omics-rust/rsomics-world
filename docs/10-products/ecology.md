# Community ecology product dossier

Status: source and upstream-operation audit complete. The target repository has
not been created.

## Boundary

`rsomics-ecology` is one product for community-table diversity, ecological
dissimilarity, ordination, metadata association, and permutation analysis. It
owns the workflow that starts with samples by observed features and may add a
phylogeny, sample metadata, or environmental variables.

The primary behavior sources are:

- [scikit-bio 0.7.3 community diversity](https://scikit.bio/docs/latest/diversity.html)
  for count validation, alpha and beta drivers, Faith PD, generalized
  phylogenetic diversity, and UniFrac;
- [scikit-bio 0.7.3 distance statistics](https://scikit.bio/docs/latest/generated/skbio.stats.distance.html)
  for ANOSIM, PERMANOVA, PERMDISP, BIO-ENV, Mantel, and pairwise Mantel
  contracts;
- [scikit-bio 0.7.3 ordination](https://scikit.bio/docs/latest/generated/skbio.stats.ordination.html)
  for PCA, PCoA, CA, CCA, RDA, biplot projection, and result semantics;
- [scikit-bio 0.7.3 gradient analysis](https://scikit.bio/docs/latest/generated/skbio.stats.gradient.html)
  for average, trajectory, first-difference, and window-difference analysis;
- [vegan 2.7-5](https://stat.ethz.ch/CRAN/web/packages/vegan/index.html) for
  the broader community-ecology workflow, especially `vegdist`, `adonis2`,
  `betadisper`, `bioenv`, `mantel`, `cca`, and `rda`;
- [biom-format 2.1.17](https://pypi.org/project/biom-format/) for rarefaction
  RNG behavior and the later BIOM table profile.

All nineteen historical product candidates correspond to real scikit-bio or
vegan operations. The problem is their packaging and duplicated infrastructure,
not fictitious operation names. They become modes and subcommands of one
installable product rather than nineteen binaries.

The product is domain-scoped. It does not become a generic matrix algebra,
statistics, or plotting suite merely because PCA, correlation, and
permutation are used internally.

## Operation map

### Initial diversity slice

| Target subcommand | Upstream operation | Initial stable surface |
|---|---|---|
| `alpha` | scikit-bio alpha driver and metrics | multi-metric pass over all samples; observed features, Shannon, Simpson family, Pielou, Chao1, Good coverage, Faith PD, and generalized phylogenetic diversity |
| `beta` | scikit-bio beta driver; SciPy pairwise distances; UniFrac | Bray-Curtis, presence/absence Jaccard, Euclidean, Canberra, city-block, unweighted UniFrac, weighted UniFrac, and normalized weighted UniFrac |
| `rarefy` | scikit-bio `subsample_counts`; biom-format sampling | fixed-depth sampling without replacement with a recorded compatibility RNG profile |

Phylogenetic and non-phylogenetic metrics are modes of `alpha` and `beta`.
Requiring a tree is a metric-specific argument contract, not a reason to
publish Faith PD, phydiv, or UniFrac as separate products.

The first release may narrow the metric list further if necessary, but any
advertised metric must have its complete degenerate-input, serialization,
oracle, and performance contract. There is no `all` value that silently
includes unfinished metrics.

### Later ordination slice

| Target surface | Upstream operation | Gate |
|---|---|---|
| `ordinate --method pca` | scikit-bio PCA; vegan `pca`/`rda` | dense and iterative method, requested dimensions, centering/scaling, rank, and covariance conventions |
| `ordinate --method pcoa` | scikit-bio PCoA; vegan `pco` | exact and truncated decomposition, negative eigenvalues, warning/correction policy, and distance provenance |
| `ordinate --method ca` | scikit-bio and vegan correspondence analysis | contingency-table validation, inertia, scaling, and score schema |
| `ordinate --method cca` | scikit-bio and vegan canonical correspondence analysis | response/constraint alignment, scaling types, rank deficiency, fitted and residual axes |
| `ordinate --method rda` | scikit-bio and vegan redundancy analysis | response scaling, constraints, ranks, fitted/residual results, and biplot scores |
| `ordinate --project` | scikit-bio `pcoa_biplot` | sample identity alignment and descriptor projection onto a declared ordination |

PCA remains here only as an ecological ordination over the product's checked
sample-feature model. It is not promoted to a generic public PCA crate.

### Later testing and association slice

| Target surface | Upstream operation | Gate |
|---|---|---|
| `test --method anosim` | scikit-bio/vegan ANOSIM | rank ties, grouping alignment, statistic range, and one-sided permutation contract |
| `test --method permanova` | scikit-bio PERMANOVA; vegan `adonis2` | initial single-factor profile; later formula, strata, sequential/marginal tests, sums of squares, and partial R-squared |
| `test --method permdisp` | scikit-bio PERMDISP; vegan `betadisper` | centroid/median, negative axes, bias adjustment, group dispersions, and permutation method |
| `associate --method mantel` | scikit-bio/vegan Mantel | Pearson/Spearman, exact/shared identities, alternative, and lookup policy |
| `associate --method pairwise-mantel` | scikit-bio `pwmantel` | streamed matrix pairs, labels, strict/shared identity policy, and complete result table |
| `associate --method bioenv` | scikit-bio/vegan BIO-ENV | variable standardization, subset limit, ties, missing data, and exhaustive-search warning |
| `trajectory` | scikit-bio gradient analysis | average, trajectory, first difference, window difference, natural ordering, weighting, and ANOVA contract |

`test` and `associate` share matrix, metadata, RNG, and report types. They do
not duplicate one parser and permutation engine per statistic.

### Further community-ecology slices

The upstream surface also includes metrics absent from the historical source
pool, partial and block beta diversity, non-metric multidimensional scaling,
distance-based RDA, Procrustes analysis, environmental fitting, diversity
partitioning, and constrained permutation designs. Each is added to the
existing product only after its own operation dossier extension and release
gate. No empty modules reserve these operations.

## Community-table contract

- The initial text profile is a delimited feature-by-sample table: the first
  column contains feature identities, the header contains sample identities,
  and cells contain abundances. Orientation is recorded and never inferred
  from dimensions.
- Feature and sample identities are nonempty and unique. Whitespace handling,
  comments, quoted delimiters, and `#OTU ID` compatibility are declared by
  input profile rather than varying among subcommands.
- An integer count table contains checked non-negative integers. A continuous
  abundance table contains finite non-negative values. `NaN` and infinities
  are never accepted merely because a comparison with zero returns false.
- Ragged rows, duplicate identities, missing cells, overflow, and an
  incompatible metric/table kind fail before computation.
- Empty features, empty samples, and zero-total samples have metric-specific
  semantics. One global “return NaN for degenerate input” rule is not applied
  to richness, entropy, dissimilarity, rarefaction, and phylogenetic metrics
  indiscriminately.
- Alpha metrics selected together share one validated table pass and one
  per-sample summary. Beta metrics may reuse validated storage but do not keep
  every pair as an auxiliary tuple vector in addition to the output matrix.
- TSV output preserves the declared sample order. JSON represents
  unavailable values as typed nulls with a reason; a compatibility text
  profile may emit the pinned upstream `nan` spelling.
- BIOM dense and sparse inputs are a later format profile. Reading a BIOM-like
  TSV header does not constitute BIOM format support.

## Distance and metadata contracts

- One internal labelled matrix type owns square shape, row-major storage,
  nonempty unique identities, row/header alignment, hollow diagonal, symmetry,
  finite values, and operation-specific non-negativity.
- A condensed view borrows or iterates over the upper triangle. It does not
  allocate a second vector unless the selected algorithm requires ownership.
- Exact identity is the safe default. A shared-identity mode records excluded
  IDs and builds a matrix whose shape is the intersection size, not the
  original matrix size with zero-filled trailing cells.
- Metadata and environmental tables use a checked sample index. Row order may
  differ, extra metadata rows may be ignored under the declared profile, and a
  matrix sample missing from metadata is an error.
- Categorical groupings and numeric environmental variables are distinct typed
  columns. Missing values, constant variables, insufficient group sizes, and
  a one-group design receive explicit errors or upstream-compatible results.
- A distance result records its metric, source table, feature filtering,
  normalization, tree profile where applicable, and identity order. A later
  test or ordination can therefore reject an incompatible matrix before
  interpreting it.

## Phylogenetic-diversity contract

- `rsomics-phylo-tree` supplies an always-valid topology, traversal, finite
  branch-length view, and checked tip index. Community-vector accumulation and
  metric formulas remain in this product.
- Faith PD and UniFrac require the declared rooted-tree profile. Generalized
  phylogenetic diversity records rooted/unrooted interpretation and abundance
  weight rather than guessing silently from root degree.
- Every table feature used by a phylogenetic metric maps to exactly one named
  tree tip. The tree may contain extra tips under the scikit-bio profile;
  duplicate or unnamed tips and duplicate table features are errors.
- Required branch lengths are finite and non-negative. Missing root length and
  a stored root length follow a named compatibility policy. Missing non-root
  lengths are not silently zero-filled.
- One vectorized postorder representation serves Faith PD, generalized PD, and
  UniFrac. Separate flattened-tree structs do not independently redefine root
  and branch semantics.
- UniFrac defines all-zero pairs, one-empty pairs, normalized denominator zero,
  and root-length behavior for every mode before parallel pair evaluation.

## Permutation and reproducibility contract

- Results record the observed statistic, alternative, permutation count,
  p-value correction rule, RNG profile, seed, and thread count.
- `permutations=0` is statistic-only mode and produces an unavailable p-value,
  not an unexplained floating-point NaN.
- A scikit-bio compatibility profile reproduces the pinned NumPy generator and
  permutation stream where bit identity is part of the claim. A faster native
  deterministic profile may use independent indexed streams, but it has a
  different name and does not claim identical p-values.
- The observed arrangement contributes to numerator and denominator exactly as
  declared. Equal-to-observed comparisons and two-sided absolute-value rules
  are tested at tie boundaries.
- Restricted permutations, blocks, and strata are absent until implemented.
  A simple free-label shuffle is not advertised as `adonis2` formula
  compatibility.
- Parallel execution never mutates a process-global pool. Repeating a run with
  the same profile, seed, input, and thread count gives the declared
  reproducibility level.

## Ordination contract

- One ordination result schema carries method, eigenvalues, explained
  proportion, sample scores, feature scores, biplot scores, constraints,
  fitted scores, and residual scores where defined.
- Axis count may be an exact integer or a declared variance threshold only
  where the upstream operation supports it. Truncating already-computed dense
  axes is reported separately from a genuinely lower-cost iterative method.
- Eigenvector sign is arbitrary. Oracle comparisons align signs, and repeated
  or near-repeated eigenspaces are compared as subspaces rather than by
  elementwise coordinates.
- PCA records centering, scaling, covariance denominator, dense/iterative
  solver, and rank. PCoA records centering, negative eigenvalues, warning or
  correction policy, and the treatment of non-Euclidean distances.
- CA, CCA, and RDA validate non-negative community response where required,
  positive margins, sample alignment, finite constraints, model rank, and
  degenerate fitted/residual spaces before calling a decomposition.
- Solver failures and non-finite outputs return typed errors. Public
  computation paths do not unwrap a fallible eigendecomposition or SVD.

## Target structure

```text
src/
├── cli.rs
├── table/
│   ├── abundance.rs
│   ├── count.rs
│   ├── metadata.rs
│   └── text.rs
├── matrix/
│   ├── distance.rs
│   ├── identity.rs
│   └── tsv.rs
├── diversity/
│   ├── alpha.rs
│   ├── beta.rs
│   ├── phylogenetic.rs
│   └── rarefy.rs
├── ordination/
│   ├── model.rs
│   ├── pca.rs
│   ├── pcoa.rs
│   ├── correspondence.rs
│   └── constrained.rs
├── inference/
│   ├── grouping.rs
│   ├── permutation.rs
│   ├── anosim.rs
│   ├── permanova.rs
│   └── permdisp.rs
├── association/
│   ├── correlation.rs
│   ├── mantel.rs
│   └── bioenv.rs
├── trajectory.rs
├── output.rs
└── report.rs
```

Only modules belonging to the current release slice are created. The initial
diversity release has no empty ordination, inference, association, or
trajectory module.

The library exposes checked table inputs, metric configurations, distance and
result types required for programmatic diversity work. CLI parsing, input
profile selection, serialization policy, and execution plumbing stay private.

## Foundation relationships

`rsomics-common` owns error-to-exit mapping, execution envelopes, aliases, and
transactional named output. `rsomics-help` owns the authoritative recursive
Clap presentation. The nineteen duplicated historical `HelpSpec` descriptions
do not survive as a second CLI tree.

`rsomics-phylo-tree` owns checked topology, Newick, traversal, branch views,
and tip identities. Ecology is a concrete consumer of those primitives through
Faith PD, generalized PD, and UniFrac. Diversity-specific vectorization,
abundance accumulation, and formulas remain product-private.

`rsomics-stats` accepts a numerical primitive only when ecology and another
named product use the same finite-value and result semantics. The initial
diversity slice requires no new stats API. Later inference may consume shared
p-value adjustment or a reviewed correlation primitive; grouping,
permutation design, ecological sums of squares, ordination, and report policy
remain in ecology.

`rsomics-distance` is not a retained foundation. Its five historical consumers
collapse into this one product, and `rsomics-pwmantel` already bypasses it with
another private copy. Its parser tests and useful API ideas are internalized
under `ecology::matrix` after the invariants are repaired.

No public count-table, distance-matrix, ordination, RNG, or permutation crate
is added during this wave. A second target-product consumer with the same
contract is required before promotion.

## Historical asset disposition

The nineteen operation-sized product repositories and packages are deleted
from the public namespace. Their external-disk clones remain implementation
assets.

| Source asset | Revision | Disposition |
|---|---|---|
| `rsomics-alpha-diversity` | `542030df8d84dca87d30e3b4a24cec6aaaf1ac0b` | refactor then merge into `alpha`; retain formulas, multi-value metrics, count goldens, and large-table clues; replace parser, degenerate policy, duplicate CLI, and unverified `all` performance claim |
| `rsomics-beta-diversity` | `10f7564b08f333b984255256cf10244c5bf3d9a5` | refactor then merge into `beta`; retain five kernels, float-format fixtures, and empty-feature cases; replace unchecked non-finite counts, pair-list allocation, private matrix, and duplicate parser |
| `rsomics-faith-pd` | `45641adf130ae241733aee06bbd44d8ea48a9759` | refactor then merge as an `alpha` metric; retain path-union kernel, fixtures, and timing clue; replace rootedness heuristic, tree flattening, count parser, and direct output |
| `rsomics-phydiv` | `46cdf30219f394d353c417604247bd4ab3abba10` | refactor then merge as an `alpha` metric family; retain BWPD formulas, rooted/unrooted cases, and weight fixtures; share the checked tree/table model |
| `rsomics-unifrac` | `a631051290342991f411b68d7c10735493162a25` | refactor then merge into `beta`; retain postorder accumulation, three mode goldens, and root-length cases; replace its third count/tree/matrix stack |
| `rsomics-subsample-counts` | `715ab363e0cabe91ddaa6457ef064930a9d278a9` | refactor then merge into `rarefy`; retain NumPy PCG64/choice compatibility, goldens, and sampling algorithms; route through the canonical count table and RNG profile |
| `rsomics-pca` | `52417ce76c3de15348bb549e8f83bc60a1a71c48` | refactor then merge into `ordinate`; retain dense covariance/SVD values and sign-aware tests; add current iterative, dimensions, rank, and result contracts |
| `rsomics-pcoa` | `39e707636e94449edbbbc10e4ed5984699962ddd` | refactor then merge into `ordinate`; retain centering, eigenvalue goldens, and invalid-matrix fixtures; reconcile current truncated method and negative-axis policy |
| `rsomics-correspondence-analysis` | `0b0c456e74e570b731f7291e858670bce644001a` | refactor then merge as `ordinate --method ca`; retain decomposition values and degenerate fixtures; share table, solver, result, formatter, and CLI models |
| `rsomics-cca` | `c723e07a01997db0226ae3ab234e5702b94ce33c` | refactor then merge as `ordinate --method cca`; retain scaling formulas and oracle fixtures; remove fallible solver unwraps and the public performance placeholder |
| `rsomics-rda` | `fa463c987ac75502051c22faccefaf39de409374` | refactor then merge as `ordinate --method rda`; retain fitted/residual decomposition and scaling fixtures; share checked constraint and result models |
| `rsomics-pcoa-biplot` | `b37b387592f5c66276cc8f7a6de2083b3d7e3d6e` | refactor then merge as ordination projection; retain sample-alignment and projection tests; replace its bespoke ordination parser and output schema |
| `rsomics-anosim` | `68e7df285331c684c96f9b5921407f10a1e1deb5` | refactor then merge into `test`; retain rank/tie kernel, grouping fixtures, and statistic goldens; replace RNG, parser, grouping, and result duplication |
| `rsomics-permanova` | `c5f243cee07587afb02ad4b726aa51a673775eff` | refactor then merge into `test`; retain single-factor pseudo-F kernel and goldens; add explicit scope versus `adonis2`, reviewed permutation streams, and typed sums-of-squares results |
| `rsomics-permdisp` | `bd2479cb86fcf5e85c86f14c0742b946c6ba2fcc` | refactor then merge into `test`; retain centroid/median fixtures and geometric median code; replace its private PCoA, solver panics, grouping, RNG, and result duplication |
| `rsomics-mantel` | `c1946b0915acfe64398990a0d7588e78d55f53c8` | refactor then merge into `associate`; retain Pearson/Spearman reductions and identity-reorder fixtures; replace allocated condensed forms, private RNG, and incomplete shared-ID policy |
| `rsomics-pwmantel` | `a25b9fe4621a1990a91e6fb496497dc99d7dcfc1` | refactor then merge as pairwise Mantel; retain streamed pair scheduling and result fixtures; discard its duplicated distance matrix, Mantel engine, formatter, and RNG |
| `rsomics-bioenv` | `f397f74f4a611926ea6cfa7ad85ba60e7275f386` | refactor then merge into `associate`; retain exhaustive combination indexing and standardization fixtures; replace unchecked ranking comparisons and matrix/environment infrastructure |
| `rsomics-gradient-trajectory` | `38358d46d0ce0fb1469e46be22bcd495c4abe9ce` | refactor then merge into `trajectory`; retain four algorithms, natural ordering, ANOVA fixtures, and RSS clue; share ordination, metadata, output, and numeric contracts |

The related `rsomics-distance` foundation candidate at
`eb66c45999cbefc3b227c383cc1eb1639c48cdba` is refactored then internalized.
It is not a twentieth product asset and is not republished.

## Audit findings that block direct consolidation

1. The nineteen products are real operations but encode one workflow as
   nineteen installations, nineteen command trees, and nineteen output paths.
2. `rsomics-distance` has five historical crate consumers but only one target
   product consumer. `rsomics-pwmantel` duplicates its parser and matrix anyway,
   demonstrating that the public boundary did not produce one shared contract.
3. Alpha, beta, rarefaction, Faith PD, generalized PD, and UniFrac define
   separate count-table structs and parsers. Duplicate and empty identities,
   integer versus continuous abundance, comments, and empty-sample behavior
   differ among them.
4. The beta and UniFrac floating parsers reject values only when `count < 0`.
   `NaN` and positive infinity therefore pass validation and can propagate into
   a confident-looking distance matrix.
5. Faith PD, generalized PD, and UniFrac independently flatten the current
   mutable `rsomics-phylo-tree`. Rootedness is inferred differently, and Faith
   PD/UniFrac accept a root with fewer than two children while describing that
   test as rootedness validation.
6. Distance matrices are represented separately in `rsomics-distance`,
   beta-diversity, PCoA, UniFrac, PERMDISP, and pairwise Mantel. Validation,
   minimum size, identity matching, non-finite behavior, and serialization are
   not one contract.
7. PCA, PCoA, CA, CCA, RDA, PCoA projection, and PERMDISP repeat table,
   decomposition, axis, formatter, and result logic. PERMDISP carries another
   PCoA implementation with its own negative-eigenvalue handling.
8. ANOSIM, Mantel, pairwise Mantel, PERMANOVA, PERMDISP, rarefaction, and
   gradient analysis use multiple SplitMix64 or PCG64 implementations and
   incompatible stream-consumption rules. Several READMEs explicitly disclaim
   scikit-bio p-value identity without giving the native mode a separate
   compatibility name.
9. Fallible `faer` SVD/eigendecomposition calls are unwrapped in production
   CCA, RDA, PCoA, and PERMDISP paths. Ranking helpers also unwrap partial
   floating comparisons. Boundary checks are not consistent enough to prove
   these panics unreachable.
10. Public table, matrix, configuration, and result fields permit callers to
    bypass parser invariants. Some kernels accept raw square buffers and a
    separate `n`, allowing mismatched shapes.
11. Every historical product opens a named destination with direct
    `File::create`. A late parse, decomposition, permutation, or write failure
    can leave a truncated output.
12. Every product depends on the old `rsomics-help` and constructs a duplicate
    `HelpSpec`. None exercises the current authoritative recursive Clap layer.
13. The CCA README still contains `PERF_PLACEHOLDER`. Other speed claims live
    only in README prose or commit subjects; their raw run distributions,
    input hashes, exact command lines, memory samples, and current upstream
    versions are absent from the tracked repositories.
14. All twenty related repositories run only `ubuntu-latest`. Only beta
    diversity and PCoA install scikit-bio in CI; the other live differentials
    can print `SKIP` and still pass.
15. Historical compatibility targets are mostly scikit-bio 0.7.2. Current
    0.7.3 PCA includes iterative and dimension controls, PCoA includes a
    truncated method and negative-eigenvalue warnings, and accelerated upstream
    paths change the relevant performance baseline.
16. Source comments repeatedly narrate upstream equivalence, implementation
    steps, and obvious loops. Selected formulas and algorithms are migrated
    without retaining that comment density.

## Compatibility plan

Required oracle jobs install the pinned oracle and fail if it is unavailable.
Committed goldens remain useful regressions but do not replace live
differentials.

| Operation | Pinned oracle | Required evidence |
|---|---|---|
| alpha diversity | scikit-bio 0.7.3; vegan 2.7-5 for overlapping metrics | every advertised metric, integer/count validation, empty and zero samples, parameter limits, multi-metric ordering, and randomized tables |
| Faith PD and phydiv | scikit-bio 0.7.3 | root profile, table/tree identity, extra tips, missing/non-finite lengths, empty samples, weight boundaries, and randomized trees/tables |
| beta diversity | scikit-bio 0.7.3 with its current SciPy path; vegan 2.7-5 for ecological dissimilarities | five non-phylogenetic kernels, zero and empty vectors, non-finite rejection, identities, byte/profile serialization, and randomized tables |
| UniFrac | scikit-bio 0.7.3 | all three modes, empty pairs, root length, branch/tip validation, extra tree tips, and randomized rooted trees/tables |
| rarefaction | scikit-bio 0.7.3 and biom-format 2.1.17 | NumPy-compatible fixed seeds across both sampling regimes, depth boundaries, large counts, zero depth, and column independence |
| PCA/PCoA | scikit-bio 0.7.3; vegan 2.7-5 | dense/truncated or iterative methods, dimensions, signs/subspaces, rank deficiency, negative axes, scaling, and randomized matrices |
| CA/CCA/RDA | scikit-bio 0.7.3 and vegan 2.7-5 profiles | eigenvalues, inertia, all score blocks, scaling 1/2, constraints, fitted/residual ranks, degeneracy, and randomized tables |
| ANOSIM/PERMANOVA/PERMDISP | scikit-bio 0.7.3; vegan 2.7-5 for declared profiles | observed statistic, group alignment, ties, centroid/median, zero permutations, fixed streams, and invalid designs |
| Mantel/pairwise Mantel/BIO-ENV | scikit-bio 0.7.3 and vegan 2.7-5 profiles | methods, alternatives, exact/shared IDs, lookups, matrix streaming, subset ties, constants, and fixed streams |
| trajectory | scikit-bio 0.7.3 | all four algorithms, axis limits, natural sort, weights, window limits, group insufficiency, and ANOVA values |

Ordination comparisons align arbitrary signs and compare repeated eigenspaces
appropriately. A byte-exact coordinate table is not used as evidence when the
mathematical result is invariant under sign or basis rotation.

## Performance and memory plan

Historical commit subjects or READMEs report useful leads: alpha at 19.81
times, beta kernels at roughly 1.0 to 3.0 times single-threaded plus a parallel
gain, Faith PD at 16.83 times, generalized PD at 72 times, UniFrac at 3.5 to
13.6 times, CCA at 3.98 times, CA at 3.31 times, RDA at 7.36 times, PERMDISP at
2.16 to 10.46 times, PCoA projection at about 2 times kernel-only, pairwise
Mantel at 1.32 to 1.37 times, and trajectory at 12.47 times with lower RSS.
These are migration clues only.

- Re-run alpha as both one metric and a representative multi-metric set on
  sparse and dense count tables. Compare one validated pass with scikit-bio
  0.7.3 and vegan 2.7-5, and report output equality plus peak RSS.
- Re-run every beta kernel at a size where pairwise computation and the dense
  output are both material. Measure single-thread and selected parallel modes
  separately and include matrix serialization.
- Measure Faith PD, generalized PD, and all UniFrac modes on the same large
  tree/table so shared vectorization and memory can be compared directly with
  scikit-bio's accelerated current driver.
- Measure rarefaction separately for the small-sample and shuffle regimes while
  verifying exact count outputs for fixed seeds.
- Measure dense and truncated ordination at representative tall, wide, and
  square shapes. Report solver threads, convergence, retained dimensions,
  input preservation, and peak working memory.
- Measure each permutation statistic at zero, 999, and a larger permutation
  count. Separate matrix parsing, observed statistic, RNG/shuffle, and
  reduction; compare identical statistical work and thread count.
- Measure pairwise Mantel with enough matrices to expose the difference between
  loading all matrices and streaming two at a time.
- Measure BIO-ENV across increasing variable counts and stop before an
  infeasible exhaustive search rather than hiding combinatorial cost.

Every record includes machine, versions, source revisions, input hashes,
commands, thread controls, timing distribution, peak RSS, and output hashes.
An established replacement operation needs a strict throughput or material
resource advantage on its relevant hot path.

## Release sequence

1. Reconstruct only the required `rsomics-phylo-tree` surface: valid topology,
   traversal, finite branch views, tip identity, and the declared Newick
   profile.
2. Build one checked count/abundance table and one internal labelled distance
   matrix. Internalize the useful `rsomics-distance` assets.
3. Migrate the first coherent `alpha`, `beta`, and `rarefy` metric set through
   those models and the unified help/common layers.
4. Run format, strict Clippy, unit, golden, live-oracle, integration,
   representative performance/RSS, and four-native-target exact-head gates.
5. Publish only the complete diversity slice. Omit ordination, tests,
   association, and trajectory from help and release documentation.
6. Add one ordination model and migrate PCA/PCoA/CA, then CCA/RDA and
   projection, with solver and eigenspace gates.
7. Add one permutation and grouping model and migrate ANOSIM,
   PERMANOVA/PERMDISP, Mantel/pairwise Mantel, BIO-ENV, and trajectory.
8. Extend to additional ecological metrics and designs only through complete
   release slices.

## Explicit exclusions

- No historical micro-crate name is revived.
- `rsomics-distance` is not retained or replaced by another public matrix
  foundation during this wave.
- Differential abundance belongs in `rsomics-composition`, `rsomics-deseq`,
  `rsomics-edger`, or `rsomics-limma`; ecological diversity does not absorb
  those workflows.
- General phylogenetic inference and tree comparison remain in
  `rsomics-phylo`; ecology consumes a tree but does not infer one in the first
  release.
- Taxonomic classification and abundance profiling remain in
  `rsomics-taxonomy` and `rsomics-metagenomics`.
- Robust Aitchison transforms and compositional zero handling remain in
  `rsomics-composition`; ecology may consume their resulting distances through
  a declared matrix profile.
- Plotting and interactive dashboards are not implied by an ordination result.
  The initial product emits reusable tables and structured reports.
- The first release does not claim complete scikit-bio, vegan, QIIME 2, SciPy,
  or biom-format replacement.
- GPL upstream code is an oracle and provenance source. Team-owned historical
  Rust code may be reused directly under the confirmed project license.
