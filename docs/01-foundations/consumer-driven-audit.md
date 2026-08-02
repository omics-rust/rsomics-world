# Consumer-driven foundation audit

Status: initial source and API audit complete. No foundation API is frozen by
this document.

The nine retained foundations are justified at the product level, but most
current APIs reflect the deleted operation-sized topology. They evolve through
real product slices, not through an independent “finish all common crates”
phase.

## Disposition

| Foundation | Decision | Initial product drivers |
|---|---|---|
| `rsomics-common` | keep; refactor command/error/output contract | `seq`, `fastq-preprocess`, `bed`; later all 30 products |
| `rsomics-help` | keep; replace the duplicate renderer with the family CLI UX adapter | `seq`, `fastq-preprocess`, `bed` |
| `rsomics-seqio` | keep; redesign around FASTA/FASTQ stream contracts | `seq`, `fastq-preprocess`, `fastq-qc`, `minimap2` |
| `rsomics-kmer` | keep; repair boundaries and expose only general primitives | `seq`; later `metagenomics`, `sketch` |
| `rsomics-intervals` | keep; repair coordinate safety and remove BED policy | `bed`, `annotation`, `peak` |
| `rsomics-bamio` | keep; narrow concrete backend types | `bam`, `call`, `count`, `methyl`, `minimap2`, `peak` |
| `rsomics-pileup` | keep; add sortedness and real compatibility gates | `bam`, `call`, `methyl` |
| `rsomics-stats` | keep; migrate only primitives used by two workflows | `composition`, DE workflows, `sc`, `ecology`, `popgen`, `plink` |
| `rsomics-phylo-tree` | keep; re-establish topology and Newick invariants | `composition`, `phylo`, `ecology` |
| `rsomics-igzip` | temporary; internalize into sequence I/O | `seqio` is the only current consumer |

## Audited source snapshot

The findings below were reviewed against these local revisions on 2026-07-30:

| Repository | Revision | Worktree state during review |
|---|---|---|
| `rsomics-common` | `ed02bcb9f813` | clean |
| `rsomics-help` | `ee7a085cbd9a` | clean |
| `rsomics-seqio` | `979b609cb87d` | source clean; untracked `Cargo.lock` |
| `rsomics-kmer` | `4a31a9ea646a` | clean |
| `rsomics-intervals` | `9ed437ea7b03` | clean |
| `rsomics-bamio` | `dc4b19df5bc6` | clean |
| `rsomics-pileup` | `5bd34dde15c5` | source clean; untracked `Cargo.lock` |
| `rsomics-stats` | `bac010ed3abf` | source clean; modified `Cargo.lock` |
| `rsomics-phylo-tree` | `25a10b1eac04` | clean |
| `rsomics-igzip` | `4c92ddfb3cd6` | clean |

The audit did not treat lockfile-only changes as source evidence and did not
modify or discard them. Each implementation wave rechecks the live revision
and worktree ownership before editing.

## Reconstruction advances

The initial audit snapshot above remains the provenance baseline. These later
commits close specific correctness blockers without freezing the foundation
APIs:

