# Native CNV final-fix candidate: green-3

Regression-red run 36218967725 at world
`651cebb5ae837cd83028e4288e14606de9866415` demonstrated three exact-value
assertion failures for subnormal medians/depths; all previous 27 tests passed.
The new three tests are unchanged in this candidate.

Relative to red-2, production median now uses `f64::midpoint` and
`sort_unstable_by(f64::total_cmp)`, preventing the demonstrated subnormal
loss and avoiding stable-sort's implicit heap allocation. No other product
file changed. The candidate has 30 tests. Lockfile, fixture TSVs and golden
tolerances remain unchanged.

`files.sha256` identifies every source file; `checksums.sha256` identifies
this receipt, that manifest and the archive. Four-native execution and final
scoped review are required before acceptance. The scope remains an unpublished
prepared-counts numerical core, not the complete workflow or a release.
