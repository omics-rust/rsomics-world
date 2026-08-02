# FASTQ preprocessing product gate — updated 2026-08-02

Status: `rsomics-fastq-preprocess 0.1.1` published and independently verified.

Version 0.1.1 at `89d4f534f90eca7ce84cf0929474047bd1139d38`
replaces the product-local thread argument with `rsomics-common 0.11.1`'s
shared `ThreadArgs`; the consumer keeps only its Rayon pool policy. Exact-head
four-native CI `30731874951` and publish run `30732312288` passed. The
independently downloaded non-yanked archive is 39,843 bytes, has SHA-256
`27e723ea3c1e3279f2fdc0c369a44da4e6c39ec9bb53ca6b5149169167ec7a95`,
and records that exact VCS identity.

## Released slice

The product exposes one shared strict single-end/paired-end engine through
three user-recognizable operations:

- `run`: fixed/poly-tail trimming and read filtering in one traversal;
- `trim`: fixed, poly-G, and poly-X trimming;
- `filter`: quality, mean-quality, N, length, and complexity filtering.

Adapter and overlap trimming, correction, UMI processing, merging,
deduplication, BBDuk-style filtering, interleaved FASTQ, instrument
auto-detection, and fastp's complete report schema remain explicit exclusions.
No placeholder command is published for them.

Historical `rsomics-fqgz` supplied an implementation asset. Its useful
chunked-libdeflate algorithm was refactored into a private product module; the
deleted micro-crate was not revived and no speculative foundation API was
added.

## Exact identities

- measured production source:
  `fd04e662426d98f414c51d16a84a2e0eb643e010`;
- deterministic broken-pipe test:
  `fd5e1ec33890c5433bfc98632196f8039e31b3b9`;
- published head and VCS identity:
  `755cd715276b01598b943f1a317b2e909b7d69c3`;
- exact-head four-native-target CI: `30726427849`;
- publish workflow: `30726551865`;
- crate archive SHA-256:
  `d12cb432e56fdeb151e91c80804301089a8b5716e07dd922b347024b9c82c016`;
- archive size: 39,876 bytes;
- registry state: 0.1.0, non-yanked, Rust 1.91, created
  `2026-08-02T01:10:22.419475Z`.

Published foundations exercised by the release:

- `rsomics-common 0.11.0`,
  `5bac25e251cc74c6a43e8302a3a6cc150886a340`;
- `rsomics-help 0.4.0`,
  `61dd6f2ce0cef6d9b4e349af5f96f96a7c95a013`;
- `rsomics-seqio 0.4.0`,
  `0c6ce988d8c90c5bfdaea00c1bcf53ae4aa443dd`.

The oracle is fastp 1.3.6 at source revision
`23d6211d4f05d61f561899f1b7702435a4b5d408`. The measured rsomics and fastp
binary SHA-256 values were respectively
`80aae1d1395627ad845f232eeda0652ab7edbcff012cd151ddf7dbf3c772422b` and
`8b0521f3d246e13178c49235c0a76230e5ee930fafcaf0db647a4210a4a65966`.

## Implementation and API decision

The product retains `rsomics-seqio` for strict parsing, validation, and FASTQ
serialization. A private `ParallelGzipWriter`:

- buffers 256 KiB chunks;
- compresses at most 16 pending chunks through the command's local Rayon pool;
- collects indexed results in source order;
- emits standards-compliant concatenated gzip members;
- produces a valid gzip stream for zero surviving reads;
- propagates compression, downstream write, flush, and finish errors;
- remains inside the transactional no-clobber output path.

This contract is product-specific until a second product needs the same
thread-controlled ordered writer. Promoting it to `rsomics-seqio` now would
violate the two-consumer foundation rule.

The release API review corrected three inherited hazards:

- public trim now rejects missing or mismatched qualities and invalid
  configuration rather than panicking;
- public filter rejects malformed record shape, invalid quality bytes, and
  invalid configuration rather than silently truncating with `zip`;
- `PipelineConfig` fields are private and typed constructors bind the report
  operation to the active stages.

The internal parsed-record path avoids repeating those public-boundary checks
inside the hot loop. Product-specific paired synchronization, output alias
policy, and best-effort two-file rollback remain local. `rsomics-common`
supplies the single-output alias and execution contracts; `rsomics-help`
renders the real nested Clap tree without a duplicate presentation model.

## Correctness gate

The release runs 22 library tests, four CLI-model tests, 22 CLI integration
tests, and four live fastp differentials in both debug and release profiles.
The tests cover:

- strict input, gzip truncation, quality encoding, and paired identity/count
  failures;
- no-clobber exact, normalized, hard-link, and symbolic-link aliases;
- paired output coordination and rollback;
- fastp filter precedence and boundary behavior;
- fixed/poly-G/poly-X trimming, including mismatch budgets and PE inheritance;
- ordered parallel gzip, empty output, flush ordering, and downstream errors;
- thread-count invariance and `trim | filter` equivalence to `run`;
- product/subcommand help and JSON/data stdout separation.

