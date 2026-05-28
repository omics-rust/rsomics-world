# perf-bed-multicov-2026-05-29

## Tool: rsomics-bed-multicov 0.1.0

## Machine
- Host: mini_m2 (Apple M2, arm64, macOS)
- CPU: Apple M2 (arm64)

## Tool versions
- rsomics-bed-multicov: 0.1.0 (commit d48f9d88 in main repo submodule)
- bedtools: v2.31.1

## Fixture
- BAM: rnaseq_perf_100k.bam (100k reads, chr1+chr2, coordinate-sorted+indexed)
- BED: multicov_perf.bed (1000 regions across chr1+chr2, 1–50kb each, seed=42)

## Results (hyperfine --warmup 2 --runs 10)

| Tool | Mean | σ | Min | Max |
|------|------|---|-----|-----|
| rsomics-bed-multicov | 98.1 ms | 27.5 ms | 66.1 ms | 152.1 ms |
| bedtools multicov | 256.6 ms | 125.0 ms | 158.8 ms | 576.9 ms |

**Ratio: 2.62× faster than bedtools multicov**

## Correctness
- diff /tmp/ours_multicov.tsv /tmp/bt_multicov.tsv → identical (1000 lines)

## PASS
> 1.0× gate: PASS (2.62×)
