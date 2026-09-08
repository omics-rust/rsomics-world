# Canonical short-window repair checkpoint

Both guards are correctness-verified but not performance-approved. A single
inlining hint is running against the corrected and released controls. No
release or secret access change has been made.

## Owning repositories

- `rsomics-kmer`: clean at `1dcb6a71b7dcab8c8095b63d474e4fbd9e9686b5`;
  production guard is `len < k || start > len - k` (commit `0ba84a6`), with
  ordinary `#[inline]` added to `next` by `0924135`.
- `rsomics-sketch`: clean at
  `8bacc91b4ad8892f93e21adbf02211eeedbf7703`; still uses registry kmer 0.2.2.
  Its short-input oracle diagnostic deliberately expects the old panic.
- `rsomics-seq`: pinned consumer
  `d9734e51c4ed557f6d8790d97a686717ebc4769e`; no source/dependency edits.

The reviewed requirements, initial failure reproductions and raw performance
interpretation are in
`docs/01-foundations/kmer-consumer-review-2026-09-09.md`.

## Exact handles to resume

| Evidence | Run | Current meaning |
|---|---|---|
| First guard consumers | `34251307513`, attempt 2 | Validation passed; artifact-only failure persisted in the retried macOS ARM job; overall run remains failed |
| First guard measurements | `34251528752` | Complete; Linux x86_64 1.199778 and Intel macOS 1.148879 block release; ARM Linux 1.002289 and ARM macOS 0.999841 |
| Second guard ordinary CI | `34252184896` | Passed exact-head four-native CI |
| Second guard consumers | `34252192237` | Passed all eight consumer/platform jobs, including artifact retention |
| Second guard measurements | `34252186622` | Complete; Linux x86_64 1.161514, ARM Linux 0.966408, Intel macOS 1.098681, ARM macOS 1.071380; performance held |
| Library assembly diagnostic | `34252843366` | Linux artifacts succeeded; macOS uploads failed; not a timing run |
| Inlining measurements | `34253569461` | Pending six balanced triplets with original and corrected controls |
| Inlining consumers | `34253574988` | Pending all eight consumer/platform combinations |

Both completed four-platform measurement sets are retained outside scratch at
`/Volumes/Zane's HDD/rsomics-fixtures/evidence/kmer-short-window-2026-09-09/`,
in `first-guard` and `invariant-boundary`. First-run raw ZIPs independently
match GitHub digests and pass integrity checks. The dossier lists their hashes.
Scratch copies and downloaded library assembly remain under
`/Volumes/KIOXIA/Developments/tmp/kmer-repair-evidence-20260909-619Btb`.
Do not overwrite these with inlining measurements.

Current source SHA-256:

```text
b994315f1c2bfab5e56d5ed7c8ceb8c5605d7d9bad16814c1ed0a322a9c0d3a7  src/hash.rs
41e7c3f50bff2f5dadf8af8a9d3cbca8b0e89db8d663f10f5d602603b6154e44  tests/canonical_windows.rs
8963d0f43ebca99cbd40d63a57f399e9b70ac81c98e7ca06f5c2dbfa9d677cd1  .github/workflows/consumers.yml
5d4b247922ea13af2a5827912155fbf8247d2eca400fe52328bef2d7e3d8a494  .github/workflows/benchmark.yml
```

## Release and storage constraints

The sparse registry index still ends at kmer 0.2.2, not yanked, checksum
`e1254977d1eaf89b29e727b7ea552ec8bd4bd0740b45fa40ac943e93ffaf9ed4`.
The crates.io API returned an unauthenticated HTTP 403; this says nothing
about token validity. The organization secret currently selects 17 other
repositories and excludes both kmer and sketch. No secret value was read.

Local product compilation remains stopped by the physical APFS boot-container
gate. `df /` shows only the system volume and must not be used to waive it.
All source edits and downloaded evidence remain on external disks. No remote
4090 build was attempted.

Do not publish either package merely because tests pass. Close the measured
regression, finish consumer and exact release-head CI gates, publish the fixed
foundation, then bump the sketch minimum registry dependency and lockfile,
remove its expected-failure diagnostic, and verify its own release head.
