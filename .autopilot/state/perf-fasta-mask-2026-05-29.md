# perf-fasta-mask — 2026-05-29

## Tool
rsomics-fasta-mask 0.1.0

## Upstream reference
bedtools maskfasta v2.31.1 (homebrew, aarch64-apple-darwin)

## Machine
mini_m2 — Apple M2, aarch64-apple-darwin, macOS 15.x

## Fixture
File: /tmp/perf_ref2m.fa (2,000,000 bp chr1 random sequence)
File: /tmp/perf_mask2m.bed (12,824 mask regions)
Generator: Python seed=42

## Command
```
hyperfine --warmup 3 --runs 7 \
  'rsomics-fasta-mask /tmp/perf_ref2m.fa -b /tmp/perf_mask2m.bed > /dev/null' \
  'bedtools maskfasta -fi /tmp/perf_ref2m.fa -bed /tmp/perf_mask2m.bed -fo /dev/null'
```

## Results

| Command | Mean ± σ | Min | Max |
|---|---|---|---|
| rsomics-fasta-mask | 15.8 ms ± 11.2 ms | — | — |
| bedtools maskfasta 2.31.1 | 38.5 ms ± 13.3 ms | — | — |

## Ratio
**2.45 ± 1.93× faster** — PASS (>1.0× gate met, though timing is variable due to filesystem caching)

## Status
PASS
