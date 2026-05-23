# rsomics-bam-import

Convert FASTQ reads to unaligned BAM. Rust port of `samtools import`.

## Usage

```
rsomics-bam-import [-0|-s|-1 R1 -2 R2] [-o out.bam] [-r RG_FIELDS] [file.fastq ...]
```

Single-end:
```
rsomics-bam-import -o out.bam reads.fastq
rsomics-bam-import -0 reads.fastq -o out.bam
```

Paired-end (separate files):
```
rsomics-bam-import -1 r1.fastq -2 r2.fastq -o out.bam
```

Interleaved paired (alternating R1/R2 in one file):
```
rsomics-bam-import -s interleaved.fastq -o out.bam
```

With read group:
```
rsomics-bam-import -0 reads.fastq -r 'ID:lib1\tSM:sample1\tPL:ILLUMINA' -o out.bam
rsomics-bam-import -0 reads.fastq -R lib1 -o out.bam   # minimal @RG with ID only
```

## Flags implemented

| Flag | Description |
|------|-------------|
| `-0`, `--single` | Single-ended reads (FLAG=4) |
| `-s`, `--interleaved` | Interleaved paired FASTQ |
| `-1`, `--read1` | Read-1 of paired run |
| `-2`, `--read2` | Read-2 of paired run |
| `-o`, `--output` | Output BAM file (default: stdout as SAM) |
| `-u`, `--uncompressed` | Write uncompressed BAM |
| `-r`, `--rg-line` | Complete @RG fields, tab-delimited (repeatable) |
| `-R`, `--rg-id` | Minimal @RG line with `ID:` only |
| `-t`, `--threads` | Worker threads for BGZF deflation |

## Flags scoped out (vs samtools import)

- `--i1`, `--i2`: Index/barcode FASTQ files and `BC`/`QT` aux tags
- `-i` / `--CASAVA`: Parse Illumina CASAVA read identifiers
- `-U` / `--UMI`: UMI extraction from read name
- `-N` / `--name2`: SRA-format second-field read name
- `-T`: Parse SAM-format tags from the FASTQ comment
- `--order TAG`: Record counter in a custom tag
- `--no-PG`: Suppress @PG header line (we never add a @PG line)
- `-O`, `--output-fmt`: Output format selection (we write BAM or SAM)
- `--input-fmt-option`, `--output-fmt-option`: htslib format options

## Origin

This crate is an independent Rust port of `samtools import` based on:
- Black-box behaviour testing against `samtools import` 1.23.1
- Reading the samtools `bam_import.c` source (MIT licence)

The core FLAG semantics (SE: 0x4; PE R1: 0x4d; PE R2: 0x8d; MAPQ=0 for
all unaligned reads) were established by black-box observation and confirmed
against the samtools source. The `bam_import.c` source (MIT) was read to
understand the read-name `/1`/`/2` stripping behaviour and the @RG line
construction.

No GPL code was used as reference.

License: MIT OR Apache-2.0  
Upstream credit: [samtools](https://github.com/samtools/samtools) (MIT)
