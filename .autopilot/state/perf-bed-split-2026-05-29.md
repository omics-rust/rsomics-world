# perf-bed-split — 2026-05-29

## Tool
rsomics-bed-split 0.1.0

## Upstream reference
bedtools split v2.31.1 (homebrew, aarch64-apple-darwin)

## Machine
mini_m2 — Apple M2, aarch64-apple-darwin, macOS 15.x

## Fixture
File: /tmp/perf_large.bed (50,000 BED records, 5 chromosomes, ~1.7MB)
Generator: Python seed=42

## Command
```
hyperfine --warmup 3 --runs 7 \
  'rsomics-bed-split -i perf_large.bed -n 8 -p /tmp/split_our' \
  'bedtools split -i perf_large.bed -n 8 -p /tmp/split_bt'
```

## Results

| Command | Mean ± σ | Min | Max |
|---|---|---|---|
| rsomics-bed-split | 72.3 ms ± 32.2 ms | 35.1 ms | 119.3 ms |
| bedtools split 2.31.1 | 124.5 ms ± 28.4 ms | 87.0 ms | 165.5 ms |

## Ratio
**1.72 ± 0.86× faster** — PASS (>1.0× gate met)

## Status
PASS
