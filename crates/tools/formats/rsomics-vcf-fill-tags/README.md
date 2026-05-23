# rsomics-vcf-fill-tags

Recompute VCF INFO population-genetics tags from FORMAT/GT across all samples.
Drop-in Rust replacement for `bcftools +fill-tags`.

## Tags computed

| Tag | Number | Type | Description |
|-----|--------|------|-------------|
| `AN` | 1 | Integer | Total allele copies in called genotypes |
| `AC` | A | Integer | Alt allele count (per ALT, same order) |
| `AF` | A | Float | Allele frequency |
| `MAF` | 1 | Float | Minor allele frequency |
| `NS` | 1 | Integer | Samples with data |
| `AC_Hom` | A | Integer | Allele copies in homozygous genotypes |
| `AC_Het` | A | Integer | Allele copies in heterozygous genotypes |
| `AC_Hemi` | A | Integer | Allele copies in hemizygous genotypes |
| `HWE` | A | Float | Hardy-Weinberg exact p-value (Wigginton 2005) |
| `ExcHet` | A | Float | Excess-heterozygosity one-tailed p-value |

Custom expression tags (`TAG=func(FMT/TAG)`) are not implemented.

## Usage

```
rsomics-vcf-fill-tags input.vcf > annotated.vcf
rsomics-vcf-fill-tags --tags AN,AC,AF input.vcf -o annotated.vcf
```

## Performance

142 MB VCF, 200 samples × 100 k variants, Apple M2 (8 cores):

| tool | mean (s) |
|------|----------|
| `rsomics-vcf-fill-tags` (parallel, 8 cores) | 1.02 |
| `bcftools +fill-tags 1.23.1` (single-threaded) | 2.52 |

**2.48× faster** wall-clock. Records processed in parallel via rayon; output order preserved.

## Origin

This crate is an independent Rust reimplementation of `bcftools +fill-tags` based on:
- The bcftools fill-tags plugin source (`plugins/fill-tags.c`, MIT licence) — read for
  exact semantics (tag types, multiallelic handling, HWE/ExcHet formulae).
- The Wigginton et al. 2005 HWE exact test (PMID:15789306).
- Black-box compatibility testing against `bcftools +fill-tags 1.23.1`.

The HWE recursion (`rsomics-stats::hwe_exact`) is ported from the bcftools
`calc_hwe()` function. The bcftools source is MIT-licenced, making direct
algorithmic reference compatible with this crate's MIT OR Apache-2.0 licence.

License: MIT OR Apache-2.0.
Upstream credit: [bcftools](https://github.com/samtools/bcftools) (MIT).
