# rsomics-read-quality

Per-base read quality heatmap and boxplot from a BAM file.

Rust port of [RSeQC](http://rseqc.sourceforge.net/) `read_quality.py`.

## Usage

```
rsomics-read-quality -i aligned.bam -o out/prefix
rsomics-read-quality -i aligned.bam -o out/prefix --mapq 20
rsomics-read-quality -i aligned.bam -o out/prefix -r 100
```

## Output

`<PREFIX>.qual.r` — an R script that, when executed, produces:

- `<PREFIX>.qual.boxplot.pdf` — per-position Phred score boxplot
- `<PREFIX>.qual.heatmap.pdf` — position × quality score heatmap

The R script is byte-compatible with RSeQC 2.6.2 output (excluding the
embedded absolute path in the `pdf(...)` calls, which are session-local).

## Options

| Flag | Default | Description |
|------|---------|-------------|
| `-i` | required | Input BAM file |
| `-o` | required | Output prefix |
| `--mapq` | 30 | Minimum MAPQ to include a read |
| `-r` | 1 | Reduce factor for boxplot `times` values |
| `-t` | all CPUs | Worker threads |

## Origin

This crate is an independent Rust reimplementation of `RSeQC read_quality.py`
based on:
- The RSeQC documentation: <https://rseqc.sourceforge.net/#read-quality-py>
- Black-box behaviour testing against the upstream binary (RSeQC 2.6.2)

No source code from the GPL-2 upstream was used as reference during
implementation. Test fixtures are independently generated.

License: MIT OR Apache-2.0.
Upstream credit: RSeQC <http://rseqc.sourceforge.net/> (GPL-2).
Reference: Wang L, Wang S, Li W. *RSeQC: quality control of RNA-seq
experiments.* Bioinformatics (2012). DOI: 10.1093/bioinformatics/bts526
