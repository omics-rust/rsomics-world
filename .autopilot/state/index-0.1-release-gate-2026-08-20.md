# rsomics-index 0.1 release gate

## Current state — 2026-09-08

Publication remains held. The previous handoff below is historical, not the
current release decision.

- Product code head: `05960a4609a3b2acc388c0a149b5e023d53027f1`.
- Last reverified four-native CI: `33843770617`, successful for repository
  head `880e9dcaa259ca71a3a37f4b4554dadcaf0fa886`.
- Current repository head: `41b161a7dac7eb3700208f025a6be6005c917002`.
  CI `34247470721` passed native Linux and macOS on x86_64 and aarch64,
  including the Linux HTSlib 1.24 oracle. The new commits repair the physical-storage
  guard and withdraw the current publication claim; product Rust code is
  unchanged.
- Stable slice remains `bgzip` and `tabix build/query/list`, with 60 ordinary
  tests and nine pinned HTSlib 1.24 oracle groups.

The August benchmark was completed from clean build/harness revision
`821d491042a92d35153efcdf160acebf381ca4ee`, after the invalid stale-binary
run described below. Product `PERFORMANCE.md` records 13 workloads, ten
alternating pairs each, eight scoped throughput wins and lower median RSS in
all 13 lanes. Those are historical reported results: on September 8 the
recorded directory
`/Volumes/KIOXIA/Developments/tmp/rsomics-index-benchmark-20260820-821d491`
was absent. Bounded checks of external scratch, retired assets, fixtures and
recovery directories did not find it. The record does not establish when or
why it disappeared. No evidence or fixture was deleted in this review.

Restore and verify the original manifest and raw trials, or rerun the entire
gate. Do not convert the stored table or historical memory into fresh evidence.
Product `PERFORMANCE.md` now explicitly holds publication on this condition.

The storage preflight also needed repair. `df /` reported 47%, but
`diskutil info -plist /` reported a 245,107,195,904-byte APFS container with
approximately 14.3 GB free: about 94% physically occupied. The harness now
uses container size and free bytes, rejects occupancy at or above 80%, and
resolves all configured Cargo/scratch paths before checking their volume.
Its isolated read-only preflight exits 2 on the present machine before any
build can start. Shell syntax and whitespace checks pass. Local builds and
benchmarks are stopped; the Linux `4090` root is also 98% full, `/data3` is
99% full, and its installed bcftools is 1.13 rather than the pinned oracle.

The organization secret metadata was re-read without accessing its value.
`CARGO_REGISTRY_TOKEN` is visible to 17 selected repositories; `rsomics-index`
is not one of them. The old user-supplied token was revoked, but the current
organization secret's validity is unknown. No publication was dispatched and
the allowlist was not changed. The last positive evidence of registry absence
is the August 20 API 404; do not describe that old response as a current check.

Resume by restoring a compliant build host and recoverable raw evidence,
reviewing the exact release source and API, and verifying exact-head CI.
Credential availability is a separate gate, not evidence that the product
itself is ready. All four target classes and the archive/install checks remain
mandatory before recording a live release.

## Historical August 20 handoff

The sections below record the earlier state before the completed benchmark
and the September recheck. They are retained for provenance only.

### Snapshot

- Product repository: `/Volumes/KIOXIA/Documents/omics-rust/rsomics-index`
- Code head: `05960a4609a3b2acc388c0a149b5e023d53027f1`
- Repository head: `821d491042a92d35153efcdf160acebf381ca4ee`
- Four-native exact-code-head CI: run `32331824268`, passed
- Four-native exact-repository-head CI: run `32340291429`, passed, including
  strict Clippy, debug and release tests, rustdoc, package verification, and
  the pinned HTSlib 1.24 compatibility suites
- Stable slice: `bgzip`; `tabix build`, `query`, and `list`
- Compatibility oracle: HTSlib 1.24

The release documentation now treats the `df8089c` benchmark as a historical
baseline because later revisions changed BGZF decompression and tabix query
algorithms.

The formal harness now owns the release build step and writes a sidecar binding
the clean Git head, release-binary SHA-256, `Cargo.lock` SHA-256, binary path,
toolchain, and build time. Smoke and formal runs refuse a missing, stale, dirty,
or checksum-mismatched build record and refuse to overwrite a nonempty evidence
directory. This prevents the stale-binary failure from recurring silently.

## Invalid measurement

A formal run was started from code head `05960a4`, but the configured release
binary had an earlier modification time and did not contain the current query
optimizations. The run was stopped as soon as this mismatch was identified.
Its incomplete evidence is retained at:

`/Volumes/KIOXIA/Developments/tmp/rsomics-index-benchmark-20260820-05960a4-invalid-stale-binary`

It is not release evidence and must not be summarized into `PERFORMANCE.md`.

## Local blocker

Before rebuilding the exact-head release binary, the required storage preflight
reported the Mac boot APFS container at 98.5% physical use and `/` at 82%, with
about 3.5 GiB available. Cargo work is stopped by the operating rule that the
boot disk must remain below 80%, even though Cargo home, target, and temporary
paths all resolve to KIOXIA.

The largest clearly disposable candidate is `~/Library/Caches` at about
7.6 GiB. No cache or user data has been deleted. KIOXIA had about 6.4 GiB free
after the invalid benchmark stopped.

## Resume sequence

1. Restore the boot disk below 80% without deleting project or session data.
2. Recheck `/` and KIOXIA capacity and resolve all Cargo paths.
3. Run the benchmark harness `build` mode from the exact repository head; it
   creates and verifies the release binary provenance record.
4. Start `run` only after the harness accepts the clean head, binary and lockfile
   hashes, pinned oracle versions, external paths, and storage preflight.
5. Run all 13 workloads with three warmups and ten alternating measured pairs
   into a new result directory.
6. Replace the historical performance decision in a new repository commit,
   then rerun package and exact-head four-native CI gates for that final head.
7. Publish only after a valid crates.io credential is available; the previous
   registry token is revoked.
