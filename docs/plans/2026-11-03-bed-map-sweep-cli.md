# BED map sweep and CLI implementation plan

> Execute one coherent feature wave with meaningful RED/GREEN tests and one fresh whole-feature review at root integration. Unattended/direct-main authority is established. Do not add helper-review ladders or delete evidence.

**Goal:** Complete the existing product's sorted map operation, using its
23 implemented reducers and repaired point-boundary behavior.

**Architecture:** Checked BED readers feed a private streaming B candidate
cache and shared-per-column reducer state. A small CLI adapter uses the
existing common/help output, JSON and alias contracts. No eager whole-B index,
new public foundation or historical micro-crate revival.

**Tech stack:** Rust1.91 and existing locked product dependencies, safe Rust.

**Spec:** AGENTS.md; `docs/10-products/bed.md`;
`.autopilot/state/bed-map-contract-2026-11-02.md` and its source/oracle packets.
ContextIntersect validation closure is retained at
`/Volumes/KIOXIA/Developments/tmp/bed-map-context-source-20261103.2HnaYS`:
member10694B SHA256aad77ed16c4f758a35cd347ab2f4db286e79a15a8318992b23ca27d007aace62.

## Constraints

- Owner `/Volumes/KIOXIA/Documents/omics-rust/rsomics-bed`, HEADcdf736337d8bd63b2770723a8a667f7af83409b9.
  Preserve and integrate the existing three-file reducer delta whose identities
  are in bed-map-execution-2026-11-02.md; do not reset it or claim it committed.
- All builds/tests/package/install/scratch through the existing physically
  verified external guard; scratch `/Volumes/KIOXIA/Developments/tmp`.
- Preserve existing ten operations, immutable releases, dependencies and
  version. Do not stage/commit/push/publish or dispatch native/performance jobs.
- Comments rare; public docs state user contracts, not implementation history.
- Keep split plus explicit fraction/reciprocal/either rejected under the
  existing unresolved aggregate-fraction boundary. Do not guess it.
- Do not import coverage's point/count/fraction defaults or genomic size bounds.

## Files and interfaces

Extend `src/map/mod.rs` and existing `reduce.rs`; add `src/map/sweep.rs` and
`src/cli/map.rs`. Small changes in `src/{bed,cli,lib}.rs` expose checked column
and initial-header access, the map module and one dispatch arm. Add
`tests/map.rs` and portable `tests/golden/map/` fixtures; extend existing CLI
tests/lifecycle assets where they already live, rather than a parallel harness.

The library contract is:

```rust
pub struct MapOptions {
    pub columns: Vec<usize>,
    pub operations: Vec<Operation>,
    pub precision: std::num::NonZeroUsize,
    pub delimiter: Vec<u8>,
    pub null: Vec<u8>,
    pub strand: crate::StrandFilter,
    pub split: bool,
    pub minimum_a: Option<f32>,
    pub minimum_b: Option<f32>,
    pub reciprocal: bool,
    pub either: bool,
    pub sort_hits: bool,
    pub header: bool,
}
pub fn map(
    a: impl std::io::Read,
    b: impl std::io::Read,
    output: impl std::io::Write,
    options: &MapOptions,
    genome: Option<&crate::Genome>,
) -> rsomics_common::Result<()>;
```

Default is column5/sum, precision10, comma delimiter, period null, any strand,
no optional selection/reporting flags. Use the existing typed Operation,
adding concise API docs and parsing for its23 names, not a second string model.
Broadcast columns/operations once; otherwise require equal nonempty lengths.
Validate1-based columns even for count. Share lazy Reducer state per unique
selected B column and emit results in original requested pair order.

## Sweep contract

Maintain raw-record ordering and B encounter ordinals, validate widths within
each input, and consume/check B's unused tail before success. Empty B errors;
empty/header-only A is valid. Do not swallow late parsing/sortedness errors.

Without genome, use per-file first-encounter chromosome tracks, no chromosome
reentry and nondecreasing raw starts. Reject conflicting known common-chromosome
orders. Only unknown relative order uses the source's conditional lexical
fallback; retain lexico-assumed/disproven state and the final nonlexical
B-chromosome-membership check. With genome, use rank/membership/order, not
coordinate clipping against declared sizes. Actual supplement witnesses own
positive nonlexical, mismatch and exclusive-chromosome cases.

