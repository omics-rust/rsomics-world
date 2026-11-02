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

## Missing source recovered; oracle work is separate

Bounded read-only retrieval freshly verifies the exact remote archive:
`/data1/liangjy/rsomics-bed-coverage-source-20261102.Kfp595/bedtools-2.31.1.tar.gz.part`,
19629373 bytes, SHA256
`fc7e660c2279b1e008b80aca0165a4a157daf4994d08a533ee925d73ce732b97`.
New external packet:
`/Volumes/KIOXIA/Developments/tmp/bed-map-contract-source-20261102.53Ijah`.
SOURCE/SUPPLEMENT/CLOSURE retain exact archive member identities and commands;
original-base64 preserves original bytes, while src is a labelled normalized
text view. Root independently decodes and verifies all23 original identities.
An initial root reader omitted the documented .b64 suffix; correcting that
reader path yields23/23, not a source repair or runtime retry.

Root reads NewChromSweep's cache/scan/chromosome-order paths and the reducer
dispatch/method bodies. Cached and newly read candidates preserve B encounter
order; sortout separately sorts by coordinates without a stable equal-key
guarantee. Without a genome, cross-file encounter-order tracking and lexical
fallback interact; a blanket lexical requirement is not source parity.
With a genome, explicit chromosome rank owns ordering. Frequency tie order
is lexical within either ascending or descending frequency direction.

Architecture-specific header recognition and invalid-number warnings are
source observations, not a requirement to reproduce malformed-data quirks
or broaden the product parser. No upstream implementation body was copied
into product code. One compact real-oracle activity is authorized to establish
complete outputs for reducer families and meaningful ordering/format/selection
contracts before the implementation plan. This is not performance acceptance.
