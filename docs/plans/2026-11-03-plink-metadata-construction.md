# PLINK metadata construction implementation plan

> **For agentic workers:** Use one bounded task and one whole-feature review. User direct-main/external-disk/no-review-ladder rules override generic worktree, approval-loop and cleanup rituals.

**Goal:** Reduce PGEN metadata preparation overhead without changing scientific outputs, metadata types or validation policy.

**Architecture:** Bind PSAM/PVAR columns once; consume compact arena rows through an owning exact-size iterator; allocate REF+ALT storage together.

**Tech Stack:** Existing Rust1.91 and product-private Arc arena; no dependency changes.

**Spec:** `AGENTS.md`, `.autopilot/state/linux-runtime-preflight-2026-11-03.md`, retained `plink-iw1-phase-evidence-20261103.8kL0NSeV/REPORT.md`.

## Global constraints

- Base owner e21010108cd40c716f2c79d2e175875446977d60, private main; no publication/remote changes.
- Owner `/Volumes/KIOXIA/Documents/omics-rust/rsomics-plink`; use its physical external Cargo/Rustup/target/tmp/cache guard before every build/test.
- Change only `src/dataset/metadata.rs` and `src/dataset/metadata/text.rs`, including existing inline consumer tests.
- Preserve Metadata/Sample/Variant/Text/Fields, compact arena ownership and existing BED/VCF Vec callers.
- Preserve first matching header column, defaults, complete width validation before typed errors, required-column order, Unicode whitespace, CRLF, preambles, unknown fields, ALT order and PR reconciliation.
- No codec/hash/durability/worker/CLI/schema/dependency/foundation/instrumentation changes. Rare source comments; no deletion/cleanup.

## Review focus

Iterator lifetime after source drop; duplicate/reordered/optional columns;
late width mismatch before typed errors; multiallelic/PR behavior; original
low-memory consumers and accurate allocation accounting. Use existing consumers
and a few integrated checks, not a scalar-helper matrix.

## Task 1: compiled columns and owning compact rows

**Interfaces:** Retain `samples(&[String], Vec<Fields>) -> Result<Vec<Sample>>` for BED/VCF. Add private `RowsBuilder::into_rows(self) -> impl ExactSizeIterator<Item=Fields>`; retain `finish(self) -> Vec<Fields>` as its collector. Check row-range widths before consuming the builder. Column schemas remain private and preserve first-match semantics.

- [x] Observe the existing10000-variant metadata-budget consumer before production edits. Pin an absolute ceiling requiring at least400000 fewer accounted owned bytes on that same fixture, justified by saving at least40 bytes per biallelic row. Record the numeric ceiling before implementation, tighten that consumer and observe RED. This is an owned-memory contract, not RSS/timing; no source-text/private-capacity assertion.
- [x] Implement column ordinals bound once, owning row iteration and REF+ALT Vec initial capacity2. Reuse construction across PGEN and existing Vec callers without other-module changes or weakened validation. Preserve bounded typed-vector capacity.
- [x] Retain existing UTF-8/unknown-column/PR roundtrip and low-memory/transaction tests. Add only an integrated reordered/duplicate-column and iterator-lifetime case if existing coverage is missing, with literal outputs. Any discovered defect gets a focused RED/GREEN.
- [x] Run guarded fmt, strict all-target/all-feature Clippy and complete debug/release suites: zero failures/ignored, previous consumers retained. No new scientific campaign or unchanged32-command rerun.
- [x] Freeze two-file delta/logs in external scratch; report tests, accounted memory before/after and remaining performance gap. Agent does not stage/commit/push; root reviews before integration.

## Preflight and rulings

One task produces/consumes the same compact Fields/arena contract. Preserving
finish and the Vec wrapper keeps BED/VCF call sites unchanged. Fix the resource
criterion before production edits; do not claim the entire0.311s preparation
phase is removable. User delegation removes another approval checkpoint;
product-local code needs no speculative public component. One fresh final
review covers the whole delta, not a helper review ladder.

## Later performance gate

After correctness/review, separately admit a bounded matched changed-source
comparison on the same complete I-w1 inputs/output pins and unchanged coarse
timers. Preserve green13 negative evidence. A memory-unit-test win, single noisy
timing or cross-campaign ratio is not whole-product acceptance.

Implementation/private integration complete at7cee8e4 after one clean final
review; baseline4106987, fixed ceiling3706987, final2911523 accounted bytes.
See `.autopilot/state/plink-metadata-construction-2026-11-03.md` for evidence
and the still-open native/source/performance gates.
