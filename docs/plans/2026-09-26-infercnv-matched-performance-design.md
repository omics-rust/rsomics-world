# inferCNV matched prepared-input measurement

## Intent and authority

Determine whether the now-correct native pre-clustering implementation has a
measured advantage over installed infercnv on the same example and machine.
The user delegates routine design/execution decisions and asks work to continue
without approval stops. This is a bounded measurement subsystem, not a product
CLI, public library, full inferCNV implementation or publication decision.

Correctness prerequisite: product `bdcc8a3a55596be8d46a774d39b0335845c75175`,
accepted shipped oracle `36221554790`, four-native run `36222541338` and receipt
`../../.autopilot/state/infercnv-shipped-native-accepted-2026-09-26.md`.
Final correctness documentation head `282d2171dc799e5ef7e94962e7b0b8c53a4a922e`
passed Control plane run `36222894885`.

The source-reviewed timing boundary in
`../../.autopilot/state/infercnv-performance-boundary-review-2026-09-26.md`
remains binding. The measurement is prepared-input preprocessing through step
14 including normal wrapper work, not pure kernels, ingestion, downstream CNV
calling, large-cohort scaling or biological validation.

## Implementation choices

Use a private opt-in harness-free Rust benchmark, one R measurement script,
and a standard-library Python trial driver/checker. The existing pinned-oracle
workflow receives an optional matched-snapshot selector. When selected it
requires shipped mode, retains the existing actual synthetic/shipped oracle
execution and checks, and runs measurement afterwards on the same native Linux
host. The R installation workflow is not copied. Ordinary oracle runs remain
unchanged when the selector is empty.

The selector must resolve an immutable committed green snapshot, not a working
checkout. A controller-created receipt, written only after the new four-native
gate is genuinely verified, binds snapshot name, archive hash, source-manifest
hash, resolved lock hash and exact native run/head/attempt. The matched workflow
checks that receipt, corresponding successful run identity and snapshot bytes,
rejects unsafe archive paths/types before extraction into new runner-temp
storage, checks every source hash before/after,
and uses --locked for release building. A later recorded binary hash is extra
provenance, not a substitute for this pre-measurement tested-source binding.
Production-core bytes must remain identical to accepted product bdcc8a3a;
only the declared benchmark, test support, manifest and lock additions may vary.

Rust uses a Linux-only dev-dependency
`nix = { version = "=0.29.0", default-features = false, features = ["resource"] }`
for checked `getrusage(RUSAGE_SELF)`. This inspected safe wrapper keeps the
no-authored-unsafe rule. Direct libc would need an unsafe exception; coarse
`/proc/self/stat` CPU ticks would weaken measurement resolution. Neither is
selected. No production dependency or public API changes.

The lockfile must be resolved remotely against the existing lock, preserving
every existing package version/checksum and common 0.12.3. Only the new
benchmark dependency, its required additional packages and root dev-dependency
edges may change. Inspect that exact lock diff before accepting it. Local
Cargo, R, dependency installation and builds remain forbidden while boot APFS
exceeds 80%; scratch, artifacts and data remain on the prescribed external disks.

The benchmark uses a separate non-default feature `infercnv-measurement` and
target `cnv_matched`, with `harness=false` and `test=false`. Native correctness
tests continue using `external-infercnv-oracle`; measurement unit tests are run
as an explicitly named test target, not by accidentally executing the custom
benchmark through `cargo test --all-targets`. Clippy must compile all features
and targets, including platform-gated benchmark code. Actual matched timing is
Linux x86_64 only; unsupported invocation elsewhere fails explicitly.

## Inputs and trial lifetime

Use only the accepted full example: incoming 9,939 genes × 184 cells, retained
8,508 × 184. Pin the six accepted full-case stage-1/stage-14 expression, gene
and cell TSV hashes in a small control-plane measurement-input receipt linked
to the accepted upstream receipt. Fresh workflow oracle exports must match
those pins before any timing; a newly self-reported hash is insufficient.
R reads its freshly generated original stage-1 RDS and verifies its expression,
genes, cells and count data against the pinned incoming state before timing.
Its path and digest must match the freshly validated oracle.json full/stage-1
checkpoint entry. Verify reference and observation group index maps against
the pinned cell/group identities, including index order, roles and complete
nonoverlapping cell coverage; require .hspike to be NULL. Do not normalize or
repair these slots during measurement: a mismatch fails preparation. Otherwise
extra hidden-spike work could alter timing without changing the primary TSV.
Rust constructs the same prepared counts from stage 1. Do not load all ten
expected matrices into a measured process; expose a test-only single-stage
loader and retain the existing ten-stage fixture entry points.

