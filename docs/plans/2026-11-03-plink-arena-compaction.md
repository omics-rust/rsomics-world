# PLINK completed arena compaction implementation plan

> **For agentic workers:** One bounded implementation task and one fresh whole-feature review. User unattended/direct-main/external-disk/no-review-ladder rules override generic approval, worktree and cleanup rituals.

**Goal:** Stop retaining unused metadata arena capacity after construction, without changing parsing, ownership or output behavior.

**Architecture:** Compact the completed String and field-range Vec once in RowsBuilder::into_rows before sharing the immutable Arc. Keep the owning iterator, all types and every caller unchanged. Do not stream or rewrite parsing in this wave.

**Tech Stack:** Existing Rust1.91 standard-library allocation methods; no dependencies or feature changes.

**Spec:** AGENTS.md and .autopilot/state/plink-metadata-construction-2026-11-03.md; actual green14 CI and full-I-w1 memory evidence.

## Global constraints

- Owner /Volumes/KIOXIA/Documents/omics-rust/rsomics-plink, clean7cee8e4366ef957981c1741055c3ff716dd5b53b.
- Modify only src/dataset/metadata/text.rs production code and the existing owned-memory consumer in src/dataset/metadata.rs.
- Preserve Text/Fields/Arena/Metadata layouts, Arc lifetime, iterator length, UTF-8/CRLF/header/width/error order, unknown fields, ALT/PR, BED/VCF callers and serialization.
- Preserve codec, SHA-256, file/directory durability, worker budgets, CLI, schema and foundation boundaries. No unsafe code, new comments, private-capacity test assertions, cleanup or new scientific campaign.
- All project writes/Cargo/Rustup/resolved target/cache/tmp must pass physical external-device/free-space guard. Keep logs/fixtures; no dependency download is needed.

## Review focus

Single compaction before Arc sharing; preserved field byte ranges and zero sentinel; iterator ownership after source drop; empty tables and existing Vec callers; allocator accounting measures owned capacity rather than RSS. Existing integrated consumers cover these contracts; do not add a scalar-helper matrix.

## Task 1: compact completed immutable arenas

**Interfaces:** Retain RowsBuilder::into_rows(self) -> impl ExactSizeIterator<Item=Fields> and finish(self) -> Vec<Fields>. The internal self binding may become mutable. No shared interface changes.

- [ ] Run the existing ten_thousand_variants_fit_metadata_budget consumer on unchanged7cee; expected2911523 owned bytes. Record current output before edits.
- [ ] Tighten that same consumer to an absolute2611523-byte ceiling, requiring at least300000 fewer owned bytes; observe the real resource RED before production edits. Do not modify the fixture or measurement.
- [ ] Compact self.text and self.fields at the end of construction, immediately before Arc allocation. Leave temporary row-range ownership and all parsing/callers unchanged. No hand-written allocator, capacity heuristic or additional data model.
- [ ] Run the same consumer GREEN, then guarded fmt, strict all-target/all-feature Clippy and full debug/release suites with zero failures/ignored. Preserve existing Unicode/unknown-column/PR, iterator lifetime, low-memory and transaction consumers; add no tests unless a concrete missing contract is found.
- [ ] Freeze the complete two-file delta, before/RED/GREEN logs, final suites and measured owned bytes in external scratch. Implementer does not stage/commit/push/publish. Root performs one fresh whole-feature review and integration.

## Rulings and later gates

Compacting once before immutable sharing is preferable to changing the parser or introducing an archived assembly dependency without demonstrated benefit. shrink_to_fit is an allocator request, not a universal RSS or speed guarantee; measure actual resource behavior before accepting those claims. Reallocation/copy time is a possible cost, retained in later full-workflow measurements.

The original green13 upstream-negative gate and green14 results remain immutable. A successful owned-memory consumer is not native-four, source, throughput/RSS or publication acceptance. Qualify any integrated source before a separately admitted changed-source matched workflow; no repeated unchanged campaign or temporary phase instrumentation.
