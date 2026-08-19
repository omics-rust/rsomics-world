# `rsomics-index` product design

Status: approved for unattended implementation by the portfolio owner on
2026-08-19. This specification refines the accepted boundary in
`docs/10-products/interval-annotation-index.md`; it does not create another
public foundation.

## Product boundary

`rsomics-index` is one installable product for preparing and querying indexed
genomic resources. Its recognizable operations are:

```text
rsomics-index bgzip
rsomics-index tabix build
rsomics-index tabix query
rsomics-index tabix list
rsomics-index fasta-index
rsomics-index dict
```

BGZF and tabix form one compressed random-access workflow. FASTA indexing and
SAM sequence-dictionary generation form the corresponding reference-resource
workflow. They share input ownership, atomic sidecar creation, format
validation, index-path conventions, and installation identity.

Exact substring search is not an indexing sidecar workflow. The historical
`rsomics-fm-search` implementation remains retired and does not become a
subcommand or justify a public FM-index crate.

The release sequence is deliberately incremental:

1. version 0.1 implements complete stable `bgzip` and `tabix` operations;
2. a later release adds `fasta-index` and `dict` after the sequence reader and
   sidecar transaction contract are revalidated against `rsomics-seqio`;
3. no placeholder subcommand is exposed before its contract is complete.

## Compatibility sources

Version 0.1 is pinned to HTSlib 1.24:

