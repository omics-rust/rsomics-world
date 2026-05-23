# rsomics-bam-signal

Binned BAM → bedGraph signal track. Rust port of `deeptools bamCoverage`.

## Usage

```
rsomics-bam-signal <input.bam> [-o out.bedgraph] [--bin-size 50] [--normalize-using CPM]
```

## Options

| Flag | Default | Description |
|------|---------|-------------|
| `--bin-size` / `-b` | 50 | Bin size in bases |
| `--normalize-using` | None | Normalisation: None, CPM, RPKM, BPM, RPGC |
| `--effective-genome-size` | — | Required for RPGC |
| `--skip-flags` | 0 | Skip reads with these FLAG bits (hex or decimal). Default matches deeptools (no skip). Use `0x400` to skip duplicates. |
| `--min-mapq` | 0 | Minimum mapping quality |
| `-t` / `--threads` | all cores | Worker threads for BGZF decompression |
| `-o` | stdout | Output bedGraph path |

## Scoped out

- bigWig output (future: via `bigtools` crate)
- `--extendReads` / `--centerReads` (paired-end fragment extension)
- `--smoothLength`
- `--skipZeroOverZero`
- Region filtering (`--region`)
- Blacklist exclusion (`--blackListFileName`)

## Origin

This crate is a Rust reimplementation of `deeptools bamCoverage` informed by the deeptools source (MIT license):

- Ramírez et al., *deepTools2: a next generation web server for deep-sequencing data analysis*, NAR 2016. DOI: 10.1093/nar/gkw257
- deeptools source: `bamCoverage.py`, `countReadsPerBin.py`, `writeBedGraph.py`

deeptools is MIT licensed. Its source was read directly and cited here per the CONVENTIONS clean-room methodology for MIT upstreams. Test fixtures are synthetically generated.

License: MIT OR Apache-2.0.
Upstream credit: deeptools <https://github.com/deeptools/deeptools> (MIT).
