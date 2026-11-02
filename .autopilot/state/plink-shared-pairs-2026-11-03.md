# PLINK shared immutable pair preparation

Plan: `docs/plans/2026-11-03-plink-shared-pairs.md`.
Owner starts at1906a48b8e6e70bf2478c8e3daf1a44c38c56572; the current
implementation delta is intentionally uncommitted. Do not discard it or
mistake it for accepted/pushed code.

Packet: `/Volumes/KIOXIA/Developments/tmp/plink-shared-pairs-20261103.w4cjwy65`.
Root reads the actual consumer and retained RED/GREEN logs. The128-variant
overlapping-window consumer first verifies8128 evaluated pairs, complete
serial/parallel output and semantic count equality, and actual4 workers.
RED then fails for416 real decodes versus128 expected, not for an unrelated
compile failure or merely requested worker count. The same test is GREEN
after shared preparation and records one passing focused test; three workflow
tests remain filtered in that focused invocation. It is not a full-suite pass.

Builder is completing mixed DS/HDS/missingness/65-founder behavior and real
low-budget fallback/zero-pair consumers, followed by full debug/release/fmt/
strict Clippy and allocation-capacity accounting. Root whole-source review,
one fresh whole-feature review and independent full verification remain open.
No native qualification, performance experiment, publication or speedup claim
follows from this focused decode-count improvement. The negative green12
measurements and unchanged output pins remain authoritative.

## Full-suite layout assertion and bounded ruling

The first full debug run has14/15 library tests passing. Its failure is the
old native_workflow.rs fixed540000-byte assumption that pair execution must
evict cached variants. Compact shared data can fit that budget, so mandatory
eviction no longer follows from the user-facing bounded-memory contract.
This is not a full-suite pass; preserve debug-full.log and all old outputs.

Root reads that assertion and the new integrated low-budget consumer. Ruling:
allow one narrow test-assertion change in native_workflow.rs, preserving the
fixed-budget operation and complete output comparison. Do not restrict the
optimization merely to retain the old storage layout. Move the actual eviction
guarantee into the existing4096-row integrated consumer at a real no-fit budget
and a moderate wider window if needed. Shared no-fit alone does not prove
eviction when a two-row resident window still fits. Preserve both bounded
fallback and output equality, without a production test flag or new matrix.
Cost if wrong: the old low-memory path could silently lose coverage; therefore
real fallback/eviction assertions remain required before full acceptance.
