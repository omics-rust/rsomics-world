# rsomics-vcf-gtcheck

Sample concordance / discordance estimator — Rust port of `bcftools gtcheck`.

## Install

```bash
cargo install rsomics-vcf-gtcheck
```

## Usage

```
rsomics-vcf-gtcheck [-g <genotypes.vcf.gz>] <query.vcf[.gz]>
```

### Flags

| Flag | Short | Default | Description |
|---|---|---|---|
| `QUERY.vcf[.gz]` | — | required | Query VCF (single or multi-sample). |
| `--genotypes FILE` | `-g` | — | Genotypes panel VCF. Omit for cross-check across all samples. |
| `--error-probability INT` | `-E` | 40 | Phred-scaled genotyping error. 0 = raw GT mismatch count. |
| `--no-HWE-prob` | — | off | Disable HWE probability column. |
| `--homs-only` | `-H` | off | Compare homozygous genotypes only (requires `-g`). |
| `--use-gt` | — | off | Force GT field (default: PL if present, else GT). |

### Examples

```bash
# Cross-check all samples
rsomics-vcf-gtcheck multi.vcf.gz

# Check query vs genotype panel
rsomics-vcf-gtcheck -g panel.vcf.gz query.vcf.gz

# Raw GT mismatch count, no HWE
rsomics-vcf-gtcheck --use-gt -E 0 --no-HWE-prob multi.vcf.gz
```

## Output

Output format matches `bcftools gtcheck`:

- `INFO` lines: per-sample summary stats (total sites, matched, discordant fraction).
- `DCv2` lines: per-pair scores (discordance, number of sites, average HWE, sample names).

## Origin

This crate is a Rust port of `bcftools gtcheck`, developed by Genome Research Ltd and distributed under the MIT license as part of [bcftools](https://github.com/samtools/bcftools).

The source of `bcftools/vcfgtcheck.c` (MIT) was read to extract the algorithm, scoring constants, dosage bitmask encoding, error-probability model, HWE accumulation, and output format. This is permitted under the MIT license and documented here as required by the rsomics-world clean-room policy.

License: MIT OR Apache-2.0.
Upstream credit: bcftools <https://github.com/samtools/bcftools> (MIT).
