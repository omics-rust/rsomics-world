# Table crosstab: local implementation

## Scope and current state

Implements the existing table dossier's long-to-wide workflow inside the owner
repository, not a revived micro-crate or new public foundation. Base owner
`ac3788aafae0d5b9a1e0829e6c1b1c4743743bc8` was clean. The initial command was
hidden pending exact-head native CI and acceptance/documentation adoption.
This implementation step made no version bump, publication, registry retry or
released-asset mutation. Subsequent promotion and GitHub0.2.0 delivery are
accepted in `table-02-delivery-2026-11-03.md`.

Private `operations/crosstab.rs` uses strict incremental records, interned byte
axes, sparse aggregate states, deterministic byte ordering and existing encoded
transactional output. `aggregate::Plan::compile_cell` reuses existing reducers
without constructing unused output labels. No dependency or foundation change.

## Evidence

- Guard verified actual external mounts `/dev/disk12s1` and `/dev/disk10s1`,
  resolved Cargo target, Rustup and all cache/scratch paths. External free space
  was about 27.2 GB / 37.0 GB after the work. Boot occupancy did not gate work.
- Official GNU datamash 1.9 archive SHA-256
  `f382ebda03650dd679161f758f9c0a6cc9293213438d4a77a8eda325aacb87d2`,
  matching the owner's existing Linux oracle pin. Local binary SHA-256
  `200b708dc7e69b1d533a72d02e115d2f57f87c9c28e77bbdb7ad2c3e71e08cef`.
- Read upstream `src/crosstab.c`, relevant `src/datamash.c` group emission,
  `tests/datamash-crosstab.pl`, and GNU manual section 5.8 before implementation.
- Two integrated regressions first failed on the missing subcommand. One fresh
  whole-feature review identified unused-header UTF-8 conversion and matrix
  header interoperability. Extended the existing workflow test; observed the
  real UTF-8 rejection, then corrected label construction in one pass.
- Three crosstab test functions pass in debug/release with the real pinned
  oracle enabled. Count/sum compare complete output including empty labels;
  native regression covers nonadjacent repeats, sparse cells, quoted CSV,
  byte headers, gzip, crosstab-to-select and failed-output preservation.
- All ordinary local tests passed; existing unrelated live-oracle tests remain
  ignored locally rather than being represented as executed. Strict Clippy and
  formatting passed. Existing CI's pinned Linux oracle job automatically runs
  the new ignored differential via `--include-ignored`.
- Corrected 5M-row count/sum measurement: 12 full-output equivalent pairs
  including warmups. RSS medians 7,290,880/624,656,384 bytes and
  7,340,032/624,623,616 bytes native/source. Resource gate passes; timing is
  retained with an explicit overlapping-test-compilation caveat. See owner
  `benchmarks/2026-11-03/crosstab.md` and `crosstab-samples.json`.

## Matrix contract and next action

Output uses datamash-style empty top-left label, byte-sorted row/column axes,
and default `N/A` for absent cells. This is a matrix, not a strict named-header
table; downstream table commands consume it with `--no-header` and positional
selectors. Do not silently rename real category values. Text reducers preserve
original input order inside each global cell.

Packet: `/Volumes/KIOXIA/Developments/tmp/table-crosstab-oracle-20261103.2rpinuLR`.
Upstream build succeeded; the wrapper's final probe initially used nonexistent
`src/datamash` instead of root `datamash`. Corrected executable was then used
successfully by actual differential tests and both performance campaigns; no
source rebuild or hidden oracle replacement was performed.

Owner commit `463ad7cf18a971c54371fe0314ebe19472574a59` was pushed to main.
Exact-head CI run `41617642479` completed successfully: all four native
platform jobs, benchmark artifact and the Linux pinned live-oracle job passed.
Each native job exercised debug/release, packaged-source tests and the existing
six-operation installed-binary smoke. This does not claim that the install
smoke already exercises crosstab; adding it is the next delivery task.
https://github.com/omics-rust/rsomics-table/actions/runs/41617642479
Corrected performance manifest SHA-256:
`d4b8d6edd3c3052deceda30e9c51ac1c6d651c8ae510c6573513b6cefae302b2`.

Independent root verification rehashed all 37 manifest source/artifact pins and
all 24 complete matrix outputs: every matrix has 500 rows and 40 value columns,
count totals 5,000,000 and sum totals 252,500,000. No substitution of shape-only
checks for upstream full-byte equality was made.

World evidence commit `8168316093bb67c1adeff10ef2ab1d43229aeafb` also passed
exact-head control-plane CI `41617712498`.

The installed-binary matrix smoke, explicit matrix/header contract, command
promotion and GitHub release are now completed in the linked delivery record.
The remaining portfolio and registry-auth holds remain unchanged.
