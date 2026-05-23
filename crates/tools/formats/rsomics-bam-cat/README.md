# rsomics-bam-cat

Concatenate BAM files into one — Rust port of `samtools cat`.

```sh
rsomics-bam-cat part1.bam part2.bam part3.bam -o all.bam
rsomics-bam-cat --header hdr.bam part1.bam part2.bam -o all.bam   # external header
```

The output carries one header (the first input's, with @RG lines from later
inputs merged in) followed by every input's alignment records in input order.

## How it is fast

Like `samtools cat`, the alignment records are **never decompressed**: each
input's compressed BGZF blocks are copied byte-for-byte to the output, so no
deflate/inflate runs on the records. Only the single output header is re-framed.
Ours additionally reads with a 1 MiB buffer and writes straight from that buffer
(no intermediate copy, no per-frame parsing), which edges out samtools'
default-buffered raw copy on the same machine.

## Options

| Flag | Meaning |
|---|---|
| `-o, --output FILE` | Output BAM (default stdout). |
| `--header FILE` | Use this BAM's header instead of the first input's. (`samtools cat` spells this `-h`; `-h` is reserved for help here.) |
| `-P, --no-PG` | Omit the @PG provenance line. |

## Origin

This crate is an independent Rust reimplementation of `samtools cat`, informed by
the upstream MIT-licensed source (`bam_cat.c`): the raw-block copy loop, the
first-header-plus-merged-@RG rule, and the trailing-EOF-marker handling.

License: MIT OR Apache-2.0.
Upstream credit: [samtools](https://github.com/samtools/samtools) (MIT/Expat).
