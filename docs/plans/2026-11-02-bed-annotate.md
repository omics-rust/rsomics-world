# BED annotate implementation plan

> Agentic workers use executing-plans for one coherent feature wave and one fresh whole-feature review. The user authorizes unattended execution; no separate design/helper approval ladder or scratch deletion.

**Goal:** Add complete BED-input multi-file annotate to the existing product.

**Architecture:** Reuse the checked parser and duplicate-preserving index.
Move coverage's clipped depth primitive to one private module; keep annotate's
point semantics, source-bin ordering, per-file results and f64 formatting local.

**Tech stack:** Rust1.91 and existing locked common/help/intervals/coitrees.

**Spec:** `docs/10-products/bed.md` and
`.autopilot/state/bed-annotate-contract-2026-11-02.md`, with its ten actual
complete-output source witnesses. Owner:
`/Volumes/KIOXIA/Documents/omics-rust/rsomics-bed`, clean80704f3 at handoff.

## Global constraints

- No new dependency, foundation, micro-crate or Layer B dependency.
- Existing nine operations, policies and qualified evidence remain intact.
- External-only verified source/build/cache/tmp/output paths; inspect and use
  the existing owning guard before builds/tests. No timed campaign yet.
- Source comments stay rare; public docs describe contracts, not process.
- One feature implementation/fix/review wave; root owns commit/push/native4/
  performance/publication. Do not stage or delete unrelated/inherited files.
- Existing coverage split/fraction hold is unchanged and does not block this.

## Review focus

The existing ten oracle witnesses plus one CLI lifecycle own these checks:

1. Point B contributes counts but no breadth; do not inherit coverage policy.
2. Source eight-level bin/input-tie ordering is not coordinate sorting.
3. Named headers preserve full BED12 width and still appear for empty A.
4. Each file retains independent duplicate counts and f64/six-place fractions.
5. Private depth extraction must preserve prior coverage's f32/seven-place,
   pooled-B, block and histogram behavior through the existing regressions.

## Task: one complete annotate operation

**Files:** new `src/annotate.rs`, private `src/depth.rs`,
`tests/annotate.rs` and compact source-witness goldens; modify `src/coverage.rs`,
`src/bed.rs`, `src/lib.rs`, `src/cli.rs` and `scripts/delivery.py` only as needed.

**Interfaces:**

```rust
pub enum AnnotateMode { Fraction, Counts, Both }
pub struct AnnotateOptions {
    pub mode: AnnotateMode,
    pub strand: StrandFilter,
    pub names: Vec<String>,
}
pub fn annotate<R: Read>(
    input: impl Read,
    annotations: impl IntoIterator<Item = R>,
    output: impl Write,
    options: AnnotateOptions,
) -> Result<()>;
```

Private `DepthBuffer` retains reusable event/run storage and exposes
`clear_events`, `add(start,end)`, `reduce(start,end)`, `covered_bases(length)`,
`runs() -> &[DepthRun]` and `histogram() -> &BTreeMap<usize,u64>`;
`covered_bases(length: u64) -> u64`. It receives already-clipped spans; point and fraction
selection do not enter this module. Existing coverage reduction logic moves
without an algorithm change. `BedRecord::field_count` supplies header width.

- [ ] Add one absent-command RED in `tests/annotate.rs`; migrate the ten
  compact full outputs, including point counts1/2/1 with all fractions0,
  bin/tie order, names/both, BED12 and empty-A headers. Run through the external
  guard; expected failure is missing annotate, not a transport/import error.
- [ ] Implement all modes and source ordering. Retain A records and sort IDs
  by chromosome/source-bin/original index, not formatted output. Load one B
  index at a time; use its original start/end to skip point depth while still
  counting eligible hits. Store count/covered metrics per A/file, then emit
  original A bytes with f64/six-place values. No per-base arrays or global B
  raw-line cache. Keep original origin-point and coordinate-limit errors;
  empty A does not silently exempt supplied B files from parsing/validation.
- [ ] Extract the narrow depth primitive and adapt coverage; do not unify
  numeric format, output order, selection or aggregate policy. Existing nine-
  operation regressions are the proof, not a new helper matrix.
- [ ] Wire `annotate -i/--input BED --files BED... [--counts|--both]
  [--names NAME...] [-s|-S] [-o/--output BED]` through existing help/common.
  Only A may be stdin. Require at least one annotation file; names, if present,
  match file count, and counts/both conflict. Validate all input aliases and
  keep JSON named-output and late-error preservation. Reject unsupported flags.
- [ ] Extend the existing delivery list to ten operations before native CI.
  Keep all twelve prior calls and add one real stdin/two-file/names+both/JSON
  annotate invocation with complete expected bytes:13 recorded calls. Do not
  repeat the previous stale help-scope failure or weaken that assertion.
- [ ] Run guarded fmt, strict all-target/all-feature Clippy, full debug/release
  and cleanly identified package/install lifecycle checks. Expected: all prior
  tests plus annotate pass, full source comparisons pass, real installed
  command executes. Dirty candidate packaging must remain labelled dirty.
- [ ] Freeze the complete delta and logs for root plus one fresh-context whole-
  feature review. No implementation worker commit/stage/push/publish or cleanup.

## Root-owned subsequent gates

Root reads source/evidence, integrates one concern and waits exact-head actual
four-native tests, packaging and installed ten-command lifecycle. One later
representative activity uses retained RefSeq nested A, exon B and merged-exon
second B with both mode, complete indexing/reduction/sort/output and full-byte
oracle equality. One warmup and five alternating pairs, wall/RSS, all outputs
retained; no per-format benchmark matrix. Review runner/capacity and authorize
that campaign separately. Only measured qualified stable behavior may ship.

Design ruling: share a narrow private span primitive, not coverage() itself or
a public generic reducer. Whole coverage reuse would import wrong point,
format and ordering policies; a foundation lacks a second product consumer.
The user already delegates ordinary implementation decisions; no new user
choice is needed for this valid-input contract.
