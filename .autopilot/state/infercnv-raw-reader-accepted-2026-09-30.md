# inferCNV strict raw reader — accepted

Product-local commit `e62fc960aeeb9dc543298885213cda6a57196fe9` accepts strict
explicit-header UTF-8 count TSV, headerless positions/annotations, plain or
gzip, into typed prepared counts and ordered cell groups. Canonical raw inputs
then pass the existing non-HMM numerical core through step 14. Product remains
library-only, version 0.1.0, `publish=false`, with no configured remote or public
product repository. Exact product bytes were tested via the controller's frozen
snapshot; this is not a product publication or complete inferCNV.

## Native and original-evidence gates

World `2aee90ce7cb984d236fcfe15aa18261099426926` passed exact-head control run
`36695918799`. Frozen `green-10` passed native run `36696131908`, attempt 1,
on Linux and macOS, x86_64 and aarch64. The
[receipt](../oracles/infercnv-raw-reader-native-2026-09-30.json) binds the exact
144 source files, production paths, archive, manifest and locked dependencies.
Both the controller and an independent reviewer checked the original evidence.

Each target's debug/release profiles executed 72 tests with zero failures or
ignored tests: 13 unit, 16 numerical contracts, three external, eight fixture,
31 reader and one oracle-equivalence test. Each also passed 11 opt-in
measurement-support tests and compiled the bench. Linux x86_64 passed format
and strict all-target/all-feature Clippy. Missing external-root configuration
was separately required to fail. Source-before/after checks verify all 144
files including the lock; current product files match the verified snapshot.

Original run/jobs/artifact JSON, logs and four ZIPs are retained under
`/Volumes/Zane's HDD/rsomics-fixtures/evidence/infercnv-ingestion-2026-09-30/run-36696131908/`.
Each ZIP passed API size/digest, CRC, safe unique member paths and byte equality
for the frozen archive/README/manifest/checksums. Logs SHA-256 is
`6cd139f998e09653fb273724b429d9d46b8aeb50708a70857413c2b90f19aef8`.

| Target | Artifact | Original ZIP SHA-256 |
|---|---|---|
| Linux x86_64 | 11087088712 | `9204b56a0c2b2bc883516de5e947717d304ab17d38a1a8e6c03a3b706604d308` |
| Linux aarch64 | 11087557864 | `e0ce13f09f8846cb1a7281c4e9ce08da749ccaf808b59d5700b6d686f4914629` |
| macOS x86_64 | 11087639974 | `08cb8b7dad93d226bfd2e3504facb925fec27c091094642d96f6d494a90559fd` |
| macOS aarch64 | 11087628451 | `f2a7a4ffa4eccadccf312e10012fc4c685a3ed0a571b92bad101b52691308805` |

## Accepted contracts and deviations

Every profile executed 40 raw checkpoint comparisons: shipped subset/full and
synthetic grouped/no-reference, ten saved stages each. Creation genes, cells,
groups/maps and values match exactly. Downstream maximum absolute delta is
`1.2732925824820995e-10`; maximum tolerance ratio is `0.010642098320408794`
under the unchanged `1e-12 + 1e-12*abs(expected)` gate. Twenty existing
prepared-shipped checkpoint comparisons also remain. Numerical-core and
measurement-code bytes are unchanged.

The reader validates every record, even discarded genes; annotations are
checked against the original header before cell filtering. It preserves
expression ordering for coordinate ties, all surviving-position chromosome
ranks, inclusive post-reduction totals, requested reference order and bytewise
observation order. Complete gzip EOF/truncation/checksum/trailing-garbage,
decoded-record caps and overlapping numeric-capacity budgets have focused
contracts and reviewed error paths.

Numeric tokens trim only ASCII spaces because actual R canonical `formatC`
exports use leading padding. Names/groups/coordinates are not trimmed. A
literal first `V1` annotation is retained rather than silently discarded.
Counts use deterministic native binary64 conversion and compensated totals;
the tested `0.076439` lower-neighbor result is not substituted or hidden.
Metadata/allocator/process memory is outside the numerical budget. Group
resolution is O(cells × groups); representative high-group scaling is pending.

Only `flate2=1.1.9` and four pure-Rust gzip dependency nodes were added; old
lock entries remain unchanged. Input policy stays internal to the one actual
consumer, not a new speculative public foundation.

## Exclusions and next work

Original shipped implicit count headers, arbitrary R fractional lexical or
threshold equivalence, general R table dialect, R locale ordering, RDS/sparse
input and sampling remain excluded. No ingestion throughput/resource advantage
has been measured. The prepared-only 41.37x measurement is not extended to
this reader or end-to-end inferCNV.

Clustering, HMM, Bayesian refinement, denoising/masking/pruning, reports, CLI
and release remain unfinished. The next
[samples-clustering witness](../../docs/plans/2026-09-30-infercnv-samples-clustering-witness-design.md)
executes actual fastcluster trees and source filter branches before native
model/algorithm work. Default Leiden and HMM i6 require separate evidence.
The [execution ledger](infercnv-raw-reader-2026-09-30.md) retains test-first and
failed-run history; no failed artifact was rewritten into an accepted receipt.
