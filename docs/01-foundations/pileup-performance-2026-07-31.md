# `rsomics-pileup` 0.2 engine performance

## Scope

This gate measures the foundation's streaming column engine, including the
no-indel partial-BAQ trigger scan. It does not compare a complete command with
samtools or bcftools and therefore supports no product-level speed claim.

Revision `b253b74bd0b13e205f6a7d8eafed7e87f538ac64` includes the deterministic
benchmark. It distributes validated 150-base match records over a 100,000-base
region and consumes every emitted entry. The 30× cases emit 3,000,000 entries;
the 250× cases emit 25,000,050.

## Environment

- Apple M2, `aarch64-apple-darwin`
- macOS 26.6 build 25G72
- rustc 1.91.0, LLVM 21.1.2
- hyperfine 1.20.0
- release profile, one process
- target and temporary data on KIOXIA external storage

The benchmark binary was built with:

```text
cargo +1.91.0 bench --bench engine --no-run
```

Each timing used five warmups and 30 measured runs:

```text
hyperfine --warmup 5 --runs 30 \
  --command-name ordinary '<engine> 30' \
  --command-name ordinary-partial '<engine> 30 partial' \
  --command-name deep '<engine> 250' \
  --command-name deep-partial '<engine> 250 partial'
```

## Results

Times are seconds. The p99 is the type-7 sample quantile. Throughput divides
the exact emitted-entry count by the median.

| Case | Median | p99 | Mean ± SD | Entries/s |
|---|---:|---:|---:|---:|
| 30× | 0.0286 | 0.0618 | 0.0308 ± 0.0081 | 104,843,557 |
| 30× partial scan | 0.0314 | 0.0350 | 0.0315 ± 0.0011 | 95,670,066 |
| 250× | 0.2103 | 0.2223 | 0.2085 ± 0.0061 | 118,857,603 |
| 250× partial scan | 0.2347 | 0.3307 | 0.2387 ± 0.0247 | 106,522,984 |

Peak RSS was measured with `/usr/bin/time -l` over five independent runs:

| Case | Median bytes | Range bytes |
|---|---:|---:|
| 30× | 1,802,240 | 1,785,856–1,818,624 |
| 30× partial scan | 1,802,240 | 1,802,240–1,818,624 |
| 250× | 1,933,312 | 1,867,776–1,949,696 |
| 250× partial scan | 1,949,696 | 1,933,312–1,966,080 |

The hyperfine JSON SHA-256 is
`a0c62a716162ecf946028490ad753e6b9251b4768c6cd54722d3722ad94dbc9b`.

## Decision

The engine retains bounded state at 250× and the partial trigger scan adds
11.6% to median wall time without material RSS growth. This closes the
foundation's representative ordinary/deep-coverage performance and memory
gate. Publication still requires a second product-side contract; the BAM
integration is not replaced by this synthetic benchmark.
