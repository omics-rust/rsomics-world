# Count CIGAR and overlap correctness

Owner baseline: b207b4e5fbb3df047e453ce62496c7013ec8e666. New read-position
projection work is deferred behind confirmed existing counting differences.
No release/version change is authorized by this record alone.

## Original source and witnesses

External packet: `/Volumes/KIOXIA/Developments/tmp/count-projection-20261103.i3Q509A7`.
Subread 2.1.1 source archive SHA-256
`6392d7c66831cdd767e58251892a79a51b6fab8ed0ba9671ad5e85ff1ab01eaa`;
official macOS ARM archive SHA-256
`58f4d9a08d88334871fd89a1a87df1fc5560fe6ebad2e4527514cd7f8e3bf49a`.
Official featureCounts executable SHA-256
`3249dea2e90721618f8af4fb547ff1e9bab987620762c1e24148b149487f8ea5`.

Frozen `FRACTION-WITNESS.json` SHA-256
`d4691d9d0cd7fbb7cca35297473a203313d00321e3670eb1e0643c45a81c69f0`
records four successful baseline/source calls. With a five-base target,
5M2I5M versus 10M at fracOverlap .45 yields source count1/native2.
Overlapping ten-base mates at positions1/6 and fracOverlap .30 yield
source count1/native0. Neither difference requires projection flags.

`readSummary.c` uses separate original denominator geometry: D/N do not
contribute bases, insertions contribute anchored events, soft clips extend
denominator intervals only. Same-reference mates use interval union with
matching insertion-event semantics, not summed read lengths. Fraction
threshold rounding also differs from a direct floating ratio comparison.
These remain open; do not describe a deletion-only correction as a full fix.

## Deletion-gap correction in progress

One integrated fixture `deletion-gap.sam` uses 5M1000D5M outside every
model.gtf exon. Official source with -O exits0 and assigns zero reads.
Actual new integration-test RED returns three gene counts1 instead of0.
Private SAM and raw BAM parsers now split D gaps while retaining N-only
split-read classification. No public API/foundation/dependency change.

Initial subsequent tests still invoked an old Nov2
debug entrypoint despite source compilation. That entrypoint was moved,
not deleted, to packet `stale-debug-entrypoint`; library and binary rebuild
investigation isolated the unchanged library fingerprint. It too was moved,
not deleted, to packet `prior-library-fingerprint`, forcing a normal rebuild.
The resulting full debug suite passes all37 test functions, including the
actually enabled featureCounts contract and new SAM/BAM deletion regression.
Formatting passes. Temporary diagnostic prints have been removed. The new
deletion fixture is also connected to the existing live oracle, which passes
against the actual official executable. Strict Clippy passes. One bounded
read-only final review finds no material issue; fraction-denominator repairs
remain separate. The deletion-only commit is being pushed for exact-head
four-native CI41601044144 at exact head
`54cbc05446af38331edf0baded726d4f7f28a8de`. Nothing is published.

Next implementation must preserve separate assignment spans and original
denominator intervals with anchored insertion events, original reference
identity, and clipping semantics. Compare the two preserved fraction
witnesses before broadening the public CLI. Keep this product-local: no
new Layer A item or public crate is justified by these fixes.

All builds/oracle scratch use the owner's external-volume guard. The
inherited unrelated Java CPPFLAGS is unset only for each guarded command.
No global configuration or unrelated user data was changed.
