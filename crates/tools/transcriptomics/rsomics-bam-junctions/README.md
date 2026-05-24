# rsomics-bam-junctions

Annotate splice junctions from spliced BAM reads against a BED12 gene model.
Rust port of `RSeQC` `junction_annotation.py`.

## Usage

```
rsomics-bam-junctions -i <input.bam> -r <genes.bed12> [OPTIONS]
```

### Options

| Flag | Default | Description |
|------|---------|-------------|
| `-i`, `--input` | required | Input BAM file |
| `-r`, `--refgene` | required | Reference gene model in BED12 format |
| `-m`, `--min-intron` | `50` | Minimum intron length (bp) |
| `--mapq` | `30` | Minimum MAPQ for a read to be considered |
| `-t`, `--threads` | auto | BGZF decompression threads |
| `--json` | off | Emit JSON instead of text |

## Output

**stdout**: `total = N` (total N-op occurrences seen, including filtered ones).

**stderr**: Two summary blocks separated by `===` lines:
1. Splicing **events** (per-read occurrences): total, known, partial novel, novel, filtered.
2. Splicing **junctions** (distinct): total, known, partial novel, novel.

### Classification

Each `(chrom, intron_start, intron_end)` is classified against the BED12 intron set
derived from consecutive exon boundaries:

- **known** — both donor (intron start) and acceptor (intron end) are present in the BED12.
- **partial_novel** — one site is known, the other is not.
- **complete_novel** — neither site appears in any transcript.

## Origin

This crate is an independent Rust reimplementation based on:
- `RSeQC`: `junction_annotation.py` (LGPL-2.1+), Wang et al. 2012
  <https://doi.org/10.1093/bioinformatics/bts356>
- The SAM/BAM format specification
- BED12 format specification
- Black-box behaviour testing against `RSeQC` 5.0.4

No source code from the LGPL upstream was used as implementation reference;
the algorithm is derived from the published method, public format specs,
and black-box behavioural testing.

License: MIT OR Apache-2.0.
Upstream credit: `RSeQC` <https://rseqc.sourceforge.net/> (LGPL-2.1+).
