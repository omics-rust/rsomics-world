# rsomics-vcf sort design

Status: product boundary, bcftools 1.24 behavior, historical assets, stable
ordering, bounded external merge, dependency alternatives, shared-foundation
requirement, compatibility differences, and release gate audited.
Implementation is not started while `rsomics-vcf` main remains the exact
unpublished 0.6.0 revision
`682942cfa69768dc3a127a8544f2f07213b704ea`. The target release is 0.10.0
after the complete `concat`, `merge`, and `isec` slices.

## Purpose and boundary

`sort` reads one typed VCF or BCF stream, validates every record against its
header, and emits the same records in deterministic coordinate and allele
order. It does not normalize variants, reconcile headers, remove duplicates,
build an index, or change biological fields. Those remain `norm`, `reheader`,
duplicate policy in the consuming operation, and `index`.

This is one `rsomics-vcf` subcommand, not a crate. It uses the product's
existing VCF/BCF readers, typed headers and records, four output encodings,
atomic named output, JSON summary, `rsomics-help` tree, and compression worker
contract. The complete stable command includes bounded in-memory runs,
external spill, hierarchical k-way merge, stable ties, standard input,
temporary-path ownership, cleanup, and performance evidence. A whole-file
`Vec` sorter is not a publishable intermediate slice.

## Authorities and evidence

The behavior oracle is bcftools 1.24 `sort`. Format authority remains the VCF
4.1 through 4.5 and BCF2 specifications. The tagged source and live binary
were inspected because temporary-run and tie behavior are not fully described
by the short manual section.

Pinned evidence:

- bcftools tag commit:
  `fb9f0f783e0f67d734f6fa7fe4df9d230522f196`;
- installed bcftools 1.24 executable SHA-256:
  `33100a6b961c529e915394d53b4737a0f8dd7a164eac352afe4e74e1ced51f60`;
- tagged `vcfsort.c` SHA-256:
  `4988146190624ccf831b141ad52cfbc8741f59f99b9009f6f31d5fdf11491b10`.

The upstream source is MIT licensed. Its option and ordering provenance are
attributed in user and compatibility documentation. The implementation may
reuse team-owned rsomics code, but does not copy bcftools's packed C structs or
internal temporary encoding.

## Historical asset disposition

Historical repository `rsomics-vcf-sort` is retained at revision
`2ba24aa3573557117fc47900892264f358bdf96d`, version 0.1.3. Relevant hashes
and decisions are:

| Asset | SHA-256 | Decision |
|---|---|---|
| `src/lib.rs` | `3cb21b7fc992280ed613a4fedfa280500e8a95339ac41242fe0bdc7c8f781dd1` | discard implementation |
| `src/cli.rs` | `933d07269afae4437331d91da42032db43fe34927cbcfcc9be6213c5c81e7c13` | discard CLI |
| `tests/compat.rs` | `844ae92a6f1b46dd1019f3ef0be67d4baa43199c6db57cbd82a2632563a80682` | retain order and malformed-input seeds |
| `benches/bench.rs` | `5275d0f9ce57576b1b519f74aa702a54da4d3584463d8168c41e3cf3077963d5` | discard launch benchmark |

The old implementation reads the complete input into one byte vector,
inflates generic gzip into another vector, copies every header and record
line, then stores every parsed record and its copied REF and ALT. Its declared
complexity is therefore not merely O(input size): several complete owned
copies coexist. It accepts only text VCF, does not validate typed INFO, FORMAT,
samples, BCF, BGZF EOF, or output encoding, and creates a named output by direct
truncation.

Its sort key approximates contig header order, numeric POS, REF bytes, and ALT
bytes. It misses complete allele-vector comparison, typed malformed-record
failures, external runs, a memory limit, a private temporary directory, stable
multi-pass ties, standard input, compression workers, atomic output, and
cleanup fault tests. JSON with standard output replaces the biological writer
with a sink. The compatibility test may skip and compares seven text records;
the benchmark repeatedly launches the binary on the same fixture without an
upstream measurement, peak RSS, or semantic hash.

Disposition:

1. direct merge: none;
2. refactor then merge: none;
3. test assets: contig-order, position, REF/ALT tie, undeclared-contig, and bad
   position fixtures;
4. discard: parser, whole-file storage, CLI, JSON behavior, writer, and
   benchmark.

