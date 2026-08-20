# rsomics-index 0.1 release gate

## Current state

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
