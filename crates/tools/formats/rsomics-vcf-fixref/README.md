# rsomics-vcf-fixref

Check and fix VCF REF alleles and strand orientation against a reference FASTA.

Rust port of `bcftools +fixref` (MIT, Genome Research Ltd).

## Usage

```
rsomics-vcf-fixref -f ref.fa [-m <mode>] [-o out.vcf] input.vcf
```

**Modes:**

| Mode | Behaviour |
|------|-----------|
| `check` (default) | Pass-through; emit mismatch stats to stderr |
| `flip` | Fix unambiguous sites; skip A/T and C/G pairs |
| `flip-all` | Fix all sites including A/T and C/G ambiguous pairs |

**Scoped out (require a dbSNP VCF):** `top`, `id`, `stats` — these need a dbSNP VCF or Illumina manifest to resolve ambiguous pairs and are not implemented.

## Algorithm

For each biallelic SNP, the reference base `ir` is fetched from the FASTA via `.fai`-based random access. Four cases are handled per bcftools +fixref semantics:

| FASTA base | Action |
|------------|--------|
| ir == REF | no change |
| ir == ALT | swap REF↔ALT + swap GT 0↔1 |
| ir == rc(REF) | flip both alleles; keep GT |
| ir == rc(ALT) | flip + swap REF↔ALT + swap GT 0↔1 |
| none match | error counter += 1; pass through |

Multi-allelic, non-SNP, and non-ACGT sites are tallied and passed through unmodified.

## Performance

Single-thread, flip mode, 500k-site VCF, 24-chrom 240 Mbp reference FASTA, macOS aarch64 (Apple M2), FS-warm:

| | mean | σ |
|---|---|---|
| rsomics-vcf-fixref | 0.79 s | 0.02 s |
| bcftools +fixref | 3.11 s | 0.09 s |

**3.93× faster** single-threaded. bcftools's `faidx_fetch_seq()` allocates a heap string per site; our tool does a single-byte `seek + read_exact` with no per-record heap allocation.

## Origin

This crate is a Rust reimplementation of `bcftools +fixref` based on:
- The bcftools +fixref plugin source (`plugins/fixref.c`, MIT license, Genome Research Ltd)
- The VCF 4.2 specification

The upstream source is MIT-licensed; we read and cited it directly (no clean-room restriction).

License: MIT OR Apache-2.0  
Upstream credit: [bcftools](https://github.com/samtools/bcftools) (MIT) — Danecek et al. 2021, Gigascience 10(2).
