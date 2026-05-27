# rsomics-bam-phase

Phase heterozygous SNPs from coordinate-sorted aligned reads — a Rust port of
`samtools phase`. Detects het sites by allele-count LOD scoring, phases contiguous
het-site blocks with a sliding-window DP, detects chimeric fragments, and writes
a text report (`PS`/`FL`/`M`/`//` lines) to stdout. Optionally splits the input
BAM into haplotype-0, haplotype-1, and chimera files.

## Install

```sh
cargo install rsomics-bam-phase
```

## Usage

```sh
# Write phase report to stdout
rsomics-bam-phase sorted.bam

# Phase and split reads into haplotype BAMs
rsomics-bam-phase sorted.bam -b hap

# Lower LOD threshold for low-coverage data
rsomics-bam-phase sorted.bam -q 20

# Disable chimera detection
rsomics-bam-phase sorted.bam -F
```

## Output format

The text output (stdout) mirrors `samtools phase`:

| Line type | Meaning |
|-----------|---------|
| `CC ...`  | Column legend (header) |
| `PS chr start end` | Phase-set bounds (1-based) |
| `FL chr start end` | Masked/filtered region |
| `M1 chr ps pos a0 a1 idx #s0 #e0 #s1 #e1` | Unmasked het marker |
| `M2 ...`  | Masked het marker |
| `//`      | Phase-set terminator |

## Flag table

| Flag | Default | Meaning |
|------|---------|---------|
| `-k, --window INT` | 13 | DP window length for local haplotype states |
| `-b, --bam-prefix STR` | — | BAM output prefix (`<prefix>.0.bam`, `.1.bam`, `.chimera.bam`) |
| `-q, --min-lod INT` | 37 | Minimum het phred-LOD to call a site heterozygous |
| `-Q, --min-bq INT` | 13 | Minimum base quality |
| `-D, --max-depth INT` | 256 | Skip pileup sites with depth > this value |
| `-F, --no-fix-chimera` | off | Disable chimeric-fragment detection and repair |
| `-A, --drop-ambiguous` | off | Route ambiguously phased reads to chimera output |
| `-t, --threads INT` | all | I/O worker threads |
| `--json` | off | Emit stats JSON on stderr |

## BAM tags written (when `-b`)

| Tag | Type | Meaning |
|-----|------|---------|
| `YP` | i32 | Haplotype assignment (0 or 1) |
| `YF` | i32 | 1 if read was identified as chimeric and flipped |
| `YI` | i32 | In-phase allele count |
| `YO` | i32 | Out-of-phase allele count |
| `YS` | i32 | Phase-set start position (1-based) |

## Origin

This crate is an independent Rust reimplementation of `samtools phase` based on:
- The `samtools phase` MIT-licensed source (`phase.c`, samtools 1.23.1)
- The SAM/BAM format specification (SAMv1)
- Black-box behavior testing against `samtools phase`

Algorithm constants extracted from `phase.c`:
- `MAX_VARS = 256` — maximum variant allele calls per fragment
- `FLIP_PENALTY = 2` — chimera-flip boundary cost
- `FLIP_THRES = 4` — minimum improvement to accept a chimera flip
- `MASK_THRES = 3` — minimum phased-read count to leave a marker unmasked
- Default `-k 13`, `-q 37`, `-Q 13`, `-D 256` thresholds

License: MIT OR Apache-2.0.
Upstream credit: [samtools](https://github.com/samtools/samtools) (MIT).
