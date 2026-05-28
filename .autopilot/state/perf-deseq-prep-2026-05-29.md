# perf-deseq-prep — 2026-05-29

## Tool
rsomics-deseq-prep 0.1.0

## Upstream reference
Pure-Python DESeq2 prep (low-count filter) reference implementation (Python 3.12.7, aarch64-apple-darwin)
No CLI upstream; Python reference implements same low-count gene filter algorithm.

## Machine
mini_m2 — Apple M2, aarch64-apple-darwin, macOS 15.x

## Fixture
File: /tmp/perf_deseq_counts.tsv (20,000 genes × 6 samples count matrix)
Generator: Python seed=42

## Command
```
hyperfine --warmup 3 --runs 7 \
  'rsomics-deseq-prep perf_deseq_counts.tsv' \
  'python3 deseq_prep_ref.py perf_deseq_counts.tsv'
```

## Results

| Command | Mean ± σ |
|---|---|
| rsomics-deseq-prep | 14.7 ms ± 6.4 ms |
| Python DESeq2 prep reference | 129.3 ms ± 12.8 ms |

## Ratio
**~8.8× faster** than pure-Python DESeq2 prep — PASS (>1.0× gate met)

## Status
PASS
