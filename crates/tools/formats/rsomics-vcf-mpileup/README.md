# rsomics-vcf-mpileup

VCF-emitting pileup with per-position genotype likelihoods from a single BAM file —
a Rust port of `bcftools mpileup` (single-sample SNP mode).

## Install

```bash
cargo install rsomics-vcf-mpileup
```

## Usage

```
rsomics-vcf-mpileup [OPTIONS] INPUT.bam

Options:
  -f, --fasta-ref <FILE>   Faidx-indexed reference FASTA (required)
  -r, --region <REG>       Restrict to region ("chrom" or "chrom:start-end", 1-based)
  -a, --annotate <LIST>    Extra FORMAT/INFO tags (no-op placeholder; AD and DP always emitted)
  -q, --min-MQ <INT>       Skip alignments with MAPQ < INT [default: 0]
  -Q, --min-BQ <INT>       Skip bases with base quality < INT [default: 1]
  -t, --threads <INT>      BGZF inflate worker threads [default: available CPUs]
      --json               Emit structured JSON progress (rsomics-common)
  -h, --help               Print help
  -V, --version            Print version
```

Writes an uncompressed VCF to stdout. Each variant position emits:
- `FORMAT` fields: `PL` (Phred-scaled genotype likelihoods), `DP` (depth), `AD` (allele depth)
- `INFO/DP`: raw read depth
- Biallelic SNPs only; REF and best ALT by count

Defaults match `bcftools mpileup 1.21`:
- Min MAPQ = 0, min base quality = 1, max depth = 250
- Reads filtered: UNMAP | SECONDARY | QCFAIL | DUP (0x704)
- Overlap-mate quality removal enabled
- Effective quality cap: min(BQ, 60, MAPQ, 60, 63), floor 4

## Not implemented in 0.1.0

The following `bcftools mpileup` features are explicitly deferred:

- **Multi-sample** input (multiple BAMs)
- **Indel** calling and indel-adjusted likelihoods
- **Multi-allelic** site handling (3+ alleles)
- **BAQ** (base alignment quality) realignment
- **BED-based** region lists (`-l`)
- **Depth-of-coverage** streaming output (`-d` per-position)
- **INFO/I16** and per-sample **RMS** fields
- **Strand bias** and Fisher's exact test tags
- **Ploidy** other than diploid
- **Reference-only positions**: bcftools emits all covered positions with `ALT=<*>`;
  we emit only positions with at least one observed non-reference base
- **BAM index jump** (`-r` uses a streaming filter; index is not required)

## Origin

This crate is an independent Rust reimplementation of `bcftools mpileup` based on:

- The bcftools 1.21 source (`mpileup.c`, `bam2bcf.c`) — MIT-licensed; read for algorithm details
  and default thresholds
- htslib 1.21 `errmod.c` — the revised MAQ error model (tables fk / beta / lhet and the
  `errmod_cal` accumulation algorithm); MIT-licensed
- The VCF 4.2 format specification for PL vector ordering (lexicographic `(i,j)` pairs, `i ≤ j`)

Upstream: [bcftools](https://github.com/samtools/bcftools) (MIT / BSD-3)
htslib: [htslib](https://github.com/samtools/htslib) (MIT / BSD-3)

License: MIT OR Apache-2.0
