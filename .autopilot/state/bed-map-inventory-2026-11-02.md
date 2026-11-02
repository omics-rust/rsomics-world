# BED map source inventory — not compatibility acceptance

A bounded read-only worker inventory follows annotate integration. No oracle,
build, test, download, timing, implementation or public API was performed.
Root has not independently adopted the detailed reducer observations yet.

Source packet:
`/Volumes/KIOXIA/Developments/tmp/bed-coverage-contract-20261102.lmPuq9`.
Historical clone: `rsomics-bed-map` at
`1fb4c6cc8d848052734a98c02170be1e094001b0`.

Reported findings to verify against complete upstream outputs:

- ContextMap.cpp:10–15,39–48 and mapFile.cpp:19–31 use sorted execution,
  left join, one B file and column reductions, unlike annotate bin ordering.
- KeyListOps.cpp:16–39 registers23 reducers; INVALID is not an operation.
  Defaults/broadcast/cardinality and selected-column validation are at61–153.
- KeyListOpsMethods.cpp:104–137,194–338 distinguishes lexical distinct/mode
  from encounter-order collapse/concat/first/last. Numeric-sort distinct uses
  a separate formatter at219–232; do not assume shared precision.
- distinct_only separator placement at206–216 and position-dependent invalid
  numeric comparisons at140–187 need actual witnesses, not silent repairs.
- ParseTools.cpp:12–22 plus KeyListOpsMethods.cpp:347–360 use permissive
  recognition/atof, not a strict full-token Rust float parser.
- BlockMgr.cpp:218–310 and mapFile.cpp:19–25 require independent split/fraction
  analysis; neither coverage nor annotate supplies the whole map policy.

The old clone is a reference/fixture candidate, not a compatible production
core: only12 reducers, wrong lexical/tie formatting semantics, permissive
missing-column handling and unsafe lifetime extension. Its compatibility test
sorts rows and can skip an absent oracle, so it does not prove output order.
Do not revive its micro-crate, parser, index or CLI.

Next source work must recover missing NewChromSweep, reader subclasses,
ContextBase and KeyListOpsMethods.h before deciding candidate/record order.
Then acquire complete-output oracle witnesses for reducer families, numeric
and string distinctions, ordering/points, columns/cardinality and selected
BED12/strand/CLI behavior. No generic public reducer is justified by this
single consumer. Existing checked parser/index are reuse candidates, not yet
proof of sorted-sweep parity. This inventory does not authorize a speculative
API, implementation, publication or repair of source anomalies.
