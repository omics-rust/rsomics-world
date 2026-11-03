# Count original-CIGAR read fraction

Correct existing read-fraction counting before adding position projection.
This is private product work, with no public API, dependency or crate change.

1. Preserve actual featureCounts2.1.1 counts for single-read insertion/clips,
   overlapping mates, and same/different/one-sided insertion events. One
   integrated SAM/BAM golden regression must first fail on the current owner.
2. Share a private CIGAR geometry parser across SAM and raw BAM. Separate
   assignment blocks from original denominator intervals; construct the
   latter only when read fraction is requested. Preserve N-only split flags.
3. Compute same-reference fragment denominator and feature-overlap unions,
   retain source threshold rounding, and keep different-reference geometry
   separate even if annotation aliases coincide.
4. Compare full count/summary output against the real oracle, run existing
   suites and strict Clippy, and obtain one bounded whole-change review.
5. Commit/push one correction after validation and require exact-head native4
   CI. Preserve previous timings; do not inherit new performance claims or
   publish before the intended stable counting contract is qualified.

Original source packet and provenance are recorded in
`.autopilot/state/count-cigar-overlap-2026-11-03.md`. Four source-only geometry
calls and original output bodies are retained in that same external packet
as `FRACTION-GEOMETRY-SOURCE.json` and `geometry-*-source.tsv*`.

Steps1–4 are complete. Step5 source commit
`1e636433bf50da9be33018d91796775e8b2ef95c` is pushed; exact-head native-four
CI41604306484 is running. Debug and release suites each pass38 actual test
functions with the source oracle enabled. No new registry/GitHub release.
