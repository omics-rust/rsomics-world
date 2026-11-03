# Count projection implementation and qualification

Owner: `/Volumes/KIOXIA/Documents/omics-rust/rsomics-count`.
Source: `2b5e450372689063736721dc1b807dcba9d62717`, pushed to main.
Exact-head native-four CI41607175618 passed. Prior original-CIGAR
correction1e63643/native-four41604306484 and Worlda5ef09f1/41604511759 passed.
No new registry or GitHub release. New public ProjectionOptions/ReadPosition
and CountOptions field require a minor version before publication.

Plan: `docs/plans/2026-11-03-count-projection.md`.
Source contract: `.autopilot/state/count-projection-contract-2026-11-03.md`.
Existing source packet is
`/Volumes/KIOXIA/Developments/tmp/count-projection-20261103.i3Q509A7`.
Official featureCounts2.1.1 SHA-256 remains
`3249dea2e90721618f8af4fb547ff1e9bab987620762c1e24148b149487f8ea5`.
Five actual source workflow calls are preserved in PROJECTION-WORKFLOWS.json,
SHA-256 `b7ca56a1b92bb1662a8d140fc24de23a5e70e5cef6c2b01fbe35ba73d1b3046a`.
The earlier three-call leading-gap witness is retained unchanged.

One joint golden RED actually failed on unknown --read-extension5. The
implemented mixed SAM/BAM and paired raw-strand/fraction workflows now pass.
Existing live oracle compares every count column and summary category for
five projection workflows. Debug/release suites each execute40 functions
with FEATURECOUNTS enabled; formatting and strict Clippy pass. One fresh
whole-feature read-only review finds no Critical/Important issue. Invalid
reduction values and extensions beyond i32::MAX preserve existing outputs.

Private geometry records the first/last original assignment sections,
including empty leading sections. Shared projection extends biological5prime
then3prime and optionally reduces to one base. It uses each mate's raw
orientation and actual header bounds; Extent and original split status are
unchanged. A private ReadOptions bundles reader configuration. No new public
foundation/dependency, placeholder operation, shift flag or sibling-product
dependency is introduced. Undocumented source read2pos suffix leniency is
deliberately not accepted: native CLI requires exactly3 or5.

Fresh external performance packet:
`/Volumes/KIOXIA/Developments/tmp/count-projection-perf-20261103.SIeiKW2O`.
RESULT.json SHA-256
`223a2c9b00e01c8355ad959a19a2495d05031594d65c914939c5b3007d0e3cb4`;
VERIFIED.json SHA-256
`1f3afc37c212937d38f54e08cb7c9b53e8f049b9d9f74957a08684e047a0d373`.
Native3022000 bytes SHA-256
`7fd617259202b77eb112d42fcd5e33e24b8a99e163d10e633c84867bf12007e8`.
Root independently rehashes64 identities, rechecks all12 complete count/
summary pairs and actual RSS reports, and verifies every measured production
source body against the committed HEAD. Tracked raw samples match originals.

AppleM2/8GiB/macOS27.2/Rust1.91.0; same2,000,000-record BAM,500 genes,
1500 exons as the preceding correction. Four threads, one warmup and five
alternating measured pairs per case. Every invocation assigns at least one
million reads. Native/source medians:

| Case | Wall seconds | Peak RSS bytes |
|---|---:|---:|
| defaults |0.517203125 /0.361356625|12042240 /68485120|
| ext5=50/ext3=20/read2pos5 |0.500470209 /0.305397917|10092544 /68501504|

RSS is82.42%/85.27% lower, with all individual native samples below every
source sample. Wall remains1.43x/1.64x slower. Resource-use gate passes,
not throughput; no real-cohort, universal, or CI/public-binary timing claim.
Full distributions/argv/output/source identities remain in the original
packet and owning benchmarks/2026-11-03/projection* records.
