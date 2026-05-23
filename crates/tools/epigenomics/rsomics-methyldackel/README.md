# rsomics-methyldackel

Per-CpG methylation extraction from bisulfite-aligned BAM. Rust port of `MethylDackel extract`.

## Usage

```
rsomics-methyldackel <ref.fa> <input.bam> -o <prefix> [--min-mapq 10] [--min-phred 5]
```

## Options

| Flag | Default | Description |
|------|---------|-------------|
| `-o` / `--output` | `out` | Output prefix; bedGraph written to `<prefix>_CpG.bedGraph` |
| `--min-mapq` | 10 | Minimum mapping quality |
| `--min-phred` | 5 | Minimum base Phred quality |
| `--ignore-flags` | `0xF00` | Ignore reads with any of these FLAG bits (secondary\|qcfail\|dup\|supplementary) |
| `--require-flags` | `0` | Require reads to have all these FLAG bits |
| `-d` / `--min-depth` | 1 | Minimum read depth to emit a CpG position |
| `-t` / `--threads` | all cores | Worker threads for BGZF decompression |

## Output

A bedGraph file `<prefix>_CpG.bedGraph` with format:

```
track type="bedGraph" description="<prefix> CpG methylation levels"
<chrom>  <start>  <end>  <pct>  <n_methylated>  <n_unmethylated>
```

Positions are 0-based half-open. Percentage is integer-truncated.

## Scoped out

- CHH / CHG context output (`--CHH`, `--CHG`)
- BigWig output
- Bias plots (`--MBias`)
- Per-read methylation output (`--perRead`)
- OT/OB/CTOT/CTOB strand filtering flags
- Merging CpG strands (`--mergeContext`)

## Origin

This crate is a Rust reimplementation of `MethylDackel extract` informed by the MethylDackel source (MIT license):

- Schultz et al., *Methyldackel: a feature-complete methylation extractor for BS-seq experiments*, GitHub 2016–2024.
- MethylDackel source: `common.c` (getStrand, updateMetrics, isCpG), `overlaps.c` (cust_tweak_overlap_quality), `extract.c` (extractCalls, writeCall).

MethylDackel is MIT licensed. Its source was read directly and cited here per the CONVENTIONS clean-room methodology for MIT upstreams. Test fixtures are synthetically generated.

License: MIT OR Apache-2.0.
Upstream credit: MethylDackel <https://github.com/dpryan79/MethylDackel> (MIT).
