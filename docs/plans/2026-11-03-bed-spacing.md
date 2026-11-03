# BED spacing implementation plan

> Execution: superpowers:executing-plans inline; one whole-feature review.

**Goal:** append source-compatible previous-record spacing inside the existing
BED product, preserving unified CLI/IO and strict sorted-input errors.

**Architecture:** a narrow streaming module retains one previous record and
closed chromosome names. Signed128 virtual point bounds and binary32 overlap
ratios reproduce the actual source. Product-local CLI wiring reuses common/help,
genome parsing, aliases, stdin and transactional output; no new public foundation.

**Tech:** Rust1.91, existing dependencies only.
**Spec:** `.autopilot/state/bed-spacing-contract-2026-11-03.md`.

## Constraints and decisions

Direct owning main under external KIOXIA; preserve inherited changes elsewhere.
Every execution uses the existing physical external storage guard. Keep
source comments rare; no placeholder commands, version/release claim or new
dependency. User-authorized autonomy supersedes repeated design approvals,
worktree creation and per-helper test/review ladders. Durable ledger is the
tracked contract; preserve all source originals and experiment output.

## One coherent task

Files: new `src/spacing.rs`, `src/cli/spacing.rs`, `tests/spacing.rs`, source
fixtures in `tests/golden/spacing/`; registration only in `src/{cli,lib}.rs`.
After implementation review, extend existing installed lifecycle to15 operations
and18 calls with one spacing full-golden call. No CI framework change.

API: `spacing(input: impl Read, genome: Option<&Genome>, output: impl Write,
options: SpacingOptions) -> Result<()>`; `SpacingOptions { header: bool }`.
CLI: `spacing --input/-i BED` (default stdin), optional `--genome/-g TSV`,
`--output/-o BED`, `--header`; existing global JSON requires named output.
Genome may use stdin only with a named BED. All selected paths participate in
alias rejection. Genome checks order/membership only; parser remains two-column.

- [x] Add one integrated consumer test and byte-exact source fixtures. It
  catches missing/wrong dispatch, raw rather than widened points, running-max
  rather than previous-only spacing, simplified overlap, lost headers/suffix,
  missing sorted/genome errors and broken named-output transaction. Include
  source's three successful outputs, three expected errors, one stdin/JSON
  success and prior destination preservation, not separate helper matrices.
- [x] Observe actual RED from missing spacing subcommand, then implement the
  narrow stream and CLI. Use write_prefix for canonical coordinates/raw suffix;
  emit header even with no records. Keep one previous record, raw start checks,
  closed chromosome set and optional genome rank; no full-input collection.
- [x] GREEN integrated test, formatting, strict all-target/all-feature Clippy,
  full debug/release suite. Review current public API/hot path once with fresh
  context; at most one necessary Important/Critical fix pass, no review ladder.
- [x] Wire one existing installed-lifecycle spacing call; scoped commits and
  push, observe exact-head native-four CI and preserve original artifacts.
- [x] Measure actual qualified native artifact versus pinned source on the
  unchanged584846-row full BED: one warmup/tool and five alternating pairs,
  full byte equality, wall/RSS/input/binary provenance and all losses. No
  release until actual correctness/performance/delivery gates are earned.

Review focus: source1e-9 ratios, raw sort versus widened points, chromosome
revisit/genome order, header-only streams, propagated parse/write errors. These
belong to the single integrated flow and ordinary existing boundary contracts.

## Consumer-driven internal reuse ruling

Actual code inspection finds identical signed point widening already duplicated
in map and coordinate. Move that unchanged geometry to private
`BedRecord::signed_bounds`, consumed by map/coordinate/spacing. Extract cluster's
existing raw-order validator to private `src/order.rs`, adding only optional
genome ranking for spacing; cluster uses the unchanged no-genome contract.
These are product-internal shared components, not speculative Layer A items.
Additional touched files are `src/bed.rs`, `src/map/sweep.rs`,
`src/coordinate.rs`, `src/cluster.rs`, `src/order.rs`. Existing map/coordinate/
cluster consumer tests cover the refactor; no helper-only matrix. Old measured
executables retain their own pins and are not relabeled as this changed source.
