# rsomics-read-duplication

Sequence-based and position-based read duplication rate — Rust port of
`RSeQC` `read_duplication.py`.

## Usage

```
rsomics-read-duplication -i input.bam -o prefix [--mapq 30] [-t threads]
```

Outputs:
- `prefix.seq.DupRate.xls` — sequence-based duplication histogram
- `prefix.pos.DupRate.xls` — position-based duplication histogram

Each file is a two-column TSV (`Occurrence\tUniqReadNumber`) sorted by
occurrence, matching the format of `RSeQC` `read_duplication.py`.

## Filters

Reads are excluded if:
- Unmapped (FLAG 0x0004)
- QC-fail (FLAG 0x0200)
- MAPQ < `--mapq` (default 30)

## Position key

Position-based grouping uses the key `chrom:start:e1_start-e1_end:e2_start-e2_end:...`
derived from CIGAR exon blocks. This matches `RSeQC`'s `bam_cigar.fetch_exon`
semantics exactly (M advances both, D/N/S advance reference, I is skipped).

## Origin

This crate is an independent Rust reimplementation of `RSeQC`
`read_duplication.py` based on:
- Wang et al. 2012 <https://doi.org/10.1093/bioinformatics/bts356>
- The public SAM/BAM format specification
- Reading the LGPL-2.1+ `RSeQC` 5.0.4 source (`SAM.py::readDupRate`,
  `bam_cigar.py::fetch_exon`) to derive exact key semantics and filter
  logic (LGPL allows reading; implementation is independent Rust)
- Black-box behaviour testing against `RSeQC` 5.0.4

License: MIT OR Apache-2.0.
Upstream credit: `RSeQC` <https://rseqc.sourceforge.net/> (LGPL-2.1+).