Use map-private widened virtual bounds (e.g. i128) for zero-length expansion,
including point0→[-1,1). Raw coordinates remain checked nonnegative values.
Retain cached B while its virtual end is >= current raw A start. Gather the
entire relevant equal-raw-start B group, since virtual starts need not be
monotone. Then independently apply real overlap/strand/fractions to each A.
The approved A-cache repair keeps count4/sum17 before/after equal-start A;
the same-class B-lookahead repair keeps point/count1/sum7 in either B tie order.
Preserve both original source defects as named known incompatibilities.

Ordinary overlap must be positive. Missing thresholds are0, not1e-9. Explicit
fractions must be finite and in(0,1]; ratios use binary32. Reject any explicit
B fraction with reciprocal, and B fraction+either without explicit A fraction.
Explicit A+reciprocal copies A threshold to B. Bare reciprocal/either and
reciprocal+either are allowed; either retains OR behavior. Do not silently
strengthen f+either when the absent B threshold is0.

Require valid BED6 +/- when strand filtering is requested (existing stricter
product boundary). Split counts each eligible B record once if any blocks
overlap; use private virtual bounds for non-BED12 records including origin
points. Sortout uses virtual(start,end) and stable B ordinal ties. Output A
coordinates canonically with original trailing-field bytes; preserve only
initial headers when requested, ignoring later comments.

## CLI

New adapter owns Args/execution; parent adds only module/variant/dispatch.
Use -a/-b, -c/--columns, --operations, -o/--output, --precision, --delimiter,
--null, --genome, --header, --sortout and existing product strand/fraction
spellings. -o remains output, not the upstream reducer alias. Only A may be
stdin. Reject aliases with A/B/genome before writing, enforce named output
under JSON, and reuse parent helpers/open_input/write_output/common/help.

## One coherent implementation task

- [x] Start with an actual failing consumer test for missing map behavior,
  using the retained complete numeric-all23 output, not getter/enum tests.
- [x] Implement streaming map, checked options and shared reducer integration.
  Import small golden inputs/outputs with original source provenance from
  bed-map-contract and bed-map-sweep-supplement packets. Correct only the two
  explicitly named point-loss expectations; do not rewrite source originals.
- [x] Exercise complete outputs for numeric-all23, text/broadcast/custom/null,
  encounter/sortout, ordinary split, strands/fractions, genome/nonlex ordering,
  header/canonical coords, point0 and both repaired point-order cases.
  Use a few integrated fixtures, not a test for every scalar helper.
- [x] Exercise real CLI common JSON/named-output behavior and all-input aliases;
  selected invalid numeric/column/empty-B, unsupported split+fractions and
  late sortedness errors must exit nonzero and preserve an existing named
  output. One installed multi-reducer JSON call extends the existing lifecycle.
- [x] Run guarded full debug/release, fmt, strict all-target/all-feature Clippy,
  package/install and existing installed lifecycle. Preserve all old calls;
  record actual populations and any real failure, no ignored diagnostics.
- [x] Freeze the whole delta, source/log hashes and rulings, index empty,
  root-ready. No public release or representative performance claim.

## Root final gates

Implementation and one fresh whole-feature review are integrated atbcf65d7;
see bed-map-execution-2026-11-02.md for the retained evidence and deferred minor.
The original owner/base and worker restrictions above describe task entry,
not the current committed state. Native-four originals and full mapping
performance are now adopted; the measured advantage is RSS only, with slower
throughput. See bed-map-native and bed-map-performance2026-11-03 receipts.
Release/publication remains separate; docs-only owner1de1e42 has its own CI.

Read the entire feature and obtain one fresh whole-feature review; independently
verify, integrate one coherent concern, then qualify actual native4 including
installed calls. One representative full mapping workflow on retained
RefSeq-derived data compares complete outputs before paired wall/RSS timing.
No small-fixture substitute or per-reducer benchmark matrix. The full portfolio
goal and separate registry/publication holds remain unchanged.
