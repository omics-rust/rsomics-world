# `rsomics-seqio 0.3.0` release gate — 2026-07-31

Status: published, registry archive independently verified, and exercised by
two product consumers.

## Boundary

The public crate provides:

- strict borrowed FASTA/FASTQ records and an optional-quality owned form;
- allocation-reusing readers for buffered, generic, and path inputs;
- transparent plain, gzip, and BGZF input detection;
- canonical record serialization to a caller-owned sink.

The final API review removed the unconsumed legacy FASTQ iterator, the
unconsumed forced-format generic opener, and serial gzip/path writer
conveniences. Compression parallelism, file transactions, and output policy
remain in products. `rsomics-seq` and `rsomics-fastq-preprocess` are the two
implemented consumers.

## Exact identities

- API review: `2ed5e3a66c67b74bde087ff28e7c731ef5ccc209`;
- published source head: `d7e1c33bb6008c0c0f59d94d07e8d5a108adaf0b`;
- source exact-head CI: `30599703477`;
- publish run: `30599794727`;
- sequence consumer contract: `060498996b669922f3188c592f73b7d98f0e21f3`,
  CI `30599557545`;
- preprocessing consumer contract:
  `6c18bedd36b3a23d7416390b2e3a09df22260079`, CI `30599557347`;
- registry-locked sequence head:
  `d4c840be2e37dc51ed38e1bd53d00ed5c3b3c84e`, CI `30599999972`;
- registry-locked preprocessing head:
  `442c202908d196f6e2fabf6283f6b6c87c1adfda`, CI `30599999790`;
- crates.io archive SHA-256:
  `d2dcd0fab1a5320834a9b0f9cba7bbdd9bfe6b26c9c4740650ac88d939fcfcc5`.

Source CI passed native Linux and macOS on `x86_64` and `aarch64`, including
strict Clippy, package verification, 38 unit tests, five compatibility tests,
and benchmark smoke. The downloaded crates.io archive matched the API checksum
and repeated all 43 tests.

## Consumer correctness

The sequence consumer passed 44 library, CLI, independent k-mer, and live
SeqKit tests. The preprocessing consumer passed 47 library, CLI, and live
fastp tests. Both products detect gzip by content, reject truncated streams,
preserve wrapped FASTQ records, propagate parse and sink errors, and exercise
the shared borrowed and owned record forms.

Preprocessing owns its thread-controlled parallel gzip writer. It delegates
record validation and serialization to `rsomics-seqio::Writer`, so the
foundation does not acquire one product's threading or transactional policy.

## Representative reader gate

The final reader candidate was exercised through `rsomics-seq` on
`SRR341550_1.fastq.gz`: 6,282,141 reads, SHA-256
`d7a15c1762d64a5434ced0cc665d7f5d167ca81a71e239f8237b9cd490dd7683`.
The host was `dell-Precision-7920-Tower`, Ubuntu 22.04, Linux
6.8.0-90-generic, `x86_64`, with two Intel Xeon Gold 6238R CPUs. Commands were
bound to cores 48–51; Hyperfine used one warmup and ten measured runs.

| Operation | rsomics | Reference | Speedup | RSS, ours / reference |
|---|---:|---:|---:|---:|
| stats | 1.142 ± 0.007 s | SeqKit 2.13.0: 1.456 ± 0.003 s | 1.27x | 6,964 / 20,604 KiB |
| strict validate | 1.122 ± 0.005 s | no semantic-equivalent oracle | baseline only | 7,056 KiB |

The stats output matched SeqKit byte for byte, SHA-256
`5f78d46bfcfe61c7af4ec8732ce01c6592b1d5c0e2f467a2f1e9cdb8ea78c555`.
This consumer gate includes gzip decode, strict parsing, validation, and the
small statistics reduction; it is not presented as an isolated parser
microbenchmark.

The release binary SHA-256 was
`62d7b59d846b5569242d306f22a5c4bddde115c3cf0215a747f6c8cd797bdf5c`.
The pinned SeqKit binary SHA-256 was
`68e55e64ca2c5123376c87dbee8f69cf3e2d41bada0639a9b7d7d56de73eea04`.

Raw evidence:
`/data1/liangjy/rsomics-linux-x86_64-20260731/seqio-gate/results-final`.
The stats JSON SHA-256 is
`7e255f55cd7def3cfbb585c37a85c5284bdd9028dbae111ef9693f121cb3a1c3`;
the validate JSON SHA-256 is
`3cdff8170eaaf2fd0b2f5364724eac77c46074b8b414bf10fd61ee1d65c748d8`.

## Follow-up

Native Linux `aarch64` has correctness CI but no representative performance
host. A future public item still requires two named consumers and
consumer-side contract tests; this release does not reserve speculative
compression or format-specific APIs.