Build the release Rust executable once and launch its exact path directly,
never Cargo, during trials. Each R trial launches Rscript directly. Packages,
fixture parsing, prepared-state validation and output-directory creation are
outside the measured region; record preparation wall time separately.

Each fresh process performs one untimed warm-up, discards that returned result,
then performs one measured call from unchanged original prepared input. R runs
an explicit full GC after dropping warm-up output and before baseline sampling;
that policy is reported. R's internal GC, INFO logging and retained count.data
remain normal call costs. Rust uses cnv::run without a snapshotting observer,
retaining normal validation/allocation. Do not time exports, hashes, expected
matrix loading or result validation. Keep the measured return alive until after
timing and validate/export it afterwards. Verify the original input unchanged,
including R's expression, count data, gene order, ordered group maps and hidden
spike slot, not merely the primary expression matrix.

Set cutoff 1, detection minimum 3, bounds references in the accepted order,
window 101 and clamp 3. Set inferCNV num_threads=1 and native-library thread
environment limits to 1 before startup. Explicitly disable reference regrouping,
scaling, HMM, denoising, outlier pruning, masking and end trimming. Use samples
analysis and inspect_subclusters=FALSE. Disable resume, all RDS/final-RDS saves,
plots/preliminary plots, expression/phylogeny exports and diagnostics; stop at
14. Preserve complete arguments and actual runtime/thread configuration.

## Metrics and result contract

Rust region wall time uses checked Instant subtraction. Region user/system CPU
comes from checked getrusage deltas with microsecond conversion. R snapshots
unclass(proc.time()) and explicitly reads user.self, sys.self and elapsed;
record child counters separately and reject unexpected child work. Do not sum
child CPU into self CPU. R's documented timer precision differs, and the exact
4.6.1 elapsed-clock backend has not been verified; do not describe it as the
same monotonic clock as Rust or manufacture precision. Negative/nonfinite
durations invalidate a trial, not become zero.

Read current RSS from Linux /proc/self/smaps_rollup after warm-up cleanup and
before timing. Require a unique Rss field, kB units and checked KiB-to-byte
conversion; missing/invalid/overflowing counters fail. Record whole-process
peak RSS and whole-process wall/user/system time with external GNU time around
each direct executable, preserving tool version and raw output. These lifetime
metrics include startup, preparation, warm-up, result validation and exit.
Never subtract a current-RSS baseline from a high-water mark and call the
result numerical-kernel memory.

Each trial writes metrics and final expression/identity exports under its own
new directory; never overwrite previous trials. Preserve failures and partials.
The driver validates every final entry against the pinned expected stage 14
using the unchanged 1e-12 + 1e-12*abs(expected) bound and exact identities. Record
per-trial output hashes, maximum absolute/scaled errors and raw stdout/stderr.
Both implementations must pass; a fast failed or incorrect run is not a sample.

## Execution and acceptance

Run a smoke pair first, excluded from statistics. Then run seven fresh-process
trials per implementation, alternating which implementation starts each pair.
Run sequentially on one native host without other task-owned heavy work.
Preserve actual order, all individual region/lifetime metrics, medians, ranges
and speed ratio; no timing assertions in unit tests. Record machine/CPU/OS,
load, compiler/profile/flags, binary/source/input hashes, R/infercnv/package/BLAS
versions, configuration, environment limits and effective thread settings.

Only claim clear elapsed-time superiority on this bounded example if every
measured Rust region time is below every R region time. If distributions
overlap, report the measurements without a speed claim and investigate before
performance acceptance. Resource-use differences remain separately reported
with their actual scope. Do not generalize a result to the complete workflow.

Tests must cover single-stage loading, RSS/unit/overflow/counter failures,
negative timing deltas, missing output/nonzero subprocess, wrong final state,
wrong pins, missing/duplicate/insufficient trials and smoke exclusion. Verify
new support on all four native target classes with existing conformance intact;
then independently review source, run the real same-host benchmark, inspect
original evidence and commit the actual result. No public release follows
automatically from a favorable measurement.
