# rsomics-fragment-size

Paired-end insert-size distribution from a coordinate-sorted BAM. One-pass
scan over proper-pair + first-in-pair reads; outputs a TSV histogram
(`size\tcount`) and a summary with ATAC-seq nucleosome-period fractions.

## Usage

```
rsomics-fragment-size <input.bam> [-o histogram.tsv]
rsomics-fragment-size <input.bam> --json
```

## ATAC nucleosome fractions

- NFR (nucleosome-free region): < 100 bp
- mono-nucleosome: 180–247 bp
- di-nucleosome: 315–473 bp

## Origin

This crate is an independent Rust implementation based on:

- `samtools stats` insert-size (`IS`) histogram semantics — one count per
  read pair at the leftmost/first mate of an inward proper pair, `|TLEN|`.
  This is the full-scan correctness oracle (`compat.rs` diffs value-for-value
  against `samtools stats -m 1.0`).
- picard `CollectInsertSizeMetrics` — full-scan histogram cross-check and the
  same-operation perfgate reference.
- ATAC-seq QC conventions from the ENCODE consortium (NFR / mono / di bands).

Both `samtools stats` (with `-m 1.0` to disable its default 99%-tail trim)
and `picard CollectInsertSizeMetrics` produce histograms that match this
crate's output value-for-value. `deeptools bamPEFragmentSize` is NOT a valid
oracle: it samples a fixed number of fragments rather than full-scanning.

License: MIT OR Apache-2.0.
Upstream credit: samtools <https://github.com/samtools/samtools> (MIT),
picard <https://github.com/broadinstitute/picard> (MIT).