| Foundation | Current revision | Verified change | Release state |
|---|---|---|---|
| `rsomics-common` | `220823993a06` | supplies the demonstrated error, exit-code, JSON-envelope, runner, single-output transaction, alias, worker, and paired-output contracts; metagenomics consumes only the single-output and runtime subset rather than inheriting unrelated controls | 0.12.0 published; exact-head four-native-target CI `30736910174`; publish run `30736966376`; downloaded archive checksum `cf3e31808c6a131ab789ee4a135bf46fd3eaafb87d3dc0d264e6e620f74bc3e7` |
| `rsomics-help` | `61dd6f2ce0ce` | replaced the duplicate `HelpSpec` renderer and argv interception with recursive styling and parsing of the authoritative Clap command tree | 0.4.0 published; exact-head four-native-target CI `30596121607`; downloaded archive checksum `5922ec5a261660869fc36aa05f731c0adb059c43344eb78c2393f05611797fe1`; registry-package tests green |
| `rsomics-intervals` | `6783f67614ae` | reduced the public crate to a validated generic zero-based half-open interval value; moved BED parsing, collections, algebra, merge policy, and COITrees indexing into products | 0.3.0 published; exact-head four-native-target CI `30597681539`; downloaded archive checksum `40cf072a5fb5900d8e4049cb9b03f28ce5ddc51e51ef2b3fed7c5c89bfa88ccd`; `bed` and `annotation` pass consumer tests against the registry release |
| `rsomics-kmer` | `d89e2df0d8ea` | retained the checked sequence-counting API and added allocation-reusing canonical DNA-window Murmur64 hashing with arbitrary nonzero `k` and a full-width seed after `rsomics-sketch` supplied the second concrete consumer | 0.2.2 published; exact-head four-native-target CI `30734719265`; publish run `30734861858`; downloaded archive checksum `e1254977d1eaf89b29e727b7ea552ec8bd4bd0740b45fa40ac943e93ffaf9ed4`; sequence and sketch consumer suites green |
| `rsomics-seqio` | `4f0e2311f9ac` | retains strict allocation-reusing FASTA/FASTQ streams, bounded gzip decode buffering, wrapped FASTQ support, and fail-loud gzip/BGZF handling; 0.5 aligns the shared runtime contract with `rsomics-common` 0.12 without adding product policy | 0.5.0 published; exact-head four-native-target CI `30744450003`; publish run `30744498235`; downloaded archive checksum `15d43fe84756988ea45f2b39b5b1b745f7ea0b090ec1f1c74be8eaddf07838c6`; `seq`, `fastq-preprocess`, `phylo`, and `metagenomics` consumer paths are green |
| `rsomics-bamio` | `3a270c7f6bfd` | retains the validated raw-record encoder, compact cached variable-record layout, and indexed SAM/BAM/CRAM reader while aligning the shared runtime contract with `rsomics-common` 0.12 for call, BAM, count, pileup, and methyl consumers | 0.5.0 published; exact-head four-native-target CI `30737055735`; publish run `30737138969`; downloaded archive checksum `42cc41695faacaa9607db04a5d8dd183aa90bfcbd0d5722b1e839b67eff35ee7` |
| `rsomics-pileup` | `7ab53a7cafc7` | retains checked projection, retry-safe borrowed columns, per-source overlap and depth state, HTSlib-compatible BAQ, and bcftools-compatible column preparation while aligning its raw-record dependency with `rsomics-bamio` 0.5; the benchmark entry point now ignores Cargo harness arguments correctly | 0.5.0 published after methyl supplied a third concrete consumer; exact-head four-native-target CI `30750094663`; publish run `30750341424`; downloaded archive checksum `089bd82c951451b21108b28f894bd5c896c75bbde33cac64f43ec17a64a7b18b`; ordinary and 250x benchmark paths pass |
| `rsomics-phylo-tree` | `63e39e14964b` | replaces public mutable topology with checked construction, immutable node views, traversal, and strict iterative Newick parsing and serialization; composition uses named topology for ILR bases and phylo uses the same API for inference, comparison, and measures | 0.2.0 published; exact-head four-native-target CI `30742259200`; publish run `30742377050`; downloaded archive checksum `07234bb701159253e249cd8fccec70e728cb46e4999ec851f51dc549ad829fde`; the second concrete product consumer shipped in `rsomics-phylo 0.1.0` |

Publication does not freeze these APIs. Every later public item still requires
two named product consumers and consumer-side tests.

## Correctness blockers

These are migration blockers because they can panic internally or silently
produce wrong biological results:

1. `rsomics-intervals/src/index.rs` previously cast public `u64` coordinates to
   `i32` without a checked boundary; `c13cb75c318` closes this blocker.
2. `rsomics-kmer/src/encode.rs` previously shifted by 64 bits for the valid
   `k = 32` reverse-complement case and trusted a debug-only constructor
   assertion; `4258ac881119` closes both blockers and supplies the fallible
   accumulator constructor exercised by `rsomics-seq`.
