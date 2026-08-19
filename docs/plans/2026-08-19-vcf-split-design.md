# rsomics-vcf split design

Status: product boundary, bcftools 1.24 `split` and `scatter` surfaces,
historical assets, multi-output transaction, bounded writer plan, compatibility
oracle, and release gates are defined. The target release is 0.11.0 after the
complete `concat`, `merge`, `isec`, and `sort` slices.

## Product boundary

`split` partitions one typed VCF or BCF stream into a directory of related
variant artifacts. It has two user-recognizable modes:

- `samples` projects one or more samples into each output;
- `records` partitions records by fixed record count or named genomic regions.

These modes belong together because they share the input model, output
encodings, region and target restriction, expression engine, safe naming,
bounded multi-output plumbing, manifest, indexing, and directory transaction.
They are modes inside `rsomics-vcf`, not standalone crates.

The operation does not split multiallelic records. Allele decomposition remains
`norm --multiallelics`. It does not split VEP or SnpEff consequence fields;
that behavior remains routed to `rsomics-annotation`. It does not schedule
distributed jobs or gather shards back together.

## Upstream and format authority

The compatibility oracles are bcftools 1.24 `+split` for sample projection and
`+scatter` for record partitioning. The VCF 4.5 and BCF2 specifications remain
the format authority, and HTSlib 1.24 supplies the reference behavior for
typed sample subsetting, region overlap, BGZF, BCF, CSI, and TBI.

The audited bcftools tag is revision
`fb9f0f783e0f67d734f6fa7fe4df9d230522f196`:

- `plugins/split.c` SHA-256:
  `86d822e0ab7fcb6b9f21d023c5d9565bd1a2181bc8413b98749eb217b48c3e97`;
- `plugins/scatter.c` SHA-256:
  `3b9c9e619ee5af7f229514dd789339fc21639b223692e7f9450244223915e223`.

Both plugin source files carry MIT licenses. The implementation may reproduce
documented behavior and black-box results but does not copy bcftools or HTSlib
code. Product attribution records both plugins and the format specifications.

## Historical asset

The retired `rsomics-vcf-split` repository is retained at revision
`4b84ce255e2ccd1292d4caa49d6011bf7e30f8bc`, version 0.1.0. Its tracked source
implements only chromosome partitioning of plain VCF text:

- it collects header strings and creates an output when a chromosome first
  appears;
- it keeps every chromosome writer open until EOF;
- it replaces most punctuation in chromosome names with `_`;
- it writes directly to final files and reports chromosome counts;
- its compatibility test compares data lines with `bcftools view -r` rather
  than the real `+split` or `+scatter` operations.

It does not support sample projection, groups, arbitrary record partitions,
BCF, gzip or BGZF output, typed validation, expressions, region restriction,
indexes, collision-free names, bounded file descriptors, a manifest, or an
atomic output set. Two distinct chromosome names can map to the same path and
silently share or overwrite an artifact. A parse or write failure leaves an
incomplete result directory.

The source classification is:

1. direct merge: none;
2. refactor then merge: none;
3. test, fixture, or benchmark asset only: the compact multi-contig fixture and
   per-contig membership expectations;
4. discard: the production line parser, writer map, lossy path conversion,
   standalone CLI and help, skip-capable compatibility harness, and tiny launch
   benchmark.

The worktree's only difference is an untracked `Cargo.lock`. It is preserved
in the historical clone and is not implementation input.

## Command surface

```text
rsomics-vcf split samples [OPTIONS] [INPUT]
rsomics-vcf split records [OPTIONS] [INPUT]
```

`INPUT` defaults to standard input. Exactly one input is accepted. Both modes
require `--output-dir DIR`; the final directory must not exist. The command
uses the product-wide `-O, --output-type v|z|b|u`, `--threads`, `--json`, quiet,
diagnostic, region, target, and expression conventions. It is rendered through
`rsomics-help` as part of the unified command tree.

Common split options include:

- `--include EXPR` or `--exclude EXPR`, never both;
- `--regions`, `--regions-file`, and the product overlap policy;
- `--targets`, `--targets-file`, and the product overlap policy;
- `--write-index csi|tbi` for compatible compressed outputs;
- `--max-open-files INT` for the private shard writer pool;
- `--output-dir DIR` and `--output-type TYPE`.

Input regions restrict records before partitioning. Partition regions in
`records` mode define destinations and are a separate typed concept. Numeric
compression levels, inferred output formats, version command-line stamps,
arbitrary HTSlib options, and permissive verbosity values remain excluded by
product-wide policy.

### Sample mode

With no plan file, `samples` creates one output per input sample in header
order. `--samples-file FILE` creates one output per non-comment row, and
`--groups-file FILE` assigns samples to one or more named outputs. The two
files are mutually exclusive.

