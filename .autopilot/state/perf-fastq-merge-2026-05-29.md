# rsomics-fastq-merge perf gate

Date: 2026-05-29
Machine: mini_m2 (aarch64-apple-darwin, Apple M2)
rsomics-fastq-merge: 0.1.0
fastp: 1.3.3 (Homebrew)
Fixture: /tmp/perfgate_merge_r1.fastq + /tmp/perfgate_merge_r2.fastq
  (1M reads × 150 bp each, plain FASTQ, ~313 MB per file, seeds 42/43)
Tool: hyperfine --warmup 2 --min-runs 5

## Operation
Paired-end overlap merge: reads overlapping by ≥30 bp (≤5 mismatches, ≤20%) → merged output.
fastp run with all non-merge processing disabled.

## Results

| Side | Mean (s) | σ (s) | Min (s) | Max (s) |
|---|---|---|---|---|
| rsomics-fastq-merge --in1 R1 --in2 R2 -m OUT | 9.361 | 0.399 | 8.949 | 10.027 |
| fastp --merge --thread 1 | 54.939 | 28.292 | 36.022 | 105.033 |

**Ratio: 5.87× faster than fastp 1.3.3 — PASS** (>1.0× gate)

## Notes
- fastp 1.3.3 shows extreme variance on macOS arm64 for merge (system time 12.7s = high I/O cost)
- min-to-min ratio: 8.949s vs 36.022s ≈ 4.0× — still clearly PASS
- Merge is the algorithmically most expensive fastq op; our Smith-Waterman overlap path via rsomics-align-core
- 4090 run recommended for authoritative comparison vs fastp 0.20.1 with real PE data
