# rsomics-fastx-convert

Convert between FASTA and FASTQ formats: `fq2fa`, `fa2fq`, `fx2tab`, `tab2fx`, and Phred quality re-encoding.

## Usage

```
rsomics-fastx-convert <SUBCOMMAND> [OPTIONS]
```

### Subcommands

| Subcommand | Description |
|------------|-------------|
| `fq2fa`    | Convert FASTQ → FASTA (strip quality scores) |
| `fa2fq`    | Convert FASTA → FASTQ (add dummy Phred+33 Q40 quality) |
| `fx2tab`   | Convert FASTA/FASTQ → tab-separated (name, seq[, qual]) |
| `tab2fx`   | Convert tab-separated back to FASTA or FASTQ |
| `phred`    | Re-encode quality scores between Phred+33 and Phred+64 |

All subcommands accept `--input` and `--output` (default `-` for stdin/stdout).

### Examples

```bash
# Strip quality from FASTQ
rsomics-fastx-convert fq2fa --input reads.fq --output reads.fa

# Add dummy quality to FASTA
rsomics-fastx-convert fa2fq --input ref.fa > ref.fq

# Tabulate FASTQ
rsomics-fastx-convert fx2tab --input reads.fq | awk '{print $1, length($2)}'

# Re-encode old Illumina 1.3 (Phred+64) to Sanger (Phred+33)
rsomics-fastx-convert phred --from 64 --input old.fq --output new.fq
```

## Install

```bash
cargo install rsomics-fastx-convert
```

## Origin

This crate is an independent Rust reimplementation of seqkit's format-conversion
subcommands (`fq2fa`, `fa2fq`, `fx2tab`, `tab2fx`, `convert`) based on:

- The FASTA/FASTQ format specification (https://maq.sourceforge.net/fastq.shtml)
- Phred quality encoding documentation (Cock et al. 2010, Nucleic Acids Research)
- Black-box behaviour testing against seqkit 2.x

No source code from the GPL-incompatible upstream was used as reference during
implementation. Test fixtures are independently generated.

License: MIT OR Apache-2.0.
Upstream credit: seqkit (https://github.com/shenwei356/seqkit) (MIT).
