# rsomics-atac-shift

ATAC-seq Tn5 insertion-bias correction. Shifts read-pair coordinates +4 bp on
the forward strand and −5 bp on the reverse strand, then writes a shifted BAM
or per-read insertion-site BED.

## Usage

```
rsomics-atac-shift <input.bam> -o <output.bam>
rsomics-atac-shift <input.bam> -o insertions.bed --bed
```

## Origin

This crate is an independent Rust reimplementation based on:

- Buenrostro JD et al. (2013) "Transposition of native chromatin for fast and
  sensitive epigenomic profiling of open chromatin, DNA-binding proteins and
  nucleosome position." *Nature Methods* 10, 1213–1218.
  DOI: [10.1038/nmeth.2688](https://doi.org/10.1038/nmeth.2688)
- deeptools `alignmentSieve --ATACshift` (MIT license) — shift constants and
  shiftRead semantics read directly from MIT source.

License: MIT OR Apache-2.0.
Upstream credit: deeptools <https://github.com/deeptools/deeptools> (MIT).
