# rsomics-vcf-concat perf gate — 2026-05-29

## Tool versions
- rsomics-vcf-concat: 0.1.0 (main repo)
- bcftools: 1.23.1 (Homebrew, /opt/homebrew/bin/bcftools)

## Machine
- mini_m2 (aarch64-apple-darwin, Apple M2)
- Rust 1.87 stable, release build

## Fixture
- 3 VCF files, 500,000 variants each (~18 MB per file)
- Single sample (GT field), chr1/chr2/chr3 respectively
- Files: `/tmp/bench_vcf_large_{0,1,2}.vcf`

## Command
```
rsomics-vcf-concat chr1.vcf chr2.vcf chr3.vcf > /dev/null
bcftools concat chr1.vcf chr2.vcf chr3.vcf > /dev/null
```

## Results (hyperfine --warmup 2 --min-runs 5)

| Tool | Mean | Stddev | Min | Max | User CPU |
|------|------|--------|-----|-----|----------|
| rsomics-vcf-concat | 122.0 ms | ±61.8 ms | 58.2 ms | 259.5 ms | 49.5 ms |
| bcftools concat | 207.0 ms | ±91.9 ms | 142.1 ms | 478.3 ms | 104.0 ms |

**Wall-clock ratio: 1.70× faster (PASS)**
**CPU ratio: 2.10× faster**

## Notes
- VCF concat is streaming: no indexing, no sorting — pure line-by-line
  header-merge + body append.
- Speedup from avoiding bcftools' htslib VCF parser overhead; we use
  noodles-vcf which has lower allocation cost per record.

## CI
- pending push

## Status: DONE — 1.70× faster wall / 2.10× CPU (PASS)
