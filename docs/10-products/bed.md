# BED product dossier

Status: [GitHub v0.4.0](https://github.com/omics-rust/rsomics-bed/releases/tag/v0.4.0)
ships twelve operations at5e5b709, adding BED-only genomecov. Exact-head
native-four originals, clean source package and fresh anonymous public
MacARM15-call installation are root-adopted. Older0.3.0 remains immutable.
Registry remains0.1.0 with authentication held. Source/binary performance
bindings and map's slower throughput remain explicit; this is not a new timing
claim. See `.autopilot/oracles/bed-native-delivery-2026-11-03-v04.json` and
`.autopilot/oracles/bed-public-delivery-2026-11-03-v04.json`.

## Historical and measured-source evidence

Immutable [GitHub v0.2.0](https://github.com/omics-rust/rsomics-bed/releases/tag/v0.2.0)
ships eight operations at `bcbee5f87aeda2e7664c279b32c5a3098635ca9b`.
Registry 0.1.0 retains `sort`, `merge`, `intersect`, `subtract`, and `complement`;
registry authentication remains separately held. The earlier 0.2.0 candidate at
`02b85a1a348c271e485cca629dc4e71fa075388a` adds `cluster`, `window`, and
`closest` and passed exact-head four-platform CI run `32329089591`. The
representative performance source is
`d7b1507b178053a087862255d84a244e4921f192`. Registry publication is blocked
by registry authentication: publish run `32329397022`
completed package verification and then received crates.io `403 authentication
failed`. The registry therefore still serves 0.1.0.

The 2026-11-02 delivery task added four-native GitHub installation for the
unchanged eight-operation0.2 slice; it does not retry registry authentication
or add later operations. Historical raw performance JSONs and runners are
freshly retained/hash-checked and their340 sample summaries recompute, including
the rejected candidate and RSS losses. This is not a new benchmark or a fresh
large-output/runtime attestation. See
`.autopilot/oracles/bed-historical-performance-2026-11-02.json` and
`.autopilot/state/bed-native-delivery-2026-11-02.md`. New delivery/release/public
installation remain separate gates. Actual ownerbcbee5f8/native4CI41348074250
now has complete originals adopted:185 Git files/four identical clean source
packages,95 distinct tests in three profiles/four hosts, and44 installed calls.
Native source/package/bundle checks pass. [GitHub v0.2.0](https://github.com/omics-rust/rsomics-bed/releases/tag/v0.2.0)
and a fresh public Mac11-call installation now pass; post-release docs97aa04a6
passes exactCI41351177412 without replacing the immutablebcbee5f8 release.
Registry remains separately held. The inherited README I/O guarantee
now explicitly distinguishes pre-commit preservation from post-commit sync
errors, without changing the locked runtime or public API. See
`.autopilot/oracles/bed-native-delivery-2026-11-02.json`.
Public release and installation proof is
`.autopilot/oracles/bed-public-delivery-2026-11-02.json`.

The pre-release coverage source `3b2314a6912a849b21e7afbbb86b45fd8d6e294e`
passed actual four-native CI `41442092689`: 103 debug, release and packaged
tests per host plus a twelve-call nine-operation installed lifecycle. One
RefSeq annotation-derived sparse/nested campaign accepts strict throughput
and maximum-process-RSS gates for summary/count/histogram reports. All 72
complete outputs match; median speedups are 2.41–85.46x on Apple M2, with the
85x result specific to nested histogram. Those measurements are not a
thread-scaling, biological-effect or universal speed claim. The split-plus-
explicit-fraction policy remains a pending user choice, explicitly rejected
in this candidate. See `.autopilot/state/bed-coverage-execution-2026-11-02.md`
and `.autopilot/oracles/bed-coverage-performance-2026-11-02.json`.

The pre-release `annotate` source `cdf736337d8bd63b2770723a8a667f7af83409b9`
passed four-native CI `41453635215`, with107 tests per debug/release/packaged
profile and13 installed calls across ten operations. Multi-file independent
count/fraction/both output retains the pinned point, bin-order and header
semantics. One complete dual-file RefSeq-derived `both` campaign has12 exact
source-matching outputs. On Apple M2 its median wall is0.392s versus37.403s
and peak process RSS medians56.9MB versus175.0MB. These are specific workflow
results, not universal speed, scaling or biological claims. Their original
source/binary bindings remain unchanged by v0.3.0 delivery. See
`.autopilot/state/bed-annotate-execution-2026-11-02.md` and
`.autopilot/oracles/bed-annotate-performance-2026-11-02.json`.

The pre-release `map` source at `bcf65d7accdf4ff956680cfdef8e5d4eeb9634f8`
adds a sorted streaming workflow with23 shared column reducers. Full local
debug verification passes118 tests; one fresh whole-feature review identifies
no critical or important issue. Exact-head native CI41495972529 passes all four
native targets with118 tests/profile and14 installed calls each; original
source/package/bundle/lifecycle evidence is root-adopted. The user-approved
point-cache repair and the same-class tied-B lookahead repair are documented
known incompatibilities, not accidental oracle mismatches. Representative
whole-mapping performance has12 byte-identical outputs: median native/source
wall0.577/0.470s and process RSS4.31/4.98MB on Apple M2. This is a13.49percent
RSS advantage but22.84percent slower throughput, not a speedup. The resource
gate is accepted under the throughput-or-resource rule. The subsequent v0.3.0
delivery and installation gates are recorded separately above. See
`.autopilot/state/bed-map-execution-2026-11-02.md`.

## Boundary

`rsomics-bed` is one BED and BEDPE interval-workflow product. It owns strict
BED parsing, optional-field interpretation, genome-file ordering, interval set
algebra, neighborhood queries, coverage summaries, interval generation, and
interval-driven reference-sequence operations. A user installs one binary and
selects operations with subcommands; an upstream executable name or flag does
not create another crate.

The primary behavior sources are:

- [UCSC BED](https://genome.ucsc.edu/FAQ/FAQformat.html#format1) for BED3–BED12,
  zero-based half-open coordinates, optional-field ordering, BED12 blocks, and
  zero-length insertion features;
- [BEDTools 2.31.1](https://bedtools.readthedocs.io/en/latest/content/bedtools-suite.html)
  for the familiar operation contracts and byte-level compatibility oracle;
- [BEDOPS 2.4.41](https://bedops.readthedocs.io/en/latest/content/overview.html)
  for multi-input set algebra, closest-feature, mapping, sorted streaming, and
  representative low-memory alternatives.

BEDTools remains the primary oracle because the historical rsomics assets were
built against it. BEDOPS is a second behavioral and performance reference, not
a source-code dependency. GPL-covered BEDOPS source is not copied.

## Product relationship map

```mermaid
flowchart TD
    record["private BED record stream"] --> set["set algebra"]
    record --> relation["overlap and distance index"]
    record --> sweep["sorted streaming sweeps"]
    record --> transform["coordinate transforms"]
    relation --> nearby["closest / window"]
    relation --> coverage["coverage / annotate / map"]
    sweep --> cluster["cluster / multiinter / unionbedg"]
    transform --> genome["genome-size policy"]
    genome --> generation["flank / slop / windows / randomization"]
    record --> sequence["FASTA-backed operations"]
    sequence --> seqio["rsomics-seqio"]
```

The record stream, indexes, overlap filters, and reducers remain private until
another Layer B product demonstrates the same policy-free API. No Layer B
product depends on `rsomics-bed`.

## Operation map

Target commands use readable product vocabulary. Familiar BEDTools short flags
may be migration aliases, but the typed long-form contract owns the help.

| Target operation | Real upstream behavior | State and decision |
|---|---|---|
| `validate` | UCSC BED3–BED12 and BEDTools parser failures | later structural slice; strict optional-field and block checks |
| `inspect` | BEDTools `summary`; historical count/stats/total-bp | later; one report replaces four count-only commands |
| `sort` | BEDTools `sort`; BEDOPS `sort-bed` | published; stable coordinate ties are an explicit rsomics guarantee |
| `merge` | BEDTools `merge`; BEDOPS `--merge` | published |
| `intersect` | BEDTools `intersect`; BEDOPS intersect/element-of | published clipped-output slice; later report modes stay options here |
| `subtract` | BEDTools `subtract`; BEDOPS `--difference` | published |
| `complement` | BEDTools `complement`; bounded BEDOPS complement | published with an explicit genome file |
| `cluster` | BEDTools `cluster` | GitHub 0.2; sorted streaming, distance, and strand policy |
| `window` | BEDTools `window` | GitHub 0.2; symmetric/asymmetric, strand-relative, and report modes |
| `closest` | BEDTools `closest`; BEDOPS `closest-features` | GitHub 0.2; complete declared single-B contract before multi-B extensions |
| `coverage` | BEDTools `coverage` | GitHub0.3; summary/count/mean/histogram/depth, pooled B and ordinary blocks; split plus fractions held |
| `annotate` | BEDTools `annotate` | GitHub0.3; independent multi-file count/fraction/both output |
| `map` | BEDTools `map`; BEDOPS `bedmap` | GitHub0.3; sorted sweep/CLI and23 private reducers; lower-RSS workflow accepted, throughput slower; registry held |
| `jaccard` | BEDTools `jaccard` | later metrics slice; base-pair set similarity and overlap filters |
| `fisher` | BEDTools `fisher` | later metrics slice; requires a genome-size universe |
| `reldist` | BEDTools `reldist` | later metrics slice; fixed historical midpoint-straddle behavior is retained |
| `multiinter` | BEDTools `multiinter`; BEDOPS partition | source inventory and five integrated plus two boundary witnesses retained; all-empty/origin policy pending, no implementation or release claim |
| `genomecov` | BEDTools `genomecov` | GitHub0.4.0 at5e5b709; native4 run41532198240/full originals and fresh public installation adopted,126 tests/profile and15 installed calls; full584846-row/93-contig histogram and bedgraph-all output/performance accepted atc6e35 in `bed-genomecov-performance-2026-11-03.json`; BAM input belongs to `rsomics-bam` |
| `unionbedg` | BEDTools `unionbedg` | later track slice |
| `flank` | BEDTools `flank` | later coordinate-generation slice |
| `slop` | BEDTools `slop`; BEDOPS range padding | later coordinate-generation slice |
| `shift` | BEDTools `shift` | later coordinate-generation slice |
| `makewindows` | BEDTools `makewindows`; BEDOPS chop/stagger | later coordinate-generation slice |
| `random` | BEDTools `random` | later seeded-generation slice |
| `shuffle` | BEDTools `shuffle` | later seeded-generation slice |
| `sample` | BEDTools `sample` | later deterministic sampling slice |
| `getfasta` | BEDTools `getfasta` | later reference slice using `rsomics-seqio` random access |
| `maskfasta` | BEDTools `maskfasta` | later reference slice; one implementation replaces two historical clones |
| `nuc` | BEDTools `nuc` | later reference slice |
| `overlap` | BEDTools `overlap` | later BEDPE/shape slice |
| `pair-to-bed` | BEDTools `pairtobed` | deferred until the BEDPE record contract is complete |
| `pair-to-pair` | BEDTools `pairtopair` | deferred with `pair-to-bed` |
| `convert` | BEDTools `bed12tobed6`; historical BED-to-GFF | later format slice with explicit `bed6` and `gff3` targets |
| `split` | BEDTools source-level `split` utility | later multi-output slice; not confused with BED12 block splitting |
| `spacing` | BEDTools source-level `spacing` utility | deferred until its under-documented contract is recovered from the pinned executable and source |

BEDOPS symmetric difference and partition remain possible modes of the set
algebra above. They do not justify duplicate commands until a workflow needs
the semantic gap that `intersect`, `subtract`, `merge`, and `makewindows` do not
cover.

## Operations routed elsewhere

| Upstream operation | Target | Reason |
|---|---|---|
| BEDTools `expand`, `groupby` | `rsomics-table` | table-column expansion and aggregation, not interval algebra |
| `bamtobed`, `bamtofastq`, `bedtobam`, `bedpetobam` | `rsomics-bam` | alignment-format conversion and validation own the workflow |
| BEDTools `multicov`, BAM mode of `genomecov`, `tag` | `rsomics-bam` | indexed alignment coverage and tagging require BAM policy and `rsomics-bamio` |
| BEDOPS Starch and indexed extraction | `rsomics-index` candidate | compression and persistent indexing are separate installation workflows |
| BEDTools `igv`, `links` | excluded | GUI snapshot scripting and HTML browser links are not data operations |
| GFF/GTF or VCF inputs accepted by BEDTools | `rsomics-annotation` or `rsomics-vcf` | each product owns its format semantics and converts at a checked interval boundary |

The historical `count`, `len`, `midpoint`, `stats`, `total-bp`, and `unique`
executables do not survive as public crates. Count and length summaries fold
into `inspect`; coordinate deduplication becomes a `sort` option. Appending a
length and collapsing to a midpoint remain deferred conveniences rather than
commands invented solely to preserve old code.

## Historical asset dispositions

The classification is about implementation reuse, not repository revival.
Every old command shell and duplicated `rsomics-help` specification is
discarded.

| Assets and recorded revisions | Classification | Retained value |
|---|---|---|
| `bed-sort` `5dce75d5a7e`, `bed-merge` `c9e39b3e760e`, `bed-intersect` `e7dc1e7e462e`, `bed-subtract` `927c257de23b`, `bed-complement` `94ea9f552620` | 2, completed | algorithms and strongest goldens were refactored into 0.1.0 |
| `bed-closest` `e85ed1339165` | 2, completed | retained corrected distance, zero-length, no-hit, and tie cases; replaced the permissive parser and full B scan |
| `bed-cluster` `b63b75567ba7` | 2, completed | retained the sorted sweep and oracle cases; moved behavior onto the product parser and output layer |
| `bed-window` `875459ee2f79` | 3, completed | retained fixtures and benchmark recipe; rewrote indexing, parsing, output modes, and CLI |
| `bed-coverage` `269e1f6ca740` | 3, reviewed | reference/contracts only; coverage rebuilt with the product parser, duplicate-preserving index and clipped depth events |
| `bed-annotate` `551632db1dce` | 3, completed | reference/fixture assets; rebuilt on the checked parser/index and private span-depth primitive, preserving distinct point, numeric and ordering rules |
| `bed-map` `1fb4c6cc8d84` | 3, source/oracle reviewed | reference/fixture only, not a direct reducer/parser merge; new private streaming/reduction design retains separate source contracts |
| `bed-jaccard` `ca81d15e39d0` | 2 | retain merged-set sweep, numeric formatting tests, and fixtures; add checked filters and product I/O |
| `bed-fisher` `fc7bb057f3d2` (dirty), `bed-reldist` `36cdfdead5f6` | 2/3 | retain fixed reldist oracle cases; audit dirty fisher state before reuse |
| `bed-multiinter` `6c07acd0a68e` | 3 | fixture/reference/benchmark-input assets only; permissive parser, incorrect tie/order assumptions and chromosome-wide event retention are not merged; new private streaming contract in `bed-multiinter-contract-2026-11-03.md` |
| `bed-genomecov` `0087cf10fc15`, `bed-unionbedg` `17190fd1bff7` | 2 | refactor into one track sweep and shared genome ordering |
| `bed-flank` `3082414312e5`, `bed-slop` `5fc4c99797b1`, `bed-shift` `06ae585329e4`, `bed-makewindows` `e3d7050be07f` | 2 | retain coordinate algorithms and oracle fixtures; centralize overflow and genome bounds |
| `bed-random` `492873673ef8`, `bed-shuffle` `c2bc689ee791`, `bed-sample` `23a95f5919e1` | 2/3 | retain deterministic fixtures; replace unrelated RNG and whole-file policies with one seeded contract |
| `bed-getfasta` `82783b773f90`, `bed-nuc` `3e20a5e4f659` | 2 | retain byte-exact sequence and nucleotide fixtures; consume the shared sequence reader |
| `bed-maskfasta` `d5e540e305d0` (dirty), `fasta-mask` `4bda5f191c91` | 3/4 | audit the dirty stronger asset, keep its oracle cases, and discard the duplicate implementation |
| `bed-overlap` `859c0e243b1e`, `bed12-to-bed6` `cd5382590b8c`, `bed-to-gff` `b215fe4e50eb` | 2/3 | retain shape/conversion fixtures; rebuild on validated optional fields |
| `bed-split` `d73fb8249648` (dirty), `bed-spacing` `fb4ede5c9a18` | 3 | retain recoverable fixtures only after the under-documented upstream contracts are pinned |
| `bed-count` `bf831e87e79f`, `bed-len` `c8c31c5c6f7d`, `bed-midpoint` `22e507b3abfd`, `bed-stats` `456bcdc41a5d`, `bed-summary` `227a928b3fa4`, `bed-total-bp` `170e5775ecdd`, `bed-unique` `91b85b570c74`, `bed-validate` `4ac82c17923a` | 3/4 | keep useful malformed and summary fixtures; discard duplicated line parsers and atomic binaries |
| `bed-multicov` `70b478da366e` | 3, routed | BAM fixture and command recipe move to `rsomics-bam`; BED product code is discarded |
| `bed-expand` `23aa4ee69ab6`, `bed-groupby` `30cf021d1c59` | 3, routed | table fixtures and reducer cases belong to `rsomics-table` |

Dirty historical worktrees are user-owned evidence. They are never staged,
cleaned, or treated as exact revision content without first recording their
diffs and recovery paths.

## Format and execution model

- Coordinates are checked `u64` half-open intervals. Optional BED fields stay
  byte-preserving and are parsed only by operations that need them.
- A strand-aware operation requires a valid BED6 strand. It does not infer or
  repair a missing strand.
- Headers, comments, zero-length records, BED12 blocks, duplicate features,
  chromosome ordering, and genome bounds have explicit operation-level policy.
- One input may be standard input. Multi-input operations reject ambiguous
  combinations instead of consuming the same stream twice.
- Named output uses `rsomics-common` transactional writes and alias rejection.
  Table-producing commands require a named output under JSON mode so BED and
  machine output cannot mix.
- The product does not silently sort an operation whose upstream contract
  requires presorted input. It reports the first ordering violation.
- `window` emits multiple matches in stable B-file order. BEDTools exposes
  UCSC-bin traversal order at some internal bin boundaries; rsomics treats that
  incidental instability as an explicit output-order divergence.
- gzip stream support is a concrete shared-code candidate with
  `rsomics-table`, but is not promoted into `rsomics-common` until both
  consumers migrate with contract tests after the table 0.1 release.

## Release history and next slice

Published 0.1.0 source head `c252c6036b4173fc95209938b45fc032ca34872d`
has crates.io checksum
`9c8883539cb78d946bdb1355704e1b8810130a2b9eadab8c37e94bf3c25f4ebe`.
The independently verified 0.1 gate is recorded in
`bed-gate-2026-07-30.md`. The 0.2 candidate evidence is recorded in
`bed-performance-2026-08-20.md`. Its implementation and performance source
`d7b1507b178053a087862255d84a244e4921f192` passed exact-head CI run
`32328054861`, the regenerated five-operation gate, and the new sparse/dense
relation gate.

Final source and public API review produced release candidate
`02b85a1a348c271e485cca629dc4e71fa075388a`. The candidate passed exact-head CI
run `32329089591` on all four native target classes. Its clean local package
archive has SHA-256
`6ce9d3cc727e385e4e66accc85c774b18f8ac7b902315c0782551e4266b1c80e`.
Publish run `32329397022` reached the upload and failed with crates.io `403
authentication failed`; it was not retried. No `v0.2.0` tag or GitHub release
was created at that earlier registry attempt; the later independent GitHub
delivery is recorded above.

The 0.2 implementation slice is complete:

- `cluster`, with the complete documented BED distance and same-strand modes;
- `window`, with symmetric, asymmetric, strand-relative, pair, any, count, and
  no-hit modes for BED inputs;
- `closest`, with one B input, overlap/no-hit behavior, same/opposite strand,
  different-name filtering, unsigned or signed distance, and all/first/last
  tie policy.

Multi-database closest, `k > 1`, and upstream/downstream preference controls
remain absent from this slice rather than accepted and ignored. BAM inputs are
not part of the BED command contract.

This slice exercises one product-private relation index and one streaming
cluster state machine. It does not create or expand a public foundation.

## Compatibility and performance gates

- CI pins the BEDTools 2.31.1 archive digest and runs the live executable on
  native Linux and macOS for `x86_64` and `aarch64`.
- Fixed goldens and deterministic differentials cover every declared option
  family, zero-length case, strand and tie mode, no-hit output, malformed
  selected fields, duplicates, nesting, and coordinate boundaries.
- The one-million-record relation gate verifies complete output before ten
  alternating measurements. Cluster passes at 4.07-5.19x, window pairs at
  4.17x, closest at 1.23-1.36x, and dense window count at 13.98x.
- The first benchmark head failed default closest at 0.943x. Compact B storage
  and index-order reuse moved it to 1.23x and reduced RSS from 577,304 to
  316,696 KiB before the candidate was accepted.
- The original one-million-record five-operation gate was regenerated after
  the shared index refactor. All five retain strict throughput advantages;
  merge is the narrowest at 1.11x and must be remeasured for later releases.
- Exact identities, fixture/output hashes, wall and CPU distributions, RSS,
  raw JSON paths, and the closest memory trade-off are recorded in
  `bed-performance-2026-08-20.md`.

## Explicit exclusions

- No revived operation-sized repositories or public `rsomics-bed-*` crates.
- No generic dataframe, annotation graph, BAM parser, or FASTA index inside
  this product.
- No GFF/GTF/VCF compatibility mode that bypasses their owning products.
- No automatic malformed-row repair, implicit sorting, coordinate clamping,
  or unsupported-option acceptance.
- No Starch format, browser HTML, IGV automation, or GUI behavior in the BED
  binary.