## Existing rsomics implementation source

`rsomics-bam` already publishes a complete external sorter. Its team-owned
`src/sort.rs` supplies a proven structural source:

- a total record-memory budget with a one-record oversize path;
- fast compressed temporary runs;
- explicit input ordinals for stable ties;
- parallel in-memory sorting;
- a maximum 32-way heap merge;
- multi-pass consolidation above 32 runs;
- temporary files owned by `tempfile` and removed on drop;
- run-header validation, EOF checks, transactional named output, and summary
  counts;
- tests forcing more than 32 runs and multiple merge passes;
- native-platform CI and a live samtools 1.24 oracle.

The VCF implementation reuses this algorithmic and test shape rather than
recreating external-sort policy from the retired micro-crate. BAM raw records,
alignment keys, headers, run codec, and output writer remain alignment-specific
and are not moved into VCF.

## Command contract

The public spelling is:

```text
rsomics-vcf sort [OPTIONS] [INPUT]
```

INPUT accepts plain, ordinary gzip, or BGZF VCF, raw or BGZF BCF, or `-`.
Omission reads standard input when it is not an interactive terminal. Exactly
one input is accepted; additional positional paths fail instead of being
silently ignored as they are by bcftools 1.24.

`-o, --output FILE` defaults to standard output. `-O, --output-type TYPE`
accepts the product-wide `v`, `z`, `u`, and `b` spellings and defaults to plain
VCF. Suffixes do not override an explicit format. Numeric compression levels
are excluded. `--threads INT` controls the bounded in-memory sort pool and
BGZF input, run, and output workers; omission uses the product default of at
most four additional workers, while zero is explicitly single-threaded.

Global `--json` requires a named output and reports only after commit. It never
suppresses or replaces the sorted stream. The summary includes input and
output, encoding, records, configured record-memory budget, maximum observed
buffered record bytes, initial run count, merge passes, peak temporary bytes,
fan-in, worker count, and whether the in-memory fast path was used.

The command exposes no region, target, filtering, sample, normalization,
deduplication, or indexing option. Composing those operations before or after
sort keeps their contracts explicit.

## Resource options

`-m, --memory SIZE` is the canonical product spelling, with `--max-mem` as a
visible bcftools compatibility alias. The default is `768M`, where K, M, and G
are metric multipliers matching the current BAM product and bcftools. A bare
integer is bytes. Parsing consumes the complete value, rejects negative,
zero, NaN, infinity, fractional bytes, unknown suffixes, and overflow, and
requires at least 1 MiB.

The value is a total buffered-record budget, not a claim about complete process
RSS. Encoded record bodies, owned comparison keys, ordinals, and per-entry
container capacity count against it. Reader, writer, thread, compression, and
fixed heap overhead are reported separately by the performance gate. One
record larger than the budget is accepted alone, so peak record buffering is
bounded by `max(budget, largest_record)` rather than failing a valid VCF.

`-T, --temp-dir DIR` names an existing parent directory. The command always
creates a unique mode-0700 child named `rsomics-vcf-sort.*`; it never interprets
trailing `XXXXXX`, deletes DIR itself, or reuses a caller-owned directory.
This canonical directory spelling also becomes the preferred spelling for the
next `rsomics-bam` help revision, while its published `--temporary-prefix`
remains a compatibility alias.

If `--temp-dir` is omitted, the runtime temporary directory is used. In this
workspace every build, test, oracle, and benchmark explicitly supplies
`/Volumes/KIOXIA/Developments/tmp`; no sort artifact is allowed on the Mac mini
boot disk. Product code remains portable and does not embed that machine path.

The scratch directory must be on a writable filesystem. Run creation, write,
flush, checksum, close, reopen, read, merge, unlink, and directory cleanup
errors propagate. Ordinary success and every caught error remove only the
private child. An uncatchable process kill may leave that uniquely named child;
the error documentation explains identification without adding an automatic
destructive scavenger.

## Typed sort key

Records are ordered by:

1. contig rank in parsed header declaration order;
2. one-based POS ascending;
3. REF followed by every ALT in source order, compared allele by allele using
   ASCII case-insensitive byte order;
4. original zero-based input ordinal.

