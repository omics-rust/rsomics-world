# `rsomics-table` 0.1 implementation plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use
> `superpowers:executing-plans`. Implement each task with tests first and do
> not expose a subcommand until its complete task passes.

**Goal:** Build and release one coherent `rsomics-table` 0.1 binary with
production-complete `select`, `filter`, `sort`, `join`, `groupby`, and
`validate` operations.

**Architecture:** One product-private byte-record stream and field grammar feed
six narrow workflow modules. Streaming operations retain bounded memory;
sorting, global grouping, and the right side of a join materialize only the
state their contracts require. `rsomics-common` owns transactions and result
contracts, and `rsomics-help` owns the complete command tree.

**Tech stack:** Rust 2024, Rust 1.91 minimum, clap 4.5, rsomics-common 0.12,
rsomics-help 0.4, serde, regex, unicode-width, flate2, tempfile, csvtk 0.37.0,
GNU datamash 1.9, and bedtools 2.31.1.

**Spec:** `docs/superpowers/specs/2026-08-20-rsomics-table-design.md`

## Global constraints

- Work only in `/Volumes/KIOXIA/Documents/omics-rust/rsomics-table`; control
  documents remain in `/Volumes/Zane's HDD/Documents/rsomics-world`.
- Before every Cargo command, verify `/` is below 80% and export
  `CARGO_HOME=/Volumes/KIOXIA/Developments/cargo-home`,
  `CARGO_TARGET_DIR=/Volumes/KIOXIA/Developments/cargo-target/rsomics-table`,
  and `TMPDIR=/Volumes/KIOXIA/Developments/tmp`.
- Keep table I/O, fields, expressions, and accumulators product-private. Do not
  recreate `rsomics-csvio` or add a new public foundation.
- Use `rsomics-help` for parsing/presentation and released `rsomics-common` for
  errors, exits, JSON, aliases, and named-output transactions.
- Standard output never mixes table bytes with JSON. Named output commits only
  after parser, operation, compression finalization, and flush succeed.
- Use byte records. Validate UTF-8 only at text-consuming boundaries.
- Comments are limited to public contracts and stable non-obvious invariants.
- Pinned oracle jobs fail if an oracle is unavailable; they never skip.
- Direct commits to `main`, one concern per commit, no coauthor, and exact-head
  CI after each pushed concern.

## Target file map

```text
rsomics-table/
├── .github/workflows/ci.yml
├── benchmarks/table-vs-upstreams.sh
├── src/
│   ├── aggregate/
│   │   ├── numeric.rs
│   │   ├── order.rs
│   │   └── text.rs
│   ├── expression/
│   │   ├── lexer.rs
│   │   ├── parser.rs
│   │   └── value.rs
│   ├── io/
│   │   ├── input.rs
│   │   ├── output.rs
│   │   ├── reader.rs
│   │   └── writer.rs
│   ├── operations/
│   │   ├── filter.rs
│   │   ├── groupby.rs
│   │   ├── join.rs
│   │   ├── select.rs
│   │   ├── sort.rs
│   │   └── validate.rs
│   ├── cli.rs
│   ├── dialect.rs
│   ├── expression.rs
│   ├── fields.rs
│   ├── io.rs
│   ├── main.rs
│   └── operations.rs
├── tests/
│   ├── cli.rs
│   ├── filter.rs
│   ├── groupby.rs
│   ├── io.rs
│   ├── join.rs
│   ├── select.rs
│   ├── sort.rs
│   ├── validate.rs
│   └── golden/
├── Cargo.toml
├── PERFORMANCE.md
├── README.md
└── THIRD_PARTY_LICENSES.md
```

---

### Task 1: Repository shell, strict record stream, and `validate`

**Files:** Create the manifest, licenses, `.gitignore`, `src/main.rs`,
`src/cli.rs`, `src/dialect.rs`, all four `src/io` modules,
`src/operations/validate.rs`, and the I/O/validation tests and fixtures.

**Produces:**

- `Dialect` with checked delimiter, output delimiter, header, and comment
  policy;
- `RecordReader<R: BufRead>::next_record() -> Result<Option<Record>>`;
- `RecordWriter<W: Write>::write(&[Field]) -> Result<()>`;
- plain/gzip input detection and finalized plain/gzip output;
- `validate(options) -> Result<Validation<ValidationReport>>`;
- a command tree containing only complete `validate`.

- [ ] Write failing parser tests for quoting, doubled quotes, embedded LF,
  CRLF, bare CR, comments, empty records, ragged rows, byte offsets, malformed
  quotes, and gzip truncation.
- [ ] Write failing validation tests for valid, invalid, duplicate-header,
  UTF-8, empty-header, empty-headerless, error-limit, JSON, and transactional
  behavior.
