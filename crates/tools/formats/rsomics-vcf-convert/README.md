# rsomics-vcf-convert

Convert between VCF text, bgzipped VCF, and HAP/LEGEND/SAMPLE format — Rust port of `bcftools convert`.

## Install

```sh
cargo install rsomics-vcf-convert
```

## Usage

```
rsomics-vcf-convert [OPTIONS] <input>
```

## Flags

| Flag | Long | Default | Description |
|------|------|---------|-------------|
| `-O` | `--output-type` | `v` | Output type: `v` (VCF text), `z`/`z0`–`z9` (bgzipped VCF) |
| `-o` | `--output` | stdout | Output file path |
| `-h` | `--haplegendsample` | — | Export to IMPUTE2 format: `<PREFIX>` or `<HAP>,<LEGEND>,<SAMPLE>` |
| `-t` | `--threads` | 1 | Worker threads |
| `-q` | `--quiet` | off | Suppress progress messages |
| | `--json` | off | Emit JSON result summary |

## Examples

```sh
# Compress plain VCF to bgzipped VCF
rsomics-vcf-convert -O z input.vcf -o output.vcf.gz

# Decompress VCF.gz to plain VCF
rsomics-vcf-convert -O v input.vcf.gz -o output.vcf

# Round-trip (compress then decompress)
rsomics-vcf-convert -O z input.vcf | rsomics-vcf-convert -O v /dev/stdin

# Export to IMPUTE2 HAP/LEGEND/SAMPLE with prefix
# (bcftools uses -h; we use --haplegendsample because -h is reserved for help)
rsomics-vcf-convert --haplegendsample chr22 input.vcf.gz
# produces chr22.hap, chr22.legend, chr22.samples

# Export with explicit file names
rsomics-vcf-convert --haplegendsample out.hap,out.legend,out.samples input.vcf
```

## Not implemented in 0.1.0

The following `bcftools convert` subcommands / flags are **not** implemented. The binary refuses them with a clear error message:

| Feature | bcftools flag | Status |
|---------|--------------|--------|
| BCF binary input/output | `-O b`, `-O u` | Deferred — requires BCF parser |
| GEN/SAMPLE (IMPUTE2) | `-g`/`-G`/`--gensample`/`--gensample2vcf` | Deferred |
| HAP/SAMPLE (SHAPEIT) | `--hapsample`/`--hapsample2vcf` | Deferred |
| HAP/LEGEND/SAMPLE → VCF | `-H`/`--haplegendsample2vcf` | Deferred |
| gVCF expansion | `--gvcf2vcf` | Deferred |
| TSV → VCF | `--tsv2vcf` | Deferred |
| Expression filters | `-i`/`-e` | Deferred |
| Region/target subsetting | `-r`/`-R`/`-t`/`-T` | Deferred |
| Sample subsetting | `-s`/`-S` | Deferred |
| Index writing | `-W` | Deferred |

## Origin

This crate is an independent Rust reimplementation of `bcftools convert` based on:

- The public VCF/BCF format specification (<https://samtools.github.io/hts-specs/>)
- The IMPUTE2 hap/legend/sample format description  
  (<https://mathgen.stats.ox.ac.uk/impute/impute_v2.html>)
- Reading the MIT-licensed `bcftools` source (`vcfconvert.c`, Petr Danecek et al., Genome Research Ltd.)
- Black-box testing against `bcftools convert` 1.23.1

No GPL source was used. License: MIT OR Apache-2.0.  
Upstream credit: bcftools (<https://github.com/samtools/bcftools>) — MIT License.