If one allele vector is a prefix of another, the shorter vector sorts first.
The ordinal makes the sort stable when contig, POS, REF, and every ALT compare
equal. ID, QUAL, FILTER, INFO, FORMAT, samples, symbolic END, and record length
do not break ties. Records are preserved byte-semantically through typed
decode and encode; the command does not merge or deduplicate equal keys.

Contig rank comes only from the header. An undeclared record contig fails.
Duplicate or conflicting contig definitions, invalid positions, malformed
REF/ALT, invalid breakends or symbolic alleles, undeclared fields, wrong
cardinality, malformed genotypes, truncated compression, and invalid BCF fail
through the existing typed format contract.

Bcftools 1.24 uses a case-sensitive packed-string comparator for an in-memory
run and a case-insensitive per-allele comparator while merging runs. A retained
probe changed the position of `A>c` relative to `A>C` and `A>G` when memory
fell from one large run to one run per record. Rsomics uses one comparator in
every path, with the source ordinal resolving case-insensitive equality, so
output does not depend on memory or temporary-run boundaries.

## Run generation

The reader parses one record at a time and assigns a checked u64 ordinal. A run
buffer holds typed encoded record data, the comparison key, and the ordinal.
Before adding a record that would exceed the budget, a nonempty buffer is
sorted and spilled. An oversize record forms a one-record run.

In-memory sorting may use the configured Rayon pool. The comparator includes
the ordinal, so an unstable parallel primitive still produces the stable total
order. If the full input fits, the sorted buffer writes directly to the final
writer and no run file is created.

Each external run is a product-private fast-compressed BCF stream containing a
canonical copy of the validated header and records in complete sort order. It
uses the existing writer and one standard BGZF EOF rather than inventing a
second record serialization. Run reopen validates the complete header against
the input header and requires the EOF marker. Run format is not a public API
and is deleted after use.

Temporary runs preserve every source record value. The final output writer
performs the only requested VCF/BCF conversion. This avoids repeated text
formatting and reparsing across merge passes.

## Hierarchical merge

At most 32 runs are opened in one merge. Each run contributes one lookahead
record to a binary heap ordered by the same key and ordinal. When more than 32
runs exist, consecutive groups of at most 32 merge into new fast-compressed
BCF runs; passes repeat until the final fan-in is bounded.

Every intermediate merge writes a complete temporary run, finishes and
validates it, then releases its source runs. A failed new run leaves all
currently owned files under the private directory and the transaction removes
the directory. Run order and stored ordinals make ties stable through any
number of passes and independent of heap iteration.

This is O(n log n) comparison work with bounded file descriptors and memory.
The release tests force at least 33 runs, at least 1,025 runs in a smaller
synthetic codec test, multiple passes, identical keys spanning every boundary,
and one record larger than the memory budget.

## Header and output preservation

The complete input header is parsed before records. Output retains its
file-format version, contig order, structured definitions, samples, and
descriptive metadata. The typed writer may add the standard PASS definition
when records use PASS and the source omitted its declaration, matching the
existing product encoding contract. No command-line, timestamp, or version
provenance line is added.

Named output rejects aliases with a named input and uses
`rsomics-common::AtomicFile`. Commit occurs only after input EOF, all run
merges, final record encoding, writer finalization, output sync, and product
quickcheck succeed. Standard output cannot roll back late failures, but every
argument and header error occurs before its header is written.

Automatic output indexing remains excluded until the variant stream and TBI
or CSI can commit as one multi-artifact transaction. Users run
`rsomics-vcf index` after a successful named sort. This is consistent with the
preceding product slices.

## Shared-foundation decision

VCF sort creates the second product consumer for a bounded external-merge
engine. The first is the published `rsomics-bam sort` and `collate` path.
Therefore a narrow `rsomics-common::external_sort` module is now justified,
provided migration passes both consumer gates.

The common module owns only:

- byte-budget run segmentation and oversize-item handling;
- input ordinals and stable tie plumbing;
- private temporary-file ownership;
- configurable bounded fan-in and multi-pass consolidation;
- heap orchestration, cancellation, cleanup, and run/pass/byte counters.

Consumers own item types, memory measurement, sort keys and comparator, run
codec, header or schema validation, compression, final writer, and biological
errors. The public API must not mention BAM, VCF, noodles records, contigs,
alleles, or output formats.

Migration uses two concrete consumer test suites:

