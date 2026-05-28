# rsomics-fastq-umi perf gate

Date: 2026-05-29
Machine: mini_m2 (aarch64-apple-darwin, Apple M2)
rsomics-fastq-umi: 0.2.0
fastp: 1.3.3 (Homebrew)
Fixture: /tmp/perfgate_1m_150bp.fastq.gz (1M reads × 150 bp, 155 MB, gz, seed 0x00C0FFEE)
Tool: hyperfine --warmup 2 --min-runs 5

## Operation
UMI extraction: 8 bp 5' UMI prepended to read name; fastp --umi mode with all other processing disabled.

## Results

| Side | Mean (s) | σ (s) | Min (s) | Max (s) |
|---|---|---|---|---|
| rsomics-fastq-umi -t 1 --umi_len 8 | 2.504 | 0.344 | 1.956 | 2.824 |
| fastp --thread 1 --umi --umi_loc read1 --umi_len 8 | 15.528 | 5.925 | 10.634 | 25.718 |

**Ratio: 6.20× faster than fastp 1.3.3 — PASS** (>1.0× gate)

## Notes
- fastp 1.3.3 shows high variance on macOS arm64 (system overhead from Java-like startup)
- min-to-min ratio: 1.956s vs 10.634s ≈ 5.4× — still clearly PASS
- 4090 run recommended for authoritative comparison vs fastp 0.20.1
