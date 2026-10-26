# Matrix smoothing worker execution

Status: implementation candidate, not accepted, measured or published.
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

Candidate green-21 is pending remote Linux correctness and whole-process
measurement. Product code remains uncommitted until useful evidence, one
overall review and four-native debug/release/strict-Clippy acceptance.
The checksum/extraction guard passed all 168 source files. Archive SHA-256:
6bf7e89885641660acbb5ad7995679a1f7913ad5bf5e56ad3b2131fa3e52888c.
Manifest SHA-256:
ea37273bd4a98444b0692fbb2584be99abdcee6a304c9b5f34fe662e2fe99cf9.

Storage check: boot APFS container 81.73% occupied; KIOXIA 264GiB free.
No local Rust/R build, installation or dependency download is allowed.
Only standalone external Rust 1.91 formatting and source serialization ran.
Preserve inherited VCF state and unrelated root scratch. No destructive,
registry, public-repository or automation action is introduced.
