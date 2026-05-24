# rsomics-bam-compare

Per-bin comparison of two BAMs as a bedGraph track. Rust port of
`deeptools bamCompare`.

Both BAMs are tiled into fixed-width bins (the shared `rsomics-coverage-core`
primitive, same binning as `bamCoverage`), each scaled, then combined per bin by
an operation (default `log2`).

## Usage

```
rsomics-bam-compare --bam1 treat.bam --bam2 ctrl.bam [-o out.bedgraph] [--operation log2] [--bin-size 50]
```

## Options

| Flag | Default | Description |
|------|---------|-------------|
| `--bam1` / `-1` | — | Treatment / numerator BAM (required) |
| `--bam2` / `-2` | — | Control / denominator BAM (required) |
| `--bin-size` / `-b` | 50 | Bin size in bases |
| `--operation` | log2 | log2, ratio, reciprocal_ratio, subtract, add, mean, first, second |
| `--pseudocount` | 1 | Pseudocount for ratio-family ops (one or two values) |
| `--skip-flags` | 0 | Skip reads with these FLAG bits (hex or decimal). Use `0x400` to skip duplicates |
| `--min-mapq` | 0 | Minimum mapping quality |
| `-t` / `--threads` | all cores | Worker threads for BGZF decompression |
| `-o` | stdout | Output bedGraph path |

## Scaling

`--scaleFactorsMethod readCount` (deeptools default) is applied: each BAM's scale
factor is `min(m1, m2) / mi`, where `mi` is the BAM's mapped read count after the
FLAG / MAPQ filter (the larger library is scaled down to the smaller). With
default flags this is the only scaling.

## Operations

With `v1 = scale[0] * cov1`, `v2 = scale[1] * cov2`, pseudocount `[pc0, pc1]`:

| Operation | Formula |
|-----------|---------|
| `log2` (default) | `log2((v1 + pc0) / (v2 + pc1))` |
| `ratio` | `(v1 + pc0) / (v2 + pc1)` |
| `reciprocal_ratio` | `r` if `r >= 1` else `-1 / r`, `r = (v1 + pc0) / (v2 + pc1)` |
| `subtract` | `v1 - v2` |
| `add` | `v1 + v2` |
| `mean` | `(v1 + v2) / 2` |
| `first` | `v1` |
| `second` | `v2` |

The pseudocount is added only for the ratio-family operations.

## Scoped out

- bigWig output (bedGraph only; future via `bigtools`)
- `--scaleFactorsMethod SES` (signal-extraction scaling — requires the SES
  p-value estimator; readCount and None are implemented)
- `--normalizeUsing` (CPM/RPKM/BPM/RPGC; deeptools only allows this with
  `--scaleFactorsMethod None`)
- `--extendReads` / `--centerReads` (paired-end fragment extension)
- `--smoothLength`, `--skipZeroOverZero`
- Region filtering (`--region`), blacklist (`--blackListFileName`)

The two BAMs must share an identical reference set (name + length + order), as
deeptools requires.

## Origin

This crate is a Rust reimplementation of `deeptools bamCompare` informed by the
deeptools source (MIT license):

- Ramírez et al., *deepTools2: a next generation web server for deep-sequencing
  data analysis*, NAR 2016. DOI: 10.1093/nar/gkw257
- deeptools source: `bamCompare.py`, `getRatio.py`, `getScaleFactor.py`,
  `writeBedGraph.py`

deeptools is MIT licensed. Its source was read directly and cited here per the
CONVENTIONS clean-room methodology for MIT upstreams. Test fixtures are
synthetically generated.

License: MIT OR Apache-2.0.
Upstream credit: deeptools <https://github.com/deeptools/deeptools> (MIT).
