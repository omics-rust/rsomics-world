# SDD ledger — plan: docs/plans/2026-09-26-infercnv-native-core-plan.md

Spec: `docs/plans/2026-09-26-infercnv-native-core-design.md`.
The accepted oracle receipt and its independent numeric review are committed
at world `156694bb765dcaa612b757e3158486d470930253`; exact-head Control plane
run `36217358283` passed. The new spec had an independent source-informed
architecture review with no material gap, then added explicit standard type
traits to support its comparison/error tests.

## Preflight

| Task/interface | Producer and consumer | Check |
|---|---|---|
| Task 1 internal model → kernels | Checked prepared counts and immutable identity views → private numerical functions | Column-major orientation, chromosome ordering, mutable-buffer privacy and stage-1 scope are explicit |
| Task 1 kernels → pipeline | Filtered/aligned finite working state → ordered borrowed stage observer | Error propagation, no retained trace matrices, and raw immutability are tested |
| Task 1 source → controller CI | Frozen test-first/green files and hashes → native snapshot workflow | Red must precede implementation; remote-generated lockfile returns before locked green runs |
| Task 1 own text | Real unpublished library and tests → exact profile/stage comparisons | No raw-import, full-CNV, public CLI, publication or speedup claim; strict four-native gate remains |

Ruling: Work on local product `main` and control-plane `main` — the user's
operating manual explicitly requires it instead of the skill's default
worktree — errors would require ordinary corrective commits, not rewriting
unrelated history.

Ruling: Keep the implementation's test-first synchronization with the controller,
not the user — local compilation is prohibited and the user delegated routine
decisions — a mistaken red classification would invalidate TDD evidence and
must be corrected before accepting green.

Ruling: Preserve durable state and skill scratch, with no cleanup/deletion —
the user requires careful preservation and `.autopilot/state` recovery — the
cost is a small amount of external-disk storage.

## Execution

Task 1 implementer: `/root/infercnv_native_core_impl`, fresh isolated context.
Only product source and its report belong to that agent; the controller owns
CI, source snapshots and Git. The local product repository was initialized on
`main`, with no commit or remote; its first test-only source is being prepared.
World plan head `30d26a4ae3ef9b2a9468f702eb664826be2ddc1c` passed Control plane
run `36217732909`. Boot APFS remains above 80%, so no local Cargo build, test,
or dependency resolution is permitted.

The new candidate workflow passed YAML/Bash parsing and independent read-only
CI review. It uses Linux-only expected-red and four-native green matrices,
verifies immutable archive/source hashes, preserves lockfiles and raw errors,
and rechecks source after tests. Actions validation and actual native execution
are still pending; no source snapshot has been accepted yet.

## Test-first snapshot

Controller workflow head `8072435876e81fa3f64f2cf069d48564a9f7692c` passed
Control plane run `36217924960`. The implementer reported `RED_READY` with
13 contract tests and one four-profile / ten-stage golden loop; `src/cnv`
remains absent. The controller independently verified all 120 fixture hashes
and original oracle bytes, and every one of the 128 archived source files.

Frozen snapshot: `.autopilot/snapshots/sc-cnv-core-2026-09-26/red-1/`.
Commit `4e8c25c7c475c855f7175a954c5bf2731a45cc74` passed Control plane
run `36218065460`. Native expected-red run `36218066698` is dispatched;
its actual result, dependency lockfile, and failure classification are pending.

## Actual red accepted

Run `36218066698` completed with the intended Rust E0583 at `src/lib.rs:3`,
missing module `cnv`. Dependency compilation and path/metadata checks passed;
only the debug test step failed. Both source integrity checks passed all
128 entries, and raw artifact upload succeeded. This is a genuine absent-core
compile failure, not an executed assertion failure or an environment error.

Preserved evidence directory:
`/Volumes/Zane's HDD/rsomics-fixtures/evidence/infercnv-native-core-2026-09-26/run-36218066698/`.
It contains terminal run/jobs/artifacts JSON, original artifact/log ZIPs, and
the extracted artifact. The controller verified CRCs, API digest, extracted
bytes, frozen source archive, and published-common checksum in Cargo.lock.

| Artifact | SHA-256 |
|---|---|
| Artifact ZIP | `f892ea8eb2b5d787d69e60d72d15b97cbfabf286fcb41405549f459a675027b3` |
| Run logs ZIP | `d747330d1ceb1726f96ba86d38fad6e97fd74793011196b22b5cfbffb5b62db9` |
| Remote-generated Cargo.lock | `636ead1961f573b303be8c02f6068201acd11dde1d0cb9bab3d81ccf86bf5554` |

The lockfile was copied byte-for-byte into the product; core implementation
is now authorized. Independent test-only review found two false-green gaps
(unchecked lengths around zip assertions and unchecked returned identities),
plus missing non-square accessor coverage. These were sent to the implementer
for correction before the green candidate. Golden tolerances remain unchanged.
