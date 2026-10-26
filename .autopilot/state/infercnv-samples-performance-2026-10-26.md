# Canonical samples CLI performance

Previous goal turn: progress, not a wait or blocker. The real raw-to-samples
CLI was implemented, committed through product f8a10a0 and accepted in actual
four-native green20/run39820518598. Resume verified current clean product,
terminal GitHub success and retained original summary/logs; control-plane
acceptance reconciliation is committed7fbbe3c1 with CI40299421920 success.

Current work implements docs/plans/2026-10-26-infercnv-samples-performance.md.
Production code and the accepted green20 snapshot are unchanged. The new
driver measures actual fresh CLI processes at1/4 workers, keeping warmups,
five alternating measured pairs, GNU-time lifetime/RSS, owner elapsed time,
all report bytes and normalized-output equality. It reuses existing timing/
process ownership rather than building another framework. Linux-only opt-in
workflow mode is explicitly not the four-native release gate.

Four driver tests first failed for the missing implementation and now pass.
The new workflow guard first failed for the missing step and now passes;
eleven workflow tests include actual shell retention behavior after a failed
benchmark invocation. Initial coherent438 tests pass50.543s with two existing
native-only skips. After the review fix, focused11 workflow tests pass0.703s;
exact-head CI will run the final full suite before actual dispatch.

One overall fresh-context static review found a retention issue: the measured
binary was copied only after success. It now copies/hashes before invocation
and measures that retained copy. The regression forces a child failure and
checks actual file bytes survive without a fabricated complete summary.
No re-review or per-helper admission campaign is needed. Actual runtime,
throughput and peak memory remain unmeasured until the hosted job executes.

Storage2026-10-26: APFS boot81.27%; KIOXIA266GiB and fixture HDD273GiB free.
No local Rust/R build, install or dependency download. Local Python -B tests
use explicit external CARGO_HOME/CARGO_TARGET_DIR/TMPDIR. Preserve unrelated
VCF state and root scratch. No repository creation, publication, deletion,
reminder, heartbeat or cron is introduced.

Next: exact-head controller CI, actual Linux benchmark using snapshotgreen20
and the existing original samples receipt, inspect retained raw measurements,
then address observed bottlenecks. Upstream R comparison, a genuinely larger
cohort and remaining inferCNV operations are still required; this thread's
goal remains the whole portfolio, not only the benchmark or samples slice.
