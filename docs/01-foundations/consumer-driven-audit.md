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
| `rsomics-common` | keep; refactor command/error/output contract | `seq`, `bed` |
| `rsomics-help` | keep; derive nested help from the command tree | `seq`, `bed` |
| `rsomics-seqio` | keep; redesign around FASTA/FASTQ stream contracts | `seq`, `fastq-preprocess`, `fastq-qc` |
| `rsomics-kmer` | keep; repair boundaries and expose only general primitives | `seq`, `fastq-preprocess` |
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
| `rsomics-intervals` | `c13cb75c318` | checked the coordinate range accepted by the COITrees backend and added fallible index/query entry points | exact-head CI green; consumer contracts, four-native-target CI, and performance evidence remain |
| `rsomics-kmer` | `e937817e629` | made `k = 32` well-defined, added checked encode/decode/canonical operations, and preserved the published constructor shape | exact-head CI green; two product contracts, four-native-target CI, and comparative performance remain |
| `rsomics-seqio` | `0c20b6af566` | replaced the ambiguous record model with strict allocation-reusing FASTA/FASTQ streams, removed direct `rsomics-igzip` use, and made gzip/BGZF failures loud | four-native-target exact-head CI and compressed-stream adversarial regressions green; comparative throughput/RSS remains |

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
   assertion; `e937817e629` closes both blockers.
3. `rsomics-pileup` documents coordinate-sorted input but does not validate it.
4. `rsomics-common` ignores JSON serialization and output write errors.
5. `rsomics-bamio::RawRecord::from(Vec<u8>)` permits unchecked bytes while
   accessors assume a valid structure.
6. `rsomics-phylo-tree::Tree::default()` does not establish a valid root and
   public node fields permit topology invariant violations.

These are fixed in their consumer wave before the dependent product migration
uses the API. They are not reasons to speculatively rewrite every foundation
first.

## API corrections

### CLI wave

`rsomics-common` and `rsomics-help` are driven by the `seq` and `bed` command
trees.

- Represent product and subcommand identity in error and JSON output.
- Derive help from one Clap command tree instead of duplicating `HelpSpec`.
- Preserve rich, plain, and JSON representations.
- Remove hard-coded old-workspace fixture paths and boot-disk cache fallbacks.
- Propagate serialization and output failures.

The API freezes only after both products exercise it.

### Sequence wave

`rsomics-seqio` becomes a strict FASTA/FASTQ reader/writer over paths and
standard streams, with borrowed streaming and owned batch forms. Concrete
threading, slab, and compression backends remain private.

`rsomics-kmer` retains checked 2-bit encode/decode, canonicalization, rolling
iteration, and general hashes. Product-specific correction tables and QC bins
remain internal.

`rsomics-igzip` accepts no new consumers. Its native backend is integrated
privately or replaced after equivalent compatibility, throughput, and memory
evidence.

### Interval wave

`rsomics-intervals` exposes coordinate-safe geometry, overlap indexing, and
generic payloads. BED parsing, header behavior, sorting policy, and writing
remain in `rsomics-bed`.

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
