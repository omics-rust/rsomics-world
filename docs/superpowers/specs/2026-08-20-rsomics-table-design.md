# `rsomics-table` 0.1 product design

Status: approved for unattended implementation by the portfolio owner on
2026-08-20. This specification refines the audited boundary in
`docs/10-products/table.md`; it does not create another public foundation.

## Product boundary

`rsomics-table` is one installable CSV/TSV product. Version 0.1 exposes six
complete operations:

```text
rsomics-table select
rsomics-table filter
rsomics-table sort
rsomics-table join
rsomics-table groupby
rsomics-table validate
```

These operations share one strict record reader, writer, dialect model, field
grammar, expression model, output transaction layer, and CLI. The remaining
operations in the product dossier are later releases, not placeholder
subcommands in 0.1.

`rsomics-table` is not a dataframe library or a collection of operation-sized
crates. The package publishes one binary. Product-private modules are not made
public merely to reuse them within the same repository.

## Compatibility sources

Version 0.1 pins three real behavior sources:

- csvtk 0.37.0 for CSV/TSV framing, field expressions, filtering, ordering,
  and joins;
- GNU datamash 1.9 for aggregation, numerical formatting, and structural
  validation;
- bedtools 2.31.1 `groupby` for consecutive grouped aggregation and its
  operation spellings.

Release CI builds or installs these exact versions. Differential tests do not
silently skip when an oracle is absent from the oracle job. csvtk and bedtools
code may inform the implementation under their MIT terms. GNU datamash is a
black-box and documentation oracle only; its GPL source is not copied.

Compatibility is operation-specific. Exact output bytes are required where
the canonical contract deliberately matches an oracle. Where the canonical
product improves an unsafe or ambiguous upstream policy, tests compare the
record set, ordering, values, and failure class after explicitly documented
normalization.

## Shared record contract

All six operations consume the same `Dialect`:

- the input delimiter is one explicit ASCII byte and defaults to comma;
- `--tsv` selects tab input and output without filename inference;
- the output delimiter defaults to the resolved input delimiter and can be
  overridden by one explicit ASCII byte;
- input has a header unless `--no-header` is supplied;
- comments are disabled unless one explicit comment byte is supplied;
- duplicate header names, malformed quoting, ragged rows, missing fields,
  and invalid field references fail by default;
- skipping empty records or ragged records requires an explicit option;
- plain and gzip input are recognized from stream bytes, including standard
  input; named gzip output is selected by `.gz`, and gzip on standard output
  requires `--gzip`;
- output records use Go `encoding/csv` quoting and LF terminators;
- CRLF record endings become LF, including CRLF inside quoted fields, while a
  bare CR remains field content;
- byte-preserving operations do not require UTF-8. Header-name lookup,
  expressions, case folding, and diagnostics validate only the bytes they
  interpret as text.

The reader is a bounded streaming state machine over `BufRead`; it does not
preload and normalize the whole input. It yields owned byte records with a
logical record number, physical line, and byte offset. A quote error that
destroys record boundaries terminates the stream. Width errors can be reported
or explicitly skipped without guessing at quote recovery.

Named outputs use `rsomics-common` transactions and are committed only after
the complete parse, operation, gzip finalization, and flush succeed. Standard
output follows ordinary Unix streaming semantics and can contain an emitted
prefix before a later input error; the command still fails nonzero.

## Field grammar

One field grammar is used by selection, ordering, joining, grouping, and
expressions:

- one-based indices: `1`, `3`, `1,3,2,1`;
- inclusive ranges: `2-5` and open ranges such as `3-`;
- exclusions: `-2`, `-2--4`, and `-name`;
- exact header names, including `${name with spaces}`;
- optional csvtk-compatible fuzzy names using `*`;
- positive selections preserve request order and duplicates;
- exclusions preserve input order;
- positive and negative selectors cannot be mixed;
- name selectors require a header, and index selectors never silently become
  names.

