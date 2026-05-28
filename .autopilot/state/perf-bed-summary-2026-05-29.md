# perf-bed-summary — 2026-05-29

## Tool
rsomics-bed-summary 0.1.0

## Upstream reference
bedtools summary v2.31.1 (homebrew, aarch64-apple-darwin)

## Machine
mini_m2 — Apple M2, aarch64-apple-darwin, macOS 15.x

## Fixture
File: /tmp/perf_large.bed (50,000 BED records, 5 chromosomes, ~1.7MB)
File: /tmp/perf_genome.txt (5 chromosomes, 10Mbp each)
Generator: Python seed=42

## Command
```
hyperfine --warmup 3 --runs 7 \
  'rsomics-bed-summary -i perf_large.bed -g perf_genome.txt > /dev/null' \
  'bedtools summary -i perf_large.bed -g perf_genome.txt > /dev/null'
```

## Results

| Command | Mean ± σ | Min | Max |
|---|---|---|---|
| rsomics-bed-summary | 7.7 ms ± 2.8 ms | — | — |
| bedtools summary 2.31.1 | 88.3 ms ± 27.2 ms | — | — |

## Ratio
**11.54 ± 5.48× faster** — PASS (>1.0× gate met)

## Status
PASS
