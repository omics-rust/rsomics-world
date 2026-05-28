# perf-bed12-to-bed6 — 2026-05-29

## Tool
rsomics-bed12-to-bed6 0.1.0

## Upstream reference
bedtools bed12tobed6 v2.31.1 (homebrew, aarch64-apple-darwin)

## Machine
mini_m2 — Apple M2, aarch64-apple-darwin, macOS 15.x

## Fixture
File: /tmp/perf_large.bed12 (15,000 BED12 records, 3 chromosomes, 1.4MB)
Exons: 2-8 per transcript, 50-500bp exons, 100-2000bp introns
Generator: Python seed=42

## Command
```
hyperfine --warmup 3 --runs 7 \
  'rsomics-bed12-to-bed6 -i perf_large.bed12 > /dev/null' \
  'bedtools bed12tobed6 -i perf_large.bed12 > /dev/null'
```

## Results

| Command | Mean ± σ | Min | Max |
|---|---|---|---|
| rsomics-bed12-to-bed6 | 26.9 ms ± 7.9 ms | — | — |
| bedtools bed12tobed6 2.31.1 | 130.1 ms ± 32.7 ms | — | — |

## Ratio
**4.84 ± 1.87× faster** — PASS (>1.0× gate met)

## Status
PASS
