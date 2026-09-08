# Canonical short-window repair checkpoint

The production guard is correctness-verified; the rejected inlining hint stays
withdrawn. Microbenchmark attribution remains inconclusive. Actual sketch
measurements now retain all four genome jobs and three full FASTQ jobs;
only Intel macOS FASTQ remains live. The completed workloads show substantial
memory advantages but mixed throughput against sourmash. No whole-gate
performance pass, release or secret access change has been made.

## Owning repositories

- `rsomics-kmer`: clean at `1a0d9aac5307644fd6d9cdaab20c648bd9193984`;
  production guard is `len < k || start > len - k` (commit `0ba84a6`).
  `0924135`'s inlining hint was withdrawn. Source, tests, Cargo files and
  benchmarks are identical to corrected control `23e42f3`.
- `rsomics-sketch`: clean at
  `f430522bb3d3c6fd38e08af758dca11e5262f47b`; still uses registry kmer 0.2.2.
  Its ordinary CI short-input diagnostic deliberately expects the old panic.
  Scale recovery and collection comparison repairs passed exact-head CI
  `34268416605`; see `docs/10-products/sketch-contract-review-2026-09-09.md`.
  Product measurements still pin the earlier head `3802b1a`, patch the
  foundation candidate only in isolated runner Cargo
  homes and test the archived, fixed-dependency binary against the full oracle.
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
| Inlining measurements | `34253569461` | Complete; rejected: median candidate/corrected ratios 1.620417 Linux x86_64, 1.122348 Linux ARM, 1.209461 macOS ARM, 0.939991 Intel macOS |
| Inlining consumers | `34253574988` | Complete failure: Linux x86_64 sketch artifact upload failed after tests passed; experiment already rejected |
| Equal-source control | `34254249238` | Complete; three retained platform artifacts; macOS ARM upload failed. Identical source and x86 library/caller assembly still yield per-pair differences of about 17% |
| Withdrawal-head ordinary CI | `34254245907` | Passed exact head `2f2eda2` on all four native targets, with lint/package/benchmark smoke |
| Same-process null control | `34256249038` | Passed all four native jobs; 60 measured triplets each. Same-source candidate/reference median ratios: Linux x86_64 0.906807, Linux ARM 1.002371, Intel macOS 0.924302, macOS ARM 1.014703. Not a performance pass |
| Interleaved-head ordinary CI | `34256246005` | Passed exact head `3d4b751` on all four native targets, with lint/package/benchmark smoke |
| Shared-scan control | `34257486430` | Passed all four native jobs at `61de048`; compiled one common scan and iterator-state address, but null ratios remain Linux x86_64 0.789447, Linux ARM 1.009003, Intel macOS 0.859599, macOS ARM 1.007179. Attribution remains inconclusive |
| Shared-scan-head ordinary CI | `34257483815` | Passed exact head `61de048` on all four native targets, with lint/package/benchmark smoke |
| Current-head ordinary CI | `34269915993` | Passed exact head `1a0d9aa` on all four native targets, with lint/package/benchmark smoke |
| Repaired-product consumers | `34269918433` | Passed all eight jobs at kmer `1a0d9aa`, pinning sketch `f430522` and seq `d9734e5`; full four-test sketch oracle passes in debug and release on every platform |
| Sketch recorder ordinary CI | `34261359503` | Passed exact sketch head `3802b1a`: four native test jobs and lint/package/benchmark smoke; ordinary short-input test still documents the registry failure |
| Sketch product measurements | `34261423289` | All four genome and three FASTQ jobs passed and artifacts retained; Intel macOS FASTQ job `102180058887` still running. Exact sketch head `3802b1a`; no whole-gate performance decision yet |
| Sketch scale recovery | `34266897367`, `34267432784` | Four-native expected failure at `63abafa`, then normal CI pass at `db59472`; source repair, not registry delivery |
| Sketch collection scale | `34267814268`, `34268416605` | Four-native expected failure at `a19e817`, then normal CI and live mixed-scale oracle pass at `f430522`; source repair, not registry delivery |

