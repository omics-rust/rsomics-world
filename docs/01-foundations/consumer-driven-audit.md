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
| `rsomics-common` | `10a8b4708d36` | narrowed the public runtime to demonstrated error, exit-code, output-mode, JSON-envelope, and runner contracts; added transactional named output after BED and VCF supplied consumers, then added format-neutral structured validation only for implemented VCF and planned BAM consumers | 0.9.0 published; exact-head four-native-target CI `30626561250`; publish run `30626628961`; downloaded archive checksum `00ecf388c7cccd2792438260c90e121f6de758c519f76ed64ff24b61f2e4ac78` |
| `rsomics-help` | `61dd6f2ce0ce` | replaced the duplicate `HelpSpec` renderer and argv interception with recursive styling and parsing of the authoritative Clap command tree | 0.4.0 published; exact-head four-native-target CI `30596121607`; downloaded archive checksum `5922ec5a261660869fc36aa05f731c0adb059c43344eb78c2393f05611797fe1`; registry-package tests green |
| `rsomics-intervals` | `6783f67614ae` | reduced the public crate to a validated generic zero-based half-open interval value; moved BED parsing, collections, algebra, merge policy, and COITrees indexing into products | 0.3.0 published; exact-head four-native-target CI `30597681539`; downloaded archive checksum `40cf072a5fb5900d8e4049cb9b03f28ce5ddc51e51ef2b3fed7c5c89bfa88ccd`; `bed` and `annotation` pass consumer tests against the registry release |
| `rsomics-kmer` | `4258ac881119` | made `k = 32` well-defined, added checked encode/decode/canonical operations and a fallible count-accumulator boundary, and removed its unused `rsomics-common` dependency | exact-head CI green; `rsomics-seq` is the first real product consumer; a second product contract and comparative performance remain |
| `rsomics-seqio` | `d7e1c33bb600` | retained strict allocation-reusing FASTA/FASTQ streams, bounded gzip decode buffering, wrapped FASTQ support, and fail-loud gzip/BGZF handling while removing unconsumed legacy, forced-format, and compression-policy APIs | 0.3.0 published; exact-head four-native-target CI `30599703477`; downloaded archive checksum `d2dcd0fab1a5320834a9b0f9cba7bbdd9bfe6b26c9c4740650ac88d939fcfcc5`; `seq` and `fastq-preprocess` pass consumer tests against the registry release |
| `rsomics-pileup` | `353e5625199e` | retains the checked projection and retry-safe borrowed-column contract, isolates mate-overlap state by input stream, applies an optional per-source depth ceiling, and adds HTSlib-compatible standard/extended BAQ plus bcftools-compatible full and partial column preparation | 0.2 API remains unpublished while consumers integrate; exact-head four-native-target CI `30651430890` passes, including pinned samtools 1.24 column and overlap oracles; the BAM consumer and representative performance evidence remain |

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
Together these two products establish the second concrete consumer contract
for the current common, help, and sequence-I/O APIs. The BED pilot is the third
help/common consumer. This does not freeze `rsomics-kmer`, which still requires
its own second product consumer.

The metagenomics/sketch dossier now supplies two concrete next call sites for
that review: canonical rolling hashes in the DNA FracMinHash builder and
checked minimizer generation in the taxonomy-labelled database builder.
Neither historical `rsomics-kmer-dist` nor `rsomics-tax-assign` demonstrates
the contract. The former keeps every exact k-mer and the latter silently drops
invalid windows and does not perform taxonomy LCA assignment. No k-mer API is
added or published until the new consumer tests and representative memory
measurements exist.

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

`rsomics-annotation` revision `0e2d4c94e990` converts inclusive GFF/GTF
features once into the shared half-open value. Annotation hierarchy, splicing,
and FASTA access remain inside the product. Both consumers resolve the
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

`rsomics-pileup` revision `353e5625199e` now supplies fallible ingestion,
low-allocation borrowed column views, retry-safe output callbacks, checked
header and projection bounds, BAM long-CIGAR replacement, exact flag-filter
semantics, raw-reference-span behavior, source-isolated overlap state, and an
optional per-source active-depth ceiling. It also supplies standard and
extended BAQ, existing `BQ`/`ZQ` conversion, full realignment, and the
bcftools 1.24 partial-realignment trigger without moving maximum-read-length or
mode selection out of products. Its live samtools 1.24 oracles cover matches,
insertions, deletions, skips, padding, clipping, strand, head/tail markers,
ordinary or indel-bearing overlapping mates, and independent input depth
policy. Exact-head four-native-target CI `30651430890` passes.

`rsomics-call` revision `81898da610d8` supplies the first published-head
integration: it validates and coordinate-merges plain or BGZF SAM, raw or BGZF
BAM, and CRAM sources; resolves source and read-group metadata into samples;
streams columns into typed multisample SNP likelihood sites; and applies the
bcftools-compatible depth and deterministic deep-evidence policies. Its
reference-only, two-sample, per-input-depth, and consecutive deep-coverage
oracles match bcftools/HTSlib 1.24. Its product-owned multiallelic caller adds
bcftools-matched ploidy, grouping, allele-selection, genotype, and quality
contracts without promoting those policies into the foundation. Its consensus
caller matches the bcftools 1.24 allele-frequency posterior for diploid and
haploid calls. Its fused typed path matches the materialized pipeline, and its
product-private format layer streams likelihood and called records through
plain VCF, BGZF VCF, raw BCF, and BGZF BCF with checked schemas, record-local
input and call errors, and fallible output finalization. Exact-head
four-native-target CI `30648436539` passes.
Local-only revision `cf735d7` then drives full and default partial BAQ through
the column-preparation API after overlap handling. Full, overlap-ordered, and
indel-triggered likelihoods match bcftools 1.24. Its formatting, strict
Clippy, debug and release tests, and rustdoc pass; package verification
correctly remains unavailable until `rsomics-pileup 0.2` is published. The
commit is not pushed or treated as release evidence yet. `rsomics-bam` must
still add the second product-side contract, and representative performance and
memory must be measured before foundation publication. Methyl extraction later
provides the third consumer for checked columns and generic mate-overlap
evidence; cytosine context and bisulfite calling do not enter the foundation.

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
