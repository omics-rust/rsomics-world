# `rsomics-pileup` 0.9 record-state gate

## Scope

Revision `8d85ae1b0cc4cea0ff5276094eb556321298aa34` lets a consumer retain one
typed value with each accepted alignment record and borrow it from every
column entry until that record is pruned. The existing `PileupEngine<()>` API
and default hot path remain source compatible.

`rsomics-call` uses the state to compute CIGAR-derived annotation data once
per record across SNP and indel evidence. The hidden `rsomics-bam consensus`
implementation retains its local mismatch, homopolymer, and adjusted-quality
context through the same API. Call passed 118 unit tests, seven ordinary CLI
tests, 21 bcftools 1.24 oracle tests, and strict Clippy against the local 0.9
candidate. BAM's caller and retained-state column tests pass against the same
candidate.

## Environment

- Apple M2, `aarch64-apple-darwin`
- macOS 26.6.1 build 25G76
- rustc 1.91.0, LLVM 21.1.2
- hyperfine 1.20.0
- release profile, one process
- build targets and temporary data on the Zane external disk

The comparison uses the existing deterministic 250x engine benchmark:
166,667 reads of 150 bases, 100,149 emitted columns, and 25,000,050 emitted
entries. Baseline revision `878619cb91bdad734808fac25682e008c6b53e70`
is version 0.8.0. The two generated lockfiles differ only in the root package
version, so dependency versions and the compiler are identical.

```text
hyperfine --warmup 2 --runs 10 \
  --command-name '0.8 default state' '<0.8-engine> 250' \
  --command-name '0.9 generic state' '<0.9-engine> 250'
```

## Results

| Revision | Mean +/- SD | Median | Range | Median entries/s |
|---|---:|---:|---:|---:|
| 0.8.0 | 202.9 +/- 1.9 ms | 202.918 ms | 200.615-207.444 ms | 123,202,683 |
| 0.9.0 | 201.1 +/- 1.1 ms | 201.358 ms | 198.909-202.610 ms | 124,157,517 |

Five independent `/usr/bin/time -lp` runs give a baseline median maximum RSS
of 1,835,008 bytes, range 1,818,624-1,835,008. The 0.9 candidate is 1,802,240
bytes in all five runs. The hyperfine JSON SHA-256 is
`a0f6fd4cab6d05084c7355dda1fdafa3cb48d1dbbb5faf352c72ad37fe3d3687`.

## Decision

The default no-state path has no measured throughput or memory regression,
while two products avoid repeated per-column record derivation through the
new policy-free contract. The candidate passed exact-head four-native-target
CI run `31469899244`, including the samtools 1.24 differential. This closes
the 0.9 foundation performance and API gate; it does not establish a
product-level speed claim for consensus or variant calling.
