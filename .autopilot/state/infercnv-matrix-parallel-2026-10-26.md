# Matrix smoothing worker execution

Status: accepted column-parallel smoothing; private unpublished product.
The whole portfolio goal remains active. Follow
docs/plans/2026-10-26-infercnv-matrix-parallel.md.

Product base f8a10a09b80b4b8f280e23abd2efbb8a02b1a336 was clean on resume.
The CLI now passes its existing worker count to unpublished PreprocessConfig.
Triangular smoothing partitions complete cell columns and preserves each
gene's accumulation order. The same two matrices remain live; no new
numeric buffer or dependency. Other preprocessing stages remain serial.
The scoped dispatcher is product-local and also used by existing distances;
spawn/join/task errors propagate after joining all started workers.

Two added kernel tests cover serial/parallel bit equality and arithmetic
failure without replacing input. Existing config rejection adds zero workers.
The canonical CLI 1/4-worker test includes the complete expression matrix.
No old golden, threshold or oracle tolerance changes. This is an integrated
feature check, not another exhaustive per-helper RED campaign.

Product commit 7495e3eb59aecd9323c67c4a9f4626a387463ecd contains exactly the
168 source files frozen/tested by green-21. No code changes after review
or native verification. The following README acceptance update is docs-only.
Exact-head world controller 40307694168 passed at
38200aba8be7073a71d945a81707225585d2c47c. Linux correctness/benchmark run
40307961363/attempt1 passed on that head: 116 tests each in debug/release,
complete canonical CLI expression equality, formatting and strict Clippy.
The unchanged candidate's full four-native run 40309311228/attempt1 passes
on the same world head: Linux/macOS x86_64/aarch64, 116 tests per debug and
release profile, complete canonical expression equality at1/4 workers.
Linux formatting/strict Clippy also pass. Original API, logs and four ZIPs
are retained externally in evidence/infercnv-matrix-parallel-2026-10-26/
run-40309311228/. Each ZIP digest matches the API; each tested source
manifest matches ea37273bd4a98444b0692fbb2584be99abdcee6a304c9b5f34fe662e2fe99cf9.

Four-native artifacts and original ZIP SHA-256:

- Linux x86_64 12643509539:
  45a0727a67b75daf63d7108fb3213759ee7f8d7de5f4a43ddb832868bd9cc5df.
- Linux aarch64 12644236556:
  ade3310051bfc248dd11542dd7d5c55dcbe5bbaf013110481b1925035b83358b.
- macOS x86_64 12644911250:
  2cf76f0583c8d85c138e55cb31c7ef4206f23a4ef6ab9b6973eca8340bc21278.
- macOS aarch64 12644851028:
  e4e854c55c5d8d7c2f6427a90c966976f8bcd6ca154f0ce8eb03ecb4ec2c49a1.

Actual canonical fresh CLI measurements (plain input, warm page cache):
one worker 0.39,0.39,0.39,0.49,0.43s; four workers 0.30,0.33,0.30,0.31,0.40s.
Medians 0.39/0.31s, ratio 1.258065. Ranges still overlap, so strict wall
separation is false. This is a useful preliminary median benefit, not a
stable cross-host, larger-cohort or upstream replacement-speed conclusion.
Median maximum RSS 60,416,000/60,751,872 bytes (about 57.6/57.9MiB).
Four workers consume more CPU time (median user 0.52 vs 0.36s), as expected
for parallel execution; wall improvement is not a claim of less CPU work.

All 12 distinct process records exit zero with empty groups; actual time
files, report byte hashes, normalized report fingerprints, retained binary
hash and recomputed summaries agree. The report fingerprint also exactly
matches the prior accepted baseline:
198e3beb943e9e6947376a8b9200e06d8729a6c530287a8ce4c1980249fb868d.
Binary SHA 50dd80358e80f4e9b6d10a4828cdc12303f7d95d7ca09240ae9482c9da178320.
Native Linux x86_64/Rust1.91, four affinity-visible CPUs, AMD EPYC9V74;
quota unavailable, load 2.60/0.99/0.40 before and 2.48/0.99/0.41 after.
Artifact 12643727211, 4,590,525 bytes, SHA
146898b6c4425e398fad27240a3ff9cd32d2adbf833a082d57921132e49838c5.
Original run/artifact API, job log and ZIP are retained externally in
evidence/infercnv-matrix-parallel-2026-10-26/run-40307961363/.

One fresh-context whole-feature static review found no Critical, Important
or material minor issue. It verified complete-column partitioning, retained
arithmetic order, finite-check relocation, all-worker joining on failure,
unchanged numeric-buffer budget, config/CLI wiring and meaningful integrated
tests. Four-native runtime gates now pass; general speed claims remain
unestablished. Do not repeat helper reviews.
The checksum/extraction guard passed all 168 source files. Archive SHA-256:
6bf7e89885641660acbb5ad7995679a1f7913ad5bf5e56ad3b2131fa3e52888c.
Manifest SHA-256:
ea37273bd4a98444b0692fbb2584be99abdcee6a304c9b5f34fe662e2fe99cf9.

Storage check: boot APFS container 81.73% occupied; KIOXIA 264GiB free.
No local Rust/R build, installation or dependency download is allowed.
Only standalone external Rust 1.91 formatting and source serialization ran.
Preserve inherited VCF state and unrelated root scratch. No destructive,
registry, public-repository or automation action is introduced.

Decision: retain the simple product-local optimization and shared execution
plumbing. It preserves accepted outputs, exercises actual Rust workers, adds
no numeric matrices/dependencies and has a useful preliminary median benefit.
Do not treat overlapping small-case timings as a strict performance gate.
The 4,103-cell real cohort is now running on4090 under the same retained
binary; host load is high and explicitly recorded. Equivalent upstream
performance, denoise/HMM/reporting, full inferCNV and the remaining products
are still open. This bounded acceptance is not publication readiness;
publish=false and no remote remain intact.
