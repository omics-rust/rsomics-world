# FASTQ 0.2 release hold: Phred+64 input validation

The pre-release review of `b6872a50` reproduced a strict-input violation in
the verified installed binary. `trim --phred-offset 64 --trim-front1 1`
accepts `ACGT` with quality `!III` and emits `CGT` with `***`; omitting the
trim returns nonzero for byte 33. The no-window branch discards fixed and
poly bases before the filter checks the offset. This predates 0.2 and is
not a new fastp compatibility contract. No release tag has been pushed.

- [x] Reproduce with CLI regressions for SE fixed clipping and PE poly-G
  trimming. Both failed on `b6872a50` by accepting invalid input.
- [x] Validate the original Phred+64 record once in the shared SE/PE
  processing path. Preserve Phred+33 reader validation, public `check`,
  valid Phred+64 normalization, filtering and trim precedence.
- [x] Run live fastp tests, fmt and strict Clippy, review the production
  diff, and verify exact-head four-native CI after pushing the fix.
- [x] Requalify the repaired installed binary before public delivery. Do
  not relabel the previous `b6872a50` benchmark or trigger crates.io.

Fix `10db8ef2cde7423b456b24eb0ef47177abf95014` is pushed to owner main.
The second read-only review found no release-relevant issue in the repair.
Both regressions pass. Local debug and release each pass 56 tests, including
the pinned live fastp 1.3.6 differentials. Fmt, strict Clippy, and rustdoc
pass. The first full-suite shell wrapper stopped on Bash 3's empty-array
expansion under `set -u`; it did not run the full suites. The successful
run's complete logs are in
`/Volumes/KIOXIA/Developments/tmp/fastq-02-exact-20261105.Z25dkh67/tmp/fastq-phred64-verified.Snhd1NPt`.

The clean package/install and same-protocol benchmark replay use
`/Volumes/KIOXIA/Developments/tmp/fastq-02-exact-20261105.Z25dkh67/tmp/fastq-phred64-candidate.WPBrclpH`.
The clean 58,025-byte crate was verified against 46 Git paths and normalized
Cargo metadata, then installed offline. All 14 installed checks passed
(one expected malformed-input failure). The installed original defect probe
now exits nonzero, with empty stdout and an explicit Phred+64 diagnostic.

The same 1M-pair synthetic protocol ran on Apple M2/macOS 27.2 arm64 with
Rust 1.91 and fastp 1.3.6. One warmup plus five alternating measured pairs
per workload; 14 full output groups match the pinned results. Measured SE
stdout is discarded identically; every PE result is fully decompressed and
matched. No claim about throughput for Phred+64 input follows from this
Phred+33 benchmark.

| Workflow | Wall median native / fastp | Peak RSS median native / fastp |
| --- | --- | --- |
| SE stdout, 1 worker | 0.462 / 0.926 s | 6,799,360 / 39,862,272 B |
| PE gzip level 4, 4 workers | 2.548 / 2.079 s | 32,800,768 / 101,548,032 B |

Every measured native RSS is below every measured fastp RSS. SE throughput
is 2.00x with 83% lower peak RSS; PE uses 68% less peak RSS but takes 23%
longer by wall median. On a shared host this is not an A/B speedup claim
against the previous candidate. RSS is maximum-process RSS. The
[qualification record](../oracles/fastq-phred64-qualification-2026-11-05.json)
binds all samples and exact package, installed binary and raw result hashes.

[FASTQ CI 41998285325](https://github.com/omics-rust/rsomics-fastq-preprocess/actions/runs/41998285325)
passed at the exact fix head, attempt 1: four native jobs and the
original-body audit. GitHub shows four original ZIP digests and the audit
receipt carrier digest; this is hosted verification, not local adoption.
The [GitHub release stage](../../docs/plans/2026-11-05-fastq-release-staging.md)
is separate. No tag, workflow dispatch, publication or credential mutation.
