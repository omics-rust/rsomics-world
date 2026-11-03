# Table crosstab implementation plan

**Goal:** turn long sample/feature measurements into a wide matrix inside
`rsomics-table`, without reviving `rsomics-tsv-crosstab`.

**Architecture:** private operation using the existing strict table reader,
transactional output, field selectors, and aggregate `Plan`. No new dependency,
foundation, or product boundary. This implements the crosstab and global-cell
contract in `docs/10-products/table.md`.

## Verified starting point

- Owner: `/Volumes/KIOXIA/Documents/omics-rust/rsomics-table`, clean
  `ac3788aafae0d5b9a1e0829e6c1b1c4743743bc8`; six operations, no crosstab.
- Historical asset: `rsomics-tsv-crosstab` at
  `da8e867f60fc6b4e8a66a4c72b8b152bb40629dd`. Retain fixtures and routing
  ideas, not its whole-input reader, unquoted line splitting, duplicate
  reducers, or silent first-run-only handling of repeated cells.
- `aggregate::Plan::{compile,new_group,new_scratch,push,finish}` already
  supplies the private reduction engine. Resolve row and column selectors
  independently through `fields::Spec`, each requiring one field.
- GNU datamash 1.9 manual, section 5.8:
  https://www.gnu.org/software/datamash/manual/datamash.html
  Count is the default; optional aggregate operates on a value field;
  absent cells use `N/A`; sorted crosstab provides the comparison oracle.
- Existing live groupby tests require an explicitly named, version-checked
  datamash 1.9. Neither `/opt/homebrew/bin/datamash` nor the previously
  suggested external `conda/envs/rs-up/bin/datamash` exists. A bounded filename
  search of external scratch returned no `datamash` or 1.9 source archive.
  Establish the oracle rather than silently skipping or substituting a version.

## Implementation and acceptance

- [x] Inspect the pinned upstream source and capture count/sum originals,
  including header behavior, empty labels, filler, and repeated cells.
- [x] Add `CrosstabArgs` in `src/cli.rs`: `--row`, `--column`, optional single
  `--aggregate FIELD:OPERATION`, `--filler` default `N/A`, existing input and
  output options. Default count uses the shared engine. Use an empty top-left
  matrix cell and byte-sorted axes; do not confuse axis labels with generated
  aggregate headers. Confirm these choices against the source witness first.
- [x] Add a complete-workflow regression in `tests/crosstab_cli.rs`: repeated
  nonadjacent cells, absent cells, named/index selectors, quoted CSV labels,
  gzip output, and malformed input preserving a named destination. Observe
  the missing-command failure before implementing.
- [x] Implement `src/operations/crosstab.rs`, wire `src/operations.rs` and
  `src/main.rs`, and reuse aggregate states rather than historical reducers.
  Read records incrementally; keep only axes and aggregate states. Preserve
  input order inside each cell for order-sensitive text reducers.
- [x] Add a pinned real datamash count/sum differential in the same test file;
  ordinary TSV records must match full output bytes under `LC_ALL=C`.
  CSV quoting is a native table contract, not a datamash capability claim.
- [x] Run guarded fmt, strict Clippy, ordinary tests and the pinned live oracle.
  Toolchains/caches/scratch must resolve to external mounted devices; invoke
  the guard's explicit fmt/clippy actions, not ambient Cargo subcommands.
- [x] Measure count and sum on at least 5M rows with full output comparison,
  warmup, alternating timed pairs and peak RSS. Bind source, binaries, input,
  flags, machine and all samples; retain negative results. Historical speedups
  do not qualify the new implementation.
- [x] One fresh whole-feature review, one necessary correction pass if needed.
  Inspect empty/header-only input, matrix shape/label ambiguity, text ordering,
  selector cardinality, and errors through the shared output transaction.
- [x] Only after the evidence gate, document stable behavior, commit the
  concern on owner main, push, and wait for exact-head four-native CI. Update
  the world dossier and delivery frontier. Publication remains subject to the
  complete release gate and the separately recorded registry-auth hold.

## Limits

No implicit dense numeric zero filling: absent measurements differ from zero.
The output is a datamash-compatible matrix, not a strict named-header table:
its top-left label is empty and category labels may also be empty. Feed it
to existing table operations with `--no-header` and index selectors. Do not
silently rename real labels or treat the matrix axis as a validated schema.
No XLSX, multi-value cells, consecutive compatibility switch, arbitrary
expression mutation, or cross-product dependency in this change. Do not publish
or advertise an unfinished command. The remaining 30-family goal is unchanged.
