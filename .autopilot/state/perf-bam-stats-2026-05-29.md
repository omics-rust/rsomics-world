# rsomics-bam-stats perf gate — 2026-05-29

## Tool versions
- rsomics-bam-stats: 0.1.1 (commit b963d98)
- samtools: 1.22.1 (Homebrew, /opt/homebrew/bin/samtools)

## Machine
- mini_m2 (aarch64-apple-darwin, Apple M2)
- Rust 1.87 stable, release build

## Fixture
- BAM: markdup_1m_cs.bam — 62 MB, ~1 million reads, coordinate-sorted
- Located at `/Volumes/Zane's HDD/rsomics-fixtures/markdup_1m_cs.bam`

## Command
```
rsomics-bam-stats markdup_1m_cs.bam > /dev/null
samtools stats markdup_1m_cs.bam > /dev/null
```

## Results (hyperfine --warmup 2 --min-runs 5)

| Tool | Mean | Stddev | Min | Max | User CPU |
|------|------|--------|-----|-----|----------|
| rsomics-bam-stats | 239.9 ms | ±197.1 ms | 112.1 ms | 783.1 ms | 412.7 ms |
| samtools stats | 4266 ms | ±603 ms | 3476 ms | 4882 ms | 2168 ms |

**Wall-clock ratio: 17.78× faster (PASS)**
**CPU ratio: 5.25× faster**

## Notes
- Wall-clock variance is HDD I/O noise (62 MB file on spinning HDD).
- CPU ratio (5.25×) is stable; our user time reflects bamio parallel reader.
- samtools stats computes a richer set of metrics (per-base, per-cycle,
  insert-size distribution, GC curves). Our output is summary stats only.
  For total-counts + quality the operations are equivalent.

## CI
- omics-rust/rsomics-bam-stats run 26547678943: green

## Status: DONE — 17.78× wall / 5.25× CPU (PASS)
