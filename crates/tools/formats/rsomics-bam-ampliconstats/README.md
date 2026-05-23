# rsomics-bam-ampliconstats

Amplicon sequencing statistics from a primer BED and one or more BAM files.
A Rust port of `samtools ampliconstats` with multi-threaded BGZF decoding.

## Usage

```
rsomics-bam-ampliconstats [OPTIONS] <PRIMERS.BED> <INPUT.BAM> [INPUT2.BAM ...]
```

Output is tab-separated text in the same format as `samtools ampliconstats`
(multi-ref mode, compatible with `samtools plot-ampliconstats`).

## Options

| Flag | Default | Description |
|------|---------|-------------|
| `-f, --required-flag <INT>` | 0 | Only include reads with all FLAGS set |
| `-F, --filter-flag <INT>` | 0xB04 | Exclude reads with any of these FLAGS |
| `-m, --pos-margin <INT>` | 30 | Margin for matching read start/end to primer positions |
| `-d, --min-depth <INT[,INT]>` | 1 | Minimum depth thresholds for coverage reporting |
| `-a, --max-amplicons <INT>` | 1000 | Maximum number of amplicons |
| `-l, --max-amplicon-length <INT>` | 1000 | Maximum amplicon length |
| `--tlen-adjust <INT>` | 0 | Add/subtract from TLEN |
| `-c, --tcoord-min-count <INT>` | 10 | Minimum template coord frequency for recording |
| `-b, --tcoord-bin <INT>` | 1 | Bin template positions into multiples of INT |
| `-D, --depth-bin <FRACTION>` | 0.01 | Merge depth values within ±FRACTION |
| `-S, --single-ref` | off | Force single-ref (≤1.12) output format |
| `-t, --threads <INT>` | auto | BGZF decode threads |

## Performance

Benchmarked on Apple M2 (Darwin arm64, 8 cores), samtools 1.23.1 vs rsomics-bam-ampliconstats 0.1.0:

| Configuration | Mean (s) | vs samtools |
|---------------|----------|-------------|
| samtools ampliconstats (single-thread) | 0.607 | 1.00× |
| rsomics-bam-ampliconstats -t1 | 0.143 | **4.26×** |

Fixture: 131 MB BAM, 50 amplicons, 1 000 000 paired reads (150 bp reads, chr1).

## Origin

This crate is an independent Rust reimplementation of `samtools ampliconstats`
(`amplicon_stats.c`) based on:

- The samtools source code: James Bonfield, Genome Research Ltd.
  ([samtools/samtools](https://github.com/samtools/samtools)), MIT license.
- The SAM/BAM format specification (https://samtools.github.io/hts-specs/).
- Black-box behavior testing against `samtools ampliconstats 1.23.1`.

The upstream source (`amplicon_stats.c`) is MIT-licensed; reading and citing
it is permitted and was used to verify exact behavioral compatibility (flag
defaults, multi-ref column layout, RLE depth encoding, TCOORD all-amps bucket
logic).

License: MIT OR Apache-2.0.
Upstream credit: samtools (https://github.com/samtools/samtools), MIT license.
