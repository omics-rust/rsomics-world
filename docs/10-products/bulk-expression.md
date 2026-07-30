# Bulk-expression product dossiers

Status: joint boundary, upstream-operation, and historical-source audit
complete. No target repository has been created.

## Portfolio decision

Retain three products:

- `rsomics-deseq` for the DESeq2 statistical workflow;
- `rsomics-edger` for the edgeR statistical workflow;
- `rsomics-limma` for the limma and voom statistical workflow.

Reject `rsomics-expression` as a public product.

The three retained products start from related matrices, designs, and
contrasts, but they do not expose interchangeable methods over one fitted
object. Each has a recognizable installation identity, state model,
normalization and filtering policy, dispersion or variance model, hypothesis
tests, diagnostics, and result semantics. Combining them behind a generic
`--method` switch would hide meaningful statistical choices and create one
version that must move in lockstep with three upstream packages.

Shared numerical code may move to `rsomics-stats` only after at least two of
these products exercise the same policy-free contract. The products never
depend on one another.

The current upstream anchors reviewed on 2026-07-31 are:

| Product | Oracle | Reviewed release | Recognizable workflow |
|---|---|---:|---|
| `rsomics-deseq` | [DESeq2](https://bioconductor.org/packages/release/bioc/html/DESeq2.html) | 1.52.0, Bioconductor 3.23 | integer counts and sample metadata → size factors → dispersion model → negative-binomial GLM → Wald or LRT results → optional effect shrinkage and transforms |
| `rsomics-edger` | [edgeR](https://bioconductor.org/packages/release/bioc/html/edgeR.html) | 4.10.1, Bioconductor 3.23 | count matrix and library metadata → expression filtering → normalization factors → quasi-likelihood or classic NB model → contrasts and result tables |
| `rsomics-limma` | [limma](https://bioconductor.org/packages/release/bioc/html/limma.html) | 3.68.4, Bioconductor 3.23 | log-expression or RNA-seq counts → linear model or voom precision model → empirical-Bayes moderation → contrasts, threshold tests, diagnostics, and gene-set analysis |

This dossier treats package documentation and the DESeq2, edgeR, and limma
user guides as the behavior source. Historical rsomics READMEs are evidence
about the old implementation, not authority for current upstream behavior.

## Shared input and execution contract

All three products need the same outer workflow discipline without pretending
that their internal objects are the same.

- Count matrices have unique feature and sample identities, non-negative
  integer values where the method requires counts, declared missing-value
  policy, and checked dimensions.
- Sample metadata is joined by identity, never by unchecked row position.
- Design formulas, factor reference levels, numeric covariates, interactions,
  blocking terms, and contrasts are parsed into a named, rank-checked design.
- A failed or non-converged fit is represented explicitly. It is not silently
  converted to zero, `NA`, or an apparently valid row.
- A run records product version, upstream oracle version, input digests, design,
  contrasts, options, filtering decisions, convergence state, and thread count.
- Named multi-file output is transactional. A late result or provenance write
  must not leave a complete-looking partial analysis.
- The primary result table retains feature order and stable names while
  exposing method-specific statistics. A generic lowest-common-denominator
  table is not the fitted-state model.
- Machine-readable output and human diagnostics use the shared
  `rsomics-common` process contract. Every command uses the current
  `rsomics-help` Clap adapter.

Each product defines a versioned analysis bundle before its first release. The
bundle must preserve enough fitted state to derive another declared contrast or
export without silently refitting under different defaults. The three bundle
schemas may share physical encodings, but they remain product-owned until a
second consumer proves a stable public type.

## `rsomics-deseq`

### Boundary

`rsomics-deseq` is a DESeq2-compatible analysis product for count data. It is
not a directory of commands corresponding to every exported R function.

The upstream dataset joins integer assays, row metadata, sample metadata, a
design formula, size factors, dispersion estimates, model coefficients,
diagnostics, and result metadata. The rsomics product must preserve those
dependencies across a complete run.

### Operation map

| Upstream contract | Target surface | Decision |
|---|---|---|
| dataset construction from matrix, HTSeq, or summarized inputs | `run` input adapters | matrix plus explicit metadata first; additional adapters only with their own schema tests |
| `estimateSizeFactors`, normalization factors | internal fit stage; normalized-count export | not a standalone public product or mandatory subcommand |
| `estimateDispersions` and trend/MAP estimates | internal fit stage and diagnostics | preserve gene-wise, fitted, final, and outlier state |
| `DESeq`, `nbinomWaldTest`, `results` | `run` and `results` | complete design/contrast-aware Wald workflow |
| `nbinomLRT` | `run --test lrt` | full and nested reduced designs, with result semantics matching the oracle |
| `lfcShrink` | `results --shrink` | method is explicit; normal, apeglm, and ashr are not conflated |
| normalized counts, `fpm`, and `fpkm` | `export` | keep length and library-size definitions in provenance |
| `normTransform`, `vst`, and `rlog` | `transform` | blind/design-aware behavior and fit reuse are explicit |
| plots and diagnostics | `inspect` data plus optional report | emit the values needed for MA, dispersion, PCA, Cook's-distance, and sample-distance views before adding a renderer |
| `unmix` | later product-local operation | outside the first release; it does not define the DE workflow |

The first release slice is one complete matrix-and-metadata to Wald-results
workflow with arbitrary full-rank designs and named contrasts. It includes
normalization, dispersion fitting, independent filtering, Cook's-distance
handling, adjusted p-values, a versioned bundle, normalized-count export, and
one stabilized transform. It does not release a collection of independently
recomputed helper commands.

LRT, shrinkage variants, additional transforms, and `unmix` remain
undocumented or feature-gated until their complete product-level
compatibility tests pass.

### Historical asset disposition

| Historical asset | Audited revision | Disposition |
|---|---|---|
| `rsomics-deseq-estimate-dispersions` | `30cfe948a68924b308d2cd61867fb194860fc4ca` | refactor then merge; central dispersion source and fixture asset |
| `rsomics-deseq-fpkm` | `ec263aa17cc91718fc5118a8436c971d22e30bb2` | test, fixture, and export-formula asset; do not keep as a binary |
| `rsomics-deseq-fpm` | `376778fd58599f8026298a12178e7cc05b5bd0e1` | test, fixture, and export-formula asset; deduplicate with normalized-count state |
| `rsomics-deseq-lfc-shrink` | `13275091e33acd3090b69508587f5718a5c06cd6` | refactor then merge for the normal-prior path; current upstream alternatives need separate implementations and evidence |
| `rsomics-deseq-lrt` | `a011348a0a61f20de84024b6ba59e56afc27c3a7` | refactor then merge; retain full/reduced design fixtures |
| `rsomics-deseq-norm-transform` | `9c4e4f2f8b9c9859e0c7b1cdfdfec56ef7feaa2b` | refactor then merge into transforms |
| `rsomics-deseq-prep` | `522a9ff561e5e559b23a05d2433a119a34b2559b` | fixture and policy-warning asset only; fixed thresholds do not replace method-aware filtering |
| `rsomics-deseq-results` | `94b1cea1e19f1e02fcf829a7f3cec7bb2f51dc5b` | refactor then merge as the initial Wald seed |
| `rsomics-deseq-rlog` | `44f7e815c2174366fd3696ae281018fcbdf9809b` | refactor then merge after general-design and blind-mode review |
| `rsomics-deseq-sizefactors` | `2aae52e2a4fc0d31d51c732d79f8dad38db7cf13` | refactor then merge; deduplicate every copied median-of-ratios implementation |
| `rsomics-deseq-unmix` | `9713ce792a3f077190f9e9488f398d2b62fade84` plus an inherited `src/cli.rs` diff | test and algorithm asset for a later operation; resolve ownership before copying |
| `rsomics-deseq-vst` | `4178daad4d14b7bb4a3c6121b631464d35aeccef` | refactor then merge into transforms |

The initial source seed is `rsomics-deseq-results`, but no repository is
merged wholesale. `rsomics-deseq-results` and
`rsomics-deseq-lfc-shrink` contain exact copies of matrix and results modules
and near-copies of the dispersion and GLM pipeline. Consolidation creates one
typed fit path rather than choosing one fork for each public command.

### Existing implementation gaps

- The main Wald and normal-shrinkage binaries accept a comma-separated
  two-group label vector rather than general sample metadata and a design.
- Standalone operations recompute size factors, dispersions, and GLMs, so
  separately invoked results and transforms do not share one auditable fitted
  state.
- The normal-prior shrinker does not cover current `apeglm` or `ashr`
  behavior, s-values, interaction constraints, or method-specific output.
- Several repositories preserve useful committed R goldens, but live DESeq2
  checks search private local R environments and return success after a loud
  skip when the oracle is unavailable.
- The goldens do not form a central matrix of upstream version, design class,
  contrast class, low-count behavior, outliers, convergence, and platform.
- CLI implementations use the retired duplicate `HelpSpec` model and direct
  file creation.
- Every repository has only Ubuntu `x86_64` CI.

## `rsomics-edger`

### Boundary

`rsomics-edger` is an edgeR-compatible count-analysis product. Its primary
identity is the `DGEList`-style lifecycle and edgeR's current
quasi-likelihood workflow, not a collection of CPM and distribution
calculators.

edgeR 4 changes the QL model relative to edgeR 3: `estimateDisp` is optional
for `glmQLFit`, and the quasi-dispersion treatment for small means or large
NB dispersions changed. Historical goldens without an exact upstream version
cannot establish current compatibility.

### Operation map

| Upstream contract | Target surface | Decision |
|---|---|---|
| `DGEList`, `readDGE`, library and gene metadata | `run` input adapters and bundle | preserve counts, sample metadata, offsets, library sizes, normalization factors, and feature metadata |
| `filterByExpr` | internal filter stage; optional decision export | group/design choice is explicit and recorded |
| `calcNormFactors` | internal normalization stage; `export factors` | TMM first; other methods are named and oracle-tested |
| `glmQLFit` and `glmQLFTest` | `run --pipeline ql` and `results` | current default first-release workflow |
| `glmFit` and `glmLRT` | `run --pipeline lrt` | later complete slice |
| `exactTest` | `run --pipeline exact` | later classic two-group slice |
| `glmTreat` | results threshold option | tested against a biological fold-change threshold, not folded into ordinary QL output |
| `estimateDisp` and robust paths | fit diagnostics and compatibility modes | upstream-version-specific behavior is explicit |
| `cpm`, `aveLogCPM`, `cpmByGroup`, `rpkm` | `export` | derived values share the fitted library state |
| `diffSpliceDGE` and `topSpliceDGE` | later `splice` operation | exon-to-gene metadata is a required typed input |
| `binomTest`, Good-Turing helpers, predictive FC | later or internal | no public surface until a real workflow requires them |

The first release slice is a current edgeR 4 QL analysis from integer counts
and sample metadata through `filterByExpr`, TMM normalization,
`glmQLFit`, `glmQLFTest`, contrast results, and a versioned bundle. A fixed
dispersion default is not a replacement for fitting the current workflow.

### Historical asset disposition

| Historical asset | Audited revision | Disposition |
|---|---|---|
| `rsomics-avelogcpm` | `7871fc7b72b149e3e7fac7b3f1a595062d7004a5` | refactor then merge into derived expression exports |
| `rsomics-cpm` | `128e1f0e61070c7844b1fb74e2292280aff21216` | refactor then merge with one library-state implementation |
| `rsomics-edger-binom-test` | `f0188271f2919e68171c32bf78dc576e4514c4ee` | test and later-operation asset |
| `rsomics-edger-cpm-by-group` | `698bf089a60cd726e3dc12a030d70c0a57d0b52c` | refactor then merge into exports |
| `rsomics-edger-diff-splice` | `b9c4afdd3f6db10259e1ce38523c38a2e6c1a21e` | refactor then merge after the core fit and gene/exon contract are stable |
| `rsomics-edger-estimate-disp` | `3eda06381596b5c574ac18448c57d7a0255b17b0` | algorithm, fixture, and legacy-compatibility asset; reconcile with edgeR 4 before merge |
| `rsomics-edger-exact-test` | `ab70f259c2ce1c5db11b50e01f31e99e44236a17` | refactor then merge for the later classic pipeline |
| `rsomics-edger-glm-lrt` | `e54fd78695c9883bc679bc5b1d18ffd9e4fc094a` | refactor then merge behind the shared design and fit state |
| `rsomics-edger-glm-qlf` | `c2f26cf9d9dc755961a8f7ea529ff28ca65262d8` | refactor then merge as the initial QL seed; replace fixed-dispersion workflow |
| `rsomics-edger-glm-treat` | `0206008c7f792ca207b9f2bb58542019af8f22b4` | refactor then merge after QL/LRT result contracts |
| `rsomics-edger-goodturing` | `b1313f13c9f22780e4becb10fc4b27b9a27bafb6` | test and niche-operation asset only |
| `rsomics-edger-predfc` | `86f1ede753ca140127912c130e14c9a2a673a485` | refactor then merge only with ranking/plotting consumers |
| `rsomics-edger-robust-disp` | archive-only local tree, no Git revision | algorithm and fixture asset only; establish provenance and current upstream relevance before copying |
| `rsomics-edger-rpkm` | `c43d59ac9f25bcad9f3c83958b702bdfa29053cf` | refactor then merge into exports |
| `rsomics-filter-by-expr` | `451580a86a4c36de536605fc08037f2c4b1b83c7` | refactor then merge into the first QL workflow |
| `rsomics-tmm-norm` | `3f37f84a8935c419a37378bc43dc09930da83067` | refactor then merge into normalization |
| `rsomics-uq-norm` | `9f53b4c7af2717ccca8d90a48762c957465cc837` | later normalization-method asset |

### Existing implementation gaps

- The QL, LRT, and exact-test crates each own another matrix parser, design
  representation, numerical helper set, p-value adjustment, writer, and CLI.
- The historical QL CLI defaults to a caller-supplied constant dispersion of
  `0.05`; it does not execute edgeR 4's complete fitted QL lifecycle.
- Current edgeR accepts counts from multiple omics types, offsets, observation
  weights, general designs, contrasts, and feature metadata. The old binaries
  cover disconnected subsets.
- The live R differentials are optional in Ubuntu-only CI. Committed goldens
  are useful but do not establish an exact current edgeR version across every
  operation.
- Numerical functions are copied between repositories and exposed as
  product-public APIs without a reviewed cross-product contract.
- Output is ad hoc TSV rather than a durable fitted analysis with diagnostics
  and provenance.

## `rsomics-limma`

### Boundary

`rsomics-limma` is a limma-compatible linear-model and empirical-Bayes
analysis product for log-expression and other omics matrices, including
RNA-seq through the current voom workflow.

The product is broader than `voom`, and `voom` is not an independent product.
The current limma guide recommends `voomLmFit` as the RNA-seq interface in
place of manually composing `voom`, `lmFit`, and
`voomWithQualityWeights`; it also incorporates sample weights and intrablock
correlation. Historical pieces therefore need to be recomposed around current
behavior rather than exposed one-for-one.

### Operation map

| Upstream contract | Target surface | Decision |
|---|---|---|
| log-expression matrix and named design | `run --input log-expression` | general linear model with optional contrasts |
| RNA-seq count matrix and library metadata | `run --input counts` | current `voomLmFit`-compatible path |
| `lmFit`, `contrasts.fit`, `eBayes`, `topTable` | `run` and `results` | first-release fitted lifecycle |
| `treat`, `topTreat`, `decideTests` | `results` options and decision export | threshold semantics remain explicit |
| `voomLmFit`, voom trend, sample weights | count-input fit and diagnostics | current behavior first; legacy composition only as a pinned mode if needed |
| `arrayWeights` and `duplicateCorrelation` | fit options and diagnostics | block/sample identities are typed and joined by name |
| `normalizeBetweenArrays` and quantile normalization | `normalize` | only for declared suitable input types |
| `removeBatchEffect` | `transform batch` | for visualization/exploration; not silently substituted for modeling batch in the design |
| `diffSplice` and `topSplice` | later `splice` operation | typed feature-to-gene mapping |
| `camera` and `fry` | later `geneset` operation | GMT parsing, universe, direction, correlation, and multiple testing are explicit |
| `genas` | later association operation | requires two fitted contrasts |
| `squeezeVar` and `propTrueNull` | internal numerical/statistical stages | not separate binaries |

The first release slice fits either a named log-expression matrix or an
RNA-seq count matrix, accepts a general full-rank design and named contrast,
executes the appropriate linear/voom model, performs empirical-Bayes
moderation, emits complete results and diagnostics, and writes a versioned
bundle. `treat` may ship in that slice only if thresholded inference passes the
same oracle and design matrix.

### Historical asset disposition

| Historical asset | Audited revision | Disposition |
|---|---|---|
| `rsomics-edger-camera` | `d05098f33384ba884016e3009d5aa22af35c8edb` | refactor then merge into later limma gene-set analysis; the name is discarded |
| `rsomics-limma-array-weights` | `546aefb55e51d2fad7a8586ae3f8aaddfe99665d` | refactor then merge behind fitted sample-weight state |
| `rsomics-limma-decide-tests` | `44bb7e715e66f24babbc574c63eb668292fc0e56` | refactor then merge into results |
| `rsomics-limma-diff-splice` | `dffbb49cae01e453af00ecc9c3c93f8d2af65ee3` | refactor then merge after core model and feature mapping |
| `rsomics-limma-duplicate-correlation` | `146dbc5f2cd793b250b54013f3a059940d2b9e5c` | refactor then merge with named block data and current voom path |
| `rsomics-limma-ebayes` | `8fbfa0b8fcb9444dcf5740a4c4feab7a398e9c30` | refactor then merge as the log-expression seed |
| `rsomics-limma-fry` | archive-only local tree, no Git revision | algorithm, R oracle, and fixture asset for later gene-set analysis |
| `rsomics-limma-genas` | archive-only local tree, no Git revision | algorithm, R oracle, and fixture asset for a later operation |
| `rsomics-limma-proptruenull` | `9299ec337f5b3eeb952f144ffe6bd994061ead63` | internalize; promote a policy-free estimator to `rsomics-stats` only with a second consumer |
| `rsomics-limma-squeeze-var` | `f95ae44a3a4511569335c74c0c0446f9eee5eafb` | refactor then merge into empirical-Bayes moderation |
| `rsomics-limma-treat` | `37f3790d5ef3670ce4bbb8d5ed39b4de43eef955` | refactor then merge into thresholded results |
| `rsomics-limma-vooma` | `b96f4fdc423d33418a5f26f1ef38f9a034c10738` | later non-count mean-variance asset |
| `rsomics-quantile-norm` | `8167d3346376573f06914fc5508a5c55e759ad98` | refactor then merge into input-aware normalization |
| `rsomics-remove-batch-effect` | `78381156c72cdeedebae0d09f4036d3ed2bf8ecc` | refactor then merge as an explicitly exploratory transform |
| `rsomics-voom` | `f51e528200ef16259f04451e1a33becc22932d57` | algorithm and fixture asset; recompose around current `voomLmFit` |
| `rsomics-voom-quality-weights` | `18ad63de6ed12827da6edc2625fd2f99cd7c36cc` | algorithm and fixture asset; recompose around current sample-weight behavior |

`rsomics-ebayes-core` has only limma consumers in the historical graph. Its
useful empirical-Bayes code is internalized in `rsomics-limma`; the old public
foundation is not revived.

### Existing implementation gaps

- The repositories duplicate expression/design/contrast parsers, QR and matrix
  code, special functions, empirical-Bayes code, multiple testing, and result
  writers.
- The main `rsomics-limma-ebayes` seed has no observation-weight,
  block-correlation, robust/trend, missing-value, or complete rank-deficiency
  model.
- The historical `voom` and quality-weight binaries implement the older manual
  composition, while current limma recommends `voomLmFit`.
- Gene-set, splicing, batch, correlation, and decision helpers consume ad hoc
  tables rather than one fitted state.
- Live limma differentials are optional and can skip in Ubuntu-only CI.
- A historical README claims a 13.55-times speedup and five-times RSS
  reduction for quality weights but references a performance document absent
  from the repository. The claim is not retained as release evidence.

## Rejected `rsomics-expression` boundary

The historical boundary had two candidates:

| Candidate | Audited revision | Decision |
|---|---|---|
| `rsomics-count-matrix` | `9a92e84f4470baa5d62a6ebcde81d56e452ee86d` | route to the `rsomics-count` source pool; retain as a collation fixture and fallback implementation, not a separate product |
| `rsomics-de-volcano` | `ce985969c2ba4b913d6c646b3e2c1c2d98af7c30` | move to a rejected DE-reporting capability pool; internalize only when a real product report consumes it |

`rsomics-count-matrix` concatenates featureCounts or htseq-count tables. The
target `rsomics-count` product already requires multi-input matrix output, so a
separate `matrix` operation ships only if legacy per-sample file collation
remains a demonstrated workflow.

`rsomics-de-volcano` does not render a volcano plot. It adds a significance
category from two selected columns and fixed thresholds. The code is 180 lines,
has no upstream behavior oracle, knows only an ad hoc TSV schema, and provides
no plot, labels, metadata, provenance, or report. DESeq2, edgeR, and limma
results also have different missing-value and significance semantics.

The `rsomics-expression` GitHub endpoint returned 404 on 2026-07-31, and the
registry reset gate had already verified the name absent from crates.io.
Removing it from the allowlist changes no live repository, package, or user
dependency.

No generic expression matrix foundation is created. A typed matrix, metadata
join, design parser, or result annotation may become shared only when two
implemented products demonstrate the same contract without erasing
method-specific policy.

## Historical source audit

The three retained products have 45 routed micro-crate candidates containing
220 Rust source files and 39,774 source lines. Exact-file hashing found 174
unique files and six duplicate groups. Forty-two repositories use the same
`main.rs`;
several substantial matrix, results, lowess, special-function, fit, and
empirical-Bayes implementations are copied or forked.

Useful evidence exists:

- method-specific R oracle scripts;
- committed result, transform, weight, and dispersion goldens;
- edge cases for zero rows, singleton designs, constant genes, contrasts, and
  thresholded results;
- algorithm implementations substantially larger than placeholders;
- Criterion recipes that can be converted into direct kernel benchmarks.

That evidence does not make any old repository release-ready:

- every micro-crate has its own public library and binary boundary;
- fit state is repeatedly flattened into unrelated TSV files;
- format and design validation vary by operation;
- live R tests usually pass after reporting a skip when a local R environment
  is missing;
- all inspected CI workflows run only on Ubuntu `x86_64`;
- source metadata points either to the control plane or a deleted micro-crate;
- output paths are commonly opened directly rather than transactionally;
- the old duplicated `HelpSpec` tree appears throughout the source pool;
- many explanatory comments narrate upstream reconstruction rather than
  preserving only stable invariants.

Selected algorithms, goldens, and fixtures are merged into product modules.
The deleted micro-crate repositories are never revived.

## Target structures

The exact private module boundaries can evolve during implementation. The
intended ownership is:

```text
rsomics-deseq/src/
├── cli.rs
├── input.rs
├── design.rs
├── dataset.rs
├── size_factor.rs
├── dispersion.rs
├── nb_glm.rs
├── test.rs
├── shrinkage.rs
├── transform.rs
├── results.rs
├── artifact.rs
└── commands/

rsomics-edger/src/
├── cli.rs
├── input.rs
├── design.rs
├── dge.rs
├── filter.rs
├── normalization.rs
├── dispersion.rs
├── glm.rs
├── ql.rs
├── exact.rs
├── splice.rs
├── results.rs
├── artifact.rs
└── commands/

rsomics-limma/src/
├── cli.rs
├── input.rs
├── design.rs
├── fit.rs
├── ebayes.rs
├── voom.rs
├── weights.rs
├── correlation.rs
├── transforms.rs
├── splice.rs
├── geneset.rs
├── results.rs
├── artifact.rs
└── commands/
```

Operation modules expose narrow typed interfaces inside their product.
Product-specific defaults, fitted objects, row filtering, dispersion models,
and result policies do not enter a public foundation.

## Consumer-driven foundation work

The three products are concrete consumers for a limited `rsomics-stats`
review:

| Candidate primitive | Named consumers | Promotion gate |
|---|---|---|
| multiple-testing adjustments | `deseq`, `edger`, `limma` | identical missing/non-finite, ordering, tie, and method contract in consumer tests |
| normal, chi-square, F, t, beta, and gamma tails/quantiles | at least `edger` and `limma`; some `deseq` paths | oracle grids over tails and boundary values, with accuracy and performance requirements |
| checked dense matrix decompositions and solves | `deseq`, `edger`, `limma` | rank, conditioning, pivoting, missing-value, and platform behavior defined independently of product policy |
| lowess/trend fitting | `deseq`, `edger`, `limma` use different policies | keep private until two products prove an actually identical model |
| NB GLM and dispersion fitting | `deseq` and `edger` have different likelihood, prior, offset, and convergence policy | keep product-local first; share only lower numerical kernels demonstrated by both |
| empirical-Bayes variance moderation | `edger` QL and `limma` are related but not interchangeable | no public API until both product tests establish a policy-free decomposition |

This wave does not create `rsomics-glm-nb`, `rsomics-ebayes-core`, a matrix
crate, a design-formula crate, or any other new public package.

`rsomics-common` supplies runtime errors, exit mapping, JSON envelopes,
transactional multi-output support, and provenance plumbing. `rsomics-help`
supplies the unified nested CLI presentation. Those foundations do not own
statistical choices or biological data models.

## Compatibility gates

Each product pins a current upstream release and records its R, Bioconductor,
dependency, BLAS, and platform environment. A committed golden is regenerated
only by a reviewed oracle script and stores that provenance beside the output.

The compatibility matrix covers:

- two-group, multi-level, covariate, interaction, paired/block, and time-course
  designs where supported;
- coefficients, named contrasts, multiple-term tests, and rank failures;
- all-zero rows and samples, low counts, sparse counts, large counts, unequal
  library sizes, normalization failure, and zero residual degrees of freedom;
- outliers, Cook's distance, robust fitting, independent filtering, missing
  values, non-finite values, and convergence failure according to the product;
- transform blind/design-aware behavior, threshold tests, splicing metadata,
  gene sets, and block correlation for the slices that expose them;
- stable row identity, filtering sets, coefficients, standard errors,
  dispersion or variance state, test statistics, p-values, adjusted values,
  diagnostics, and output schema.

Tolerance is defined per field and data regime. One global relative epsilon
cannot justify tails, near-zero values, discrete decisions, rank changes, or
different convergence solutions. Discrete rows and decisions should match
exactly; numerical fields use documented absolute, relative, or ULP bounds.

The live oracle is mandatory in release CI or a provenance-checked release-gate
environment. An unavailable oracle fails that gate instead of converting it
into a passing test.

## Performance and memory gates

Performance is measured on representative complete workflows and separately on
reviewed kernels:

- small interactive analysis, where startup and parsing matter;
- medium bulk RNA-seq data with realistic genes, samples, covariates, and
  contrasts;
- large or sparse count/expression matrices that expose memory behavior;
- designs that exercise dispersion, QL, empirical-Bayes, transform, and block
  paths rather than only a two-column arithmetic helper.

Each report records target commit, upstream versions, R and BLAS configuration,
machine, architecture, thread controls, input provenance, warmup, repetitions,
timing distribution, CPU time, and peak RSS. R is invoked directly; environment
manager startup is not charged to the oracle. Kernel and end-to-end results are
reported separately.

For a declared replacement slice, the relevant hot path must have a strict
throughput or resource-use advantage while meeting compatibility. A faster
approximation that changes filtering, convergence, estimates, or decisions is
not a performance win.

## Release gates and exclusions

Before any of the three product names is published:

1. the first complete workflow slice and fitted-state model pass public API
   review;
2. all declared operations use the shared CLI and runtime layers;
3. strict formatting, Clippy, unit, property, golden, live-oracle, and
   representative benchmark gates pass;
4. exact-head CI passes natively on Linux and macOS for `x86_64` and
   `aarch64`;
5. repository metadata, documentation, upstream versions, licenses, and
   attribution are correct;
6. the performance and memory decision is recorded from the target commit;
7. incomplete later operations are absent from public help and documentation.

Explicit exclusions from the initial releases:

- single-cell marker testing and pseudobulk policy;
- transcript quantification, tximport/tximeta aggregation, and effective-length
  correction;
- pathway databases, enrichment knowledge bases, and a generic plotting
  product;
- sleuth, NOISeq, EBSeq, DEXSeq, and other independent method families;
- GPU paths without a measured numerical and end-to-end benefit;
- cross-product fitted-object interchange;
- public scalar/statistical helper CLIs.

The historical Rust implementations are team-owned and may be reused.
DESeq2 is LGPL-3-or-later; edgeR and limma are GPL-2-or-later. Every migrated
operation records the upstream package, exact version, documentation or source
material used, paper where relevant, oracle script, and fixture provenance.
