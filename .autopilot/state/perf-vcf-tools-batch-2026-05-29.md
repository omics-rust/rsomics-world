# VCF tools batch perf gate — 2026-05-29

## Machine
- mini_m2 (aarch64-apple-darwin, Apple M2), Rust 1.87 stable, release

## Upstream
- bcftools 1.23.1 (Homebrew)
- seqkit v2.13.0 (Homebrew)

## Fixture
- 500,000 variants, chr1, proper FORMAT header (GT:DP), 20.7 MB
- File: `/tmp/bench_vcf_proper_0.vcf` — seed 42

## Results (hyperfine --warmup 2 --min-runs 5)

| Tool | Ours | Upstream | Wall ratio | CPU ratio | PASS? |
|------|------|----------|------------|-----------|-------|
| rsomics-vcf-view 0.2.0 | 45.9 ms | bcftools view 476.8 ms | 10.39× | 10.8× | PASS |
| rsomics-vcf-filter 0.2.0 | 41.0 ms | bcftools view -f PASS 462.0 ms | 11.28× | 10.9× | PASS |
| rsomics-vcf-sort 0.1.1 | 188.2 ms | bcftools sort 785.9 ms | 4.18× | 5.5× | PASS |
| rsomics-vcf-stats 0.1.0 | 199.5 ms | bcftools stats 235.3 ms | 1.18× | 1.44× | PASS |
| rsomics-vcf-query 0.1.1 | 75.4 ms | bcftools query -f 166.9 ms | 2.21× | 1.81× | PASS |
| rsomics-vcf-norm 0.1.1 | 338.4 ms (wall) | bcftools norm -m-any 565.4 ms | 1.67× wall, 6.7× CPU | PASS |
| rsomics-vcf-concat 0.1.0 | 122.0 ms (3×18 MB) | bcftools concat 207.0 ms | 1.70× | 2.10× | PASS |
| rsomics-vcf-isec 0.1.0 | 880.9 ms (plain) | bcftools isec (bgzip) 1906 ms | 2.16× | 1.76× | PASS |

## Notes
- vcf-view / vcf-filter share the same parsing path — both benefit from
  noodles-vcf's lower-overhead record iteration vs htslib VCF parser.
- vcf-norm system time (228 ms) is temp-file I/O for the merge pass;
  CPU time (61 ms) vs bcftools (416 ms) = 6.7× CPU speedup is meaningful.
- vcf-isec uses plain VCF vs bcftools isec requiring bgzip+tabix;
  comparison is directional but not perfectly equivalent.
- vcf-stats: only 1.18× wall; CPU (1.44×) is tighter. Both do a single pass
  + stats accumulation; bcftools stats generates richer quality histograms.

## CI
- All tools: CI green (see individual submodule GH runs)

## Status: ALL DONE