1. `rsomics-bam` reruns its coordinate, natural-name, bytewise-name,
   template-coordinate, collate, >32-run, EOF, cleanup, and performance gates;
2. `rsomics-vcf` runs the complete matrix in this document, including memory-
   invariant ordering and malformed temporary runs.

API review compares the real existing BAM call site with the VCF call site
before publication. If the generic boundary would force either product to
allocate owned domain objects outside its budget or expose codec policy, the
engine remains duplicated privately until a smaller common contract is found.
No `rsomics-sort` crate is created.

## External dependency audit

Existing Rust external-sort crates are evaluated before extracting our own
engine. `extsort` 0.5.0 is Apache-2.0 and supports user codecs, custom
comparators, disk segments, and optional Rayon sorting, but its public segment
limit is an item count rather than a byte budget. That cannot directly enforce
the VCF/BAM memory contract when record sizes vary widely.

`spillover` 0.2.0 advertises pluggable codecs, keys, deduplication, k-way
merging, and a `GetSize` trait and is therefore a closer candidate, but its
recent interface has not yet passed our stable-tie, bounded-fan-in,
multi-pass, truncation, cleanup, cancellation, cross-platform, or performance
gates. `ext_sort`, `external_sort`, and `extsort_iter` likewise require a live
API and ownership audit before acceptance; documentation alone is not release
evidence.

The implementation spike runs the two strongest candidates against the same
BAM and VCF consumer tests and benchmarks. A dependency is adopted if it
satisfies the public contract without forks or hidden unbounded ownership and
does not regress either hot path. Otherwise the current proven BAM engine is
generalized into `rsomics-common`. This is a measured reuse decision, not an
assumption that any crate named external sort fits genomic records.

## Product structure

The intended VCF-local structure is:

```text
src/
├── sort.rs
└── sort/
    ├── key.rs
    ├── record.rs
    ├── run.rs
    └── output.rs
```

If the common extraction passes, `run.rs` becomes a thin consumer adapter over
`rsomics_common::external_sort`. The VCF module keeps header parsing, key
construction, BCF run codec, format validation, summary, and command policy.

Source comments remain rare. They document only the cross-pass stability
invariant, the oversize-record budget rule, and final transaction boundary
where names and types do not already make the reason clear. Migration history,
stages, and obvious control flow stay in this dossier rather than code.

## Deliberate compatibility differences

Rsomics differs from bcftools 1.24 where necessary for deterministic and
fail-loud behavior:

- extra positional inputs fail instead of being ignored;
- memory syntax is complete and positive instead of accepting junk, zero,
  negative, NaN, or infinity;
- one case-insensitive per-allele comparator is used in memory and on disk, so
  memory limits cannot change record order;
- `--temp-dir` is always a parent for a private child and never a caller-owned
  deletion target or magic `XXXXXX` template;
- the record-memory budget is measured explicitly and reported;
- named output is atomic and cannot alias input;
- JSON never discards sorted records;
- output suffix does not override an explicit type;
- command provenance, diagnostic verbosity, numeric compression levels, and
  automatic indexing are excluded.

All uppercase valid-record ordering, stable exact ties, input formats, stdin,
four output encodings, memory spill, hierarchical merging, and malformed-input
failures remain compatible with the 1.24 oracle.

## Failure contract

The command exits nonzero for:

- interactive omission, more than one input, unreadable input, input/output
  alias, or invalid JSON/output combination;
- invalid memory, thread, temporary directory, output type, or ignored option;
- malformed headers, duplicate or conflicting contigs, undeclared contigs or
  fields, invalid POS or alleles, wrong Number cardinality, malformed FORMAT or
  genotype data, corrupted BCF, truncated compression, or trailing garbage;
- record-count or ordinal overflow;
- temporary create, encode, checksum, finish, sync, reopen, header mismatch,
  EOF, decode, merge, unlink, or cleanup failure;
- output create, write, finish, sync, quickcheck, close, rename, or parent-sync
  failure.

No parser, record, run, or writer error is converted into a skipped record.
No named partial output becomes visible, and cleanup never targets a directory
that the command did not create.

## Test matrix

Unit tests cover:

- strict memory parsing, metric suffixes, overflow, minimum size, thread
  choices, temp-parent validation, and summaries;
