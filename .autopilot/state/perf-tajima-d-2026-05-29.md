# perf-tajima-d — 2026-05-29

## Tool
rsomics-tajima-d 0.1.0

## Upstream reference
Pure-Python Tajima's D reference implementation (Python 3.12.7, aarch64-apple-darwin)
vcftools not available on mini_m2; Python reference implements same algorithm.

## Machine
mini_m2 — Apple M2, aarch64-apple-darwin, macOS 15.x

## Fixture
File: /tmp/perf_sfs_large.tsv (100,000 derived allele count entries, n=100 haploid samples)
Generator: Python seed=42

## Command
```
time rsomics-tajima-d sfs_large.tsv -n 100 > /dev/null
time python3 tajima_ref.py sfs_large.tsv > /dev/null
```

## Results

| Command | Wall time |
|---|---|
| rsomics-tajima-d | 0.035s |
| Python Tajima's D reference | 0.294s |

## Ratio
**~8.4× faster** than pure-Python reference — PASS (>1.0× gate met)

## Status
PASS