The implementation stores parsed selectors separately from width/header
resolution. A resolved field is always a checked zero-based index. Missing
fields fail rather than producing an empty cell.

## Operation contracts

### `select`

`select` streams one input and emits the requested fields in request order.
Repeated fields are repeated. Index, range, exclusion, name, and fuzzy-name
selection all use the shared grammar. A header is projected by the same
resolved plan and can be omitted from output explicitly.

The operation does not rename or mutate fields in 0.1. Those are later
`mutate` responsibilities. It does not implement a second TSV-only line
splitter or accept a missing column as an empty value.

### `filter`

`filter` compiles one `--where` expression before consuming data rows and
streams records whose result is Boolean true. Version 0.1 has a deliberately
closed expression language:

```text
field       := $1 | $name | ${arbitrary header name}
literal     := finite-f64 | quoted-UTF-8-string | true | false | null
unary       := ! | -
arithmetic  := + | - | * | / | %
comparison  := == | != | < | <= | > | >= | =~ | !~ | in
logical     := && | ||
functions   := len(value) | ulen(value)
```

Parentheses control precedence and `in` takes a literal list. Regex patterns
must be string literals and are compiled once. Fields are numeric when they
parse as a finite `f64`, otherwise UTF-8 text; `--numeric-as-string` disables
automatic numeric typing. Equality of unlike types is false, relational
comparison of unlike types is false, and arithmetic on a non-number is an
input error. A final non-Boolean value is an input error. Division by zero,
non-finite results, invalid UTF-8 consumed as text, and invalid regexes fail
loud.

csvtk's bitwise operators, shifts, dates, exponentiation, ternary operator,
and null coalescence are excluded from 0.1. Unsupported syntax is rejected at
parse time; it is not accepted and evaluated approximately.

### `sort`

`sort` materializes complete records and orders them with repeatable keys.
Each key selects one or more fields and has one of these complete 0.1 modes:

- lexical byte order;
- case-insensitive Unicode lexical order;
- finite numeric order, with non-numeric cells ordered after numbers;
- csvtk-compatible natural order;
- reverse of any supported mode;
- multiple keys in declared order.

The retained csvtk comparator and deterministic unstable quicksort define tie
permutation. The parallel path must produce the same permutation as the serial
path. Numeric values are parsed once per row/key. `--threads` belongs to this
operation only and is not exposed on operations that cannot use it.

Date ordering and custom level files are excluded from 0.1. Their syntax
fails with an unsupported-mode error instead of falling back to lexical sort.

### `join`

`join` combines exactly two inputs in 0.1 and supports `inner`, `left`, and
`full` joins over one or more fields. It materializes the right input in a hash
index and probes it in left input order.

- every matching left/right duplicate pair is emitted as a Cartesian product;
- right matches retain right input order;
- left rows retain left input order;
- unmatched left rows are emitted in place for left and full joins;
- unmatched right rows are appended in right input order for a full join;
- the left key columns hold the right key for a right-only full-join row;
- right key columns are omitted from the appended right payload;
- missing payload cells use `--fill`, which defaults to an empty field;
- empty key fields match by default; `--null-never-matches` disables that;
- optional case-insensitive matching validates key fields as UTF-8;
- output schema collisions fail unless explicit left/right suffixes make
  every header unique.

Input aliases and output aliases are checked before writing. Headers are
resolved independently, so differently named key columns are supported. A
headerless join accepts index keys only and emits no synthetic header.

### `groupby`

`groupby` accepts repeatable `<field>:<operation>[=<alias>]` aggregate specs.
Global grouping is the default: equal composite keys are combined regardless
of input position and output keys are byte-sorted deterministically. With no
group selector the complete input is one group.

`--consecutive` provides bounded run aggregation for already grouped data. It
does not silently sort. A key that reappears after another key is an input
error, preventing an accidentally fragmented result.

Version 0.1 implements these operations completely:

