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
| `rsomics-common` | keep; refactor command/error/output contract | `seq`, `fastq-preprocess`, `bed` |
| `rsomics-help` | keep; replace the duplicate renderer with the family CLI UX adapter | `seq`, `fastq-preprocess`, `bed` |
| `rsomics-seqio` | keep; redesign around FASTA/FASTQ stream contracts | `seq`, `fastq-preprocess`, `fastq-qc`, `minimap2` |
| `rsomics-kmer` | keep; repair boundaries and expose only general primitives | `seq`; later `metagenomics`, `sketch` |
| `rsomics-intervals` | keep; repair coordinate safety and remove BED policy | `bed`, `annotation`, `peak` |
| `rsomics-bamio` | keep; narrow concrete backend types | `bam`, `vcf`, `count`, `methyl`, `minimap2`, `peak` |
| `rsomics-pileup` | keep; add sortedness and real compatibility gates | `bam`, `vcf`, `methyl` |
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
| `rsomics-common` | `9f11f37c0fa4` | narrowed the public runtime to demonstrated error, exit-code, output-mode, JSON-envelope, and runner contracts; removed speculative thread, RNG, logging, file, fixture, and tool abstractions; added fail-loud JSON emission fallback | exact-head four-native-target CI, strict Clippy, 26 tests, and package verification green; unpublished; local coordinated graph verified with `seq`, `fastq-preprocess`, `bed`, `seqio`, and `intervals` |
| `rsomics-help` | `c615aa8b8522` | replaced the duplicate `HelpSpec` renderer and argv interception with recursive styling and parsing of the authoritative Clap command tree | exact-head four-native-target CI, strict Clippy, six tests, and package verification green; unpublished; all three pilot product suites pass in local patched worktrees |
| `rsomics-intervals` | `491b14c0d43b` | checked the COITrees coordinate boundary; aligned version 0.3 with common 0.7; repaired package metadata and four-native-target CI; removed narrative comments without changing behavior | exact-head four-native-target CI, strict Clippy, 48 unit tests, six property tests, and package verification green; unpublished; `bed` and `annotation` now exercise the coordinate model, while the index still lacks a second consumer and its BED-policy and performance gates remain |
| `rsomics-kmer` | `4258ac881119` | made `k = 32` well-defined, added checked encode/decode/canonical operations and a fallible count-accumulator boundary, and removed its unused `rsomics-common` dependency | exact-head CI green; `rsomics-seq` is the first real product consumer; a second product contract and comparative performance remain |
| `rsomics-seqio` | `b23cf8ad29fd` | replaced the ambiguous record model with strict allocation-reusing FASTA/FASTQ streams, bounded gzip decode buffering, wrapped FASTQ support, and fail-loud gzip/BGZF handling; aligned version 0.3 with common 0.7 | exact-head four-native-target CI, strict Clippy, 45 unit tests, five compatibility tests, benchmark smoke, and package verification green; exercised by both `rsomics-seq` and `rsomics-fastq-preprocess`; unpublished |

None of these revisions has been published. A green foundation CI establishes
the implementation baseline; it does not replace the two-consumer completion
gate below.

## Correctness blockers

These are migration blockers because they can panic internally or silently
produce wrong biological results:

1. `rsomics-intervals/src/index.rs` previously cast public `u64` coordinates to
   `i32` without a checked boundary; `c13cb75c318` closes this blocker.
2. `rsomics-kmer/src/encode.rs` previously shifted by 64 bits for the valid
   `k = 32` reverse-complement case and trusted a debug-only constructor
   assertion; `4258ac881119` closes both blockers and supplies the fallible
   accumulator constructor exercised by `rsomics-seq`.
3. `rsomics-pileup` documents coordinate-sorted input but does not validate it.
4. `rsomics-common` previously ignored JSON serialization and output write
   errors; `9f11f37c0fa4` closes this blocker and supplies a plain-diagnostic
   fallback when the JSON output path itself fails.
5. `rsomics-bamio::RawRecord::from(Vec<u8>)` permits unchecked bytes while
   accessors assume a valid structure.
6. `rsomics-phylo-tree::Tree::default()` does not establish a valid root and
   public node fields permit topology invariant violations.

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

