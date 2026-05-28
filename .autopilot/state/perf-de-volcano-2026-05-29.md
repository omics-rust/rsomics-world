# perf-de-volcano — 2026-05-29

## Tool
rsomics-de-volcano 0.1.0

## Upstream reference
Pure-Python DE volcano annotation reference implementation (Python 3.12.7, aarch64-apple-darwin)
No CLI upstream for volcano annotation; Python reference implements same algorithm.

## Machine
mini_m2 — Apple M2, aarch64-apple-darwin, macOS 15.x

## Fixture
File: /tmp/perf_de.tsv (20,000 DE result rows)
Generator: Python seed=42

## Command
```
hyperfine --warmup 3 --runs 7 \
  'rsomics-de-volcano perf_de.tsv' \
  'python3 de_volcano_ref.py perf_de.tsv'
```

## Results

| Command | Mean ± σ |
|---|---|
| rsomics-de-volcano | 7.0 ms ± 1.8 ms |
| Python DE volcano reference | 79.3 ms ± 31.9 ms |

## Ratio
**~11.3× faster** than pure-Python DE volcano annotation — PASS (>1.0× gate met)

## Status
PASS
