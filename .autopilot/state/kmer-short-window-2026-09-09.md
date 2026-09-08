# Canonical short-window repair checkpoint

Both guards are correctness-verified but not performance-approved. The
inlining hint made performance worse and was withdrawn. An equal-source
control is running before further structural changes. No release or secret
access change has been made.

## Owning repositories

- `rsomics-kmer`: clean at `2f2eda21729dc049524cfedb77b77ffdead41aaa`;
  production guard is `len < k || start > len - k` (commit `0ba84a6`).
  `0924135`'s inlining hint was withdrawn. Source, tests, Cargo files and
  benchmarks are identical to corrected control `23e42f3`.
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
| Inlining measurements | `34253569461` | Rejected: median candidate/corrected ratios 1.620417 Linux x86_64, 1.122348 Linux ARM, 1.209461 macOS ARM; Intel macOS still running |
| Inlining consumers | `34253574988` | Linux x86_64 sketch artifact upload failed after tests passed; Intel seq still running; experiment already rejected |
| Equal-source control | `34254249238` | Candidate and corrected reference source are identical after withdrawal; measurement running |
| Withdrawal-head ordinary CI | `34254245907` | Passed exact head `2f2eda2` on all four native targets, with lint/package/benchmark smoke |

Both completed four-platform measurement sets are retained outside scratch at
`/Volumes/Zane's HDD/rsomics-fixtures/evidence/kmer-short-window-2026-09-09/`,
in `first-guard` and `invariant-boundary`. First-run raw ZIPs independently
match GitHub digests and pass integrity checks. The dossier lists their hashes.
The three completed inlining platforms are also copied to `inlining`; add its
Intel macOS artifact when complete, without overwriting the existing platforms.
Scratch copies and downloaded library assembly remain under
`/Volumes/KIOXIA/Developments/tmp/kmer-repair-evidence-20260909-619Btb`.
Do not overwrite these with inlining measurements.

Current source SHA-256:

```text
39415b62c6a989ced3815f324e027c55751607889f0e88d1923754e66e35ae4b  src/hash.rs
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

Before another production optimization, finish the equal-source control and
review the iterator's internal representation. The three local optimization
attempts have not cleared the gate. Do not add more guard permutations or
`inline(always)` blindly. Any new internal window-count design needs its own
explicit invariants, existing consumer regressions and matched measurements;
no public API expansion is required.