The published 0.3 `rsomics-help` API duplicates a second `HelpSpec` tree and is
used only by the unreconstructed `rsomics-minimap2`. The committed 0.4
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

`rsomics-seq` revision `2727daa3bf4f` consumes the checked accumulator and the
strict `rsomics-seqio` stream API directly. Its complete five-command first
slice passes exact-head CI on all four native targets with live SeqKit
differentials, an independent ordered k-mer oracle, the unified help layer, and
only the shared JSON output option. Its representative Linux gate also matches
Jellyfish for 104,521 canonical count rows.

`rsomics-fastq-preprocess` revision `f217fc4902b2` consumes
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
Together these two products establish the second concrete consumer contract
for the current common, help, and sequence-I/O APIs. The BED pilot is the third
help/common consumer. This does not freeze `rsomics-kmer`, which still requires
its own second product consumer.

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

The release target for `rsomics-intervals` is the smallest coordinate-safe
geometry API demonstrated item by item. Version 0.3 still exposes a
non-generic overlap index plus BED parsing, sorting, merging, and writing.
Those items are not release-approved foundation API until two products
demonstrate the same policy-free contract. Otherwise they move into the
consuming product.

`rsomics-bed` revision `9f4ba8ee945c` is the first concrete checked-index
consumer. Intersect uses the foundation's fallible build/query boundary
directly; subtract uses a separate merged `u64` coverage map and no longer
constructs an unused overlap tree. CI patches intervals 0.3 and one common 0.7
instance without a committed path dependency. The earlier representative
million-record gate matches bedtools output and passes throughput on all five
operations without adding another shared crate; the revised subtract hot path
still requires representative remeasurement before publication.
`rsomics-annotation` revision `80920fb9e72b` provides the second
coordinate-model contract through one checked conversion from inclusive
GFF/GTF features to the shared half-open interval type. Final-head CI run
`30574846937` passes all 40 product tests on four native targets, including
live gffread differentials for selection, conversion, and sequence extraction.
Extraction adds no speculative public foundation: annotation-specific
hierarchy and splicing stay inside the product, while FASTA random access uses
the aligned external noodles format graph. The BED parsing and sorting
functions currently exposed by the foundation still require a fresh policy
review before intervals 0.3 is published. Annotation does not naturally need
the checked tree for its streaming operations, so it is not a second
`IntervalIndex` consumer. That item remains unpublished until `peak`, `signal`,
or another real product demonstrates the same policy-free query contract.

### Alignment wave

`rsomics-bamio` exposes validated records and stable reader/writer contracts,
not every current batch or work-stealing implementation type. `rsomics-count`
adds a concrete sequential and parallel record-reader consumer without moving
feature assignment or annotation policy into the foundation. `rsomics-methyl`
adds BAM/CRAM records, indexed regions, and bisulfite-specific aux-tag
consumption while keeping methylation policy product-private.

`rsomics-pileup` adds input-order validation, low-allocation column views,
complex CIGAR and overlap tests, filter-combination tests, and real oracle
comparison before product use. Methyl extraction provides the third consumer
for checked columns and generic mate-overlap evidence; cytosine context and
bisulfite calling do not enter the foundation.

### Analysis wave

`rsomics-stats` absorbs a numerical primitive only when two products use the
same typed semantics. It does not become a container for all 91 historical
statistics binaries.

`rsomics-composition` supplies concrete contracts for p-value adjustment and
selected statistical tests. ANCOM-specific orchestration and cutoff policy
remain in the product.

`rsomics-phylo-tree` closes node mutation and root invariants and supports the
declared Newick grammar before `composition`, `phylo`, and `ecology` depend on
it. Composition consumes only validated topology and tip identities for
tree-derived ILR bases.

The `rsomics-phylo` dossier supplies the second concrete contract: checked
construction, immutable topology, traversal, root interpretation, tip identity,
and Newick parsing/emission. Inference, split-distance policy, tree measures,
and result schemas stay in the product. The current foundation's public mutable
nodes, invalid `Default` tree, incomplete label grammar, and unchecked
non-finite branch lengths block its next release.

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
