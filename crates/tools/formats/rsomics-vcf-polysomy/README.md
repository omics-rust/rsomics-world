# rsomics-vcf-polysomy

Estimate per-chromosome copy number from B-allele frequency (BAF) distributions
in a single-sample VCF. Rust port of `bcftools polysomy`.

## Install

```
cargo install rsomics-vcf-polysomy
```

## Usage

```
rsomics-vcf-polysomy -o <output-dir> [OPTIONS] <INPUT.vcf[.gz]>
```

Results are written to `<output-dir>/dist.dat` with three record types:

- `DIST` — per-bin normalised BAF histogram
- `FIT` — fitted Gaussian mixture function for each CN model
- `CN` — per-chromosome copy-number estimate and goodness-of-fit

## Options

| Flag | Long | Default | Description |
|------|------|---------|-------------|
| `-o` | `--output-dir` | *(required)* | Output directory |
| `-s` | `--sample` | *(auto)* | Sample name (required for multi-sample VCF) |
| `-f` | `--fit-th` | 3.3 | Goodness-of-fit threshold (higher = more permissive) |
| `-c` | `--cn-penalty` | 0.7 | Penalty for increasing copy number (0–1) |
|      | `--peak-symmetry` | 0.5 | Peak symmetry threshold (0–1) |
| `-b` | `--peak-size` | 0.1 | Minimum peak size fraction |
| `-m` | `--min-fraction` | 0.1 | Minimum detectable aberrant-cell fraction |
| `-i` | `--include-aa` | off | Include AA peak in CN2/CN3 evaluation |
| `-q` | `--quiet` | off | Suppress progress output |
| `-v` | `--verbose` | off | Verbose output |

## Examples

```bash
# Estimate CN for a single-sample VCF
rsomics-vcf-polysomy -o outdir sample.vcf.gz

# Analyse one sample from a multi-sample VCF
rsomics-vcf-polysomy -s NA12878 -o outdir cohort.vcf.gz

# Stricter fit threshold
rsomics-vcf-polysomy -o outdir -f 2.0 sample.vcf.gz
```

## Algorithm

1. Build a 150-bin BAF histogram from `FORMAT/BAF` values.
2. Smooth with a centred moving-average (window=7, mirrors bcftools `-S -3`).
3. Locate RR/RA/AA boundaries: `irr` = argmin in [0, n/2), `iaa` = argmin in [n/2, n).
4. Classify trivially as CN=1 (RA absent) or CN=-1 (ambiguous ratio).
5. Otherwise fit CN2, CN3, and CN4 Gaussian mixture models using
   Levenberg-Marquardt with 50 Monte Carlo restarts per model.
6. Select the best model using a CN-penalty tiebreaker (default 0.7).

## Performance

On a 300 k-record VCF (3 chromosomes, 13.7 MB, macOS aarch64):

| Tool | Mean ± σ | Ratio |
|------|----------|-------|
| rsomics-vcf-polysomy 0.1.0 | 435.8 ms ± 8.2 ms | **5.88×** |
| bcftools polysomy 1.23.1 | 2564 ms ± 32 ms | 1.00× |

See `.autopilot/state/perf-vcf-polysomy-2026-05-27.md` for full provenance.

## Origin

This crate is an independent Rust reimplementation of `bcftools polysomy` based on:
- Reading the MIT-licensed bcftools source: `polysomy.c`, `peakfit.c`
  (samtools/bcftools, https://github.com/samtools/bcftools)
- The bcftools/htslib format specification

The upstream source is MIT-licensed and was used as an implementation reference.
Our Levenberg-Marquardt solver and LCG Monte Carlo are independent Rust
implementations that produce equivalent copy-number decisions while achieving
better per-chromosome wall-clock performance through avoiding GSL overhead.

License: MIT OR Apache-2.0.  
Upstream credit: bcftools polysomy (https://github.com/samtools/bcftools), MIT license.
