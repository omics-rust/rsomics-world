# rsomics-bigwig-compare

Per-bin comparison of two bigWig files as a bedGraph track — a Rust port of
deeptools `bigwigCompare`.

The genome is tiled into `--bin-size`-wide bins. For each chromosome common to
both files the per-base bigWig values are read (via the `rsomics-bbi` Layer A
reader), averaged per bin, then combined per bin by `--operation` (default
`log2`). Output is bedGraph with adjacent equal-value bins merged unless
`--fixed-step`.

```
rsomics-bigwig-compare -b1 a.bw -b2 b.bw -o out.bg --operation log2 --bin-size 50
```

## Operations (deeptools `getRatio`)

Per bin, with `v1 = scale1 * cov1`, `v2 = scale2 * cov2`. If either scaled value
is NaN the result is NaN. Otherwise:

| operation | value |
|---|---|
| `log2` (default) | `log2((v1 + pc0) / (v2 + pc1))` |
| `ratio` | `(v1 + pc0) / (v2 + pc1)` |
| `reciprocal_ratio` | `r` if `r >= 1` else `-1 / r` |
| `subtract` | `v1 - v2` |
| `add` | `v1 + v2` |
| `mean` | `(v1 + v2) / 2` |
| `first` | `v1` |
| `second` | `v2` |

The pseudocount `[pc0, pc1]` (default `[1, 1]`) is added only for the
ratio-family operations.

Output is **bedGraph only**. bigWig output is out of scope: it needs a BBI
writer this workspace does not yet ship (`rsomics-bbi` is read-only).

## Origin

This crate is an independent Rust reimplementation of deeptools `bigwigCompare`
based on the deeptools source (`bigwigCompare.py`, `getRatio.py`,
`writeBedGraph_bam_and_bw.py`, MIT) and black-box differential testing against
the deeptools binary.

The combine formulas were reimplemented from deeptools `getRatio.py`; the
per-bin averaging, chromosome-set selection (common chromosomes, first file's
order, `min` length on disagreement), `--skipZeroOverZero`, run-length merge and
`{:g}` value formatting follow `writeBedGraph_bam_and_bw.py`.

License: MIT OR Apache-2.0.
Upstream credit: [deeptools](https://github.com/deeptools/deepTools) (MIT),
paper DOI [10.1093/nar/gkw257](https://doi.org/10.1093/nar/gkw257).