- [ ] Initialize the repository and verify the focused tests fail because the
  reader and report do not exist.
- [ ] Implement the streaming state machine and Go-compatible writer without
  importing the old whole-input normalization model.
- [ ] Wrap gzip by stream bytes on input and explicit/suffix policy on output;
  surface CRC, truncation, finalization, and broken-write errors.
- [ ] Implement `validate`, shared error context, `rsomics-help` parsing, and
  `rsomics-common::run_validation` integration.
- [ ] Run focused tests, all library/unit tests, strict Clippy, and help-tree
  assertions.
- [ ] Commit `feat(table): add strict table validation`.

### Task 2: Shared field plans and streaming `select`

**Files:** Create `src/fields.rs`, `src/operations/select.rs`,
`tests/select.rs`, and field fixtures; extend CLI and README only with the now
complete operation.

**Produces:** parsed unresolved selectors, checked header/width resolution,
and a streaming projection workflow.

- [ ] Port the useful csvio field tests first, adding `${...}`, repeated
  fields, exclusions, duplicate headers, invalid mixes, and byte-only index
  mode.
- [ ] Add failing CLI goldens for CSV/TSV, quoted cells, embedded newline,
  headerless input, gzip, malformed late input, named-output preservation, and
  JSON separation.
- [ ] Implement parsing and resolution without regex work on index-only paths.
- [ ] Implement one-record-at-a-time projection through the shared writer.
- [ ] Run the select suite and a live csvtk 0.37.0 differential locally.
- [ ] Commit `feat(table): select delimited fields`.

### Task 3: Closed expression engine and streaming `filter`

**Files:** Create all expression modules, `src/operations/filter.rs`,
`tests/filter.rs`, and expression/compatibility fixtures.

**Produces:** a tokenized Pratt parser, typed AST, precompiled literal regexes,
resolved field references, and a streaming Boolean filter.

- [ ] Write lexer/parser tests for every supported token, precedence,
  parentheses, field spelling, literals, lists, and rejected csvtk syntax.
- [ ] Write evaluator tests for numeric/text/Boolean/null values, mixed types,
  regex, membership, `len`, `ulen`, non-finite results, invalid UTF-8, and row
  context.
- [ ] Add failing end-to-end goldens for simple csvtk `filter`, representative
  `filter2`, comments, headerless fields, gzip, and a late evaluation error.
- [ ] Implement lexing and parsing with no runtime script engine.
- [ ] Resolve field references once, compile regex literals once, and evaluate
  each row without allocating unchanged text values.
- [ ] Implement output and summary contracts, then run live csvtk 0.37.0 exact
  and normalized differentials.
- [ ] Commit `feat(table): filter with typed expressions`.

### Task 4: Deterministic multi-key `sort`

**Files:** Create `src/operations/sort.rs` and private comparator/natural-sort
modules as needed; migrate selected sort tests and fixtures.

**Produces:** lexical, case-folded, numeric, natural, reverse, range, and
multi-key ordering with identical serial/parallel tie permutation.

- [ ] Migrate the strong csvtk goldens and add tests for cached numeric keys,
  non-numeric ordering, Unicode, duplicate keys, thread counts, malformed
  input, date/custom-level rejection, and output transaction failure.
- [ ] Port and reduce the historical natural comparator and deterministic
  quicksort, retaining only invariant comments.
- [ ] Parse and resolve all sort keys before allocating prepared key columns.
- [ ] Parallelize only disjoint partitions above a measured threshold and use
  the requested worker pool rather than the global Rayon pool.
- [ ] Run serial/parallel identity tests and the full live csvtk 0.37.0 sort
  differential.
- [ ] Commit `feat(table): sort by checked table keys`.

### Task 5: Duplicate-correct `join`

**Files:** Create `src/operations/join.rs`, `tests/join.rs`, and join fixtures.

**Produces:** two-input inner, left, and full joins with composite keys,
Cartesian duplicate products, deterministic row order, null policy, fill, and
checked schema suffixes.

- [ ] Write failing tests for every join type, left/right duplicate counts,
  composite/differently named keys, empty keys, case folding, right-only rows,
  headerless mode, suffix requirements, ragged inputs, and all aliases.
- [ ] Add exact csvtk fixtures for the overlapping contract, including the
  four-row Cartesian product from two duplicate keys.
- [ ] Implement a compact right-row store and key-to-row-index vectors; never
  overwrite an existing key.
- [ ] Track matched right row identities for full joins and append unmatched
  rows in original order.
- [ ] Build and validate the output schema before opening the output
  transaction.
- [ ] Run live csvtk 0.37.0 inner/left/full differentials and the complete join
  suite.
- [ ] Commit `feat(table): join duplicate table keys`.

### Task 6: Unified accumulator engine and `groupby`

