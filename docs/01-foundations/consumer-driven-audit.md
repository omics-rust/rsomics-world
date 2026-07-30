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
| `rsomics-seqio` | keep; redesign around FASTA/FASTQ stream contracts | `seq`, `fastq-preprocess`, `fastq-qc` |
| `rsomics-kmer` | keep; repair boundaries and expose only general primitives | `seq`; later `metagenomics`, `sketch` |
| `rsomics-intervals` | keep; repair coordinate safety and remove BED policy | `bed`, `annotation` |
| `rsomics-bamio` | keep; narrow concrete backend types | `bam`, `vcf` |
| `rsomics-pileup` | keep; add sortedness and real compatibility gates | `bam`, `vcf` |
| `rsomics-stats` | keep; migrate only primitives used by two workflows | DE workflows, `sc`, `ecology`, `popgen`, `plink` |
| `rsomics-phylo-tree` | keep; re-establish topology and Newick invariants | `phylo`, `ecology` |
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
| `rsomics-intervals` | `c13cb75c318` | checked the coordinate range accepted by the COITrees backend and added fallible index/query entry points | exact-head CI green; consumer contracts, four-native-target CI, and performance evidence remain |
| `rsomics-kmer` | `4258ac881119` | made `k = 32` well-defined, added checked encode/decode/canonical operations and a fallible count-accumulator boundary, and removed its unused `rsomics-common` dependency | exact-head CI green; `rsomics-seq` is the first real product consumer; a second product contract and comparative performance remain |
| `rsomics-seqio` | `ce9c5514c235` | replaced the ambiguous record model with strict allocation-reusing FASTA/FASTQ streams, bounded gzip decode buffering, added wrapped FASTQ and tab-bearing header support, removed direct `rsomics-igzip` use, made gzip/BGZF failures loud, and accelerated printable-byte validation without changing its public contract | exact-head four-native-target CI and compressed-stream adversarial regressions green; exercised by both `rsomics-seq` and `rsomics-fastq-preprocess`; real-fixture throughput and RSS pass on macOS arm64, while Linux and full operation coverage remain |

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
   errors; `1c51f7d0b356` closes this blocker without expanding its public
   error or exit-code vocabulary.
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

The 0.3 `rsomics-help` API duplicates a second `HelpSpec` tree and is used only
by the unreconstructed `rsomics-minimap2`. The 0.4 prototype removes that
model, recursively decorates the authoritative Clap tree, and reduces the
normal consumer call to `rsomics_help::parse::<Cli>()`. It has passed the
existing command, compatibility, and benchmark test suites of all three pilot
products. See
[`help-consumer-contract.md`](help-consumer-contract.md).

### Sequence wave

`rsomics-seqio` becomes a strict FASTA/FASTQ reader/writer over paths and
standard streams, with borrowed streaming and owned batch forms. Concrete
threading, slab, and compression backends remain private.

`rsomics-kmer` retains checked 2-bit encode/decode, canonicalization, rolling
iteration, and general hashes. Product-specific correction tables and QC bins
remain internal.

`rsomics-seq` revision `02f8268931b0` consumes the checked accumulator and the
strict `rsomics-seqio` stream API directly. Its complete five-command first
slice passes exact-head CI on all four native targets with live SeqKit
differentials and an independent ordered k-mer oracle. Its representative
Linux gate also matches Jellyfish for 104,521 canonical count rows.

`rsomics-fastq-preprocess` revision `8e483fc95556` consumes
`rsomics-common` and `rsomics-seqio` without depending on `rsomics-kmer`.
Its initial trim/filter pipeline passes exact-head CI on all four native
targets with live fastp differentials. The product internalizes the historical
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

The consumers expose a real difference in the current common runtime contract:
preprocessing uses `--threads` to size its Rayon work, while `rsomics-seq`
initializes the same shared pool without using it. `rsomics-bed` also accepts
`--threads` and `--seed`; none of its five current operations uses either
value, and its published intervals dependency re-enables common's default
Rayon feature. One- and four-thread controls show no sequence-product scaling.
Before freezing the common CLI API, a product must not advertise an
inapplicable shared flag. The resolution needs capability-selective
consumer-side command-tree tests while preserving preprocessing's concrete
thread control. No common API change is selected yet.

Parallel gzip remains product-private because only preprocessing currently
needs the thread-controlled contract. If `rsomics-seq` demonstrates the same
need with consumer tests and representative measurements, the backend can move
behind the existing `rsomics-seqio` writer contract without exposing
product-specific policy.

`rsomics-igzip` accepts no new consumers. Its native backend is integrated
privately or replaced after equivalent compatibility, throughput, and memory
evidence.

### Interval wave

`rsomics-intervals` exposes coordinate-safe geometry, overlap indexing, and
generic payloads. BED parsing, header behavior, sorting policy, and writing
remain in `rsomics-bed`.

`rsomics-bed` revision `97f5fe31662e` is the first concrete checked-index
consumer. Because the fallible API at `c13cb75c318` is not published, the
product pins 0.2.0 and validates the same backend range before every infallible
build or query. The representative million-record gate matches bedtools output
and passes throughput on all five operations without adding another shared
crate. `rsomics-annotation` must provide the second consumer-side contract
before the checked foundation API is released; until then the product guard is
deliberate temporary duplication rather than a frozen alternative abstraction.

### Alignment wave

`rsomics-bamio` exposes validated records and stable reader/writer contracts,
not every current batch or work-stealing implementation type.

`rsomics-pileup` adds input-order validation, low-allocation column views,
complex CIGAR and overlap tests, filter-combination tests, and real oracle
comparison before product use.

### Analysis wave

`rsomics-stats` absorbs a numerical primitive only when two products use the
same typed semantics. It does not become a container for all 91 historical
statistics binaries.

`rsomics-phylo-tree` closes node mutation and root invariants and supports the
declared Newick grammar before `phylo` and `ecology` depend on it.

## Completion gate per wave

A foundation wave is complete when:

- at least two named products call the public API;
- consumer-contract tests pass in those products;
- historical compatibility assets are preserved or superseded explicitly;
- hot-path performance and memory are measured where relevant;
- a fresh public-API review finds no product-specific policy exposed.