The broken-pipe integration test originally raced with a small pipe buffer on
Linux `x86_64`. Revision `fd5e1ec` replaced it with a Unix socket whose read
end is closed before process launch. The implementation already propagated the
write error; the corrected test now exercises that contract deterministically.

Exact-head CI run `30726427849` passed formatting, strict Clippy, rustdoc,
clean package verification, debug/release tests, live fastp differentials, and
benchmark smoke on native:

- Ubuntu 24.04 `x86_64`;
- Ubuntu 24.04 `aarch64`;
- macOS 15 `x86_64`;
- macOS 15 `aarch64`.

Linux `aarch64` has native correctness evidence but no representative
performance host.

## Representative performance

The host was `dell-Precision-7920-Tower`, Ubuntu 22.04, Linux 6.8.0,
`x86_64`, with two Intel Xeon Gold 6238R CPUs. Rust was 1.91.0. All source,
toolchains, targets, temporary files, fixtures, and results were under
`/data1`; the server root filesystem was not used for build output.

Inputs:

- `SRR341550_1.fastq.gz`:
  `d7a15c1762d64a5434ced0cc665d7f5d167ca81a71e239f8237b9cd490dd7683`;
- `SRR341550_2.fastq.gz`:
  `18a8e61af21d276dfaf12035307d673e3f52c9f3ac57658ee2f593d1aabeb1a4`.

Times are five-run Hyperfine means and sample standard deviations after one
warmup. Peak RSS is from a separate `/usr/bin/time -v` run.

| Mode | Threads | rsomics | fastp 1.3.6 | RSS, rsomics / fastp |
|---|---:|---:|---:|---:|
| paired | 4 | 10.914 ± 0.493 s | 14.690 ± 0.715 s | 31.5 / 99.2 MiB |
| single | 4 | 5.969 ± 0.431 s | 5.503 ± 0.862 s | 18.0 / 51.1 MiB |

Decision:

- paired compressed output is 1.35 times faster and uses 68% less peak memory
  on this host;
- single-end compressed output does not claim a throughput win;
- single-end demonstrates 65% lower peak memory;
- these are operation- and host-specific results, not a blanket replacement
  claim.

Decompressed rsomics and fastp outputs matched at each corresponding position:

- paired R1:
  `f13cb655feedf78cf1f3c512675ad73323409f5862b0b3a6e5e3d48e21e6e365`;
- paired R2:
  `452c78a98878e56bf1e5e7728b749e0277e0e14607fa465f7da3e83e551c078c`;
- single R1:
  `9cc5172922740e7291bdf9fdfadc3d03370665fb0a8d4d4c4c5d4b930c800b58`.

The concatenated-member files also passed `gzip -t` and were consumed by
SeqKit 2.13.0 and fastp 1.3.6.

## Publication verification

The crates.io archive was downloaded independently after publication. Its
checksum and `.cargo_vcs_info.json` matched the registry record and release
head. `cargo install --locked rsomics-fastq-preprocess@0.1.0` succeeded from
the registry archive on an isolated external-disk target. The installed
binary passed:

- top-level command/help discovery;
- a stdin/stdout identity filter smoke;
- a malformed-quality smoke that returned non-zero with the strict parse
  error.

The registry token was exposed to the repository only for publish workflow
`30726551865`; repository access was removed immediately after success and
verified absent.

`rsomics-fastq-qc` is a separate cross-product integration consumer, not a
dependency of this release. Standard FASTQ output was already covered by
strict seqio parsing, live fastp differentials, stream composition, gzip
interoperability, SeqKit, and registry-install smoke. Its later independent
release confirms the handoff without changing this product boundary.

## Raw evidence

Tracked product artifacts:
`benchmarks/linux-x86_64-fastp-1.3.6/` at release head `755cd71`.

Principal artifact SHA-256 values:

- paired Hyperfine JSON:
  `df3123f7986fef7e9e1970eadc15a6fbf5fd78839bcc0adc36c7cc3238a753ad`;
- single Hyperfine JSON:
  `f2cabf8b2d16c6c4ef08ed8aa694f0f88e49a4a56418864eab7ce42ff135d8a6`;
- rsomics paired RSS:
  `7220ff0e0292c8b9d9151ecb1d3aa4e736d074c1ef2e2c4d0ab5cf79567a5e44`;
- fastp paired RSS:
  `32bc2a0ffebb4a4d3add83eaf2bab4964c4a4d202574812ff0a31a52046be474`;
- rsomics single RSS:
  `67eca864f0b0992bbca386f98da7c6fe332479f4ace763421d49fffd106e3522`;
- fastp single RSS:
  `fed3ff187cda59dced0f9f99836e27c648f3de4cddbd4fcb88c87b0a6519e6fd`.

The remote result directory remains
`/data1/liangjy/rsomics-linux-x86_64-20260730/results/release-fd04e66`.
