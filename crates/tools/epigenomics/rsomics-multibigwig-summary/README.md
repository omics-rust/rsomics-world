# rsomics-multibigwig-summary

Multi-bigWig per-bin / per-region mean-signal matrix — Rust port of deeptools
`multiBigwigSummary`. Computes the mean bigWig signal per genome bin or per
supplied BED region across multiple samples, producing the correlation / PCA
input matrix.

## Usage

```
# Bins mode (10 kb default)
rsomics-multibigwig-summary -b chip1.bw chip2.bw -o counts.tab

# Custom bin size
rsomics-multibigwig-summary -b chip1.bw chip2.bw -o counts.tab --bin-size 1000

# BED-file mode (per-region)
rsomics-multibigwig-summary -b chip1.bw chip2.bw --bed peaks.bed -o counts.tab
```

## Output (`--outRawCounts`)

Tab-delimited table with a `#`-prefixed header row and per-bin/region data rows,
matching deeptools `--outRawCounts` byte-for-byte:

```
#'chr'	'start'	'end'	'chip1.bw'	'chip2.bw'
chr1	0	10000	2.3	0.5
chr1	10000	20000	nan	nan
```

Values are Python-formatted floats (`2.0`, `0.6666666666666666`, `nan`).

## Origin

This crate is an independent Rust reimplementation of
[deeptools](https://github.com/deeptools/deepTools) `multiBigwigSummary` based
on:

- Ramírez et al., "deepTools2: a next generation web server for deep-sequencing
  data analysis", Nucleic Acids Research, 2016.
  DOI: [10.1093/nar/gkw257](https://doi.org/10.1093/nar/gkw257)
- The deeptools source (MIT), read to confirm per-bin mean semantics
  (`pyBigWig.stats` nanmean, common-chromosome handling, last-partial-bin
  inclusion, and `--outRawCounts` format).

License: MIT OR Apache-2.0  
Upstream credit: deeptools <https://github.com/deeptools/deepTools> (MIT).
