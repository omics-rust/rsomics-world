# rsomics-bam-reheader

Replace a BAM's header, passing the alignment records through unchanged — Rust
port of `samtools reheader`.

```sh
rsomics-bam-reheader new_header.sam in.bam -o out.bam
rsomics-bam-reheader new_header.sam in.bam > out.bam      # stdout, like samtools
```

The replacement header is SAM text. Its @SQ reference dictionary must match the
input's record refIDs (reheader does not re-map references — it is for fixing
@RG/@PG/@CO and SN tags, not reordering).

## How it is fast

The alignment records are **never decompressed**: after writing the new header,
the input's compressed BGZF blocks are copied byte-for-byte (only the one block
straddling the header/record boundary is re-framed). Reading uses a 1 MiB buffer
and writes straight from it, matching samtools' `bgzf_raw_read`/`bgzf_raw_write`
path while cutting a buffer copy.

## Options

| Flag | Meaning |
|---|---|
| `-o, --output FILE` | Output BAM (default stdout). |
| `-P, --no-PG` | Omit the @PG provenance line. |

Out of scope vs samtools: `-c CMD` (pipe the header through a program) and `-i`
in-place (CRAM-only; this crate is BAM-only — `samtools reheader -i` also rejects
BAM).

## Origin

This crate is an independent Rust reimplementation of `samtools reheader`,
informed by the upstream MIT-licensed source (`bam_reheader.c`): the
write-new-header-then-raw-copy streaming path, the boundary-block leftover
handling, and the stdout output target.

License: MIT OR Apache-2.0.
Upstream credit: [samtools](https://github.com/samtools/samtools) (MIT/Expat).
