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

## First native candidate

The implementer froze `green-1` with seven focused CNV modules, 14 public
contract tests, eight private kernel tests, and one four-profile golden test.
The test-audit changes are present; formatting passed without local Cargo.
The controller verified all 136 archive files against both source and manifest.
The remote lockfile remains unchanged.

Frozen candidate commit `cf55b05589719373890d9189e424d56fd4b0b954` dispatched
run `36218401374` for Linux/macOS x86_64/aarch64 debug/release validation and
strict static checks. Acceptance is pending. Independent task review by
`/root/infercnv_native_task_review` is running alongside CI. The implementer
flagged missing explicit arithmetic-overflow witness coverage; checked
arithmetic exists, but this is not yet accepted as sufficient testing.

Related control-plane heads: `0624b8d422c7dca30d73ec042825a2dc221e5d06`
passed run `36218177756`; shipped-data preflight head
`64b3387b53034018c3e9fc80c57c46cab5e39d31` passed run `36218341241`.

### green-1 result and review

Run `36218401374` completed at the frozen head. All four native targets passed
23 tests in both debug and release, including all 40 stage comparisons each.
Every target/mode reported maximum absolute delta
`7.105427357601002e-15`. Formatting passed, but Linux x86_64 strict Clippy
failed two `manual_is_multiple_of` diagnostics in `config.rs` and
`normalize.rs`. The candidate is not accepted.

Original run/jobs/artifact JSON, four artifact ZIPs and extracted files, and
the run-log ZIP are preserved under external fixtures in
`evidence/infercnv-native-core-2026-09-26/run-36218401374/`. The controller
verified all API digests, ZIP CRCs, extracted bytes, frozen source archives,
136 before/after source hashes per target, test counts and per-stage reports.
The run-log ZIP SHA-256 is
`3915199c77378ef6f21c78be6e65e342d8a1bf674515974cea8bae08285ec53f`.

| Target artifact | ZIP SHA-256 |
|---|---|
| Linux x86_64 | `62aa7e22038d1f8f4a053915e1bd3cd9ae0475d8597f1de776e8bb6e5b8ea341` |
| Linux aarch64 | `d4da409423bc3659af42bd253b31c9f65f67ca2079579c0268eedb5a067232ab` |
| macOS x86_64 | `ded8de36ac0e5734fd59779bdd81655029717d1cd3fdec69506ebda92106fc49` |
| macOS aarch64 | `6c71e115cd2816dcb366bdc010e2720121faf422f49e996909699bd94462c716` |

Task review found no clear numerical/core-contract defect, but required three
missing independent witnesses: successful post-filter depth normalization,
budget arithmetic overflow via a private helper, and the 100-gene linear
smoothing case. Fix round 1/5 bundles these with the two observed Clippy
diagnostics; the original implementer owns all product changes. Golden
tolerances and dependency lock remain fixed. Control plane run `36218400071`
passed for `cf55b05589719373890d9189e424d56fd4b0b954`.

### green-2 verified; task fix review clean

Frozen `green-2`, world head `b9c5f33fb276e63f196f17f7d0c3ba4bda89973c`,
changes only the four paths reviewed in fix round 1. Run `36218605570`
completed success on all four native targets. Each passed 27 tests in debug
and release (11 private, 15 public contract, one four-profile golden), with
40 checkpoint reports per mode and maximum absolute delta
`7.105427357601002e-15`. Format and strict Clippy passed without diagnostics.
Control plane run `36218603731` also passed at that exact head.

Preserved directory:
`/Volumes/Zane's HDD/rsomics-fixtures/evidence/infercnv-native-core-2026-09-26/run-36218605570/`.
The controller verified terminal head/jobs, every API artifact digest and ZIP
CRC, extracted bytes, frozen archives, lockfile identity, all 136 before/after
source hashes per target, actual test counts, 40 deltas per mode, and clean
static logs. Original full logs ZIP SHA-256:
`573aa0804e13210d4d64921756f1049781e5c71b836b1a1755344d32cc55f7f9`.

| Target artifact | ZIP SHA-256 |
|---|---|
| Linux x86_64 | `3164ff62c894c94e107529556aa830f2b21a4c1ebfe19800d55508061ef43097` |
| Linux aarch64 | `8f87e07c4b8cb2eee7e203938b11bd7baaf98c665689c212a4b01613fc2e106b` |
| macOS x86_64 | `8a8608fe6a7ae3b45c99c5987d6ea3a0b7d5f887c185b1e71adb85d1dfa235ab` |
| macOS aarch64 | `da5a96f237472ebe76d74965905029de29929888fe69d2b400ee72ff6aedd2f3` |

Task 1: fix round 1/5 (all three review findings and both CI diagnostics
addressed, zero open; snapshots green-1..green-2). Scoped independent review
confirmed the new expected values and overflow paths, read the Linux debug,
release, Clippy and formatting evidence, and found no new breakage. Its
remaining cross-platform/provenance verification item is resolved by the
controller's checks above. A fresh whole-change review is still required
before the product's first local commit and final acceptance.

## Final whole-change review: two important findings

Fresh reviewer `/root/infercnv_native_final_review` found two concrete gaps
despite green-2's passing ordinary-input tests. Even median uses `a/2+b/2`,
which turns two smallest positive subnormal values into zero; this can silently
zero a valid input's stage-3 depths. Stable `sort_by` also allocates hidden
infallible numeric scratch, bypassing the promised fallible allocation path.
The latter is not a demonstrated violation of the conservative byte ceiling.

The single final fix wave first adds exact-equality private/public subnormal
regressions with production behavior unchanged. The controller will capture
real assertion failures before authorizing safe midpoint and allocation-free
sorting changes. Numerical tolerances will not mask the subnormal defect.
Expected-red workflows now generate Cargo.lock only if absent, preserving the
already pinned lock in regression-red snapshots.

Reviewer exclusions accepted: raw-state/downstream/CLI work and performance
remain explicit future gates; the accepted upstream oracle is not re-audited
in this implementation review; string/observer allocation is outside the
numeric-workspace contract. None is silently promoted to a completed result.
