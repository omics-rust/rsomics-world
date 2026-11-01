# Index BGZF/query hot-path implementation plan

> **For agentic workers:** Use superpowers:executing-plans for one coherent
> owning change, one integrated verification and one whole-feature review.

**Goal:** Remove demonstrated extra execution/memory work from the two failed
Index primary paths while preserving complete format/query contracts.

**Architecture:** Keep the existing product/public CLI and private frame,
writer, index and query modules. A real inline serial writer and a buffered,
bounded indexed reader share existing framing, not a new crate/framework.

**Tech stack:** Rust1.91, existing locked libdeflater/noodles/common/help.

**Spec:** AGENTS.md, docs/10-products/interval-annotation-index.md and adopted
index-bgzf-complete-failure-2026-11-01.json. Root fully read the complete
source review index-hot-path-review-20261101.1k9SRz/REPORT.md,
SHA3162f6476109dfa2380deee27d596f80365dc31e7d6a81903c930378bc84353a.

## Fixed scope

Owner /Volumes/KIOXIA/Documents/omics-rust/rsomics-index is clean
030be20895cb7806f78b2923e1c1ac3afa05cd96. Existing full13-lane correctness
passes, but serial binary compression and sparse query fail their fixed
performance gates. Do not change data, primary names, thresholds, workers,
timers, gates, historical originals or accepted FAI/reference receipts.
No new public foundation/API, dependency/version update, source/foreign
execution on Mac, installation, cleanup, registry/release or CI dispatch.

All Cargo/rustup/target/TMP/cache/scratch must physically resolve to approved
external disks through the existing owner guard. Do not stop for unrelated
boot occupancy. Source comments stay rare; encode ownership/control flow in
types/functions, not phase/audit narration. Preserve error propagation, CRC/
EOF/index validity and transactional output/directory sync.

## Review focus

- T1 and N4 encode the same block/input framing at the same actual codec effort.
- Ordered/duplicate/unique/targets queries preserve long interval overlap.
- Early stop uses the actual uncovered fragment, not its parent selection.
- N4 drains scheduled decode results and remains bounded after early stop.
- Corrupt frames/indexes, malformed records and I/O failures stay nonzero.

## One concentrated implementation

Files: src/bgzip/writer.rs, narrow reuse helpers in src/bgzip/frame.rs only as
needed; src/tabix/query.rs, query/blocks.rs, index.rs; existing corresponding
tests and README/merge record for the real effort correction. Do not rewrite
FAI/dictionary/CLI/common or their accepted fixtures.

- [ ] Extend existing connected tests first for the actual behavior change:
  public BGZF level6 must use libdeflate7 effort, not6. Root read original
  HTSlib1.24 bgzf.c:586 mapping {0,1,2,3,5,6,7,8,10,12}; align levels0..9,
  retaining level0 stored framing. Observe one actual regression, not source
  grep or self-derived expected constants. This corrects effort/quality;
  old timing/output claims do not automatically transfer.
- [ ] T1 executes inline without producer/worker/writer queues. Reuse initialized
  input/frame storage and existing compressor; deflate directly into the
  final frame payload, then fill header/CRC/ISIZE. Do not keep per-block
  input.to_vec/compressed-Vec/frame-copy behind a new serial wrapper.
  Retain text newline/carry/short-read/long-line behavior. Keep N4 bounded
  pipeline and serial/parallel stream equivalence. Use bounded output
  buffering where appropriate; do not remove transaction sync to win timing.
- [ ] Give query input a bounded BufReader and truthful physical/logical cursor;
  skip unnecessary seek for contiguous blocks and use in-buffer relative
  seeks when possible. Cache hits must not advance the real file cursor.
  Preserve variable BGZF extra headers, CRC and EOF validation.
- [ ] Do not disable user/default decoded cache merely because region coordinates
  advance: their physical chunks may overlap. Retain cache_bytes=0 as a real
  uncached mode. Keep cache metadata and decoded bytes bounded; an efficient
  FIFO policy is sufficient if it avoids current O(entries) hit refresh and
  has no public LRU contract. No generic cache framework or bigger hidden cache.
- [ ] Add narrow typed early-stop control through chunk/block/line visitation.
  Stop only on valid sorted-record bounds for the actual interval fragment;
  keep complete start/end overlap, reference identity, duplicate region,
  targets, header/separator and unique-visited semantics. In N4 stop scheduling
  new work and resolve/drain outstanding work without losing errors or growing
  the ready map. If lazy stopping conflicts with an existing malformed-index
  contract, report it rather than silently weakening validation.
- [ ] Remove the second query chunk sort/merge already guaranteed by pinned
  noodles-csi0.56.0 optimize_chunks. Reuse frame/decoded workspace only where
  ownership actually allows; do not add copies merely to claim reuse.

Do not add an unsafe uninitialized-memory path or a generic forward record
cursor that loses long BED/symbolic-VCF intervals. No purported allocation,
syscall or causal-performance percentages without measurements. The source
also fdatasyncs; query stdout is already1MiB buffered. These are not new fixes.

## One verification and handoff

Use real connected tests, not mocks or per-helper ladders. Extend the existing
stream/query tests for changed fragment/long-overlap/cache0/cache-enabled/N4
drain behavior and actual output; preserve all existing always-run regressions.
Run format, strict Clippy and full debug/release suites once after the coherent
change; record any optional oracle skips by name, never as executed coverage.
No upstream C/tool execution on Mac, real-data benchmark or unchanged campaign.

Freeze full before/after source, diff, protected paths, raw checks and REPORT
in a new private /Volumes/KIOXIA/Developments/tmp packet. Agent does not stage,
commit/push/dispatch/publish/delete. Root owns one whole independent feature
review/concentrated correction, commit and native-four gate. Only a substantively
changed, accepted binary may enter the original same-input13-lane performance
campaign after fresh custody/capacity admission. No speed or release is earned
by the source changes or local tests alone.
