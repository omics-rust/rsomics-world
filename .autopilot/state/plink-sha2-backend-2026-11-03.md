# PLINK maintained SHA-256 dependency candidate

Implementation admission only; no new candidate, throughput/RSS improvement,
native/source qualification or publication is claimed here.
Plan: `docs/plans/2026-11-03-plink-sha2-backend.md`.

## Baseline and reason

Private base270caf52d2aa87375cfd1ab5c6ef8d0ca2acf29c is green15 native/source
qualified. Its complete paired comparison has no material cohort RSS advantage
over7cee; do not pursue further capacity-only changes as an RSS fix. The older
e210 one-off attribution identifies output hashing as a substantial coarse
interval, not a current precise crypto-only cost. Preserve SHA-256 and every
durability guarantee rather than deleting useful work to meet a timing target.

Root reads cached sha2-0.11.1 manifest/changelog/default runtime dispatch and
software-unroll source, and verifies the official tagged
[manifest](https://raw.githubusercontent.com/RustCrypto/hashes/sha2-v0.11.1/sha2/Cargo.toml)
and [dispatch](https://raw.githubusercontent.com/RustCrypto/hashes/sha2-v0.11.1/sha2/src/sha256.rs).
MSRV1.88 fits product Rust1.91. Default runtime CPU checks/hardware/software
backends differ from0.10.9; no CPU feature is forced and no speedup is inferred.
SHA-512's AVX2 backend is not an x86 SHA-256 acceleration claim.

Fresh public sparse read under the existing external guard confirms exact
sha2-0.11.1 is not yanked, rust_version1.88, checksum
47d7069beb7d6ac7b9acd1039986e73443f24234f41074da099d6f994ac9ad19.
Only metadata is read; no dependency archive, credentials or shared config is
downloaded/copied/changed. Cached sources are already external.

## Bounded candidate scope

Pin sha2 exactly0.11.1 with unused default features disabled; update its lock
closure offline. Only Cargo.toml/Cargo.lock may change, plus transaction.rs
solely if necessary for the same incremental Digest/lowercase output API.
No custom crypto, archived sha2-asm, unsafe product code, schema/algorithm/
64KiB-read/worker/codec/atomic-I/O changes or new public component.

Accepted green15 consumers are the unchanged baseline; do not rerun them merely
to manufacture a RED. Desired behavior is identical, so do not add tests of
manifest dependency text or a scalar helper matrix. Final existing golden,
tamper, transaction and full CLI consumers must pass with the actual candidate.
Run external guarded fmt/strict Clippy/full debug+release, no network retry,
compiler fallback, SSH, performance campaign, stage/commit/publish by producer.
Initial missing-cache/resolution failures stop that affected action for bounded
inventory, not an automatic online fetch. Report unrelated lock movement.

Retain a new external evidence packet<=128MiB plus8GiB headroom; all cache,
Cargo/Rustup/resolved target/tmp physically on KIOXIA. Preserve but exclude
mutable .remember application state from immutable evidence. Root performs
one fresh whole-feature review, adjudicates private integration, and separately
qualifies immutable native/source then real-workflow measurements before any
performance/release claim. Existing negative gates and BED input-policy holds
remain unchanged.