- constant-state numeric reductions: `sum`, `min`, `max`, `absmin`, `absmax`,
  `range`, `mean`, `geomean`, `harmmean`, `pvar`, `svar`, `pstdev`, `sstdev`,
  `pskew`, `sskew`, `pkurt`, and `skurt`;
- order statistics: `median`, `q1`, `q3`, `iqr`, `perc:N`, `mad`, and
  `madraw`;
- textual or mixed reductions: `count`, `first`, `last`, `unique`,
  `collapse`, `countunique`, `mode`, and `antimode`.

Online moments use a mergeable stable state. Exact quantiles, median absolute
deviation, collapse, unique values, and modes keep only the values required by
their contract. The operation never stores a complete input merely because
one different aggregate needs it. Invalid numeric cells fail unless
`--ignore-non-numeric` is explicit. Numeric output uses a tested portable
14-significant-digit contract; oracle tests retain documented tolerances for
GNU long-double higher moments.

### `validate`

`validate` consumes the complete stream without emitting table data and
returns a structured report containing compression, delimiter, header state,
record count, width, physical lines, uncompressed bytes, and checked errors.
Plain success prints one concise summary; JSON uses the shared result envelope.

It checks gzip integrity, quoting, row width, duplicate headers, delimiter and
comment validity, and optionally full UTF-8. Safe width errors can be collected
up to `--max-errors`; an unframeable quote or compression error ends the scan.
Invalid input returns the shared invalid-input exit code and includes record,
line, and byte-offset context. Empty header-mode input is invalid; empty
headerless input is valid with zero fields and records.

## CLI and machine output

Every command tree is parsed through `rsomics-help`. `rsomics-common` owns
error classes, exit mapping, path aliases, atomic named output, and the JSON
envelope. Table-producing commands allow `--json` only with a named data
output; the table goes to the file and the operation summary goes to standard
output. Table bytes and JSON never share one stream.

The command tree contains no compatibility-only hidden micro-tools and no
generic global `--threads`. Help documents stable behavior, not implementation
history. Production errors reach the command boundary and exit nonzero.

## Repository architecture

```text
src/
├── cli.rs
├── dialect.rs
├── fields.rs
├── expression/
│   ├── lexer.rs
│   ├── parser.rs
│   └── value.rs
├── io/
│   ├── input.rs
│   ├── output.rs
│   ├── reader.rs
│   └── writer.rs
├── aggregate/
│   ├── numeric.rs
│   ├── order.rs
│   └── text.rs
├── operations/
│   ├── filter.rs
│   ├── groupby.rs
│   ├── join.rs
│   ├── select.rs
│   ├── sort.rs
│   └── validate.rs
├── operations.rs
└── main.rs
```

`cli` translates Clap types into narrow operation options. `io` owns record
framing and compression but no operation policy. `fields` resolves one checked
selection plan. `expression` compiles and evaluates the closed filter grammar.
`aggregate` owns reusable private accumulator states. Each operation owns only
its workflow and output schema.

The package is binary-only in 0.1. A public Rust table API requires a real
second product consumer and a separate review; internal module reuse is not
evidence for a public foundation.

## Shared foundations

Version 0.1 consumes released `rsomics-common` and `rsomics-help`. It does not
depend on `rsomics-stats`: formatting, text aggregation, grouping policy, and
exact quantiles are table-product contracts, and the existing public stats
crate has no demonstrated policy-free table accumulator API.

`rsomics-csvio` is internalized rather than retained. Its strict quote cases,
Go-compatible writer, field grammar, CRLF cases, and goldens are migration
assets. Its whole-input ownership model and public crate boundary are not.

No foundation change is planned before a product-side test demonstrates a
missing shared contract. If implementation reveals one, promotion still
requires a second named product consumer and consumer-side tests.

## Historical implementation disposition

