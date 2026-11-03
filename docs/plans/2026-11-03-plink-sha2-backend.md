# PLINK maintained SHA-256 backend candidate plan

> **For agentic workers:** One bounded dependency-maintenance task, then one fresh whole-feature review. User unattended/direct-main/external-disk/minimal-test rules override approval, worktree and reviewer ladders.

**Goal:** Evaluate the maintained RustCrypto backend in the real durable genotype workflow without changing file identity or durability contracts.

**Architecture:** Pin existing sha2 dependency to0.11.1 without unused default features. Keep the existing incremental SHA-256 reader and64KiB buffer. No new crate, wrapper, parallel worker, custom crypto or forced CPU feature.

**Tech Stack:** Existing Rust1.91; sha2-0.11.1's declared MSRV1.88 fits. Exact sources are already cached externally; resolve/build offline initially.

**Spec:** AGENTS.md; .autopilot/state/plink-arena-compaction-2026-11-03.md; prior full-I-w1 phase evidence. Primary upstream [manifest](https://raw.githubusercontent.com/RustCrypto/hashes/sha2-v0.11.1/sha2/Cargo.toml) and [runtime dispatch](https://raw.githubusercontent.com/RustCrypto/hashes/sha2-v0.11.1/sha2/src/sha256.rs).

## Constraints and rationale

- Clean private base270caf52d2aa87375cfd1ab5c6ef8d0ca2acf29c. Existing green15 four-native/source and12-call arena observations are the baseline; do not rerun unchanged baseline suites.
- Allowed owner files: Cargo.toml, generated Cargo.lock, and src/io/transaction.rs only if required for source-compatible Digest/output-format API migration. No other production, schema, version, CLI, foundation, codec, validation, worker or atomic-I/O changes.
- Use `sha2 = { version = "=0.11.1", default-features = false }`. Existing caller uses only incremental hashing/formatting, not OIDs or alloc-specific SHA APIs. Exact pin avoids relying on the upstream patch-MSRV policy.
- New default runtime dispatch includes AArch64 SHA2 and x86 SHA-NI with software fallback; software implementation also differs from locked0.10.9. This is a concrete candidate, not proof of faster hashing or workflow performance. Read actual cached upstream source before edits.
- Do not introduce the archived sha2-asm library, native target flags, unsafe product code, hidden parallelism, digest algorithm changes, chronological PGEN stream hashes or fewer file/directory syncs.
- All project/Cargo/Rustup/resolved-target/cache/tmp paths physically external. Existing bash guard and Rust1.91,2 build jobs; no default-home writes or toolchain fallback. Initial dependency resolution/build must be offline; missing cache stops only that action for bounded inventory, not an automatic network retry.
- Preserve the existing manifest's64-character lowercase SHA-256 values and complete output bytes. Existing full CLI/source fixtures and transaction/tamper consumers prove the contract; do not add a scalar hash-helper matrix or a source-text assertion about dependency selection.

## Task 1: private dependency candidate

**Interfaces:** Keep transaction::sha(&Path) -> Result<String>, Manifest and commit/verify APIs unchanged. Cargo.lock changes only the SHA dependency closure or strictly required shared constraints; report any other package movement before broadening scope.

- [x] Confirm clean base and read Cargo/transaction callers plus cached0.11.1 manifest/backend/changelog. Confirm actual package/transitive MSRVs are compatible with1.91.
- [x] Apply the exact dependency pin/feature choice and update only its lock closure with guarded offline Cargo resolution. Preserve any missing-cache/resolution failure; no invented RED or lockfile-source test. Desired public behavior is unchanged, so existing green baseline plus final consumer verification is appropriate for this maintenance candidate.
- [x] Compile and fix only necessary Digest/output-format API migration in transaction.rs if encountered. Preserve64KiB streaming read, I/O error propagation and exact lowercase digest formatting. No additional abstraction or performance claim.
- [x] Run guarded fmt, strict all-target/all-feature Clippy and complete debug/release suites; expect existing17-library+5-workflow executions with zero failed/ignored and all74878 raw cells. Use the unchanged real consumers, no new campaign or extra tests without a concrete regression.
- [x] Freeze exact delta, resolved dependency/version/license/MSRV notes and actual logs in a new external packet<=128MiB. Exclude mutable .remember application sidecars from immutable evidence pins; preserve them. No stage/commit/push/publish/SSH/benchmark by implementer.

## Review and later acceptance

One fresh whole-feature review covers dependency scope, actual runtime dispatch/fallback, Rust1.91 compatibility, private API adaptation, digest formatting and unchanged golden/transaction consumers. Root adjudicates private integration only after actual successful logs/source verification.

Native-four and same-file source qualification must consume the new immutable candidate before any separately admitted matched full-I-w1 delivery-binary measurement. No toy hash microbenchmark, rebuilt comparator, old-binary relabeling, throughput/RSS claim, negative-gate waiver or publication follows from a dependency upgrade alone. Retain or reverse the candidate according to actual evidence; do not chase unchanged reruns. The portfolio's30-family scope and other product holds are unchanged.

Private candidate integration is223e342f845bdb280bd369395483f8f72a1d445f after
one clean whole-feature review and root verification. Green16 preparation is
separate; new native/source/performance/publication gates remain pending.
