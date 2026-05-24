# rsomics-compute-matrix

bigWig signal → per-region score matrix. Rust port of `deeptools computeMatrix`
(the matrix that feeds `plotHeatmap` / `plotProfile`).

## Usage

```
rsomics-compute-matrix reference-point -S signal.bw -R regions.bed -o matrix.gz \
    --reference-point TSS -b 1000 -a 1000 --bin-size 50

rsomics-compute-matrix scale-regions -S signal.bw -R regions.bed -o matrix.gz \
    -m 1000 -b 500 -a 500 --bin-size 50
```

## Output format

A gzipped file, byte-for-byte compatible with deeptools `save_matrix`:

- Line 1: `@` followed by a JSON dict of parameters (no spaces; deeptools' fixed
  key order; the per-sample "special" params — `upstream`, `downstream`, `body`,
  `bin size`, `ref point`, `unscaled 5/3 prime` — are emitted as one-element
  lists, with `group_boundaries` / `sample_boundaries` / labels appended).
- One TAB row per region: `chrom`, start, end, name, score, strand, then the
  per-bin signal values formatted with `%f` (six decimals; missing → `nan`).

## Options

| Flag | Default | Description |
|------|---------|-------------|
| `-S` / `--score-file` | — | bigWig signal file (required) |
| `-R` / `--regions-file` | — | BED6 regions file (required) |
| `-o` / `--out-file-name` | — | Gzipped output matrix (required) |
| `--reference-point` | TSS | `reference-point` anchor: TSS \| TES \| center |
| `-b` / `--before-region-start-length` | 500 (ref) / 0 (scale) | Upstream flank |
| `-a` / `--after-region-start-length` | 1500 (ref) / 0 (scale) | Downstream flank |
| `-m` / `--region-body-length` | 1000 | scale-regions: body scaled to this length |
| `--bin-size` | 10 | Bin width in bases |
| `--average-type-bins` | mean | Per-bin statistic: mean \| median \| min \| max \| std \| sum |
| `--missing-data-as-zero` | false | Treat NaN bases as 0 |
| `--skip-zeros` | false | Drop regions whose binned mean is 0 |
| `--min-threshold` / `--max-threshold` | — | Skip a region if any bin ≤ / ≥ this |
| `--scale` | 1 | Multiply all values |
| `--samples-label` | bigWig basename | Sample label in the header |
| `-t` / `--threads` | all cores | Worker threads |

## Modes

- **reference-point** — feature-complete: TSS / TES / center anchors, plus and
  minus strands, asymmetric flanks, off-chromosome NaN padding, every bin
  statistic, `--missing-data-as-zero`, `--scale`, thresholds, `--skip-zeros`.
  Verified byte-exact vs deeptools 3.5.6 on small and large (2100-region,
  3-chromosome) fixtures.
- **scale-regions** — feature-complete: scaled body + flanks, both strands, the
  short-body (< binSize) all-NaN special case. Verified byte-exact likewise.

## Scoped out

- Multiple bigWig samples (`-S a.bw b.bw`) and multiple region groups
  (`-R a.bed b.bed`).
- `#`-delimited multi-group BED files (rejected with an error rather than
  silently mis-grouped). Single "genes" group only.
- GTF input, `--unscaled5prime` / `--unscaled3prime`, `--nanAfterEnd`.
- `--blackListFileName`, `--sortRegions` other than `keep`, on-the-fly
  clustering, the `--outFileNameMatrix` / `--outFileSortedRegions` side files.

## Origin

This crate is a Rust reimplementation of `deeptools computeMatrix` informed by
the deeptools source (MIT license):

- Ramírez et al., *deepTools2: a next generation web server for deep-sequencing
  data analysis*, NAR 2016. DOI: 10.1093/nar/gkw257
- deeptools source: `computeMatrix.py`, `heatmapper.py`. The bin layout
  (`compute_sub_matrix_worker`, `coverage_from_big_wig`, `coverage_from_array`),
  the `numpy.linspace` per-bin partitioning, the strand handling, and the
  gzipped `@`-header format (`save_matrix`) were read directly and matched.

The bundled bigWig reader (`src/bigwig.rs`) is an independent pure-Rust BBI
reader; the on-disk layout was read from bigtools 0.5.6 (`src/bbi/bbiread.rs`,
`src/bbi/bigwigread.rs`, MIT, Jack Huey) and Jim Kent's published BBI format.
We carry our own reader rather than depend on bigtools because bigtools pins
`libdeflater = "0.13"`, whose C-FFI `libdeflate-sys` collides at link time with
the workspace's `rsomics-fqgz` (`libdeflater = "1"`) — two crates may not link
the same native library. Block inflation uses the workspace `flate2` zlib-rs
backend (pure Rust).

deeptools and bigtools are MIT licensed; their source was read directly and
cited here per the CONVENTIONS clean-room methodology for MIT upstreams. Test
fixtures are synthetically generated.

License: MIT OR Apache-2.0.
Upstream credit: deeptools <https://github.com/deeptools/deeptools> (MIT),
bigtools <https://github.com/jackh726/bigtools> (MIT).