- contig rank, POS, complete allele-vector ordering, case, prefix vectors,
  exact ties, ordinal overflow, symbolic and breakend values;
- memory accounting, exact-boundary flush, one oversize record, empty input,
  in-memory fast path, run counts, 32-way boundary, 33 and 1,025 runs, and
  multi-pass stable ties;
- run header identity, EOF, checksum or decode failure, heap order, cancellation,
  and cleanup.

Integration tests cover plain, ordinary gzip, and BGZF VCF, raw and BGZF BCF,
stdin, every output encoding, sites-only and sampled records, mixed fields,
contig order, same-position REF/ALT vectors, identical records, large records,
worker counts, named and standard output, JSON, and atomic replacement
behavior.

Failure tests cover extra inputs, invalid memory values, missing temporary
parents, unwritable scratch, input/output aliases, undeclared contigs and tags,
bad position and alleles, INFO/FORMAT cardinality, malformed BCF, truncated
gzip/BGZF, output failure, and injected run and commit failures. Tests verify
that only private temporary children are removed.

The live bcftools 1.24 oracle compares decoded headers and every record in
order for the retained surface. Separate deliberate-difference tests record
the upstream result for extra inputs, memory parser defects, and mixed-case
memory-dependent order. Oracle setup cannot silently skip in release CI.

## Performance gate

The representative matrix includes:

- 10 million unsorted biallelic records with small INFO;
- 2 million sampled records with large FORMAT vectors and variable record
  sizes;
- duplicate-heavy coordinates with multiallelic and symbolic records;
- in-memory, two-run, >32-run, and multi-pass budgets;
- plain VCF, BGZF VCF, and BCF input with VCF, BGZF VCF, and BCF output;
- one and default worker configurations on local NVMe scratch.

Each case alternates command order after warmups and records exact revisions,
machine, input hash, flags, memory budget, temporary filesystem, run count,
merge passes, peak scratch bytes, wall time, CPU, peak RSS, bytes read and
written, and complete order-sensitive semantic hashes. The dependency spike
and generalized BAM engine use the same fixtures and accounting.

Publication requires a strict throughput or resource-use advantage over
bcftools 1.24 on the relevant hot path, with no regression to the published
BAM sort and collate claims if common extraction occurs. Equal VCF performance
is insufficient without another measured material benefit. Safety and
temporary cleanup are required but do not substitute for performance.

## Release gate

Implementation begins only after 0.6.0 publication and the preceding stable
VCF slices establish their current format and transaction APIs. The command
remains absent from public help and README until the complete contract passes.

Release requires formatting, strict Clippy, debug and release tests, the full
bcftools 1.24 oracle, dependency or common-engine review, BAM and VCF consumer
gates, fault injection, formal performance evidence, package verification, and
a fresh public-API and hot-path review. Exact-head CI must pass native Linux
and macOS on x86_64 and aarch64.

The registry archive is then downloaded independently, matched to the release
head and package tree, installed with fresh external Cargo state, and smoke-
tested on VCF, BCF, stdin, forced spill, multi-pass, named output, and JSON.

## Audit evidence

The retained external audit directory is:

```text
/Volumes/KIOXIA/Developments/tmp/rsomics-vcf-sort-audit-20260819
```

It contains `ties.vcf`, `probe.sh`, VCF and BCF results, stderr and exit-status
captures, `results.sha256`, and `oracle-summary.md`. Representative observed
results are:

| Probe | bcftools 1.24 result | Contract use |
|---|---|---|
| contig order chr2 then chr1 | chr2 records precede chr1 | retained header rank |
| equal coordinate, uppercase alleles | REF/ALT vector order, exact ties stable | retained comparator |
| one large run versus one-record runs | same valid uppercase order | external oracle |
| mixed `A>c`, `A>C`, `A>G` | order changes with memory | rsomics deterministic correction |
| `-m 2x`, `-m 0`, `-m -1` | exit 0 with one-record runs | strict parser difference |
| two positional inputs | exit 0 after sorting only the first | rsomics rejects extra input |
| VCF stdin | exit 0, sorted VCF | retained streaming input |
| BCF input and raw BCF output | exit 0, decoded records identical | encoding oracle |
| BGZF VCF output | exit 0, one valid stream | encoding oracle |
| `-T prefix.XXXXXX` | private directory created and removed | rsomics uses parent semantics |
