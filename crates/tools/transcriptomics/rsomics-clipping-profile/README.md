# rsomics-clipping-profile

Per-position soft-clipping profile from a BAM file.

For each read cycle position (0-based, 0..read_length-1), reports how many reads are
soft-clipped (CIGAR 'S') at that position and how many are not.

## Usage

```
rsomics-clipping-profile -i input.bam -o prefix [-s SE|PE] [--mapq 30]
```

Writes:
- `<prefix>.clipping_profile.xls` — tab-separated table: `Position`, `Clipped_nt`, `Non_clipped_nt`
- `<prefix>.clipping_profile.r` — R script to reproduce the upstream plot

## Filters

- Unmapped reads (FLAG 0x0004): skipped.
- QC-fail reads (FLAG 0x0200): skipped.
- MAPQ < threshold (default 30): skipped.
- Secondary / supplementary reads: **not** filtered (matching upstream behaviour).

## Strand handling

For reverse-strand reads (FLAG 0x10), clip positions are mirrored — position `i`
in read-coordinate space maps to `read_length - 1 - i` in the output table.
This matches `clipping_profile.py` semantics (clips reported in sequencing order).

## Origin

This crate is an independent Rust reimplementation of `RSeQC` `clipping_profile.py`
based on:
- The published method: Wang et al. 2012 <https://doi.org/10.1093/bioinformatics/bts356>
- The public SAM/BAM format specification
- Black-box behaviour testing against `RSeQC` 5.0.4
  (`clipping_profile.py` — GPL-v3; source not read; clean-room implementation)

No source code from the GPL upstream was used as reference during
implementation. Test fixtures are independently generated.

License: MIT OR Apache-2.0.
Upstream credit: `RSeQC` <https://rseqc.sourceforge.net/> (GPL-v3).