The samples file has up to three tab-separated columns:

```text
input-sample-list    output-sample-list    label
```

The groups file has three tab-separated columns:

```text
input-sample    output-sample    label-list
```

Lists use comma separation with `\,` and `\\` escapes. `-` preserves sample
names. Blank lines and lines whose first non-space byte is `#` are ignored.
Each row is parsed completely. Input samples must exist, rename cardinality
must match, an output cannot contain a duplicate final sample name, and every
label must identify one unambiguous output. Missing or malformed rows fail
before any final artifact appears.

`--keep-tags LIST` retains the bcftools model for `INFO`, `INFO/TAG`, `FMT`,
and `FMT/TAG`, with `FORMAT` accepted as the canonical long spelling. Every
named tag must exist with the requested category. Unknown tags fail instead
of silently restoring all fields.

Record INFO values are preserved by default. In particular, cohort-level AC,
AN, and AF are not silently recalculated after sample projection. Users can
run the explicit fill-tags annotation mode when recalculation is intended.
Expressions are evaluated after each sample projection, matching the useful
`+split` contract where genotype predicates can select different records for
different outputs.

### Record mode

Exactly one partition definition is required:

- `--records-per-part INT` creates sequential parts with at most that many
  selected records;
- repeated `--part REGION[=LABEL]` assigns records by region;
- `--parts-file FILE` reads a region and optional label from each row.

The record count is a strictly positive integer. Fractional, zero, negative,
overflowing, and suffixed values fail. Sequential output labels are zero-based
part ordinals. `--prefix STRING` changes their display labels, not path
boundaries.

Part files accept blank lines and comments. Multiple regions with the same
label form one output. A record that matches different labels is copied to
each output once; overlapping regions with the same label never duplicate it.
`--unmatched LABEL` creates an output for records matching no part, including
an empty output when no record is unmatched. Every declared region label gets
an output even when its record count is zero.

Partition-region membership follows the selected product overlap mode and
uses the decoded variant span when record or variant overlap is requested.
The default is POS membership for compatibility with `+scatter`. The operation
preserves input order within every part and does not sort. Index requests add
the existing product sort-order and contig-block validation before commit.

Expressions are evaluated before record routing. This deliberately implements
the advertised filter rather than bcftools 1.24 `+scatter`, whose parsed
`--include` and `--exclude` expressions are not applied by its processing path.

## Output names and manifest

User-controlled sample, group, contig, region, and prefix strings never become
paths directly. Every output filename begins with a 16-digit hexadecimal
output ordinal and includes a bounded escaped display label. The ordinal makes
names unique on case-sensitive, case-insensitive, and Unicode-normalizing
filesystems. Bytes outside ASCII alphanumeric, `.`, `_`, and `-` are
percent-encoded. Labels longer than 96 encoded bytes are truncated and gain
the first 12 hexadecimal digits of their SHA-256. `.` and `..` can never form
a path component.

The output extension is determined only by `--output-type`. Explicit labels
that resemble `.vcf`, contain separators, or contain an extension do not alter
encoding or escape the destination. The manifest preserves the original label
and the exact sample or region plan, so filenames are not the data model.

`manifest.json` is written last and contains:

- schema version, operation mode, input identity, and output format;
- output ordinal, relative path, display label, and semantic role;
- ordered input and output sample names or normalized region definitions;
- record count, byte size, and SHA-256 for each variant artifact;
- index path, format, size, and SHA-256 when indexing is requested;
- total input, selected, routed, duplicated, unmatched, and output records;
- maximum simultaneously open files and private scratch bytes.

The manifest never contains absolute staging paths. JSON mode reports the
committed directory and the same summary; it cannot replace or suppress the
variant artifacts.

## Typed processing model

Both modes decode through the current product format layer. Headers and every
record are validated against the input schema before routing. VCF text, BGZF
VCF, raw BCF 2.2, and BGZF BCF 2.2 use one typed record contract. Truncated
compression, malformed records, undeclared fields, invalid cardinality,
unknown contigs, failed filters, and writer or index failures propagate to a
nonzero top-level exit.

Sample mode constructs an immutable projection plan after parsing the header.
Each output owns an ordered input-sample index vector, optional output names,
a projected header, tag projection, and optional bound expression. FORMAT
values are subset without changing allele indices, ploidy, phase, missing
values, or BCF vector-end semantics.

Record mode constructs either a sequential counter or a region index whose
payload is a compact output bitmap. Region definitions are normalized and
validated before record processing. Matching a record produces a deduplicated
destination list in output ordinal order.

## Bounded multi-output engine

