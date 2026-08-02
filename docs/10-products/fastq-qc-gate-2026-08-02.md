# FASTQ QC product gate — 2026-08-02

Status: `rsomics-fastq-qc 0.1.0` published and independently verified.

## Released contract

The `report` command analyzes multiple FASTQ inputs and writes one report
directory per input containing:

- FastQC/MultiQC-compatible `fastqc_data.txt`;
- `summary.txt` with module status tokens;
- a self-contained rsomics HTML report with inline SVG charts and accessible
  data tables.

Plain, gzip, and BGZF inputs are detected by content through `rsomics-seqio`.
Multiple files use a command-local Rayon pool controlled by the shared
`rsomics-common::ThreadArgs`; `rsomics-help` renders the nested command tree.
BAM/SAM, Casava grouping, Nanopore `fast5`, custom limits, custom adapter and
contaminant lists, and FastQC ZIP packaging are excluded from 0.1.0. No
placeholder command is published for them.

## Exact identities

- published source and VCS identity:
  `f24f0e1766d16e667a153b2dc6ac1d2c11d96e0d`;
- exact-head four-native-target CI: `30733371110`;
- publish workflow: `30733480364`;
- crate archive SHA-256:
  `979cf2d2340c8d4b6db2eab342cdefe526191a3a01a908a2b7554e9646eeb08b`;
- crate archive size: 37,285 bytes;
- FastQC 0.12.1 distribution SHA-256:
  `5f4dba8780231a25a6b8e11ab2c238601920c9704caa5458d9de559575d58aa7`.

Published foundations exercised by the release are `rsomics-common 0.11.1`,
`rsomics-help 0.4.0`, and `rsomics-seqio 0.4.0`. The product owns scheduling,
analysis policy, report transactions, and presentation; no speculative public
foundation was added.

## Historical source disposition

The retired `rsomics-fastqc` source head was
`8ea4621251d988d68cd900e51e47a404e2c6cdfc`. Its verified Git bundle SHA-256 is
`5362b07364495bba6c47c9cf65ba1747e013349dba9b3ebc4c99d4a3440b9944`;
the retired crates.io archive SHA-256 is
`355ce3a9debfc6219e396b3c2f4e75104e847414f84a8094da0659dc86c302bc`.

The analyzers and small fixtures were refactored into the product. The old
per-read dynamic dispatch, parallel mutable module state, JSON-only report
path, placeholder thread flag, stale defaults, and inherited compatibility
claims were discarded. No deleted micro-crate was revived.

## Correctness and compatibility

The release implements the eleven modules enabled by FastQC 0.12.1 defaults,
with Per tile sequence quality emitted only for compatible Illumina headers.
Tests cover module ordering and schemas, PASS/WARN/FAIL thresholds, base and
length grouping, GC modeling, quality histograms, adapter accumulation,
overrepresented-sequence ordering, and the finite-population correction after
FastQC's 100,000-unique-sequence duplication limit.

Full `fastqc_data.txt` output is byte-identical to FastQC 0.12.1 on both
6,282,141-read SRR341550 mates:

| Mate | Input SHA-256 | Shared report-data SHA-256 |
|---|---|---|
| R1 | `d7a15c1762d64a5434ced0cc665d7f5d167ca81a71e239f8237b9cd490dd7683` | `907e1e3f35583c62929c2d51cafe2c7904319e873f45a12ef5934fe2acfcb36f` |
| R2 | `18a8e61af21d276dfaf12035307d673e3f52c9f3ac57658ee2f593d1aabeb1a4` | `e163695bf49275ea96ca0192f89149e927efba0629cfb75f844bb1ec3377d945` |

Controlled black-box grids freeze boundary behavior independently of the real
fixture. CI downloads the exact FastQC distribution, verifies its archive
hash, and executes a live whole-report differential. The oracle executable is
not included in the crate.

The output transaction stages every report before commit, rejects colliding
input-derived report names before analysis, preserves existing destinations,
and rolls back already committed outputs on a later failure. Linux and macOS
use an atomic no-replace directory rename. Truncated gzip, malformed FASTQ,
partial multi-input failure, aliases, thread-count invariance, and JSON/stdout
separation have integration coverage.

Exact-head CI passed formatting, strict Clippy, rustdoc with warnings denied,
clean package verification, debug/release tests, the live oracle, and benchmark
smoke on native Ubuntu 24.04 and macOS 15 for both `x86_64` and `aarch64`.

## Representative performance

The host was an Apple M2 Mac mini running macOS 26.6. Rust 1.91.0 built the
release binary; FastQC 0.12.1 ran on OpenJDK 26.0.2. Both commands processed
the two compressed SRR341550 mates concurrently with two worker threads.
Hyperfine 1.20.0 performed one warmup and five measured runs. Peak RSS is from
a separate `/usr/bin/time -l` run.

| Implementation | Mean wall time | Standard deviation | Peak RSS |
|---|---:|---:|---:|
| FastQC 0.12.1 | 25.003 s | 0.888 s | 624.5 MiB |
| rsomics-fastq-qc 0.1.0 | 16.563 s | 0.063 s | 43.0 MiB |

The rsomics workflow is 1.51 times faster and uses 93.1% less peak resident
memory on this host. Both commands produced complete reports. Packaging is not
identical: FastQC writes separate image assets, while rsomics embeds SVG charts
in its HTML. The performance claim is therefore an end-to-end user-workflow
comparison, not an analysis-only microbenchmark.

Tracked raw artifact SHA-256 values:

- Hyperfine JSON:
  `7cae65a5a442963f548654884a5f86aef1303c57ca16241502405053e7df76f3`;
- FastQC memory observation:
  `6540d32d8bd691c5e176b7e8f861be54185e46fca4d81ce7017cc15801a07cb6`;
- rsomics memory observation:
  `d211fe637f7b20e4767eed408bb28e41ad465c89d482f16ecbf390e6f50a8b6b`.

The raw files and reusable driver are tracked under
`benchmarks/2026-08-02-macos-arm64-fastqc-0.12.1/` in the product repository.

## Publication verification

The crates.io archive was independently downloaded after publication. Its
checksum and `.cargo_vcs_info.json` match the recorded archive and source head.
An isolated external-disk `cargo install --registry crates-io --version 0.1.0
--locked rsomics-fastq-qc` succeeded. The installed binary reported version
0.1.0, rendered the shared top-level and nested help, and generated all three
report artifacts from the packaged tiny FASTQ fixture.

The registry token was exposed to the repository only for publish workflow
`30733480364`; repository access was removed immediately after success and
verified absent.