Both completed four-platform measurement sets are retained outside scratch at
`/Volumes/Zane's HDD/rsomics-fixtures/evidence/kmer-short-window-2026-09-09/`,
in `first-guard` and `invariant-boundary`. First-run raw ZIPs independently
match GitHub digests and pass integrity checks. The dossier lists their hashes.
All four inlining platforms are copied to `inlining`; the three available
equal-source artifacts are in `equal-source`, and the new four-platform
same-process controls are in `interleaved-control` and `shared-scan-control`.
All copies were compared recursively. These retain native binaries, generated harness source,
manifest, lockfile, metadata, fixture, provenance and raw CSV observations.
Scratch copies and downloaded library assembly remain under
`/Volumes/KIOXIA/Developments/tmp/kmer-repair-evidence-20260909-619Btb`.
Do not overwrite any of these experiment directories.

Current source SHA-256:

```text
39415b62c6a989ced3815f324e027c55751607889f0e88d1923754e66e35ae4b  src/hash.rs
41e7c3f50bff2f5dadf8af8a9d3cbca8b0e89db8d663f10f5d602603b6154e44  tests/canonical_windows.rs
a18b40f5addda72a5ccc9c3b08a7ecef627654e29978a329d151f63c6e7b4f88  .github/workflows/consumers.yml
c16dbcf5c76f99cac1315910582a0bc0b21667a6519a1a6f3aece34796594577  .github/workflows/benchmark.yml
8b32ba71f8426bc8722226e3cff7a0070841b415cfb6fe8c898811e915175aec  .github/benchmarks/compare-hashes.rs
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

Do not publish either package merely because tests pass. Close the current
consumer performance gate, finish consumer and exact release-head CI gates, publish the fixed
foundation, then bump the sketch minimum registry dependency and lockfile,
remove its expected-failure diagnostic, and verify its own release head.

## Next bounded work

The consumer refresh at `1a0d9aa` changes only the workflow's sketch pin and
adds the release-mode full oracle command. Production sources, tests, benches
and Cargo files remain identical to `61de048`. All eight artifact ZIPs match
GitHub SHA-256 digests and pass integrity checks. Their metadata, lockfiles
and Cargo configurations resolve exactly one kmer package at `1a0d9aa`.
Raw logs show all four sketch oracle tests executing successfully in both
profiles, not merely ignored during ordinary tests. Sequence retains its
independent k-mer oracle and pinned SeqKit contracts.

Artifacts, exact-head CI metadata and eight raw job logs are preserved at
`/Volumes/Zane's HDD/rsomics-fixtures/evidence/kmer-short-window-2026-09-09/consumers-34269918433/`.
The copy was recursively compared with scratch
`/Volumes/KIOXIA/Developments/tmp/kmer-consumers-34269918433-Yvr8UM/`.
This closes the current source-head consumer refresh, not the future release
head or registry-adoption gates.

Stop the guard and minor microbenchmark permutations. Both independent review
and executable inspection confirm that the latest diagnostic shares one scan
and iterator-state address, but library/jump-table and hasher-buffer addresses
remain distinct. The guard's causal timing effect is not established.

The real sketch product refresh is implemented and dispatched. All local
fixtures were independently hash-checked, with live NCBI/ENA source metadata
checked. The 4,699,745-byte E. coli FASTA and 87,439,836-byte gzip FASTQ are
complete historical inputs, not downsampled replacements. A read-only strict
FASTQ scan verified 6,282,141 records, 634,496,241 bases and uniformly 101-base
reads; there are no shorter-than-k31 reads in this timing workload. The separate
full candidate oracle retains mixed/all-short FASTA and FASTQ regressions.

