# rsomics-bam-checksum

Order-independent BAM checksum — Rust port of `samtools checksum`.

Computes reproducible, order-independent checksums over alignment records: the
same set of reads produces the same checksum whether the BAM is coordinate-sorted
or name-sorted. Per-record hashing is parallelised across a rayon batch while the
order-independent fold (multiplication mod a Mersenne prime) runs serially.

## Usage

```
rsomics-bam-checksum [OPTIONS] <INPUT.bam>

Options:
  -b, --flag-mask <INT>        Flag bits used in checksums [default: 193 = PAIRED|READ1|READ2]
  -F, --exclude-flags <INT>    Skip records with any of these flags [default: 2304 = SECONDARY|SUPPLEMENTARY]
  -f, --require-flags <INT>    Skip records missing any of these flags [default: 0]
  -c, --no-rev-comp            Do not reverse-complement sequences on the reverse strand
      --aux-tags <TAGS>        Aux tags to checksum [default: BC,FI,QT,RT,TC]
  -t, --threads <N>            BGZF inflate worker threads [default: 1]
```

## Origin

This crate is an independent Rust reimplementation of `samtools checksum` based on:
- The samtools source: `bam_checksum.c` (MIT, Copyright 2024-2025 Genome Research Ltd.,
  Author: James Bonfield). URL: <https://github.com/samtools/samtools/blob/1.23.1/bam_checksum.c>
- The SAM/BAM format specification (SAMv1 §4.2)
- Black-box behaviour testing against the upstream binary (samtools 1.23.1)

The upstream source is MIT-licensed; reading and citing it is permitted.

License: MIT OR Apache-2.0.
Upstream credit: samtools <https://github.com/samtools/samtools> (MIT).