| Asset | Revision | Version 0.1 use |
|---|---|---|
| `rsomics-csvio` | `0fccfb8cc2085a117ae88dc4b993c8b71b9c693b` | Refactor strict framing, writer, field rules, and goldens into private modules. |
| `rsomics-tsv-select` | `ba997aa55e050e4f40f25c84e657e5b0c2dd1dd0` | Retain fixtures and benchmark recipe; discard implementation. |
| `rsomics-tsv-filter` | `f694c99adab05a70800e93b3217e9a5507a68d63` | Retain numeric-condition cases and csvtk goldens; rebuild on the typed expression engine. |
| `rsomics-tsv-sort` | `1df47552324b55952ccd5e057f764833d24583e3` | Refactor comparator, natural ordering, quicksort, and strong differential fixtures. |
| `rsomics-tsv-join` | `635603c8e2ff683707ef77827bc5520e482ad778` | Retain small fixtures and benchmark recipe; rewrite all operation code. |
| `rsomics-bed-groupby` | `30cf021d1c59785076912c59bade457ea4a4bc7a` | Retain bedtools formatting and edge cases; replace value-vector-per-op design. |
| `rsomics-tsv-stats` | `108d43936350dafdbde2d1bd1cf6d4427941efd3` | Refactor operation list, numeric goldens, and tolerances into one accumulator engine. |

The merge record names these revisions. Narrating comments, audit history,
duplicated CLI shells, TSV line splitting, whole-output buffering, silently
missing fields, and optional-oracle tests are not migrated.

## Correctness evidence

Ordinary tests cover:

- quoted delimiters, doubled quotes, embedded LF and CRLF, bare CR, comments,
  empty records, gzip concatenation and truncation, invalid UTF-8, ragged rows,
  duplicate headers, and transactional failures;
- every selector form, repeated selection, missing and duplicate names,
  headerless mode, and fuzzy matching;
- expression precedence, type behavior, regexes, membership, Unicode width,
  invalid syntax, non-finite arithmetic, and row-context diagnostics;
- lexical, numeric, natural, reverse, multi-key, Unicode, tied-key, serial,
  and parallel ordering;
- all join types, independent key names, composite and empty keys, every
  duplicate-key product, right-only rows, order, fill, suffixes, and aliases;
- every aggregate, global and consecutive grouping, composite keys, key
  reappearance, invalid numerics, quantile edge cases, online-state merging,
  and deterministic output;
- validation reports, error limits, empty input, broken gzip, and JSON/data
  stream separation.

Pinned live tests cover representative exact and failure behavior for all six
operations. Frozen goldens make ordinary tests self-contained but do not
replace the live release oracle.

## Performance gate

The release fixture set includes:

- at least five million quoted CSV and TSV records for `select`, `filter`, and
  `validate` streaming throughput and peak RSS;
- a multi-gigabyte repeated-key table for lexical, numeric, natural, serial,
  and parallel `sort`;
- large many-to-one and repeated-key joins whose output is large enough to
  exercise the writer;
- high-cardinality global grouping and low-cardinality consecutive grouping,
  with streaming and order-statistic aggregates measured separately;
- plain and gzip streams.

Each result records exact revisions, machine, input generator and hashes,
flags, warmups, alternating trial order, timing distribution, CPU time, peak
RSS, output hashes, and semantic equality. At least one representative hot
path must have a strict throughput or resource-use advantage over its relevant
oracle. Every measured regression is reported; unused worker flags and tiny
process-launch wins cannot pass the gate.

## Release gates

Before 0.1 publication:

1. formatting, strict all-target/all-feature Clippy, debug and release tests,
   rustdoc where applicable, package verification, live compatibility, and the
   performance decision pass from a clean exact head;
2. exact-head CI passes natively on Linux and macOS for `x86_64` and
   `aarch64`; the full oracle job pins csvtk 0.37.0, datamash 1.9, and bedtools
   2.31.1;
3. the parser, expression evaluator, comparator, hash join, accumulators,
   public CLI, and production failure paths receive a fresh review;
4. README and help expose only the six complete operations;
5. the crates.io archive, VCS revision, checksum, clean install, and one smoke
   per operation are verified after publication.
