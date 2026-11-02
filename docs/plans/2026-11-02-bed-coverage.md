# BED coverage migration

Implement one coherent ninth BED operation, not annotate/map or a new crate.
Owner: `/Volumes/KIOXIA/Documents/omics-rust/rsomics-bed`, starting main
`97aa04a6`. Contract and actual BEDTools 2.31.1 evidence:
`.autopilot/state/bed-coverage-contract-2026-11-02.md` and its pinned packet.

## Scope and design

Keep default summary, counts, mean, histogram and per-base depth together.
Pool multiple B inputs without losing duplicates; stream A in original order.
Use existing checked coordinate/strand/parser contracts and candidate-level
fraction/reciprocal/OR filters. Preserve f32 ratios with seven decimals,
observed-depth histogram ordering and aggregate A-length multiplicity.

Reuse `IntervalIndexBuilder`/`IntervalIndex`, which retain per-record IDs.
Do not reuse union-merging `CoverageBed` or copy `RelationBed`'s unused raw
B bytes/neighbour orders. Coverage-local metadata and sorted clipped depth
events avoid an allocation proportional to A span. Reuse buffer capacity.
Put the engine in `src/coverage.rs`; split depth reduction into a private
module only if it materially improves the code. Add typed BED12 block parsing
to the existing parser on demand; preserve outer-span output/intronic zeros
and observed block-hit counting. No new Layer A item or dependency.

The confirmed upstream split-plus-fraction defect is a separate pending human
choice. Implement ordinary filters and block reduction, but do not choose or
advertise a default for that combination before the answer. This gate does
not block other modes or unrelated products. Do not claim complete source
option coverage: BAM, genomecov, sorted execution and header passthrough are
outside this wave, with unsupported flags rejected rather than ignored.

## One implementation and review wave

1. Add a compact full-output regression that fails because coverage is absent.
   One duplicate/nested/no-hit/out-of-order fixture covers five output modes.
   Add retained source witnesses for filters, multi-B and ordinary blocks.
2. Implement typed options and engine; wire `lib.rs` and existing `cli.rs`
   through common/help, transactional output and all-input alias validation.
   Keep source comments rare. Do not change existing eight operations.
3. Focused invariants, full golden/source outputs and one CLI lifecycle cover
   stdin, JSON, named output, late errors and aliases. Run existing regressions,
   formatting and strict Clippy. No helper-by-helper test/review ladder.
4. One fresh integrated review follows the complete wave. Root owns staging,
   commits, push, exact-head four-native acceptance and publication decisions.
   Update delivery/help/docs only for actually qualified stable behavior.
5. Before release, one representative campaign uses non-trivial BED inputs,
   complete index construction/output and sparse plus nested fan-out. Compare
   distinct summary/count/hist paths, warmup and five alternating pairs with
   full output equivalence, wall/RSS and exact identities. Per-base output
   needs correctness smoke, not a huge text benchmark matrix. Require a strict
   throughput or resource advantage; record failure honestly if absent.

Input-provenance ruling: the retained million-record BED benchmark generators
produce deterministic synthetic workloads, not biological measurements. Do
not label them real biological data or reuse the relation-window B lane as a
coverage-hit lane without checking overlap geometry. A declared deterministic
sparse/dense/nested interval workload is appropriate for this interval engine;
record generation provenance and actual fan-out. A biological annotation lane
is optional if already available, not a new blocking download requirement.

All source/build/cache/tmp/fixtures remain on verified external disks. No
deletion, registry retry, speculative public API, placeholder or release before
the relevant gates. Performance execution requires root review of the runner
and capacity; preparing it does not authorize a timed campaign.
