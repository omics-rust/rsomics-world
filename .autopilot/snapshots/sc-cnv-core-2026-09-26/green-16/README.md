# Private native samples-distance candidate

Compared with red-10, add only src/cnv/samples/{condensed,distance}.rs and
move Ward's existing pair-count/index helpers into condensed.rs. Ward's
arithmetic, chain, sorting, output model, resource accounting and all prior
assertions remain unchanged. No test, fixture or expected scalar changes.
Cargo.lock remains byte-identical to accepted private product4c9e789.

The immutable borrowed view validates shape/finiteness once per route;
distance calls validate indices/output budget before fallible allocation.
The independent two-lane f64 reduction retains odd-tail handling, finite
arithmetic rejection and positive-zero/underflow semantics, without FMA,
reassociation, term buffer or clamp. Shared mechanics stay product-local.

Actual native RED39776437129 passed source/metadata/path checks, then failed
for absent condensed.rs, exit101; all157 source pre/postchecks passed.
Fresh independent admission/test and static kernel reviews approve with no
Critical/Important finding. Four-native debug/release comparisons of every
original107331 distance and all152 computed-vector trees remain pending.

No native filtering, groups, full step15, public API, performance or release
acceptance is implied. The selected original-matrix witness is authenticated
through the existing original artifact verifier and full semantic validators.
Plan: docs/plans/2026-10-22-infercnv-native-distance-plan.md.
