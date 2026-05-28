# perf-count-matrix — 2026-05-29

## Tool
rsomics-count-matrix 0.1.0

## Upstream reference
Pure-Python count-matrix merge reference implementation (Python 3.12.7, aarch64-apple-darwin)
No CLI upstream for merging featureCounts outputs; Python reference implements same algorithm.

## Machine
mini_m2 — Apple M2, aarch64-apple-darwin, macOS 15.x

## Fixture
Files: /tmp/perf_counts_s1.txt, /tmp/perf_counts_s2.txt (20,000 genes × 2 samples each)
Generator: Python seed=42

## Command
```
hyperfine --warmup 3 --runs 7 \
  'rsomics-count-matrix perf_counts_s1.txt perf_counts_s2.txt' \
  'python3 count_matrix_ref.py perf_counts_s1.txt perf_counts_s2.txt'
```

## Results

| Command | Mean ± σ |
|---|---|
| rsomics-count-matrix | 18.6 ms ± 6.0 ms |
| Python count-matrix reference | 161.7 ms ± 74.0 ms |

## Ratio
**~8.7× faster** than pure-Python count-matrix merging — PASS (>1.0× gate met)

## Status
PASS