3. `rsomics-pileup` previously documented coordinate-sorted input without
   enforcing it; `2b2cb7071381` closes the ingestion, coordinate, CIGAR, and
   sorted-watermark blocker before consumer integration.
4. `rsomics-common` previously ignored JSON serialization and output write
   errors; `9f11f37c0fa4` closes this blocker and supplies a plain-diagnostic
   fallback when the JSON output path itself fails.
5. `rsomics-bamio::RawRecord::from(Vec<u8>)` previously permitted unchecked
   bytes while accessors assumed a valid structure. The fallible constructor
   and `RawRecordEncoder` at `3bcbe0ed9bb2` close this boundary.
6. `rsomics-phylo-tree::Tree::default()` did not establish a valid root and
   public node fields permitted topology invariant violations. Checked
   construction and immutable views at `71af2cd` close this blocker.

These are fixed in their consumer wave before the dependent product migration
uses the API. They are not reasons to speculatively rewrite every foundation
first.

## API corrections

### CLI wave

`rsomics-common` and `rsomics-help` are driven together by the `seq`,
`fastq-preprocess`, and `bed` command trees.

- `rsomics-help` owns help, version, terminal/color policy, Clap errors,
  suggestions, and command navigation.
- `rsomics-common` owns typed runtime errors, exit codes, JSON result envelopes,
  and result-to-process mapping.
- The real Clap command tree is the only argument and help model.
- Semantic argument groups remain on the product's existing Clap types.
- Serialization and output failures propagate rather than being swallowed.

The superseded 0.3 `rsomics-help` API duplicated a second `HelpSpec` tree and
was used only by the unreconstructed `rsomics-minimap2`. The published 0.4
implementation removes that model, recursively decorates the authoritative
Clap tree, and reduces the normal consumer call to
`rsomics_help::parse::<Cli>()`. It has passed the existing command,
compatibility, and benchmark test suites of all three pilot products. See
[`help-consumer-contract.md`](help-consumer-contract.md).

### Sequence wave

`rsomics-seqio` becomes a strict FASTA/FASTQ reader/writer over paths and
standard streams, with borrowed streaming and owned batch forms. Concrete
threading, slab, and compression backends remain private.

`rsomics-kmer` retains checked 2-bit encode/decode, canonicalization, rolling
iteration, and general hashes. Product-specific correction tables and QC bins
remain internal.

`rsomics-seq` revision `d4c840be2e37` consumes the checked accumulator and the
strict `rsomics-seqio` stream API directly. Its complete five-command first
slice passes exact-head CI on all four native targets with live SeqKit
differentials, an independent ordered k-mer oracle, the unified help layer, and
only the shared JSON output option. Its representative Linux gate also matches
Jellyfish for 104,521 canonical count rows.

`rsomics-fastq-preprocess` revision `442c202908d1` consumes
`rsomics-common` and `rsomics-seqio` without depending on `rsomics-kmer`.
Its initial trim/filter pipeline passes exact-head CI on all four native
targets with live fastp differentials. Its private `--threads` control builds a
local Rayon pool for each execution instead of mutating process-global state.
The product internalizes the historical
`rsomics-fqgz` chunked-libdeflate algorithm behind its transactional writer;
it does not add a public foundation or bypass `rsomics-seqio` validation and
serialization. On provenance-checked SRR341550 paired input, the four-thread
Linux `x86_64` path is byte-identical, 1.28 times faster, and uses about 69%
less peak RSS than the aligned fastp slice. The single-end path is slower but
uses about 63% less peak RSS, so it is recorded as a resource advantage rather
than a throughput claim.
Together these products establish the current common, help, and sequence-I/O
contracts. The BED pilot is another help/common consumer.

`rsomics-sketch 0.1.0` is now the second concrete `rsomics-kmer` product
consumer. It drove the 0.2.2 canonical Murmur64 iterator, including arbitrary
nonzero `k`, a full-width seed, allocation-reusing scratch buffers, ambiguity
boundaries, and sourmash-compatible byte order. Randomized consumer tests and
real-fixture output hashes match sourmash 4.9.4, while the foundation retains
no FracMinHash, signature, comparison, or search policy. The future
metagenomics minimizer call site remains unimplemented and cannot justify more
public API on its own.

