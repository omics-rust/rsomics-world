# rsomics-bam-strandedness

Infer RNA-seq library strand protocol from a BAM file and a BED12 gene model.

```
rsomics-bam-strandedness -i sample.bam -r genes.bed12
```

Outputs the same text format as RSeQC `infer_experiment.py`:

```
This is PairEnd Data
Fraction of reads failed to determine: 0.0290
Fraction of reads explained by "1++,1--,2+-,2-+": 0.9650
Fraction of reads explained by "1+-,1-+,2++,2--": 0.0060
```

## Options

| Flag | Default | Description |
|---|---|---|
| `-i` / `--input` | required | Input BAM file |
| `-r` / `--refgene` | required | Reference gene model (BED12) |
| `-s` / `--sample-size` | 200000 | Max usable reads to sample |
| `--mapq` | 30 | Minimum MAPQ |
| `-t` / `--threads` | all cores | BGZF inflate threads |

## Origin

This crate is an independent Rust reimplementation based on:

- RSeQC: `infer_experiment.py` (LGPL-2.1+), Wang et al. 2012
  <https://doi.org/10.1093/bioinformatics/bts356>
- The SAM/BAM format specification (MIT)
- BED12 format specification
- Black-box behaviour testing against RSeQC 5.0.4

No source code from the GPL/LGPL upstream was used to derive correctness-critical
logic beyond what is described in the paper and observable from the LGPL script's
public output (the LGPL classification logic is short and paper-described; it was
read for format-accuracy).

License: MIT OR Apache-2.0.
Upstream credit: RSeQC <https://rseqc.sourceforge.net/> (LGPL-2.1+).