- [`bgzip` 1.24 manual](https://www.htslib.org/doc/1.24/bgzip.html) and
  [`bgzip.c`](https://github.com/samtools/htslib/blob/1.24/bgzip.c);
- [`tabix` 1.24 manual](https://www.htslib.org/doc/1.24/tabix.html),
  [`tabix.c`](https://github.com/samtools/htslib/blob/1.24/tabix.c), and the
  TBI/CSI definitions in the SAM and CSI specifications;
- the installed 1.24 `bgzip` and `tabix` binaries are development oracles, and
  release CI builds the tagged 1.24 source rather than accepting an arbitrary
  executable from `PATH`.

The later reference-resource release is pinned to samtools 1.24
[`faidx`](https://www.htslib.org/doc/1.24/samtools-faidx.html) and
[`dict`](https://www.htslib.org/doc/1.24/samtools-dict.html), plus the
[`faidx` file contract](https://www.htslib.org/doc/faidx.html).

Compatibility means identical decoded content, region selection, metadata,
index semantics, and failures where those are observable. Raw compressed BGZF
bytes are not required to match because valid block boundaries and deflate
streams may differ. TBI/CSI bin serialization order is compared structurally
when the format does not define byte order.

## Version 0.1 command surface

### `bgzip`

The stable operation accepts one named input or standard input. It supports:

- compression and decompression;
- integrity testing;
- standard output or one explicit named output;
- compression levels 0 through 9 and an explicit worker count;
- text-aware newline block boundaries and `--binary` behavior;
- `.gzi` creation while compressing and rebuilding from an existing BGZF
  stream;
- decompression from an uncompressed offset with an optional byte limit;
- an explicit index path;
- overwrite only with `--force`.

Unlike upstream's implicit in-place lifecycle, 0.1 never deletes an input
file. Named output is staged and atomically committed only after BGZF
finalization, sink flush, and optional `.gzi` finalization all succeed. An
output and sidecar are one recoverable transaction. Input/output aliases,
truncated frames, corrupt CRCs, malformed `.gzi` files, impossible offsets,
worker failure, and finalization errors fail nonzero without exposing a
partial named output.

Recompression to reproduce an earlier block layout (`--rebgzip`), multiple
independent input files in one invocation, permission/time metadata copying,
and interactive overwrite prompts are explicit exclusions from 0.1. They do
not affect the compressed random-access workflow and would add destructive or
platform-specific policy before it is needed.

### `tabix build`

The build operation accepts one BGZF-compressed, coordinate-sorted
tab-delimited file. It supports:

- `gff`, `bed`, `sam`, and `vcf` presets;
- explicit sequence, start, end, comment, skipped-line, and coordinate-base
  configuration;
- TBI and CSI output, including configurable CSI minimum shift;
- type detection only when no preset or column option is supplied;
- default or explicit index paths and explicit overwrite;
- transactionally committed indexes.

The builder validates every record instead of treating compression as proof of
syntax. It rejects unsorted positions, a contig that reappears after a later
contig, invalid configured columns, zero or inverted coordinates, incompatible
preset/custom-column combinations, TBI coordinates beyond its representable
range, malformed headers, truncated BGZF, and input/index path aliases.

### `tabix query`

The query operation accepts one BGZF data file and a compatible TBI or CSI
index. It supports:

- inline one-based inclusive regions;
- an explicit index path;
- header inclusion or header-only output;
- BED or one-based tabular region files;
- sequential target filtering;
- duplicate suppression or region separators as mutually exclusive modes;
- standard output or one transactional named output;
- worker count and bounded BGZF cache configuration.

Inline and file regions preserve requested region order. Candidates selected
by index chunks are reparsed and overlap-filtered before emission. `--unique`
deduplicates by physical record offset rather than record text. Sequential
targets preserve file order. Malformed regions, missing contigs, incompatible
index headers, stale/truncated indexes, malformed records encountered in a
selected chunk, and write failures propagate to the command boundary.

Remote URI access, remote index downloading, header replacement, and
authentication are excluded from 0.1. They require a separate transport and
credential model and are not needed to prove local indexed access.

### `tabix list`

The list operation prints reference names from the selected TBI or CSI in
stored order. It accepts an explicit index path, validates the index header,
and supports standard output or one transactional named output.

## Architecture

The repository remains a flat independent crate and has no path dependencies.
Its intended structure is:

```text
src/
├── bgzip/
│   ├── index.rs
│   ├── reader.rs
│   └── writer.rs
├── commands/
│   ├── bgzip.rs
│   └── tabix.rs
├── tabix/
│   ├── build.rs
│   ├── config.rs
│   ├── index.rs
│   ├── query.rs
│   └── record.rs
├── cli.rs
├── lib.rs
└── main.rs
```

`commands` owns CLI validation, output selection, diagnostics, and JSON
separation. `bgzip` owns BGZF and GZI mechanics without command policy.
`tabix::record` owns one checked interval parser used by both build and query.
`tabix::build` owns the forward sorted scan and bin accumulation.
`tabix::query` owns chunk planning, deduplication, and final overlap tests.

No file should combine argument parsing, index encoding, and record scanning.
Narrow option and summary types are the only public library surface. Internal
bin maps and noodles representations are not public API.

## Shared layers

Version 0.1 consumes the released `rsomics-common` error, output, path-alias,
and atomic-file contracts and the released `rsomics-help` parser and command
presentation. It does not add an index-specific item to either crate.

BGZF remains private to the product for 0.1. After `rsomics-vcf` 0.6 is
published and its 0.7 concat work is committed, the VCF, BAM, and index paths
will be compared. A policy-free framing/reader/writer API may then move to
`rsomics-seqio` only with at least two consumer-side contracts and no
product-specific output or compatibility policy.

`rsomics-intervals` is not used for tabix parsing. Tabix coordinates and index
headers are format policy, not interval geometry, and using the public crate
would not create a second natural `IntervalIndex` consumer.

## Historical asset disposition

| Asset | Revision | Disposition |
|---|---|---|
| `rsomics-bgzip` | `1d5c8ee57e62a66d2982f15fc85dec53444d248e` | Refactor then merge framing, round-trip tests, EOF fixture, and performance seed. Replace its non-transactional output, input deletion, incomplete option model, and command shell. |
| `rsomics-tabix` | `5c3bddba67051eca06713197aadfacf62f178f9c` | Refactor then merge the optimized TBI/CSI accumulation, interval parser, structural goldens, query tests, and benchmark. Replace the combined CLI, direct output overwrite, missing sort validation, linear reference lookup, and incomplete query surface. |
| `rsomics-fasta-index` | `fb86ed124f069141b2934641f1291a8adc83a4be` | Later refactor merge of FAI/fetch/dict algorithms and goldens after re-auditing compressed FASTA, duplicate names, wrapping, and coordinated sidecars. |
| `rsomics-bam-dict` | `e3dd9a007965f9dd741882690d973b9e51044bf3` | Fixture and behavior cross-check only. The standalone shell and untracked `Cargo.lock` are not imported; the overlapping implementation is superseded by `rsomics-fasta-index`. |
| `rsomics-fm-search` | `6daef14ea73d9510f9d5060bd604ed0795686605` | Discard as a product asset for this boundary. Preserve the retired repository; do not expose FM search or retain `rsomics-fm-index` publicly without a real second consumer. |

All historical Rust code is team-owned. Selected algorithms retain Git source
revisions in this ledger. HTSlib and samtools behavior or code that materially
influences the implementation is attributed under their MIT/Expat notices in
`THIRD_PARTY_LICENSES.md`; the product remains MIT OR Apache-2.0.

## Correctness evidence

Ordinary tests cover:

- valid and malformed BGZF headers, extra fields, block sizes, CRCs, EOF
  markers, concatenation, truncation, GZI ordering, offsets, worker failures,
  and transactional finalization;
- BED, GFF, SAM, VCF, and custom tabular records, CRLF, long lines, empty
  files, header-only files, coordinate limits, sorting, contig order, TBI and
  multiple CSI depths;
- inline and file regions, open-ended regions, overlapping regions, repeated
  chunks, header modes, unique/separate modes, stale indexes, explicit index
  paths, output aliases, broken pipes, and write failures;
- command-tree validity, shared help layout, JSON separation, exit classes,
  and absence of undocumented subcommands.

Pinned 1.24 differential tests cover both directions of BGZF decompression,
text and binary block behavior, GZI partial reads, every tabix preset, custom
columns, TBI and CSI structural equality, list order, header modes, repeated
and overlapping queries, regions and targets files, and malformed-input exit
behavior. Oracle tests fail when the tagged tools cannot be built in release
CI; they do not silently skip.

## Performance gate

The retained evidence is a seed, not a release claim. The old BGZF benchmark
used HTSlib 1.23.1 on an Apple M2 and the old tabix benchmark used 1.21 on a
loaded Linux server. Version 0.1 is remeasured against 1.24 from a clean exact
head.

Representative inputs are:

- a multi-gigabyte textual VCF-like stream and incompressible binary stream
  for one- and four-worker compression, decompression, and indexed partial
  reads;
- at least six million sorted VCF records across multiple contigs for TBI and
  CSI build;
- sparse, dense, overlapping, and repeated region sets for query.

Every timing records machine, versions, binary and input hashes, flags, worker
budgets, warmups, alternating trials, wall/user/system time, peak RSS, output
sizes, and complete semantic equality. At least one representative hot path
must have a strict throughput or resource-use advantage. Regressions on other
paths are reported explicitly; a tiny process-launch fixture cannot pass the
gate.

## Release gates

Before 0.1 publication:

1. formatting, strict Clippy for all targets/features, tests, rustdoc, package
   verification, pinned compatibility, and the performance decision pass;
2. exact-head CI passes natively on Linux and macOS for `x86_64` and
   `aarch64`, with Linux `x86_64` building the full 1.24 oracle;
3. the public API and compression/index hot paths receive a fresh review;
4. README and help expose only the implemented stable surface;
5. the downloaded crates.io archive, VCS revision, checksum, fresh install,
   and representative command smokes are verified.

