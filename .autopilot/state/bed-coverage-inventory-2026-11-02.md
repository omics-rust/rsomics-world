# BED coverage workflow source inventory

Read-only next-wave inventory, not implementation or compatibility acceptance.
Owning clean main is97aa04a6acfba702af35639f32be9e427ae29f90; the public
eight-operation0.2 slice is unchanged. Do not revive historical micro-crates.

Verified clean historical sources match portfolio-inventory.tsv entries76/81/92:

| Asset | Exact HEAD | Disposition |
|---|---|---|
| bed-coverage | 269e1f6ca740d4e6aae41558be275c47eb66660b | refactor union/depth-bases reduction; retain format and zero-length goldens; discard parser/driver |
| bed-annotate | 551632db1dceaa4918f9f5e7ea79527af6f979d5 | refactor union/count algorithms; retain multi-B/strand and ordering evidence; discard shell |
| bed-map | 1fb4c6cc8d848052734a98c02170be1e094001b0 | refactor12 reducers and retain tests; discard unsafe lifetime/manual driver |

One coherent candidate wave is coverage plus annotate/map, sharing a private
candidate/reducer model. This is not yet an approved implementation contract:
recover actual pinned upstream operation semantics before locking flags,
row ordering, numeric formatting and release scope. Dossier histogram/depth
remain real requirements even though old coverage implements only summary,
count and mean. Do not silently count the narrower old asset as complete.

## Concrete reuse and hazards

Owner BedRecord/BedReader and existing common/help/intervals contracts suffice.
IndexedBed preserves B multiplicity. Existing CoverageBed instead merges B
intervals at load: reuse only for union breadth, never feature counts, depth
or selected-column aggregation. RelationBed retains raw records but also
closest-specific order arrays; do not inherit those allocations gratuitously.

Historical coverage src/lib.rs108–161 computes clipped union-covered bases,
hit count and summed depth-bases without per-base storage. Its sorted-start
clips assumption is not guaranteed by owner B-file-order candidates. Sort
clips explicitly or use an order-independent reduction. Its effective_span
uses unchecked end+1; retain owner's checked zero-length coordinate policy.
The old moving cursor is not amortized linear with nested long intervals.

Historical annotate src/lib.rs242–265 deliberately emits UCSC-bin traversal
order, not A-file order. Source row-order evidence must precede the public
contract; do not silently normalize or copy a surprising order as an accident.
Its permissive strand handling also differs from the current strict BED6
boundary and needs explicit compatibility classification.

Historical map src/ops.rs has12 operations, original-byte collapse/distinct,
numeric poisoning and mode tie rules. Fresh oracle cases must cover nonfinite
values, ties, missing fields, count-only validation and distinct ordering.
Its driver lifetime transmute and whole-A buffering are not required in this
owner and must not be copied.

Root independently reads the owner CoverageBed load/query, historical clipped
coverage reducer, map reducers and annotate bin-order code after the bounded
inventory. No source execution, builds, test pass, new API or speed claim is
inferred. Historical coverage's tiny-golden benchmark is not representative;
future gates need sparse/dense/nested inputs and complete useful output.

Current next action is upstream source/oracle recovery, not implementation.
Sequence strand extraction and actual PLINK41421169780 proceed independently.
