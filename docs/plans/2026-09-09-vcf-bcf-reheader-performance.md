# BCF reheader performance gate

Status: accepted implementation plan; no BCF measurements exist yet. Keep the
historical VCF harness and results intact. Implement a product-local BCF
harness and its own tests, not another public foundation. The first execution
target is native Linux x86 on GitHub Actions; local execution is prohibited by
boot-disk occupancy. Other native classes remain required for release.

## Questions

1. Does the complete current BCF reheader command offer a measured throughput
   or resource advantage over pinned bcftools 1.24 on the named workloads?
2. What is the aggregate cost of the decoder/framing repair on canonical valid
   BCF, compared with an exact pre-repair binary built in the same job?

These are separate comparisons. Neither identifies an allocator-level cause.
Named-output timing includes each command's actual durability contract:
rsomics-common 0.12.3 synchronizes the file and parent directory; bcftools
reheader closes its output without matching explicit synchronization.
Do not remove the product's transaction guarantees to improve a timing.

## Fixed matrix

| Synthetic workload | Records | Samples | Purpose |
|---|---:|---:|---|
| record-heavy | 2,000,000 | 8 | sustained record/frame processing |
| sample-heavy | 100,000 | 128 | FORMAT decoding and encoding |

Generate deterministic varying GT, DP, AD and PL, including supported missing
and phased genotypes. Retain the seed, generator identity, complete source VCF
hash and schema. These synthetic inputs do not establish population-wide
performance. Use pinned bcftools to encode BGZF BCF and independently validate
the result before measuring sample renaming with a complete two-column map.

Each workload has zero-worker and two-worker cells. Zero workers is the
primary comparison. Two workers is product-level evidence: bcftools shares
its pool between input and output, whereas rsomics workers compress output.
Both use their default compression settings because reheader has no matching
compression-level selector. Record the actual backends and output sizes.
These differences follow the [pinned bcftools implementation](https://raw.githubusercontent.com/samtools/bcftools/1.24/reheader.c)
and current product code; equal flags do not imply identical pipelines.

Use three alternating warmup pairs followed by 12 alternating measured pairs
per cell: 96 measured candidate/oracle invocations across four cells. Smoke
mode uses smaller fixed inputs and fewer runs solely to validate the harness.
Freeze formal input sizes before comparing ratios; any resizing decision must
be based on absolute duration/storage, not which implementation is ahead.

## Harness correctness

Write harness tests first for resource-field parsing and unit conversion,
paired summaries, workload identity/resume rejection and strict framing.
Do not call product binaries locally to run those tests. Exercise remote
smoke before formal measurements.

Before timing and after every invocation, outside the timer, validate BGZF
CRC/ISIZE, complete frame boundaries, one terminal canonical EOF, BCF magic,
header length and complete record framing. Zero shared length and trailing
bytes are invalid, not EOF. Combine that physical validation with complete
bcftools-decoded normalized header and ordered record-body hashes against a
frozen rename expectation. Check sample order and all INFO/FORMAT values.
Product view must not be the sole validator. Record each physical output hash
and size, command status and stderr; reject failures before summarizing.

Keep timing output separate from program stderr. Preserve raw wall, user CPU,
system CPU and maximum RSS observations, including their original units.
Normalize Linux KiB explicitly if using GNU time. Report median, IQR, range,
paired wall/RSS ratios and user+system CPU. Do not label the maximum of twelve
observations a useful p99 estimate or declare success from any smaller median.
Performance decisions require a review of paired distributions and tradeoffs.

## Provenance and storage

Before the first tool invocation, record the frozen source manifest, lockfile,
build flags/features and exact executable hashes; prove the relationship to
the tested source in the job. Include Rust, bcftools/HTSlib versions/backends,
native host/CPU/RAM, filesystem, environment, input hashes and exact commands.
The pre-repair control needs its own source/executable identity and must be
restricted to canonical valid inputs supported by that version.

Use unique cell/pair/tool output directories. Keep every output initially;
preflight estimated full retention plus build/input space and a safety margin.
If storage is insufficient, stop that benchmark without silently shrinking the
formal workload or removing evidence. Completion manifests are written only
after validation. A matching fingerprint may resume completed observations;
changed tools, inputs, harness or options require a new directory. Failed or
partial attempts remain visible and are never overwritten as successful runs.

Formal Linux results do not replace four-native correctness CI, the concat
per-mode/many-sample ligation benchmarks or a fresh public API/hot-path review.
No product publication follows from a smoke or isolated median improvement.
