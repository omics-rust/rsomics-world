# perf-seq-stats — 2026-05-29

## Tool
rsomics-seq-stats 0.1.0

## Upstream reference
seqkit stats v2.13.0 (homebrew, aarch64-apple-darwin)

## Machine
mini_m2 — Apple M2, aarch64-apple-darwin, macOS 15.x

## Fixture
File: /tmp/perf_seq_stats.fa (10,000 sequences, 500–5000 bp each, 27.5 MB total)
Generator: Python seed=42

## Command
```
hyperfine --warmup 3 --runs 7 \
  'rsomics-seq-stats /tmp/perf_seq_stats.fa' \
  'seqkit stats /tmp/perf_seq_stats.fa'
```

## Results

| Command | Mean ± σ |
|---|---|
| rsomics-seq-stats | 27.8 ms ± 3.6 ms |
| seqkit stats v2.13.0 | 32.8 ms ± 4.1 ms |

## Ratio
**~1.18× faster** than seqkit stats — PASS (>1.0× gate met)

## Status
PASS
