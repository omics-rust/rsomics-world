# rsomics-bam-fingerprint

ChIP-enrichment fingerprint — the cumulative-coverage (Lorenz) curve. Rust port
of `deeptools plotFingerprint`.

The genome is sampled at evenly spaced windows; each window's per-base read
coverage is summed; sorting those values and plotting their normalised
cumulative sum against the normalised window rank gives a Lorenz curve. A
diagonal line means uniform coverage (no enrichment); a sharp elbow near the
right means a few windows hold most of the signal (strong ChIP enrichment).

## Usage

```
rsomics-bam-fingerprint -b <input.bam> --out-raw-counts counts.tab [--bin-size 500] [--number-of-samples 500000]
```

## Options

| Flag | Default | Description |
|------|---------|-------------|
| `--bam` / `-b` | — | Input indexed coordinate-sorted BAM (required) |
| `--out-raw-counts` | — | Per-bin per-base coverage table (deeptools `--outRawCounts`); `-` for stdout |
| `--out-fingerprint` | — | Cumulative fingerprint curve: `rank<TAB>fraction`; `-` for stdout |
| `--out-quality-metrics` | — | Summary metrics (AUC, X-intercept, elbow); `-` for stdout |
| `--bin-size` / `-s` | 500 | Window size in bp to sample the genome |
| `--number-of-samples` / `-n` | 500000 | Number of bins sampled across the genome |
| `--min-mapq` | 0 | Minimum mapping quality |
| `--sam-flag-exclude` | 0 | Skip reads with any of these FLAG bits set |
| `--sam-flag-include` | 0 | Keep only reads with all of these FLAG bits set |
| `-t` / `--threads` | all cores | Worker threads for BGZF decompression |

At least one of `--out-raw-counts`, `--out-fingerprint`, `--out-quality-metrics`
must be given.

## Determinism

deeptools does **not** sample bins randomly. It sets
`stepSize = max(genomeSize / numberOfSamples, 1)` and walks each chromosome at
`stepSize` intervals inside `mapReduce` genome chunks; a bin crossing a chunk
boundary is dropped. `chunkSize` derives from the BAM's mapped-read count. This
crate reproduces all of it, so the sampled-bin set and per-bin counts are
identical to `plotFingerprint -p 1` (single-process, no task shuffle). Output is
fully reproducible — no RNG involved.

## Scoped out

- The PNG/SVG/PDF plot — this crate emits the data tables (raw counts,
  fingerprint curve, summary metrics), the same split as `rsomics-bam-signal`
  emitting bedGraph rather than bigWig.
- `--JSDsample` Jensen-Shannon distance, CHANCE statistics, and the
  Synthetic AUC / Synthetic X-intercept / Synthetic Elbow / Synthetic JS
  Distance columns (require a reference BAM and a scipy Poisson model;
  multi-input, out of scope for this single-input crate).
- `--extendReads` / `--centerReads`, `--region`, `--blackListFileName`,
  `--minFragmentLength` / `--maxFragmentLength`, `--skipZeros`.

## Origin

This crate is a Rust reimplementation of `deeptools plotFingerprint` informed by
the deeptools source (MIT license):

- Ramírez et al., *deepTools2: a next generation web server for deep-sequencing
  data analysis*, NAR 2016. DOI: 10.1093/nar/gkw257
- deeptools source: `plotFingerprint.py`, `sumCoveragePerBin.py`,
  `countReadsPerBin.py`, `mapReduce.py`

The bin-sampling geometry (`stepSize`/`chunkSize`), the `SumCoveragePerBin`
per-base spread arithmetic (including its tiled-path full-middle-tile behaviour),
the pysam `get_blocks()` CIGAR splitting (M/=/X runs broken by I, D and N), and
the AUC / X-intercept / elbow formulas were all read directly from the source and
matched value-exact against `plotFingerprint 3.5.6` `--outRawCounts` /
`--outQualityMetrics`. deeptools is MIT licensed; its source was read and cited
here per the CONVENTIONS clean-room methodology for MIT upstreams. Test fixtures
are synthetically generated.

License: MIT OR Apache-2.0.
Upstream credit: deeptools <https://github.com/deeptools/deeptools> (MIT).