The engine never keeps one final VCF or BCF writer per output. That design
fails on cohorts or region plans larger than the process file-descriptor
limit. Instead it uses two phases inside the private directory transaction:

1. decode and route each selected record into appendable, checksummed internal
   shards through an LRU writer pool bounded by `--max-open-files`;
2. finalize one output at a time by writing its projected header and decoding
   its shard into the selected VCF or BCF encoding.

Internal shards use a product-private length-delimited typed record format and
carry an ordinal and checksum. They are not VCF, BCF, or public API. A reopen
validates the shard header and append offset. Finalization validates complete
EOF, record count, and checksum before removing the shard. The staging
directory cannot commit while any shard remains or any manifest entry lacks a
validated file.

Sequential record-count splitting can write one final staged artifact at a
time without shards. The same manifest and validation path is retained, so
the optimization cannot change visible behavior.

`--max-open-files` reserves descriptors for input, output, indexes, manifest,
directory sync, and runtime libraries. The effective pool is capped below the
process soft limit and fails before processing when the requested budget
cannot support the minimum. It is never silently raised.

Compression workers are bounded independently of the file pool. `--threads`
is accepted only for compressed output and cannot multiply the number of open
part writers.

## Directory transaction and indexes

The operation uses `rsomics-common::AtomicDirectory`. It creates a private
sibling staging directory on the destination filesystem, writes and syncs
every variant and index, validates hashes and the manifest, removes private
shards, syncs the staging directory, renames it to the absent destination, and
syncs the parent. Failure removes only the private staging directory. An
existing destination is rejected and is never merged, truncated, or replaced.

An index is part of the same transaction as its variant file and manifest.
TBI is valid only for coordinate-sorted BGZF VCF within its coordinate limits;
CSI supports BGZF VCF and compressed BCF. Raw VCF and raw BCF reject index
requests. Each index is reopened and queried at boundary coordinates before
commit.

This is the planned second VCF call site for the directory transaction after
`isec`. The second product consumer remains the `rsomics-cnv call` report
bundle. The public foundation owns only same-filesystem staging, target-exists
policy, sync, commit, rollback, and the staging path. Split plans, shards,
manifests, naming, indexes, and record routing stay product-private.

No writer-pool, shard, or partition crate is created. There is no second
product consumer for that policy.

## Product structure

```text
src/
├── split.rs
├── split/
│   ├── manifest.rs
│   ├── names.rs
│   ├── records.rs
│   ├── samples.rs
│   ├── shards.rs
│   └── transaction.rs
└── commands/
    ├── split.rs
    └── split/
        ├── records.rs
        └── samples.rs
```

`split.rs` owns typed options, mode dispatch, counters, and the result summary.
`samples.rs` owns plan parsing and typed projection. `records.rs` owns
sequential and region routing. `names.rs` maps display labels to relative
artifact paths. `shards.rs` owns the private appendable format and LRU pool.
`manifest.rs` validates committed contents. The command modules convert Clap
values, bind expressions, use `rsomics-help`, and render the summary.

No public split library API is added. Shared VCF headers, records, expressions,
regions, encoders, indexes, and output selection are reused inside the product.
Any `rsomics-help` evolution must preserve the existing BED and VCF product
consumers rather than creating split-specific presentation policy.

## Compatibility contract

The stable sample-mode contract covers:

- one output per sample in header order;
- multi-sample outputs, renaming, explicit labels, and groups;
- one sample in multiple groups and multiple samples per group;
- strict plan escaping, missing samples, duplicates, and rename cardinality;
- INFO and FORMAT tag retention with header pruning;
- include and exclude expressions evaluated on projected outputs;
- VCF, BGZF VCF, raw BCF, compressed BCF, stdin, regions, and targets;
- empty selected outputs, indexes, manifests, and transactional failure.

The stable record-mode contract covers:

- exact positive records-per-part boundaries and a short final part;
- contig, interval, overlapping, repeated-label, and empty regions;
- one record routed to multiple labels without same-label duplication;
- unmatched output, prefix labels, input restrictions, and expressions;
- all four encodings, stdin, indexes, manifests, bounded descriptors, and
  transactional failure.

Output contents are compared semantically after removing allowed command-line
header stamps. File naming is deliberately safer and is tested against the
manifest rather than required to match bcftools bytes.

## Deliberate fail-loud differences

Live bcftools 1.24 probes recorded several behaviors that are not copied:

| Probe | bcftools 1.24 | rsomics contract |
|---|---|---|
| `+split` with one existing and one missing sample in a row | exits 0 and writes an invalid header with a duplicated existing sample | reject the plan before output |
| `+split -k` with only unknown tags | exits 0 and keeps all INFO and FORMAT fields | reject every unknown or category-mismatched tag |
| `+split` malformed second record | reports an HTSlib parse error, exits 0, leaves output files | nonzero exit and no visible destination |
| `+scatter -i 'QUAL>10'` | writes all three probe records | apply the bound expression before routing |
| `+scatter -n 1.5` | exits 0 and creates one-record parts | reject non-integers |
| `+scatter -S` label `../escape` | writes `escape.vcf` outside the output directory | label remains manifest data; generated relative path cannot traverse |
| `+scatter` malformed second record | reports a parse error, exits 0, leaves a header-only part | nonzero exit and rollback |
| pre-existing output file | silently truncated and replaced | reject an existing destination directory |
| thousands of sample or region outputs | one live final writer per output | bounded shard writer pool |

Bcftools's collision suffixes for sanitized sample names are useful evidence
for deterministic ordering, but rsomics uses ordinal-prefixed escaped names so
lossy collisions never occur. Warnings do not substitute for an invalid sample
plan.

## Tests

Unit tests cover:

- escaped sample and label lists, exact column counts, rename cardinality,
  missing and duplicate samples, and deterministic group order;
- cross-platform name generation, separators, `.` and `..`, extensions,
  Unicode normalization, case-only differences, long labels, and stable
  ordinals;
- INFO and FORMAT tag plans, unknown tags, projected typed values, ploidy,
  phase, missing values, and vector ends;
- positive integer parsing, part boundaries, region normalization, overlap
  modes, repeated labels, duplicate suppression, and unmatched records;
- shard checksums, truncated frames, reopen and append, LRU eviction, descriptor
  budgets, empty shards, and finalization order;
- manifest hashes, counters, relative paths, indexes, and schema validation.

Golden and differential tests cover plain VCF, BGZF VCF, raw and compressed
BCF, stdin, all sample and record plans, filters, region restrictions, target
streams, empty outputs, and every deliberate divergence. Output headers,
samples, records, tags, and indexes are compared with bcftools 1.24 where its
result is valid.

Fault injection covers input decode, expression evaluation, shard create,
append, flush, sync, reopen, checksum, final encode, index build, file sync,
manifest write, directory sync, rename, and parent sync. Before commit, no
final destination may exist. After commit, every manifest path must exist and
no private shard may remain.

Descriptor tests lower the process soft limit and split more outputs than that
limit while asserting the measured open-descriptor ceiling. Windows path tests
and native macOS case-insensitive filesystem tests supplement ordinary
temporary-directory tests.

## Performance gates

Formal comparison uses pinned bcftools and HTSlib 1.24 with version, revision,
machine, filesystem, input hash, flags, warmups, alternating runs, timing
distribution, peak RSS, peak open descriptors, bytes read and written, scratch
bytes, and semantic output hashes recorded.

Representative workloads include:

- 5 million biallelic records and 32 samples, one output per sample;
- 1 million records and at least 512 samples under a lowered descriptor limit;
- 10 million records split into fixed 100,000-record parts;
- 10 million records routed into at least 512 named genomic parts with overlap
  and unmatched records;
- BGZF VCF and compressed BCF output, with and without indexes.

The release requires a strict throughput or resource-use advantage on a
representative hot path. The primary expected advantage is bounded peak open
descriptors for large sample and region plans; this must be measured, not
inferred from source. Ordinary 32-sample and fixed-part workloads must remain
operationally competitive and cannot hide an unbounded scratch or memory
regression. Every measured output must reproduce the semantic oracle hash.

## Release gate

Release 0.11.0 is complete only when:

- both command modes and every declared stable option are implemented without
  placeholders or undocumented partial modes;
- `AtomicDirectory` has its VCF and CNV consumer-side fault-injection tests and
  passes public API review;
- formatting, strict Clippy, unit, integration, differential, malformed-input,
  descriptor, transaction, index, and benchmark smoke suites pass;
- the formal performance gate records a strict useful advantage;
- package contents, repository metadata, README, unified help, licenses, and
  attribution are reviewed from a clean exact head;
- native Linux and macOS CI pass on `x86_64` and `aarch64` at that exact head;
- the crate is published only after all earlier declared release slices and
  this complete split slice are present.

The audit fixtures and live outputs are retained outside the repository at:

```text
/Volumes/KIOXIA/Developments/tmp/rsomics-vcf-split-audit-20260819
```

Primary references are the
[bcftools plugin guide](https://samtools.github.io/bcftools/howtos/plugins.html),
the [bcftools 1.24 split source](https://github.com/samtools/bcftools/blob/1.24/plugins/split.c),
the [bcftools 1.24 scatter source](https://github.com/samtools/bcftools/blob/1.24/plugins/scatter.c),
and the [VCF and BCF specifications](https://github.com/samtools/hts-specs).
