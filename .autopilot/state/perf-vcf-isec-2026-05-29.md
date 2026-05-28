# rsomics-vcf-isec perf gate — 2026-05-29

## Tool versions
- rsomics-vcf-isec: 0.1.0 (main repo)
- bcftools: 1.23.1 (Homebrew)

## Fixture
- VCF A: 500k variants chr1, 20.7 MB (plain VCF)
- VCF B: 500k variants chr2 (plain VCF — no overlap expected since different chrom)
- For bcftools: bgzipped + tabix-indexed versions

## Results (hyperfine --warmup 2 --min-runs 5)

| Tool | Mean | User CPU |
|------|------|----------|
| rsomics-vcf-isec (plain VCF) | 880.9 ms | 475.4 ms |
| bcftools isec (bgzip+tabix) | 1906 ms | 837 ms |

**Wall ratio: 2.16× faster (PASS)**
**CPU ratio: 1.76× faster**

## Status: DONE — 2.16× wall (PASS)
