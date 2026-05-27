# rsomics-vcf-roh

Detect runs of homozygosity (ROH) in VCF/BCF files using a 2-state Hidden Markov Model — a Rust
reimplementation of `bcftools roh`.

## Install

```sh
cargo install rsomics-vcf-roh
```

## Usage

```
rsomics-vcf-roh [OPTIONS] <INPUT.vcf[.gz]>
```

## Flags

| Flag | Default | Description |
|------|---------|-------------|
| `--AF-dflt <FLOAT>` | — | Default allele frequency when AF is unknown (skip site if not set) |
| `--AF-tag <TAG>` | — | INFO tag to read ALT allele frequency from (e.g. `AF`) |
| `-a, --hw-to-az <FLOAT>` | `6.7e-8` | P(AZ\|HW): transition probability from HW to autozygous state |
| `-H, --az-to-hw <FLOAT>` | `5e-9` | P(HW\|AZ): transition probability from AZ to HW state |
| `-G, --GTs-only <FLOAT>` | — | Use GT field only; FLOAT is the Phred-scaled error for non-called genotypes (e.g. 30) |
| `-I, --skip-indels` | off | Skip indels (SNPs only) |
| `-i, --ignore-homref` | off | Skip hom-ref genotypes (0/0) |
| `-O, --output-type <TYPE>` | `sr` | `s`=per-site ST lines, `r`=ROH region RG lines |
| `-o, --output <FILE>` | stdout | Output file |
| `-s, --samples <LIST>` | all | Comma-separated list of samples to analyze |
| `-q, --quiet` | off | Suppress progress messages |
| `-v, --verbosity <INT>` | — | Verbosity level |

## Output format

**RG lines** (regions, `-Or`):
```
RG  <sample>  <chrom>  <start>  <end>  <length_bp>  <n_markers>  <avg_phred_quality>
```

**ST lines** (sites, `-Os`):
```
ST  <sample>  <chrom>  <pos>  <state>  <phred_quality>
```
where state 0 = Hardy-Weinberg (normal) and 1 = autozygous (ROH).

## Examples

```sh
# Detect ROH regions with GT-only mode, default AF 0.4
rsomics-vcf-roh -G30 --AF-dflt 0.4 -Or input.vcf.gz

# Per-site state output using INFO/AF tag
rsomics-vcf-roh --AF-tag AF -Os input.vcf.gz

# Both regions and sites, subset to specific samples
rsomics-vcf-roh -G30 --AF-dflt 0.4 -s NA12878,NA12877 input.vcf.gz
```

## Algorithm

The tool implements a 2-state HMM with states:
- **HW** (Hardy-Weinberg): normal diploid, genotype frequencies follow Hardy-Weinberg equilibrium.
- **AZ** (autozygous/ROH): both haplotypes are identical by descent.

**Emission probabilities** per site with alternate allele frequency `f`:
- `P(data|HW) = (1-f)² P(data|RR) + 2f(1-f) P(data|RA) + f² P(data|AA)`
- `P(data|AZ) = (1-f) P(data|RR) + f P(data|AA)`

**Default transition probabilities** (matching bcftools):
- `P(AZ|HW) = 6.7e-8` (entering a ROH)
- `P(HW|AZ) = 5e-9` (leaving a ROH)

Viterbi decoding determines the most likely state sequence. Forward-backward
computes per-site posterior probabilities for quality scores, following the same
one-step-offset convention as bcftools (first-site quality reflects the initial
prior ≈ 3.0 Phred).

## Origin

This crate is an independent Rust reimplementation of `bcftools roh` (Narasimhan et al., 2016).
The algorithm was derived from reading the bcftools source (MIT license), which is permitted
for a clean MIT OR Apache-2.0 reimplementation.

Reference: Narasimhan V, Danecek P, Scally A, Xue Y, Tyler-Smith C, Durbin R.
BCFtools/RoH: a hidden Markov model approach for detecting autozygosity from next-generation
sequencing data. Bioinformatics. 2016 Jun 1;32(11):1749-51.
DOI: [10.1093/bioinformatics/btw044](https://doi.org/10.1093/bioinformatics/btw044)

License: MIT OR Apache-2.0.  
Upstream credit: [samtools/bcftools](https://github.com/samtools/bcftools) (MIT).
