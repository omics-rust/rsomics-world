# Phylogenetics product dossier

Status: source and upstream-operation audit complete. The target repository has
not been created.

## Boundary

`rsomics-phylo` is one phylogenetic analysis product. It owns alignment
trimming for phylogenetic input, evolutionary distance calculation,
distance-based tree inference, tree comparison and summarization, and
host-associate phylogeny tests. Later slices may add likelihood inference,
species-tree estimation, placement, consensus, and tree-set quality control
without creating operation-sized products.

The primary behavior sources are:

- [trimAl 1.5.1](https://github.com/inab/trimal/releases/tag/v1.5.1) for
  alignment trimming, scoring, sequence filtering, backtranslation, and
  format behavior;
- [scikit-bio 0.7.3 tree algorithms](https://scikit.bio/docs/latest/tree.html)
  for nucleotide distances, NJ, UPGMA/WPGMA, GME/BME, NNI, consensus,
  patristic matrices, RF-family distances, and cophenetic comparison;
- [DendroPy 5.0.11 tree measures](https://jeetsukumaran.github.io/DendroPy/library/treemeasure.html)
  and
  [tree comparison](https://jeetsukumaran.github.io/DendroPy/library/treecompare.html)
  for node timing, balance statistics, directional split errors, and
  branch-score behavior;
- [scikit-bio 0.7.3 Hommola cospeciation](https://scikit.bio/docs/latest/generated/skbio.stats.evolve.hommola_cospeciation.html)
  for host-associate correlation and permutation inference;
- [IQ-TREE 3.1.3](https://github.com/iqtree/iqtree3/releases/tag/v3.1.3) for
  the later maximum-likelihood, model-selection, support, concordance,
  topology-test, ancestral-state, simulation, and checkpoint contracts;
- [ASTER 1.25](https://github.com/chaoszhang/ASTER/releases/tag/v1.25) for
  later gene-tree, gene-family, whole-alignment, raw-read, and distance-based
  species-tree workflows;
- [UShER 0.6.6](https://github.com/yatisht/usher/tree/v0.6.6) for later
  mutation-annotated-tree placement and manipulation;
- [TreeShrink 1.4.0](https://github.com/uym2/TreeShrink/releases/tag/v1.4.0)
  for later long-branch outlier detection over tree collections.

This is one user workflow family, not one crate per distance, tree metric, or
upstream executable. An algorithm with a different input shape is a subcommand.
An algorithmic choice sharing the same input and result contract is normally a
typed mode.

Faith PD, generalized phylogenetic diversity, and UniFrac are community
diversity operations in `rsomics-ecology`. Tree-derived ILR bases are in
`rsomics-composition`. Population-genetic and genotype workflows remain in
`rsomics-popgen` and `rsomics-plink`.

## Operation map

### Initial release slice

| Target subcommand | Upstream operation | Decision |
|---|---|---|
| `trim` | trimAl threshold and gap-score trimming | initial `gap` method with complete threshold, alignment, column-map, and format contracts |
| `distance` | scikit-bio Hamming, JC69, and K2P | aligned nucleotide sequences to one canonical labelled distance matrix |
| `infer` | scikit-bio `nj`, `upgma`; SciPy average linkage | `--method nj\|upgma`; rootedness, negative branches, and tie behavior are explicit |
| `compare` | scikit-bio RF/wRF/KF/cophenetic; DendroPy symmetric difference, Euclidean branch score, FP/FN | one comparison engine and one result schema, not four binaries |
| `tip-distance` | scikit-bio `TreeNode.cophenet`; DendroPy patristic distances | branch-length or edge-count matrix with checked tip identities |
| `summarize` | DendroPy node ages, coalescence ages, Colless, Sackin, B1, treeness, N-bar, gamma | one or several selected measures in one typed result table |
| `cospeciation` | scikit-bio `hommola_cospeciation` | Hommola statistic, one-sided permutation p-value, and optional null distribution |

The first `trim` release does not claim the whole trimAl surface. It claims one
complete, named method. The command is extended in place as additional methods
pass their own oracle and performance gates.

`infer --method nj` emits an unrooted tree. `infer --method upgma` emits a
rooted ultrametric tree. They do not share a misleading generic rootedness
default merely because both consume a distance matrix.

### Later tree and alignment slices

| Target surface | Upstream operation | Gate |
|---|---|---|
| `trim --method similarity\|consistency\|overlap` | trimAl manual thresholds and compare-set selection | complete score, window, sequence-removal, and column-map behavior |
| `trim --method gappyout\|strict\|strictplus\|automated1` | trimAl automated methods | pinned 1.5.1 decision and tie behavior over diverse real alignments |
| `trim --backtranslate` | trimAl backtranslation | identifier, frame, stop-codon, and codon-gap validation |
| `infer --method wpgma\|gme\|bme` | scikit-bio WPGMA, GME, and BME | topology and branch-length differential plus representative scaling |
| `rearrange` | scikit-bio NNI | objective, rootedness, starting tree, and convergence contract |
| `consensus` | scikit-bio majority rule; IQ-TREE consensus/support assignment | tree weights, burn-in, thresholds, rootedness, and support serialization |
| `convert` | DendroPy Newick, NEXUS, and NeXML I/O | complete declared format profiles and round-trip fixtures |
| `edit` | DendroPy prune, shear, root, reroot, label, and annotate | identity-preserving transformations with transactional output |
| `outlier` | TreeShrink | per-gene, all-genes, and per-species modes with current defaults and alignment filtering |

### Later inference and placement slices

| Target surface | Upstream workflow | Gate |
|---|---|---|
| `model` | IQ-TREE ModelFinder | complete model vocabulary, AIC/AICc/BIC, partitions, and reproducible candidate search |
| `infer --method ml` | IQ-TREE maximum likelihood | substitution models, partitioning, checkpoint/resume, search, and likelihood evidence |
| `support` | IQ-TREE ultrafast and nonparametric bootstrap, branch tests | replicate generation, convergence, support assignment, and seed contract |
| `evaluate` | IQ-TREE fixed-topology evaluation | optimized parameters, likelihood report, and input-tree identity |
| `topology-test` | IQ-TREE RELL, KH, SH, expected-likelihood weight, and AU tests | candidate alignment, resampling, correction, and complete result table |
| `concordance` | IQ-TREE gene and site concordance factors | locus identity, gene-tree alignment, sampling, and annotated-tree output |
| `ancestral` | IQ-TREE ancestral state reconstruction | node identity, marginal probabilities, ambiguity, and state-table output |
| `simulate` | IQ-TREE AliSim | model, indel, partition, seed, and generated-alignment provenance |
| `species-tree` | ASTER ASTRAL-IV, ASTRAL-Pro3, wASTRAL, CASTER-site/pair, WASTER, SISTER, and D* | each mode has a complete input, algorithm, support, and oracle contract |
| `place` | UShER and matUtils | MAT format, parsimony placement, optimization, query, conversion, and scale evidence |

These operations remain absent from help and release documentation until
implemented. A subprocess wrapper or a command that only reports “not yet
implemented” is not a release slice.

## Alignment and distance contracts

- Aligned records have unique nonempty identities, a checked common width, an
  explicit nucleotide or amino-acid alphabet, and preserved full descriptions
  where the selected output format supports them.
- `rsomics-seqio` supplies the common FASTA stream and record boundary.
  Alignment validation, gap semantics, residue scoring, and trimming decisions
  stay in this product.
- Gap, similarity, consistency, and overlap scores use the pinned trimAl
  definitions. A user-facing gap fraction is not silently treated as trimAl's
  complementary gap score.
- Thresholds, windows, conservation floors, selected columns, and selected
  sequences are range checked before work begins. Ragged input is an error;
  missing bytes are not counted as non-gaps.
- Column maps preserve original zero-based coordinates independently of output
  wrapping. A complementary alignment and its retained alignment partition the
  input columns exactly.
- Distance matrices use a labelled square TSV profile with both header and row
  identities. The current unlabelled-body rsomics format is accepted only
  through an explicit legacy input profile.
- Matrix identities are unique and row identities must match the header.
  Shape, hollow diagonal, symmetry, finite values, and operation-specific
  negativity are checked once at the boundary.
- Hamming is a proportion over all aligned positions. JC69 and K2P use
  canonical-site pairwise deletion under the pinned scikit-bio profile.
  Lowercase, RNA `U`, ambiguity symbols, and gaps receive explicit behavior.
- Saturated or unidentifiable JC69/K2P pairs are reported as non-estimable.
  A compatibility output may serialize `nan`; tree inference always rejects a
  non-finite matrix rather than silently constructing a tree from it.

## Tree and comparison contracts

- Newick input follows a declared grammar. The initial grammar includes the
  required terminating semicolon, quoted labels and doubled quotes, unquoted
  underscore conversion policy, whitespace, comments, internal labels, and
  signed finite branch lengths. The
  [Newick specification](https://phylipweb.github.io/phylip/newick_doc.html)
  and scikit-bio 0.7.3 profile are the initial references.
- A parsed tree always has a valid root, reciprocal parent/child links,
  reachable nodes, no cycles, and stable node identities. Empty child lists,
  multifurcations, and unifurcations have deliberate typed meanings rather than
  arising from missing tokens or parser accidents.
- Operations using taxon identity reject duplicate or unnamed tips unless a
  documented format contract permits them. Identity alignment never falls back
  to input position.
- Rooted and unrooted interpretation is explicit. An `auto` compatibility mode
  may reproduce an upstream root-degree rule, but the result report records the
  chosen interpretation.
- Missing branch lengths are allowed for topology-only operations. Weighted
  comparison, patristic distances, treeness, timing, and gamma reject missing
  or non-finite lengths by default. An upstream-compatible zero-fill policy is
  a separately selected mode.
- RF count/proportion, weighted RF city-block distance, Kuhner-Felsenstein
  Euclidean branch score, directional FP/FN, and cophenetic comparison share
  one taxon index and split encoding. The implementation has no fixed 64-taxon
  ceiling.
- Exact-taxon and shared-taxon comparison are separate policies. The safe
  default is exact identity; scikit-bio compatibility may select shared taxa
  where the pinned operation does so.
- Comparison output records rootedness, taxon policy, common and excluded taxa,
  terminal-branch inclusion, branch-length policy, metric, and normalization.
- Node ages and coalescence ages state whether they mean backward time on an
  ultrametric tree or forward depth. Those quantities are not conflated.
- Colless and gamma validate strict bifurcation where required. Unary nodes,
  zero total length, too few tips, non-ultrametric input, and degenerate
  normalizations return typed errors instead of panics or incidental NaNs.

## Permutation contract

- Host and associate distance matrices are validated under the same labelled
  matrix contract and aligned to interaction rows and columns by identity.
- The interaction matrix is binary, rectangular, uniquely labelled, and has
  enough edges and edge pairs for a defined correlation.
- The observed statistic, alternative, inclusion of the observed value in the
  p-value numerator, permutation count, RNG algorithm, seed, and thread count
  are recorded.
- A pinned scikit-bio compatibility mode reproduces its generator stream for a
  fixed seed. A parallel native RNG mode may use independent deterministic
  streams, but it is named separately and does not claim bit-identical
  p-values.
- `permutations=0` is an explicit statistic-only mode. The result distinguishes
  “not calculated” from a numerical NaN produced by degenerate data.

## Target structure

```text
src/
├── cli.rs
├── alignment/
│   ├── model.rs
│   ├── score.rs
│   ├── trim.rs
│   └── column_map.rs
├── distance/
│   ├── model.rs
│   ├── sequence.rs
│   ├── simd.rs
│   └── tsv.rs
├── infer/
│   ├── nj.rs
│   └── upgma.rs
├── compare/
│   ├── taxa.rs
│   ├── splits.rs
│   ├── rf.rs
│   └── cophenetic.rs
├── measure/
│   ├── paths.rs
│   ├── timing.rs
│   └── balance.rs
├── cospeciation/
│   ├── interaction.rs
│   ├── correlation.rs
│   └── permutation.rs
├── output.rs
└── report.rs
```

Later likelihood, species-tree, placement, and tree-set modules are added only
with their release slices. The initial source tree contains no empty
`model_finder`, `astral`, or `usher` module.

The library exposes checked alignments, distance matrices, inference
configuration, comparison results, and statistical results required for
programmatic use. CLI parsing, compatibility profiles, serialization policy,
and execution plumbing stay private.

## Foundation relationships

`rsomics-common` owns error-to-exit mapping, execution envelopes, aliases, and
transactional named output. `rsomics-help` owns the complete recursive Clap
presentation for every current and later subcommand. The three historical
products that omit `rsomics-help` and the products that carry duplicate
`HelpSpec` trees do not establish a second CLI model.

`rsomics-phylo` is the main concrete consumer driving reconstruction of
`rsomics-phylo-tree`. The foundation owns:

- an always-valid tree and immutable topology view;
- checked construction and mutation;
- node, edge, root, traversal, and tip-identity primitives;
- declared Newick parsing and emission profiles;
- reusable topology and taxon-index validation.

NJ, UPGMA, RF policy, branch-score reductions, node measures, trimming,
permutation inference, CLI options, and result tables remain in the product.
The foundation does not expose public mutable node fields or an invalid
`Default` tree.

Composition consumes validated topology and tip identity for tree-derived ILR
bases. Ecology consumes the same topology and branch view for phylogenetic
diversity. Those consumer tests, not the number of retired phylo crates,
justify the foundation API.

`rsomics-stats` may own Pearson correlation or permutation-count primitives
only after phylo and a second product use the same finite-value, degeneracy,
tail, and RNG contract. Hommola edge pairing and relabelling stay private.

The canonical labelled distance matrix initially remains inside
`rsomics-phylo`. Ecology is a likely second consumer, but that does not justify
a new public matrix foundation before both products demonstrate the same
identity and missing-value contract.

## Historical asset disposition

The eleven operation-sized GitHub repositories and crates.io packages are
deleted. Their external-disk clones remain implementation assets.

| Source asset | Revision | Disposition |
|---|---|---|
| `rsomics-msa-trim` | `773aa0d375d0cfa4aaed24d366d438b9201521ad` | refactor then merge; retain gap-column kernel, trimAl 1.5.1 golden, smoke cases, and benchmark fixture; replace whole-file policy, ragged handling, direct output, and duplicate CLI |
| `rsomics-seq-dist` | `798e8d2be02e91842bd045338f05bb59065332ba` | refactor then merge; retain Hamming/JC69/K2P kernels, SIMD tallies, goldens, and measured speedups; replace bespoke FASTA, matrix layout, NaN pipeline policy, and missing help integration |
| `rsomics-nj-tree` | `7830b61a366026480a0b29ad55aad4a8617387fc` | refactor then merge; retain candidate bound-based NJ kernel and validation cases; require a real topology/branch oracle because current tests only inspect Newick shape |
| `rsomics-upgma` | `41fefd4b401ca60fe1dfe71b4066c96dee9e89bc` | refactor then merge; retain nearest-neighbor-chain average linkage, SciPy goldens, large-fixture evidence, and tie cases; replace matrix parser, unchecked public kernel, label emission, and direct linkage output |
| `rsomics-tree-rfd` | `c30431245bc77c6c5aa910fa2ff182035af7c55f` | refactor then merge into `compare`; retain scikit-bio 0.7.2 goldens, large RF fixture, and rooted/unrooted cases; replace duplicated split engine and silent taxon loss |
| `rsomics-tree-wrfd` | `af721fafc871611cd8cc29ebb161839aa2679bc9` | refactor then merge into `compare`; retain wRF/KF formulas, shared-taxon fixtures, and performance evidence; make this and RF use one checked taxon/split model |
| `rsomics-tree-branch-score` | `eefb6a023bbe9b221b1c3e8d437f9cca21a5f0a6` | test and formula asset only until rewritten; the `u64` split mask cannot represent the claimed 300-taxon benchmark and duplicates KF/FP/FN already covered by the unified comparison engine |
| `rsomics-tree-tipdist` | `6a43074ef1dcb635daa76c6cd056580a77ca18fe` | refactor then merge; retain one-pass patristic matrix algorithm, byte goldens, large output fixture, and formatter tests; reject missing lengths by default and share matrix serialization |
| `rsomics-node-ages` | `2c121467a34fda7bc9e0d9ed377af358e4ab8e76` | refactor then merge into `measure`; retain ultrametric recurrence, timing fixtures, and DendroPy values; reconcile current DendroPy node-age semantics and share traversal/length validation |
| `rsomics-tree-balance` | `95a9e9074354c220b0b2af841cea0396a76aa34d` | refactor then merge into `measure`; retain formulas, normalization fixtures, and DendroPy values; replace duplicated age traversal, missing-length zero fill, and unary/degenerate panic paths |
| `rsomics-hommola` | `9af268ffe34a8add00edef5849844e1d6e391ae4` | refactor then merge; retain allocation-free Pearson reduction, deterministic parallel-permutation design, goldens, and memory evidence; replace public unchecked structs, duplicate matrix parsers, positional identity, and ambiguous RNG compatibility |

The untracked `target/` directories in the node-ages and tree-balance clones
are inherited local state and are not source assets or deletion targets.

## Audit findings that block direct consolidation

1. The current `rsomics-phylo-tree` permits invalid construction:
   `Tree::default()` has no root node, all topology fields are publicly mutable,
   and `to_newick()` can index an invalid root.
2. Its Newick parser silently maps invalid UTF-8 labels to an empty string,
   accepts a missing semicolon, empty child positions, and non-finite branch
   lengths, and does not implement quoted labels, escaping, comments, or
   underscore semantics. Its emitter does not quote labels.
3. Four comparison crates independently implement taxon extraction,
   traversals, rooting inference, subset/bipartition encoding, and reduction.
   Their exact-taxon versus shared-taxon and missing-length policies disagree.
4. `rsomics-tree-branch-score` assigns taxa to one `u64` bitmask. At more than
   64 taxa its shifts overflow or alias. The recorded 300-taxon result therefore
   cannot establish correctness or a valid performance comparison.
5. The RF and wRF paths silently discard unnamed tips and collapse duplicate
   names into sets. They do not error when two nonempty trees share no taxa.
6. The old products use three incompatible distance-matrix layouts:
   unlabelled body rows for NJ/UPGMA/sequence distance, labelled scikit-bio
   matrices for Hommola, and labelled output for tip distances.
7. `rsomics-seq-dist` intentionally emits `nan` for saturated pairs while
   NJ/UPGMA only reject NaN under their own parsers. A composed distance-to-tree
   workflow has no explicit non-estimable policy.
8. The NJ tests never compare topology and branch lengths to a real oracle.
   The historical performance journal reports a RapidNJ command-not-found
   message and cannot be treated as a valid competitor result.
9. The trim implementation does not validate threshold range or aligned
   widths. Short records can be processed as if absent columns were non-gaps.
   Its old timing record omits the upstream version and measures an earlier
   dirty revision.
10. Node-age and balance code duplicate traversals and ultrametric logic.
    Balance treats missing lengths as zero, uses a partial comparison that can
    panic on NaN, and indexes a second child without rejecting unary nodes.
11. Hommola's readers do not validate matrix symmetry, diagonal, finiteness,
    duplicate identities, or interaction alignment. Its public structs permit
    too few edges, which can underflow the pair-count expression.
12. Named outputs use direct `File::create`, so a late parse, computation, or
    write failure can leave a truncated destination.
13. Three product crates omit `rsomics-help`; the others duplicate the
    historical help model. All use the deleted operation identity instead of
    one family command tree.
14. CI is mostly one Ubuntu job. Three repositories add a generic macOS job,
    but none explicitly proves native Linux and macOS on both `x86_64` and
    `aarch64`.
15. Source comments repeatedly narrate implementation history, upstream
    equivalence, phases, and obvious loops. Selected algorithms are retained
    without carrying that comment style into the target.

## Compatibility plan

Oracle tests are first-class jobs. A missing required oracle fails the job; it
does not return success after printing `SKIP`.

| Operation | Pinned oracle | Required evidence |
|---|---|---|
| gap trimming | trimAl 1.5.1 | thresholds 0/1/interior, `-cons`, gap symbols, ragged rejection, column map, complement, formats, and randomized aligned fixtures |
| sequence distance | scikit-bio 0.7.3 | Hamming/JC69/K2P finite values, ambiguity and gap deletion, saturation, identities, TSV serialization, and random aligned fixtures |
| NJ | scikit-bio 0.7.3 canonical NJ; RapidNJ 2.3.2 as performance comparator | compare unrooted topology and branch lengths independent of sibling order/root placement; both negative-branch policies |
| UPGMA | scikit-bio 0.7.3 and its pinned SciPy linkage | topology, linkage, branch lengths, ties, WPGMA exclusion, and random finite matrices |
| RF/wRF/KF/FP/FN | scikit-bio 0.7.3 and DendroPy 5.0.11 profiles | rooted/unrooted, exact/shared taxa, duplicate/unnamed tips, missing lengths, terminal branches, polytomies, unifurcations, 64/65/300/100,000-taxon cases |
| tip distances | scikit-bio 0.7.3 and DendroPy 5.0.11 | branch lengths, edge counts, missing-length policy, tip order, identities, and byte goldens |
| timing and balance | DendroPy 5.0.11 | ultrametric and non-ultrametric trees, unary/polytomy cases, all normalizations, missing/non-finite lengths, and current node-age semantics |
| cospeciation | scikit-bio 0.7.3 | statistic, exact fixed-seed compatibility mode, native deterministic mode, zero permutations, degenerate matrices, and label permutations |

Newick foundation tests cover the declared grammar directly plus round trips
through scikit-bio, DendroPy, IQ-TREE, trimAl label output, and ASTER tree
fixtures. NEXUS, NeXML, MAT, and multi-tree streams are not accepted until
their own declared profiles pass.

## Performance and memory plan

Historical results are migration clues, not automatic release evidence.

- Re-run trim against trimAl 1.5.1 on nucleotide and protein alignments with
  enough rows and columns to dominate startup. Record retained columns and
  peak RSS.
- Re-run every sequence-distance metric thread-for-thread against scikit-bio
  0.7.3 on an alignment whose pairwise work and full output matrix are both
  material. Separate compute-only from parse/emit.
- Measure NJ against RapidNJ 2.3.2 on finite 2,000-taxon and larger matrices.
  Report input plus working-set RSS; the current implementation allocates a
  four-times-expanded square matrix in addition to parsed rows.
- Re-run UPGMA against current SciPy through scikit-bio, preserving the
  2,000-taxon linkage fixture and measuring both algorithm-only and end-to-end
  paths.
- Measure unified RF and weighted comparison on the existing 100,000-tip and
  4,000-tip fixtures plus a 65-tip regression. Report exact values, time, and
  RSS; discard the invalid fixed-mask 300-tip result.
- Preserve the 2,000-tip full patristic-matrix output gate. Separate matrix
  construction from formatting and disk I/O.
- Re-run node measures on at least 500 tips and Hommola on the existing
  150-host, 200-associate, 510-edge, 999-permutation fixture under pinned
  current oracles.
- Measure single-thread and selected parallel modes separately. Thread counts,
  CPU affinity, host load, versions, input hashes, run distribution, output
  hashes, peak RSS, and exact source revisions are mandatory.

For an established replacement operation, at least one relevant hot path must
be strictly faster or materially lower-memory than the oracle. Python import
time may be reported as user-facing latency but does not replace a
compute-only comparison.

## Release sequence

1. Reconstruct `rsomics-phylo-tree` only far enough to support the simultaneous
   phylo consumer slice: valid construction, immutable topology, declared
   Newick, traversal, and tip identity.
2. Build the canonical alignment and labelled distance models inside
   `rsomics-phylo`; migrate `trim`, `distance`, NJ, and UPGMA.
3. Build one taxon/split/path engine; migrate RF, wRF/KF, FP/FN,
   `tip-distance`, timing, and balance without duplicated encodings.
4. Migrate Hommola through the same matrix identity contract and a pinned RNG
   profile.
5. Run format, strict Clippy, unit, golden, live-oracle, integration,
   representative performance/RSS, and four-native-target exact-head gates.
6. Publish the first release only if all seven advertised subcommands pass.
   Otherwise ship a smaller internally coherent subset and omit every later
   command from help and documentation.
7. Extend trimAl modes, tree editing/consensus/outlier operations, likelihood
   inference, species-tree estimation, and placement as separate complete
   releases.

## Explicit exclusions

- No historical micro-crate name is revived.
- `rsomics-phylo` does not implement Faith PD, generalized PD, UniFrac,
  ordination, PERMANOVA, or other community ecology statistics.
- The first release does not claim complete trimAl, IQ-TREE, ASTER, UShER,
  TreeShrink, DendroPy, or scikit-bio replacement.
- General FASTA/FASTQ transformation remains in `rsomics-seq`; general
  alignment generation is a later phylo decision, not an empty initial
  subcommand.
- Bayesian MCMC workflows such as BEAST and MrBayes are outside the current
  implementation plan.
- GPL or AGPL upstream code is an oracle and provenance source unless a
  separately reviewed reuse decision permits more. Team-owned historical Rust
  code may be reused directly.
- No public distance, matrix, split, RNG, or alignment foundation is added
  merely because the corresponding module is reusable within this product.
