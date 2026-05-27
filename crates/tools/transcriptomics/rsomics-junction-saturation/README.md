# rsomics-junction-saturation

Subsample-based splice-junction saturation analysis from aligned RNA-seq reads —
a Rust reimplementation of RSeQC `junction_saturation.py`.

## Install

```bash
cargo install rsomics-junction-saturation
```

## Usage

```
rsomics-junction-saturation -i INPUT.bam -r REFGENE.bed12 -o PREFIX [OPTIONS]

Options:
  -i, --input <FILE>         Input BAM (coordinate-sorted)
  -r, --refgene <FILE>       BED12 gene annotation
  -o, --output-prefix <STR>  Output prefix
  -l, --low-bound <INT>      Lower sampling fraction % [default: 5]
  -u, --upper-bound <INT>    Upper sampling fraction % [default: 100]
  -s, --step <INT>           Step between fractions % [default: 5]
      --mapq <INT>           Minimum mapping quality [default: 0]
      --min-intron <INT>     Minimum intron length to count as splice junction [default: 50]
      --seed <INT>           RNG seed for reproducible subsampling
  -t, --threads <INT>        BGZF inflate threads [default: available CPUs]
      --json                 Structured JSON progress
  -h, --help                 Print help
  -V, --version              Print version
```

Writes `<prefix>.junction_saturation.txt` (tab-separated):

```
pct    known    partial_novel    complete_novel
5      342      18               4
10     521      29               7
...
100    1234     87               23
```

Columns:
- `pct`: sampling fraction (percent of total mapped reads)
- `known`: junctions where both donor and acceptor are in the annotation
- `partial_novel`: junctions where exactly one splice site is annotated
- `complete_novel`: junctions where neither splice site is annotated

## Origin

This crate is an independent Rust reimplementation of `junction_saturation.py`
from RSeQC, based on:

- RSeQC documentation and observed tool behaviour
- The RSeQC paper: Wang L, Wang S, Li W. RSeQC: quality control of RNA-seq
  experiments. Bioinformatics. 2012. https://doi.org/10.1093/bioinformatics/bts356
- BAM/CIGAR format specification (SAMv1 §1.4)

No source code from the upstream Python implementation was used as a direct
algorithm reference beyond what is observable from the tool's output format.

Upstream: [RSeQC](http://rseqc.sourceforge.net/) (GPL-2.0)

License: MIT OR Apache-2.0
