# Composition product dossier

Status: source and upstream-operation audit complete. The target repository has
not been created.

## Boundary

`rsomics-composition` is one compositional-data analysis product. It owns
Aitchison geometry, log-ratio transforms, zero handling, balance construction,
proportionality, structural-zero detection, and composition-native
differential-abundance workflows.

The primary behavior sources are:

- [scikit-bio 0.7.3 composition statistics](https://scikit.bio/docs/latest/generated/skbio.stats.composition.html)
  for the complete numerical primitive and initial inference contract;
- [ANCOMBC 2.14.0](https://bioconductor.org/packages/release/bioc/html/ANCOMBC.html)
  for ANCOM, ANCOM-BC, ANCOM-BC2, and SECOM workflows;
- [ALDEx2 1.44.0](https://bioconductor.org/packages/release/bioc/html/ALDEx2.html)
  for Dirichlet Monte Carlo, scale-aware differential abundance, effect size,
  and generalized-model workflows;
- Aitchison geometry, ILR, multiplicative replacement, rCLR, proportionality,
  ANCOM, ANCOM-BC, and Dirichlet-multinomial method papers cited by those
  packages.

The boundary is a user-recognizable analysis family, not one Python function
or R entry point. The input matrix, feature and sample identities, zero policy,
metadata alignment, numerical validation, and output provenance are shared
across its operations.

Diversity indices, ecological distances, ordination, PERMANOVA, and UniFrac
belong to `rsomics-ecology` or `rsomics-phylo`. Generic differential-expression
models belong to `rsomics-deseq`, `rsomics-edger`, and `rsomics-limma`.

## Operation map

### Initial operations

| Target subcommand | Upstream operation | Decision |
|---|---|---|
| `closure` | scikit-bio `closure` | close every composition along the selected component axis |
| `perturb` | scikit-bio `perturb`, `perturb_inv` | one-row broadcast or sample-aligned perturbation; inverse is a typed mode |
| `power` | scikit-bio `power` | Aitchison scalar multiplication |
| `centralize` | scikit-bio `centralize` | center a matrix around its compositional center |
| `inner` | scikit-bio `inner` | scalar or pairwise Aitchison inner products with explicit identity alignment |
| `alr` | scikit-bio `alr`, `alr_inv` | forward and inverse ALR with an explicit reference component |
| `clr` | scikit-bio `clr`, `clr_inv` | forward and inverse CLR |
| `rclr` | scikit-bio `rclr` | robust CLR with an explicit missing-value policy |
| `ilr` | scikit-bio `ilr`, `ilr_inv` | forward and inverse ILR under default, SBP, or tree-derived bases |
| `basis` | scikit-bio `sbp_basis`, `tree_basis` | validate and emit a named orthonormal balance basis |
| `multi-replace` | scikit-bio `multi_replace` | multiplicative zero replacement |
| `vlr` | scikit-bio `vlr`, `pairwise_vlr` | one pair or a symmetric feature-by-feature VLR matrix |
| `struc-zero` | scikit-bio `struc_zero` | structural-zero grid by metadata group |
| `ancom` | scikit-bio `ancom`; ANCOMBC `ancom` | one-way ANOVA, Holm/BH/none adjustment, W decisions, and group percentiles |

Forward and inverse transforms remain modes of the same subcommand because
they share one mathematical identity and basis contract. `pairwise-vlr`,
`ilr-inv`, or every ANCOM significance test do not become separate crates or
top-level products.

### Later inference slices

| Target subcommand | Upstream workflow | Gate |
|---|---|---|
| `ancom-bc` | scikit-bio `ancombc`; ANCOMBC `ancombc` | sampling-fraction and taxon-bias estimation, covariates, structural zeros, confidence intervals, and complete result tables |
| `ancom-bc2` | ANCOMBC `ancombc2` | multi-group, repeated-measure, trend, sensitivity, and mixed-model behavior |
| `dirmult-test` | scikit-bio `dirmult_ttest` | Dirichlet-multinomial Monte Carlo test with reproducible RNG |
| `dirmult-lme` | scikit-bio `dirmult_lme` | fixed and random effects with explicit convergence evidence |
| `aldex` | ALDEx2 | Monte Carlo CLR, two-group, paired, Kruskal-Wallis, GLM, correlation, effect-size, and scale-uncertainty contracts |
| `secom` | ANCOMBC SECOM | sparse compositional correlation with its own numerical and sparsity oracle |

These names stay out of CLI help and release documentation until their full
declared contracts pass. The first release is not delayed merely to reserve
them.

## Input, identity, and zero contracts

- A table has unique nonempty sample and component identities plus a checked
  rectangular numeric matrix. TSV and CSV are initial formats. A BIOM adapter
  requires separate format fixtures; it is not simulated by accepting a
  vaguely delimited file.
- The component axis is explicit in the library API. The CLI's normal table
  form is samples by components and never silently transposes input.
- Operations combining a table, composition, basis, tree, or second matrix
  align by component identity. Matching widths with different identities is
  an error unless the user explicitly selects positional mode.
- A perturbation operand is either one named composition broadcast to all
  samples or a table aligned one-to-one by sample identity. Row order alone
  does not pair two labeled tables.
- Metadata is aligned by unique sample identity. Missing, duplicated, or
  unused rows are diagnosed before inference.
- Finiteness, negativity, zeros, and missing values are operation-specific
  policies. `rclr` may preserve missing observations; ordinary log-ratio
  transforms require positive finite values; count-based inference does not
  acquire a hidden pseudocount.
- Zero replacement accepts a finite positive `delta` whose total replacement
  mass is valid for every row. The chosen method and parameter are recorded in
  the execution report.
- Inverse transforms reject non-finite coordinates and verify that every
  output row is finite, non-negative, and closed within a documented numerical
  tolerance.
- SBP rows, basis dimensions, orthonormality, tree topology, unique tip names,
  and correspondence between tips and component identities are validated
  before a transform.

Pseudocount addition and multiplicative replacement are distinct policies.
Passing `--pseudocount` to a transform is an explicit convenience composition,
not a claim that scikit-bio itself performs that preprocessing.

## Numerical and inference contracts

- Closure and all log-ratio operations use scaling and log-sum formulations
  that avoid avoidable overflow and underflow.
- Forward and inverse transforms preserve component identities and record the
  selected reference or balance basis.
- VLR defines degrees of freedom and zero masking explicitly. Full pairwise
  output is symmetric with an exact zero diagonal and is emitted without
  retaining a second dense copy.
- ANCOM records the significance test, p-value adjustment, alpha, tau, theta,
  group ordering, percentile requests, and upstream compatibility version.
- The initial ANCOM contract supports the upstream default one-way ANOVA plus
  Holm, Benjamini-Hochberg, or no adjustment. Paired, rank-based, repeated,
  or arbitrary callable tests are absent from help until their design and
  identity contracts are implemented and differentially tested.
- Constant ratios, zero within-group variance, insufficient residual degrees
  of freedom, failed model convergence, and non-finite test statistics receive
  explicit outcomes. They are not converted to significance by incidental
  NaN ordering.
- Monte Carlo workflows record the RNG algorithm, seed, draw count, filtering,
  denominator policy, scale model, and effective parallelism.
- Statistical output includes effect direction and uncertainty where the
  upstream method defines them. A boolean significance column alone does not
  replace an effect estimate.

## Target structure

```text
src/
├── cli.rs
├── table/
│   ├── model.rs
│   ├── delimited.rs
│   ├── metadata.rs
│   └── align.rs
├── geometry/
│   ├── closure.rs
│   ├── perturb.rs
│   ├── power.rs
│   └── inner.rs
├── transform/
│   ├── alr.rs
│   ├── clr.rs
│   ├── rclr.rs
│   └── ilr.rs
├── basis/
│   ├── model.rs
│   ├── sbp.rs
│   └── tree.rs
├── zeros/
│   ├── replacement.rs
│   └── structural.rs
├── proportionality.rs
├── inference/
│   ├── ancom.rs
│   ├── tests.rs
│   └── summary.rs
├── output.rs
└── report.rs
```

Later inference methods receive their own modules when implemented. The
initial tree does not contain empty `ancom_bc`, `aldex`, or `dirmult` modules.

The library exposes checked table, basis, transform, and result types needed
for programmatic use. Parsers, allocation strategy, CLI option models, and
product-specific inference policy remain private.

## Foundation relationships

`rsomics-common` owns error-to-exit mapping, JSON execution envelopes, aliases,
and transactional named output. `rsomics-help` owns the complete recursive
Clap command presentation. There is no duplicate help specification or argv
interceptor inside this product.

`rsomics-composition` is a concrete consumer of general p-value adjustment and
statistical-test primitives from `rsomics-stats`. A public stats item moves
there only when composition and a second named product exercise the same
finite-value, tail, tie, and degeneracy contract. ANCOM's log-ratio matrix,
W statistic, cutoff staircase, percentile layout, and bias-correction policy
stay private.

Tree-derived ILR bases make composition a concrete consumer of the validated
Newick and immutable topology contract in `rsomics-phylo-tree`, alongside
`rsomics-phylo` and `rsomics-ecology`. Tip-to-component alignment and balance
construction stay in composition.

The shared labeled matrix is not a new public foundation. It first lives in
this product. `rsomics-composition` does not depend on the Layer B
`rsomics-table`, `rsomics-ecology`, or differential-expression products.

## Historical asset disposition

The ten operation-sized GitHub repositories and crates.io packages are
deleted. Their clean external-disk clones remain implementation assets.

| Source asset | Revision | Disposition |
|---|---|---|
| `rsomics-aitchison-ops` | `2dcf4147902a7f130d3121f1cf679838faadbe4c` | refactor then merge; retain closure, perturbation, power, centralization, inner-product algorithms, properties, and goldens |
| `rsomics-alr` | `99869f5f8f1633179e6d5c366000d8199836d895` | refactor then merge; retain forward/inverse kernels, reference-index cases, and goldens |
| `rsomics-ancom` | `aa6e72a86403560daa311201474143df0c2a3812` | refactor then merge; retain ANOVA, Holm/BH, W-cutoff code, degenerate fixtures, and oracle cases |
| `rsomics-clr` | `ecbf335ae4fff31fea5a7ebfbbb86928d564e77d` | refactor then merge; retain forward/inverse kernels, properties, and goldens |
| `rsomics-ilr` | `799ab17a5d0f30a2d5c76db61803da4af27f3cae` | refactor then merge; retain the linear-time default-basis transform and round-trip evidence |
| `rsomics-ilr-basis` | `c2c5e59f98e34d5a373c2ea2cd604e0ecc2082bb` | refactor then merge; retain SBP/tree basis formulas and goldens, replace the positional table I/O and bespoke Newick parser |
| `rsomics-multi-replace` | `7f5c5d4f6b695fa8e591dca50676993514e0e004` | refactor then merge; retain the closed-form replacement kernel and fixtures |
| `rsomics-rclr` | `e19d52d1e3680917f7ecfaef66f7263ce7c08986` | refactor then merge; retain observed-mask behavior and NaN-position goldens |
| `rsomics-struc-zero` | `bbdfd792a07adc7350bf3185a3c47c2b06dcc211` | refactor then merge; retain groupwise detection, negative-lower-bound cases, and fixtures |
| `rsomics-vlr` | `68b9ce52155878fa7229329cdfccff3e10bfc0c9` | refactor then merge; retain the covariance identity, robust pair cases, faer experiment, and goldens |

The source code and fixtures in these repositories are team-owned and may be
reused. Their repeated `Table`, CLI, output, runtime, and help implementations
are discarded rather than merged ten times.

## Existing implementation gaps

- Nine repositories carry separate labeled-table parsers. Four parser files
  are byte-identical, and all ten binaries repeat nearly the same main/runtime
  path. The current shape is operation-per-crate duplication, not useful
  modularity.
- Most named outputs are opened with `File::create` before the operation
  finishes. A parse, numerical, or write failure can leave a plausible partial
  result.
- Aitchison perturbation and inner products compare dimensions without
  comparing component identities. ALR selects its reference only by position,
  and the custom-basis path discards all table identities and applies a tree
  basis positionally.
- CLR, ALR, ILR, and custom-basis inverse paths do not consistently reject
  non-finite coordinates. Several can complete successfully with NaN output.
- `multi_replace` accepts a negative or NaN `delta`; the row check then permits
  negative or NaN replacement values.
- The custom-basis Newick parser implements only a narrow grammar, does not
  provide the claimed comment handling, and does not connect tip names to
  matrix columns.
- The historical ANCOM result omits percentile summaries and supports only
  one significance test and three correction modes. It predates scikit-bio
  0.7.3's current vectorized contract and the broader composition-native
  inference scope.
- The full ANCOM p-value matrix and VLR result are dense. Their allocation,
  feature limits, streaming output, and failure behavior are not governed by
  a product-level memory contract.
- Live scikit-bio differentials return success after printing a skip message
  when Python or the oracle package is absent.
- Every historical CI workflow runs only Ubuntu `x86_64` and uses the old
  `rsomics-common` 0.6 and `rsomics-help` 0.3 contracts.
- The source contains extensive API narration, upstream history, and
  line-by-line algorithm comments. Reconstruction keeps only user contracts
  and non-obvious numerical or compatibility invariants.

## Retained evidence

All ten assets contain unit tests. Nine contain Criterion microbench recipes.
Every operation has committed scikit-bio-derived goldens and an optional live
differential; the retained cases cover:

- closure, perturbation, inverse perturbation, power, centralization, and
  Aitchison inner products, including zeros and dimension mismatch;
- ALR, CLR, and ILR forward/inverse examples, scale invariance, reference
  positions, custom SBP and tree bases, orthonormality, and round trips;
- multiplicative replacement under default and custom delta;
- rCLR masks, NaN locations, zeros, negative values, and all-zero rows;
- pair and pairwise VLR, degrees of freedom, symmetry, and robust zero masks;
- structural-zero group grids with and without a negative lower bound;
- ANCOM Holm and BH results, zero and non-finite inputs, and degenerate groups.

These are migration inputs, not current release proof. Goldens are recaptured
against pinned current oracle versions with command, environment, package
hash, and tolerance provenance.

The `rsomics-ilr-basis` README claims 5.2 times lower wall time and 2.8 times
lower peak RSS than scikit-bio on a 50,000 by 50 matrix, but no tracked
measurement record supports the claim. It is not carried into product
documentation without remeasurement. The Criterion harnesses measure only
Rust kernels on generated matrices and contain no upstream comparison.

## First release slice

The first release contains every initial operation in the operation map:

- one strict labeled-table and metadata contract;
- Aitchison closure, perturbation, power, centralization, and inner products;
- ALR, CLR, rCLR, and ILR, including inverse transforms where defined;
- default, SBP, and tree-derived ILR bases with identity alignment;
- multiplicative zero replacement;
- pair and pairwise VLR;
- structural-zero detection;
- ANCOM with pinned scikit-bio one-way ANOVA, Holm/BH/none adjustment, W
  decisions, and requested percentile summaries;
- stdin and stdout where one-pass ownership is unambiguous, transactional
  named outputs, and complete execution reports;
- a unified `rsomics-help` command tree with no advertised later subcommands.

ANCOM-BC, ANCOM-BC2, Dirichlet-multinomial tests, ALDEx2, and SECOM are later
slices. The package is not published until the complete first slice passes.

## Compatibility gates

- Pin scikit-bio 0.7.3 for the initial numerical oracle. Refresh the pin before
  publication if a newer release changes semantics.
- Run live differentials for every initial operation. A missing oracle fails
  the compatibility job; committed goldens remain the all-platform regression
  layer.
- Compare values, shapes, sample/component order, missing-value masks,
  boolean grids, W statistics, significance decisions, percentiles, and
  failures. Normalize only documented float formatting and path/version text.
- Cover duplicate and empty identities, reordering, transposition requests,
  dimension mismatch, malformed rows, zero and all-zero compositions,
  negative and non-finite values, extreme magnitudes, invalid delta,
  insufficient degrees of freedom, constant ratios, degenerate groups, and
  metadata mismatch.
- Exercise default, permuted, SBP, and tree bases; malformed and nonbinary
  SBPs; duplicate or missing tree tips; quoted labels, comments, branch
  lengths, polytomies, and non-orthonormal explicit bases.
- Verify algebraic properties independently of scikit-bio: closure
  idempotence, perturbation inverse, scale invariance, CLR zero sum,
  forward/inverse closure recovery, basis orthonormality, VLR symmetry, and
  deterministic inference.
- Exact-head CI runs strict Clippy, tests, package verification, and native
  execution on Linux and macOS for `x86_64` and `aarch64`.

ANCOMBC and ALDEx2 later receive pinned R-oracle workflows. ALDEx2 is GPL-3.0
or later, so its source is an oracle and provenance reference, not mergeable
implementation material. Independent implementation is derived from public
method contracts and papers.

## Performance gates

Measurements record product revision, oracle version, machine, threads, input
generator or dataset hash, dimensions, sparsity, flags, warmup, timing
distribution, peak RSS, output hash, and numerical tolerance.

The initial profiles include:

- dense transforms on a matrix large enough to exceed cache and expose parse,
  allocation, compute, and write costs separately;
- sparse rCLR and zero replacement on a representative microbiome-like count
  table;
- VLR at feature counts where the quadratic output and memory traffic are
  material;
- ANCOM across representative sample, feature, group, and constant-ratio
  regimes.

Each profile compares end-to-end CLI behavior and isolated kernels against the
pinned upstream. Python or R startup is reported separately and is not the
sole basis of a speed claim. Thread scaling must show useful work and bounded
memory. Full-matrix operations must fail before allocation when checked
dimensions exceed addressable or configured limits.

At least one relevant hot path must show a strict throughput or peak-memory
advantage over its established oracle before publication. Existing README
claims and Rust-only Criterion timings do not satisfy this gate.

## Explicit exclusions

- No plotting, dashboard, or report-design subcommands.
- No generic matrix calculator or delimited-table transformation surface.
- No diversity, ordination, phylogenetic distance, or ecological permutation
  analysis.
- No DESeq2, edgeR, limma, or single-cell differential-expression aliases.
- No public crate for each transform, test, matrix, basis, or zero policy.
- No copying of GPL upstream source and no claim that team-owned historical
  Rust code changes the license obligations of external oracle packages.