`rsomics-metagenomics 0.1.0` consumes the same strict `rsomics-seqio` reader,
gzip/BGZF detection, shared transaction boundary, JSON envelope, and unified
help layer for its amplicon abundance lifecycle. Its consumer tests required no
new `rsomics-seqio` or `rsomics-kmer` item. The classifier minimizer call site
therefore remains a dotted future relationship rather than speculative public
API.

Neither historical `rsomics-kmer-dist` nor `rsomics-tax-assign` demonstrates
an additional contract. The former keeps every exact k-mer and the latter
silently drops invalid windows and does not perform taxonomy LCA assignment.
Publication does not freeze the current API; later additions still require two
named consumers and their tests.

The consumers exposed and resolved a real difference in the old common runtime
contract: preprocessing uses `--threads` to size its Rayon work, while
`rsomics-seq` and `rsomics-bed` did not use the shared thread, seed, quiet, or
verbose flags. Common 0.7 removes those speculative controls. The committed
product migrations keep thread ownership inside preprocessing and remove
inapplicable flags from the other command trees.

Parallel gzip remains product-private because only preprocessing currently
needs the thread-controlled contract. If `rsomics-seq` demonstrates the same
need with consumer tests and representative measurements, the backend can move
behind the existing `rsomics-seqio` writer contract without exposing
product-specific policy.

`rsomics-igzip` accepts no new consumers. Its native backend is integrated
privately or replaced after equivalent compatibility, throughput, and memory
evidence.

### Interval wave

`rsomics-intervals 0.3.0` is the smallest coordinate-safe geometry API
demonstrated by two products. `Interval<C>` validates `start <= end`, keeps its
fields private, permits borrowed or owned chromosome identifiers, and exposes
only accessors and basic half-open geometry.

`rsomics-bed` revision `e8898dbcb0db` embeds the shared value in its retained
and streaming BED records. BED parsing, zero-length behavior, merge policy,
coverage maps, and the checked COITrees adapter remain product-private. Its
million-record Linux gate matches bedtools byte for byte for all five
operations; intersect is 3.99 times faster with roughly one third the peak RSS,
and subtract is 5.09 times faster with about one eighteenth the peak RSS.

`rsomics-annotation` release revision `8e7beed4d51e` converts inclusive GFF/GTF
features once into the shared half-open value and consumes
`rsomics-common 0.11.0` for single-output transaction and alias safety.
Annotation hierarchy, splicing, coordinated extraction outputs, and FASTA
access remain inside the product. Both interval consumers resolve the
published registry archive rather than a path patch. No public overlap index
remains to justify speculatively; another product may propose one only after a
second concrete consumer exists.

### Alignment wave

`rsomics-bamio` exposes validated records and stable reader/writer contracts,
not every current batch or work-stealing implementation type. `rsomics-count`
adds a concrete sequential and parallel record-reader consumer without moving
feature assignment or annotation policy into the foundation. `rsomics-methyl`
adds BAM/CRAM records, indexed regions, and bisulfite-specific aux-tag
consumption while keeping methylation policy product-private.

`rsomics-pileup` revision `7ab53a7cafc7` supplies fallible ingestion,
low-allocation borrowed column views, retry-safe output callbacks, checked
header and projection bounds, BAM long-CIGAR replacement, exact flag-filter
semantics, raw-reference-span behavior, source-isolated overlap state, and an
optional per-source active-depth ceiling. It also supplies standard and
extended BAQ, existing `BQ`/`ZQ` conversion, full realignment, and the
bcftools 1.24 partial-realignment trigger without moving maximum-read-length or
mode selection out of products. Its live samtools 1.24 oracles cover matches,
insertions, deletions, skips, padding, clipping, strand, head/tail markers,
ordinary or indel-bearing overlapping mates, and independent input depth
policy. Exact-head four-native-target CI `30750094663` passes. The published
0.5.0 archive has checksum
`089bd82c951451b21108b28f894bd5c896c75bbde33cac64f43ec17a64a7b18b`.

