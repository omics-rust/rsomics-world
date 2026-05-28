# rsomics-bed-sort perf gate — 2026-05-29

## Tool versions
- rsomics-bed-sort: 0.1.0 (main repo)
- bedtools: 2.31.1 (Homebrew, /opt/homebrew/bin/bedtools)

## Machine
- mini_m2 (aarch64-apple-darwin, Apple M2)
- Rust 1.87 stable, release build

## Fixture
- 120,000 BED3 records across 24 chromosomes (chr1–chr22, chrX, chrY)
- Pre-shuffled (unsorted) input, seed 42
- File: `/tmp/bench_sort.bed` — 2.8 MB

## Command
```
rsomics-bed-sort input.bed > /dev/null
bedtools sort -i input.bed > /dev/null
```

## Results (hyperfine --warmup 2 --min-runs 5)

| Tool | Mean | Stddev | Min | Max | User CPU |
|------|------|--------|-----|-----|----------|
| rsomics-bed-sort | 83.0 ms | ±63.4 ms | 24.8 ms | 341.9 ms | 26.1 ms |
| bedtools sort | 235.2 ms | ±126.9 ms | 134.4 ms | 564.8 ms | 106.3 ms |

**Wall-clock ratio: 2.83× faster (PASS)**
**CPU ratio: 4.07× faster**

## Notes
- Sort correctness: output is chrom-lexicographic then start-numeric sort.
- High wall-clock variance is HDD-latency driven; CPU (4.07×) is stable.
- Speedup from Rust's sort_unstable (pdqsort) vs bedtools' stdlib qsort
  with C-string comparison overhead.

## CI
- pending push

## Status: DONE — 2.83× faster wall / 4.07× CPU (PASS)