**Files:** Create the aggregate modules, `src/operations/groupby.rs`,
`tests/groupby.rs`, and migrated datamash/bedtools fixtures.

**Produces:** typed aggregate specs, global and checked-consecutive grouping,
mergeable numeric states, bounded textual/order states, deterministic keys,
and portable numerical output.

- [ ] Write per-accumulator tests for empty/singleton/large-magnitude inputs,
  online merges, quantile definitions, ties, invalid numbers, aliases, and
  formatting.
- [ ] Migrate all useful datamash 1.9 and bedtools 2.31.1 goldens before
  implementation, separating exact output from tolerance-based moments.
- [ ] Implement constant-state counts, sums, extrema, means, moments, and
  variance; implement explicit value stores only for operations that require
  order, distinct values, modes, or collapse.
- [ ] Implement collision-free composite byte keys. Global mode stores one
  accumulator set per key and sorts output keys; consecutive mode keeps one
  active set and rejects key reappearance.
- [ ] Resolve aggregate fields once and make every numeric parse error include
  record, line, field, and operation context.
- [ ] Run live datamash and bedtools differentials plus the complete groupby
  suite.
- [ ] Commit `feat(table): aggregate grouped records`.

### Task 7: Unified CLI, attribution, and four-native CI

**Files:** Complete `src/cli.rs`, `.github/workflows/ci.yml`, README,
`THIRD_PARTY_LICENSES.md`, and CLI tests.

- [ ] Assert the exact six-operation tree, nested help, suggestions, stable
  option headings, `NO_COLOR`, JSON/data separation, and absence of unfinished
  operations.
- [ ] Remove duplicated operation flags by composing checked Clap `Args`
  types, while keeping `--threads` sort-local.
- [ ] Record historical source revisions and csvtk/bedtools attribution;
  identify datamash only as a behavior oracle.
- [ ] Build pinned csvtk 0.37.0 in native CI. Build pinned datamash 1.9 and
  bedtools 2.31.1 in the full Linux `x86_64` oracle job.
- [ ] Run formatting, strict Clippy, debug/release tests, help, and package
  checks on Linux and macOS `x86_64` and `aarch64`; run live differentials in
  the oracle job with no skip path.
- [ ] Commit `ci: validate table workflows on four native targets` and wait
  for the exact-head run.

### Task 8: Representative compatibility and performance gate

**Files:** Create `benchmarks/table-vs-upstreams.sh`, fixture generators, and
`PERFORMANCE.md`; add only small deterministic compatibility fixtures to Git.

- [ ] Make the harness refuse wrong upstream versions, dirty target heads,
  missing `/usr/bin/time`-equivalent provenance, output inequality, or boot-disk
  fixture/result paths.
- [ ] Generate and hash at least five million quoted records plus large
  repeated-key sort/join and high-cardinality groupby inputs on external disk.
- [ ] Run warmups and at least ten alternating measured pairs for streaming,
  sort, join, global/consecutive grouping, and gzip workloads.
- [ ] Verify exact output hashes or operation-appropriate semantic equality
  before accepting any timing row.
- [ ] Record medians, distributions, CPU, peak RSS, versions, machine, flags,
  generator, inputs, outputs, and explicit losses.
- [ ] Require one strict representative throughput or resource win. If none
  exists, optimize a real hot path and repeat rather than weakening the gate.
- [ ] Commit `test(table): add upstream performance gate` followed by
  `docs(table): record release performance` when the measured decision is
  stable.

### Task 9: Final review, package, publish, and verification

**Files:** Modify only issues found by the final review; update release
metadata and control-plane state after live publication.

- [ ] Review the parser and gzip finalizers for partial-success paths; the
  expression engine for type confusion and repeated compilation; sort for
  comparator consistency; join for duplicate loss; groupby for hidden whole
  input/value retention; and every production `unwrap`.
- [ ] Remove narrating comments, stale historical language, and unnecessary
  public items. Confirm the package is binary-only and has no accidental table
  foundation API.
- [ ] From a clean candidate head run formatting, strict all-target/all-feature
  Clippy, debug and release tests, live oracles, documentation checks, package
  verification, and the performance harness integrity check.
- [ ] Push the exact candidate and wait for its four-native exact-head CI.
- [ ] Inspect package contents and checksum, publish 0.1 only when registry
  credentials are available, verify the downloaded archive and VCS revision,
  clean-install to external storage, and smoke all six operations.
- [ ] Record the release, source-asset dispositions, performance decision, and
  next table slice in `rsomics-world`.

## Completion evidence

The release is complete only when the repository, CI run, package checksum,
crates.io archive, installed binary, six command smokes, pinned oracle results,
and performance result all identify the same Git head. A locally green build,
an unpublished package, or an available registry name is not completion.
