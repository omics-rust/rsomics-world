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
