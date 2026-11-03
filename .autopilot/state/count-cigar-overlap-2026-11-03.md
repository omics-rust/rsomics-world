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

## Deletion-gap correction adopted

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
`54cbc05446af38331edf0baded726d4f7f28a8de` passes. World2bd3a1ad and exact-head
CI41601672350 also pass. Nothing is published.

Next implementation must preserve separate assignment spans and original
denominator intervals with anchored insertion events, original reference
identity, and clipping semantics. Compare the two preserved fraction
witnesses before broadening the public CLI. Keep this product-local: no
new Layer A item or public crate is justified by these fixes.

## Original read-fraction correction

Plan: `docs/plans/2026-11-03-count-read-fraction.md`.
Four actual upstream geometry calls freeze complete original outputs in
`FRACTION-GEOMETRY-SOURCE.json`, SHA-256
`537fe309918bb908750815422e69b5a2ebb4cc0640ca37f416a551338a75edbf`.
The combined single fixture includes insertion, leading/trailing soft clips
and a plain read; actual native RED is4 versus source1. Paired fixtures cover
reference unions, shared coverage, identical/differing/one-sided insertion
events. Source counts at .3/.45/.7 are preserved, not inferred from new code.

One private `geometry` module now supplies both SAM and raw BAM, with
conditional original denominator intervals. Same-reference mate union uses
source event matching; numerator union preserves reference partition and
per-mate strand selection. Original read-fraction threshold rounding is
retained. No public API/dependency/crate or version change. New tests add one
combined SAM/BAM golden function and extend the existing full-output live
oracle to five geometry/rounding cases. Full debug38 functions, actually
enabled live source oracle and strict Clippy pass. One fresh whole-change
read-only review has no Critical/Important finding.

Fresh representative performance packet:
`/Volumes/KIOXIA/Developments/tmp/count-read-fraction-perf-20261103.jmlsTpR5`.
RESULT.json SHA-256
`fd87cd4a329939b9bfbfda3cca9716c06c09efa40b26365970244c0789307fea`;
VERIFIED.json SHA-256
`db953177df8aee1025944c29e21cef97f6f442fe86cd8e65dc55e3a2484bd0ba`.
Root independent replay rehashes66 identities, rechecks all12 complete
count/summary pairs, and compares actual RSS reports. Two million single
records and one million paired fragments use500 genes/1500 exons, four
threads, one warmup plus five alternating measured pairs per case.
Native/source median RSS is10,108,928/68,501,504 bytes for default singles,
12,107,776/69,091,328 bytes for paired fracOverlap .45:85.24%/82.48% lower.
Native wall remains1.48x/1.67x slower. All individual native RSS values are
below every source RSS in each case. Resource-use gate passes; no throughput
advantage, real-cohort result or CI-binary timing is claimed. Machine,
toolchain, raw samples, binary/input/source-file pins and argv remain in the
original packet and owning benchmark record. Release-suite38 functions and
the enabled source oracle pass. Source commit
`1e636433bf50da9be33018d91796775e8b2ef95c` is pushed; root verifies all measured
production-file identities against that commit. Exact-head native-four
CI41604306484 is running. No release is made by this entry.

All builds/oracle scratch use the owner's external-volume guard. The
inherited unrelated Java CPPFLAGS is unset only for each guarded command.
No global configuration or unrelated user data was changed.
