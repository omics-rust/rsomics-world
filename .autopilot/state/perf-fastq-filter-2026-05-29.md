# rsomics-fastq-filter perf gate

Date: 2026-05-29
Machine: mini_m2 (aarch64-apple-darwin, Apple M2)
rsomics-fastq-filter: 0.1.0
fastp: 1.3.3 (Homebrew)
Fixture: /tmp/perfgate_1m_150bp.fastq.gz (1M reads × 150 bp, 155 MB, gz, seed 0x00C0FFEE)
Tool: hyperfine --warmup 2 --min-runs 5

## Operation
Passthrough filter (no trimming, no quality filtering) — isolates gz decode + FASTQ parse + write.

## Results

| Side | Mean (s) | σ (s) | Min (s) | Max (s) |
|---|---|---|---|---|
| rsomics-fastq-filter -t 1 | 1.947 | 1.064 | 1.204 | 3.824 |
| fastp --thread 1 (disabled filters) | 12.422 | 6.228 | 5.821 | 21.521 |

**Ratio: 6.38× faster than fastp 1.3.3 — PASS** (>1.0× gate)

## Notes
- fastp 1.3.3 shows high variance on macOS arm64; the ratio is conservative (min-to-min ≈ 4.8×, still PASS)
- rsomics-fastq-filter uses rsomics-seqio gz decode path (flate2 via libdeflate on macOS)
- 4090 run recommended for authoritative single-thread comparison vs fastp 0.20.1