`rsomics-call` revision `85579cb94f9a` validates and coordinate-merges plain or
BGZF SAM, raw or BGZF BAM, and CRAM sources; resolves source and read-group
metadata into samples; streams columns into typed multisample SNP likelihood
sites; and applies the bcftools-compatible depth, overlap, BAQ, and
deterministic deep-evidence policies. Full, overlap-ordered, and default
partial BAQ likelihoods match bcftools 1.24. Its product-owned consensus and
multiallelic callers, checked VCF/BCF likelihood schema, and fused typed path
remain product policy. The repository consumes the published pileup and bamio
archives rather than Git or path patches. It passes 117 release library tests,
seven ordinary CLI tests, and 21 live bcftools 1.24 oracle groups. On the
declared deterministic 5 Mb, 30x Linux fixture, fused `run` has 1.6% lower
median wall time and 52.8% lower peak RSS than bcftools/HTSlib 1.24 while all
5,024 normalized calls match. Release head `b34cc226242` passes exact-head
four-native-target CI `30722248488`; publish run `30722470067` succeeds, and
the non-yanked 0.1.0 archive checksum is
`83e2750f2b73b477da315d76f619db11c74e369528851a5108a69f4dd52bbde5`.

`rsomics-bam` revision `d3be2001212a` supplies the second concrete pileup
consumer through `mpileup`. Coordinate-sorted SAM, BAM, and CRAM feed the same
pileup engine; unchanged BAM bodies use the raw reader while generic records
use the shared bamio encoder. Default, quality/flag/depth, overlap, all-position,
reference, standard/full/redo BAQ, insertion, deletion, skip, and head/tail
text behavior matches samtools 1.24 across its declared oracle matrix.
Transactional named output and the shared rsomics JSON envelope remain product
concerns. Exact-head four-native-target CI `30654810659` passes.

The ordinary and 250× engine gate records bounded RSS and more than 95 million
entries/s in every case, including the partial trigger scan. This establishes
the two-consumer gate that permitted the original publication. Methyl
extraction is now the third consumer for checked columns; cytosine context,
bisulfite filtering, and methylation-specific mate adjustment remain inside
the product.

### Analysis wave

`rsomics-stats` absorbs a numerical primitive only when two products use the
same typed semantics. It does not become a container for all 91 historical
statistics binaries. Composition's optional-value p-adjustment implementation
remains product-local because no second implemented product currently exercises
that contract.

`rsomics-phylo-tree 0.2.0` closes node mutation and root invariants and supports
the declared strict Newick grammar. Composition consumes only validated
topology and tip identities for tree-derived ILR bases. The iterative parser
and serializer also pass a 20,000-level ladder-tree regression without relying
on recursive descent or recursive formatting.

`rsomics-phylo 0.1.0` now supplies the second implemented contract: checked
construction, immutable topology, traversal, root interpretation, tip identity,
and Newick parsing/emission. Its NJ/UPGMA builders, RF-family split policy,
patristic matrices, measures, and result schemas remain product-local. Exact-head
four-native-target CI `30747064784`, the live scikit-bio/DendroPy/trimAl oracle,
and the published archive demonstrate the foundation boundary without moving
phylo-specific policy into `rsomics-phylo-tree`.

The `rsomics-ecology` dossier supplies the community-diversity contract:
validated topology, postorder traversal, finite branch views, and checked tip
identity for Faith PD, generalized PD, and UniFrac. Abundance accumulation,
diversity formulas, distance matrices, ordination, and permutation policy stay
in the product. The historical `rsomics-distance` consumers all collapse into
ecology, so that crate is internalized rather than added to the foundation set.

Ecology's initial diversity slice requires no new `rsomics-stats` API. Later
inference work promotes a numerical item only when composition or another
named product demonstrates the same finite-value and result contract.

## Completion gate per wave

A foundation wave is complete when:

- at least two named products call the public API;
- consumer-contract tests pass in those products;
- historical compatibility assets are preserved or superseded explicitly;
- hot-path performance and memory are measured where relevant;
- a fresh public-API review finds no product-specific policy exposed.
