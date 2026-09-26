# Accepted native inferCNV pre-clustering core

Product: `/Volumes/KIOXIA/Documents/omics-rust/rsomics-sc`, local main commit
`13961bf6a42cb4cbbf36c528331aafb98a52937b`. The worktree is clean; there is
no remote, public repository, binary or registry publication. `publish=false`.

## Verified scope

Prepared, ordered stage-1 counts → filtering, depth normalization, log2,
reference subtraction, clipping, chromosome-local triangular smoothing,
global cell centering, repeated reference subtraction and fold change.
The checked column-major model preserves identities and immutable raw input;
observation callbacks borrow stage state and propagate errors. The numerical
workspace bound is checked before fallible working-buffer allocation.

Pinned upstream baseline: infercnv 1.28.0 source
`b421d9405c97a309b081ef86d455e976df93eae4`, accepted real-package oracle run
`36216764707`. Four reference profiles and ten checkpoints per profile are
compared in one native execution each, including every intermediate matrix
and final returned identities. No historical approximate kernel was reused.

## Acceptance evidence

Frozen source: `.autopilot/snapshots/sc-cnv-core-2026-09-26/green-3/`.
World head: `fc512a97f0d97b42099a096ff1533878cb14a556`.
Native run: https://github.com/omics-rust/rsomics-world/actions/runs/36219136430

| Native target | Debug | Release |
|---|---|---|
| Linux x86_64 | 30 passed | 30 passed |
| Linux aarch64 | 30 passed | 30 passed |
| macOS x86_64 | 30 passed | 30 passed |
| macOS aarch64 | 30 passed | 30 passed |

Each mode/target reported 40 checkpoint comparisons, maximum absolute delta
`7.105427357601002e-15`, with unchanged criterion
`abs(delta) <= 1e-12 + 1e-12*abs(expected)`. Formatting and strict Clippy passed.
Tests include exact subnormal-median and public depth regressions whose old
production behavior demonstrably failed in run `36218967725`; minimum-value
loss and stable-sort's hidden infallible allocation were fixed before acceptance.

Original API metadata, four artifact ZIPs, complete logs and extracted files
are retained in `/Volumes/Zane's HDD/rsomics-fixtures/evidence/infercnv-native-core-2026-09-26/run-36219136430/`.
The execution ledger records all earlier red/green runs, reviews, hashes,
unchanged lockfile, source integrity and evidence-verification checks:
`infercnv-native-core-2026-09-26.md`.

## Still required

This is not raw-input end-to-end compatibility, full inferCNV, biological
validation or a performance/release claim. Shipped real-example conformance,
representative speed/RSS evidence, persisted/raw-state integration, clustering,
HMM/Bayesian policies, denoising, output reports and the unified rsomics-help
CLI remain open. Continue with
`docs/plans/2026-09-26-infercnv-shipped-data-plan.md`; no new foundation is needed.
