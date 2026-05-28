# perf-ld-matrix — 2026-05-29

## Tool
rsomics-ld-matrix 0.1.0

## Upstream reference
Pure-Python pairwise r² reference implementation (Python 3.12.7, aarch64-apple-darwin)
No CLI plink/vcftools available on mini_m2; Python reference is the baseline.

## Machine
mini_m2 — Apple M2, aarch64-apple-darwin, macOS 15.x

## Fixture
File: /tmp/perf_ld.vcf (500 biallelic SNPs, 50 diploid samples, 112KB)
Generator: Python seed=42, Hardy-Weinberg genotype frequencies

## Command
```
time rsomics-ld-matrix perf_ld.vcf > /dev/null
time python3 ld_ref.py perf_ld.vcf > /dev/null
```

## Results

| Command | Wall time |
|---|---|
| rsomics-ld-matrix | 0.072s |
| Python pairwise r² reference | 1.948s |

## Ratio
**~27× faster** than pure-Python pairwise LD computation — PASS (>1.0× gate met)

## Status
PASS
