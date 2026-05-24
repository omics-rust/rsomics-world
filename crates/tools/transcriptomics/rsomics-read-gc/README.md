# rsomics-read-gc

Per-read GC% distribution from a BAM file.

Rust port of `RSeQC` `read_GC.py`. For each mapped read passing MAPQ and flag
filters, computes GC% = `(G+C) / read_length * 100` formatted to 2 decimal
places, and writes a `<prefix>.GC.xls` histogram.

## Usage

```
rsomics-read-gc -i input.bam -o out_prefix [--mapq 30] [-t THREADS]
```

Writes `<out_prefix>.GC.xls` — a two-column TSV (`GC%\tread_count`).

## Algorithm details

- Reads are processed in BAM scan order.
- Filters: skip unmapped (FLAG 0x0004), skip QC-fail (FLAG 0x0200), skip
  MAPQ < threshold (default 30).  Secondary and supplementary reads are NOT
  filtered — matching RSeQC behaviour.
- GC% = `(G+C) / len * 100` formatted as `"%.2f"`.  N bases and ambiguity
  codes count in the denominator but not as GC, matching RSeQC's
  `len(RNA_read)` denominator.
- Output rows sorted by GC% ascending (RSeQC emits in dict insertion order;
  compat tests sort both sides before comparison).

## Origin

This crate is an independent Rust reimplementation of `RSeQC`
`read_GC.py` based on:
- The published method: Wang et al. 2012 <https://doi.org/10.1093/bioinformatics/bts356>
- The public SAM/BAM format specification
- Reading the LGPL-2.1+ `RSeQC` 5.0.4 source (`SAM.py::readGC`)
  to derive exact GC% formatting, filter logic, and output format
  (LGPL allows reading; implementation is independent Rust)
- Black-box behaviour testing against `RSeQC` 5.0.4

No source code from the LGPL upstream was used as template; the Rust
implementation was written independently after reading the algorithm.

License: MIT OR Apache-2.0.
Upstream credit: `RSeQC` <https://rseqc.sourceforge.net/> (LGPL-2.1+).