The recorder is in sketch `.github/benchmarks/`; eight local recorder tests,
Ruff, Rustfmt and YAML/Bash syntax checks passed without local Rust compilation.
Independent review caught and resolved asymmetric test-build feature unification:
all three measured binaries are now archived immediately after ordinary release
builds, before test builds. Normalized external package identities and actual
compiler-artifact feature sets are compared. The live oracle explicitly uses
`RSOMICS_SKETCH_BIN` to execute the archived candidate, whose digest is checked
again afterward. That recorder commit did not change production source or
public manifests; subsequent product-local repairs have their own CI evidence.

Next: finish Intel macOS FASTQ job `102180058887`; investigate any actual
failure, then download its new artifact to external scratch, verify it, and
preserve a verified copy outside scratch. Review paired time/RSS distributions
and full output equality before a whole-gate performance decision. Neither
dispatch nor the recorder's `complete.json` is a performance or release pass.
Old August numbers remain historical only.

Ordinary CI has now passed. All four genome jobs also passed, including the
archived candidate's full three-test oracle and checksum. The four original
ZIPs independently match GitHub digests and pass integrity checks; all 320 raw
trials, complete output bytes, 12 product binaries, lockfile hashes and actual
compiled dependency features were checked. Genome candidate/sourmash paired
median wall ratios are 1.110640 Linux x86_64, 0.515002 Linux aarch64, 1.095327
Intel macOS and 0.704724 ARM macOS. Peak-RSS ratios range from 0.058394 to
0.063158. This supports a scoped memory advantage but not universal speedup.
Only Intel macOS FASTQ abundance is still running; do not release from this
partial workload result.

Genome evidence and completed job logs are preserved and recursively compared
under the permanent evidence root's `sketch-construction-34261423289/` directory.
Scratch is `/Volumes/KIOXIA/Developments/tmp/sketch-construction-34261423289-mjkyFe`.
The dossier records all seven retained artifact IDs/digests. Collect only the
new Intel macOS FASTQ artifact into fresh child paths, preserving existing evidence.

ARM FASTQ verification covers 36 raw trials per platform, eight measured
four-tool rounds, exact dependency/build-feature identities, binary/lock hashes,
complete output equality and raw resource values. Candidate/sourmash median
time and RSS ratios are 0.885291 / 0.082669 on Linux ARM and 1.360956 / 0.187212
on macOS ARM. Candidate/published-baseline time ratios are 1.033869 and 1.014050;
candidate/same-source-reference ratios are 1.000000 and 1.003284. These scoped
differences do not establish a guard-specific causal effect. Both ARM ZIPs and
completed job logs were copied into the permanent evidence directory and
recursively compared with scratch.

Linux x86_64 FASTQ artifact `10072798104`, SHA-256
`29e68c60307ab1af1e8ae680ec3eb724407f19f8b326071ff25cf1598a306020`,
has also been verified and permanently retained. Its candidate/sourmash paired
median time and RSS ratios are 1.864916 / 0.090191; it is slower in all eight
pairs but uses materially less memory. Candidate/baseline time is 0.699100 and
candidate/same-source-reference time is 0.966135. The latter's persistent
difference precludes a guard-specific attribution. Full output, build, raw
resource and cross-platform signature checks passed independently.

A fresh independent API/hot-path review found no correctness or API blocker
for a patch-compatible foundation repair; it does not approve performance or
publication. Live secret metadata still selects 17 repositories and excludes
kmer/sketch. Older successful publish runs precede the secret's latest update
and cannot prove it remains valid. No credential or access setting was changed.

The kmer `benchmark.yml` deliberately pins dependency `3d4b751` as an equal-source
microbenchmark control. The separate sketch workflow pins corrected reference
`23e42f3`, current candidate `61de048`, published registry kmer 0.2.2 and sourmash
4.9.4. Neither workflow automatically follows a future production HEAD.
Do not reuse the microbenchmark's successful status as a release gate.
No public API expansion or additional production optimization is needed to
start the consumer performance refresh.
