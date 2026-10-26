# Canonical samples CLI performance

Actual result: run40301640764/attempt1 is terminal success at world
7cb53b5995467442c37262b6d53618bb64c5875e. Exact-head controller40301195690
also passes. The Linux-only benchmark mode executed12 distinct successful
processes (two warmups, ten measured runs); all outputs share the same full
report fingerprint after removing only settings.threads. Existing source/
oracle admission, native debug/release and strict Clippy also pass.

Five measured serial wall values:0.64,0.64,0.64,0.64,0.64s.
Five measured4-worker values:0.63,0.62,0.62,0.62,0.64s.
Medians0.64/0.62s; ratio1.032258, overlapping ranges, no strict separation.
Median maximum RSS:60,346,368/60,329,984bytes (about57.6MiB each).
GNU-time wall has centisecond granularity. Owner elapsed medians are
0.645378523/0.627440003s and include observer overhead, not only CLI time.
This is warm-input plain raw-to-default-report CLI behavior for the real
9939-gene/184-cell shipped input (8508 retained/6187 selected), not a cold,
large-cohort, R-comparison, HMM or full-inferCNV performance result.

Runner: native Linuxx86_64, Rust1.91, four affinity-visible CPUs; CPU quota
was unavailable, not proven unlimited. Baseline load2.66/1.05/0.41;
after2.40/1.05/0.41. Do not infer dedicated-host performance from this.
The measured retained binary SHA is
20cb15f0b465a2320a3199a7dbd75587ed1d10ce191b93964cb5a6ab09ce8bfb.
Artifact12640687144,4,574,045bytes,SHA
54c265abc55376839eade8bf824c811a5b2ad92573204b19d61fedf5dfa4bd7e.
Raw API/log/ZIP files are retained externally in
evidence/infercnv-samples-performance-2026-10-26/run-40301640764/.
Root checks actual time files,12 distinct zero-exit/empty-group process
records, report-byte hashes, retained binary hash and recomputed medians.
This is a focused measurement check, not another general admission campaign.

Decision: four distance workers alone have little end-to-end effect on this
small case; do not advertise a material overall speedup. The source still
runs matrix preprocessing serially. Inspect/profile that path and measure a
larger real input before changing algorithms or making replacement claims.
No user approval is required for ordinary product-local optimization; retain
existing oracle tolerances and validate integrated output, not every helper.
The next larger real source is now downloaded and inspected, not yet adapted
or benchmarked:36,601 features/4,103 cells/12,410,405 integer nonzeros from
the public10x GBM sample. See infercnv-real-cohort-2026-10-26.md for provenance,
matching reference-build requirements and the absence of normal-cell labels.
Earlier implementation notes below are historical.

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
