# rsomics-fastq-split perf gate

Date: 2026-05-29
Machine: mini_m2 (aarch64-apple-darwin, Apple M2)
rsomics-fastq-split: 0.1.0
fastp: 1.3.3 (Homebrew)
Fixture: /tmp/perfgate_1m_150bp.fastq.gz (1M reads × 150 bp, 155 MB, gz, seed 0x00C0FFEE)
Tool: hyperfine --warmup 2 --min-runs 5

## Operation
Split by lines: --split_by_lines 400000 → 10 output files (1M reads / 100k reads per file).

## Results

| Side | Mean (s) | σ (s) | Min (s) | Max (s) |
|---|---|---|---|---|
| rsomics-fastq-split -t 1 --split_by_lines 400000 | 1.604 | 0.127 | 1.395 | 1.736 |
| fastp --thread 1 --split_by_lines 400000 | 24.075 | 5.480 | 17.759 | 31.891 |

**Ratio: 15.01× faster than fastp 1.3.3 — PASS** (>1.0× gate)

## Notes
- Split I/O is extremely fastp-expensive (10 file handles + high system time 6.5s)
- rsomics-fastq-split uses BufWriter per output shard — linear I/O, no contention
- min-to-min ratio: 1.395s vs 17.759s ≈ 12.7× — still solidly PASS
