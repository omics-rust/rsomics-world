# BED map execution

Current status: the complete map feature is committed and pushed at
bcf65d7accdf4ff956680cfdef8e5d4eeb9634f8. Native exact-head CI run41495972529
was queued at the latest observation. Performance and release gates remain open.
The user approves repairing the point-cache data-loss defect and recording
known incompatibility. Split-plus-fraction is still held.

The following Task1/Task2 entries preserve earlier implementation evidence;
their uncommitted-owner descriptions are historical, not current status.

Owner remains at cdf736337d8bd63b2770723a8a667f7af83409b9 with an intentional
uncommitted three-path source delta. Do not discard it, stage unrelated files,
publish it or mistake it for an already-pushed complete operation.

| Path | SHA256 |
|---|---|
| src/lib.rs | 2519461523f4a6f53afd18d38341ba4922f4c301848f976ce0237cb5b73a56e7 |
| src/map/mod.rs | 19430c4317a7a4f83f5696242d8af8cb948e993c6b6d2dab4fd85809792a0359 |
| src/map/reduce.rs | 98eba201bb88b7d033e13647b4d3e4170158935f9dc829f570085cd013f5d432 |

Evidence packet:
`/Volumes/KIOXIA/Developments/tmp/bed-map-reducers-20261102.PtWJHK`.
FROZEN.md records actual RED for absent typed reducer APIs followed by six
focused debug and six release passes, fmt success and all final log hashes.
Strict Clippy exits101 on five private unintegrated dead-code diagnostics;
no allowance suppresses them and no whole-product Clippy pass is claimed.

Root reads both complete new source bodies and verifies all three source
identities. The implementation has no source narration comments, unsafe code,
new dependency, public foundation, duplicate CLI or speculative point mode.
It keeps per-column lazy numeric/sorted/lexical state, linear min/max and
safe significant-digit formatting; complete scalar/string/nohit/singleton
outputs, numeric input errors and custom bytes have direct behavior tests.

Root executes the existing physical-external guard and focused tests again:
logs show6 passed,0 failed/ignored,71 filtered, after external disk12s1,
37GiB free and Cargo resolved target validation. The outer tool-return cell
stalls despite completed logs; exact command/test processes are absent in a
read-only process check. Root closes only that stale tool cell, does not
restart tests, and does not invent an unreturned shell exit status.
root-focused.stdout/stderr retain this observation. The same return-channel
issue affects a read-only control validator invocation; it prints internally
consistent and its process is absent. Hosted exact-head CI remains the final
control-plane gate, not the stalled return channel.

One fresh whole-feature review remains for after Task2, not another kernel
review ladder now. Full existing-suite/package/install/native4/performance
and release gates remain open. No benchmark or public map feature claim is
made by these six focused tests.

## Task2 started 2026-11-03

Plan: `docs/plans/2026-11-03-bed-map-sweep-cli.md`.
Root independently replays43 supplemental oracle identities and all eight
actual outputs/statuses, plus reads the pinned ContextIntersect validation
closure. Source gaps, point0 support, canonical A coordinates and the second
tied-B point-loss repair are recorded in the contract, not guessed by builder.
All3 inherited Task1 source hashes still match their original freeze.

Implementation packet:
`/Volumes/KIOXIA/Developments/tmp/bed-map-sweep-cli-20261103.xzaa4v`.
Actual RED red-consumer.log exits101 because the complete numeric-all23 CLI
consumer cannot invoke the absent map subcommand; this is not an unrelated
compile/import failure. The full feature, source review/native4/performance
and publication acceptance remain pending. No reducer-only feature is shipped.

## Integrated feature 2026-11-03

The source commit contains47 exact feature paths, including the existing reducer
kernel, sorted sweep, typed options, CLI dispatch and golden fixtures. No new
dependency, public foundation, unsafe code or version bump was introduced.
Both point-cache and tied-B lookahead data-loss repairs are documented as known
incompatibilities; ordinary valid-input behavior retains its upstream oracle.

Root independently matches250 frozen owner bodies and the complete feature
patch. One fresh whole-feature review finds no critical or important issue.
Its minor suggestion for an additional invalid-unused-B-tail rollback test is
deferred: the implementation already validates that tail, and no current bug
was identified. No extra defensive test expansion is required for integration.

Worker evidence records full debug/release/packaged-release118-test passes,
strict Clippy, fmt, rustdoc and14 installed CLI calls. Root independently reruns
the full debug suite through guarded-cargo after validating physical external
paths:118 passed,0 failed/ignored, command exit0. Local packaging used
--allow-dirty before commit and does not replace clean exact-head CI.

The owner is clean after commit and push. CI, native-four qualification,
representative whole-mapping performance and publication remain separate gates;
this source integration does not change any already-published release.
