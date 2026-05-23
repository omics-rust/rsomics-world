# rsomics-bam-head

Print a BAM's header and the first N alignment records as SAM — Rust port of
`samtools head`.

```sh
rsomics-bam-head in.bam              # all header lines, no records (default)
rsomics-bam-head -n 5 in.bam         # header + first 5 records
rsomics-bam-head -H 2 in.bam         # first 2 header lines only
rsomics-bam-head -H 1 -n 3 in.bam    # first header line + first 3 records
```

Reading stops as soon as the record budget is met, so on a multi-GB BAM head
touches only the first block or two.

## How it is fast

The header is emitted verbatim from the BAM's stored text. Records are formatted
straight from their raw BAM payloads by a hand-written BAM→SAM encoder (seq
decoded two bases per byte via a lookup table, integers/aux written without
allocation, output batched ~64 KiB per write) — avoiding the per-record
allocation and trait-object dispatch of a generic record writer, which is what
made the first cut lose to samtools' `sam_format1`.

## Options

| Flag | Meaning |
|---|---|
| `-H, --headers INT` | Print only the first INT header lines (default: all). (`samtools head` spells this `-h`; `-h` is reserved for help here.) |
| `-n, --records INT` | Also print the first INT alignment records (default: 0). |
| `-o, --output FILE` | Output SAM (default stdout). |

## Origin

This crate is an independent Rust reimplementation of `samtools head`, informed
by the upstream MIT-licensed source (`sam_view.c` `main_head`): the all-headers
default, the `-h` newline-counting header truncation, the zero-records default,
and the raw stored-header-text output. The SAM record encoding follows the SAMv1
spec and matches htslib `sam_format1` field-for-field (integer aux types collapse
to `i`, `B` arrays comma-joined, float `%g`).

License: MIT OR Apache-2.0.
Upstream credit: [samtools](https://github.com/samtools/samtools) (MIT/Expat).
