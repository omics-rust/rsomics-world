# rsomics-vcf-popgen perf gate

Date: 2026-05-29
Machine: mini_m2 (aarch64-apple-darwin, Apple M2)
rsomics-vcf-popgen: 0.1.0 (release build)
vcftools: 0.1.17 (/opt/homebrew/bin/vcftools)
Fixture: /tmp/vcf_popgen_200k.vcf (200k variants × 5 samples, 17 MB, seed 42)
Tool: hyperfine --warmup 2 --min-runs 5

## Operation
Allele frequency calculation: `freq` subcommand vs `vcftools --freq --stdout`.
Both output CHROM\tPOS\tN_ALLELES\tN_CHR\t{ALLELE:FREQ} tab-delimited.

## Results (200k-variant representative fixture)

| Side | Mean (ms) | σ (ms) | Min (ms) | Max (ms) |
|---|---|---|---|---|
| rsomics-vcf-popgen freq | 651.1 | 201.5 | 418.9 | 887.4 |
| vcftools --freq --stdout | 878.7 | 92.4 | 789.2 | 984.9 |

**Ratio: 1.35× faster than vcftools 0.1.17 — PASS** (>1.0× gate)

## Small fixture (10k variants, 890KB) for reference

| Side | Mean (ms) | σ (ms) |
|---|---|---|
| rsomics-vcf-popgen freq | 37.2 | 14.3 |
| vcftools --freq | 53.6 | 26.8 |

Ratio: 1.44× — consistent with large fixture.

## Notes
- VCF parsing: noodles-based streaming reader vs vcftools' C htslib reader
- rsomics-vcf-popgen uses single-pass streaming; vcftools also streams but with C overhead
- Ratio is modest (1.35×) — vcftools is written in C with htslib; headroom limited
- CPU: 316.9ms vs 530.8ms → 1.67× CPU win (user time); wall time limited by I/O (150ms sys)
- 4090 comparison: authoritative Linux x86_64 comparison pending

## Status: PASS (1.35× wall, 1.67× CPU on 200k-variant fixture)
