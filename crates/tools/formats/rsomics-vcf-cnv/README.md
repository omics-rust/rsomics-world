# rsomics-vcf-cnv

HMM-based copy-number variation (CNV) caller from B-allele frequency (BAF) and Log R Ratio (LRR)
signals stored in a single-sample VCF — a Rust port of `bcftools cnv`.

## Install

```
cargo install rsomics-vcf-cnv
```

## Usage

```
rsomics-vcf-cnv [OPTIONS] <INPUT.vcf[.gz]>
```

The input VCF must have `FORMAT/BAF` (float) and `FORMAT/LRR` (float) per-sample fields, as
produced by array-processing pipelines (Illumina GenomeStudio, etc.).

Output is written to the directory specified by `-o` (default: current directory):
- `cn.<sample>.tab` — per-site copy-number state and posterior probabilities
- `summary.<sample>.tab` — per-region summary with CN state, quality, site count, HET count

## Flag reference

| Short | Long | Default | Description |
|-------|------|---------|-------------|
| `-s` | `--sample` | (auto) | Query sample name; required for multi-sample VCF |
| `-o` | `--output-dir` | `.` | Output directory |
| `-f` | `--AF-file` | — | Allele frequency file (CHR TAB POS TAB REF,ALT TAB AF) |
| `-a` | `--aberrant` | `1.0` | Fraction of aberrant cells |
| `-b` | `--BAF-weight` | `1.0` | Relative weight of BAF evidence |
| `-d` | `--BAF-dev` | `0.04` | Expected BAF standard deviation |
| `-e` | `--err-prob` | `1e-4` | Uniform measurement error floor |
| `-k` | `--LRR-dev` | `0.2` | Expected LRR standard deviation |
| `-l` | `--LRR-weight` | `0.2` | Relative weight of LRR evidence |
| `-L` | `--LRR-smooth-win` | `10` | LRR moving-average smoothing window size |
| `-x` | `--xy-prob` | `1e-9` | HMM off-diagonal transition probability |
| | `--summary-only` | — | Skip per-site cn.tab; write only summary.tab |
| `-t` | `--threads` | (auto) | Worker threads |
| `-q` | `--quiet` | — | Suppress stderr progress |

## Origin

This crate is an independent Rust reimplementation of `bcftools cnv` based on:

- The public file-format spec (VCF 4.2, FORMAT/BAF and FORMAT/LRR fields)
- The bcftools source code (MIT license) — specifically `vcfcnv.c` in the
  [samtools/bcftools](https://github.com/samtools/bcftools) repository

The bcftools source is MIT-licensed, so direct algorithm reading and citing is permitted.
Algorithm constants transcribed from source: LRR means (-0.45, 0.00, +0.30 for CN1/CN2/CN3),
default genotype frequencies (fRR=0.76, fRA=0.14, fAA=0.098), default transition probability
1e-9, BAF σ=0.04, LRR σ=0.2.

License: MIT OR Apache-2.0.
Upstream credit: bcftools cnv <https://github.com/samtools/bcftools> (MIT).
