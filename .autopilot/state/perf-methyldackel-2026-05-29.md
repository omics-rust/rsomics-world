# perf-methyldackel — 2026-05-29

## Tool
rsomics-methyldackel 0.1.0

## Upstream reference
MethylDackel 0.6.1 (using HTSlib 1.21, conda methyldackel-env, aarch64-apple-darwin)

## Machine
mini_m2 — Apple M2, aarch64-apple-darwin, macOS 15.x

## Fixture
File: /tmp/bs_perf.bam (sorted+indexed, 50,000 reads, 839 KB)
Reference: /tmp/bs_ref.fa (50,000 bp chr1, CpG every 20bp)
Generator: Python seed=42, pysam, 80% methylation rate, 100bp reads with XM/XR/XG tags

## Command
```
hyperfine --warmup 3 --runs 7 \
  'rsomics-methyldackel /tmp/bs_ref.fa /tmp/bs_perf.bam -o /tmp/ours_methyl' \
  'MethylDackel extract /tmp/bs_ref.fa /tmp/bs_perf.bam -o /tmp/ref_methyl'
```

## Results

| Command | Mean ± σ | Min | Max |
|---|---|---|---|
| rsomics-methyldackel | 31.4 ms ± 5.0 ms | 25.5 ms | 39.3 ms |
| MethylDackel 0.6.1 (HTSlib) | 139.4 ms ± 24.3 ms | 122.4 ms | 188.9 ms |

## Ratio
**4.43 ± 1.05× faster** — PASS (>1.0× gate met)

## Status
PASS
