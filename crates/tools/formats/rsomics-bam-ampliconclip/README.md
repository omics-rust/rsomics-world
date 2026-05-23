# rsomics-bam-ampliconclip

Clip amplicon primer regions off aligned reads given a BED of primer
coordinates — a Rust reimplementation of `samtools ampliconclip`. By
default it soft-clips the primer bases from each read's 5' end (rewriting
the CIGAR with a leading/trailing `S`); `--hard-clip` physically removes
the bases from SEQ/QUAL, updates POS for a 5' cut, and recomputes the
CIGAR.

## Install

```
cargo install rsomics-bam-ampliconclip
```

Single binary.

## Usage

```
rsomics-bam-ampliconclip -b primers.bed in.bam -o clipped.bam
rsomics-bam-ampliconclip -b primers.bed --hard-clip --both-ends in.bam -o clipped.bam
```

- `-b FILE` BED file of primer regions (required).
- `-o FILE` output BAM (default stdout).
- `--hard-clip` hard clip (remove SEQ/QUAL) instead of the default soft clip.
- `--both-ends` clip on both the 5' and 3' ends.
- `--strand` use the BED strand column to match read direction.
- `--tolerance INT` match a region within this many bases (default 5).
- `--fail` mark unclipped, mapped reads as QCFAIL.
- `--clipped` only output clipped reads.
- `--no-excluded` do not write excluded (unmapped or QCFAIL) reads.
- `--filter-len INT` drop reads whose active query length is ≤ INT.
- `--fail-len INT` mark as QCFAIL reads whose active query length is ≤ INT.
- `--unmap-len INT` unmap reads whose active query length is ≤ INT (default 0).
- `--keep-tag` keep the NM/MD tags on clipped reads (default deletes them).
- `--no-PG` do not add a @PG line.
- `-t N` inflate/deflate worker threads.

The input must be coordinate-sorted (the primer match seeks per
reference). Clipping changes POS, so the output header's `SO:coordinate`
is downgraded to `SO:unknown` — re-sort downstream if a sorted output is
needed.

### Deferred flags

These `samtools ampliconclip` options are not yet implemented and are
documented here so the gap is explicit:

- `--original` (the `OA` tag recording pre-clip RNAME/POS/strand/CIGAR/
  MAPQ/NM),
- `--rejects-file FILE` (write filtered reads to a separate file),
- `--primer-counts FILE` (per-primer clipped-read counts in bedgraph
  form; the counts are tracked internally but not yet emitted),
- `-f FILE` (write the stats block to a file rather than stderr),
- `-u` (uncompressed BAM output).

Implemented: default soft clip, `--hard-clip`, `--both-ends`, `--strand`,
`--tolerance`, `--fail`, `--clipped`, `--no-excluded`, `--filter-len`,
`--fail-len`, `--unmap-len`, `--keep-tag`, `--no-PG`.

## Why it can be faster

`samtools ampliconclip` decodes every record, rewrites the CIGAR (and for
hard clip the SEQ/QUAL), and re-emits BGZF — the same work this crate
does. Two levers make ours faster on the same machine and thread count:

1. The clip is byte-level surgery on the raw BAM record payload (the same
   layout htslib's `bam1_t` holds), with no decode/re-encode round-trip
   through a fully-materialised record. The CIGAR walk and the SEQ/QUAL
   memcpy mirror the C exactly, so the per-record cost is a tight copy.
2. Output BGZF is deflated through [rsomics-bamio]'s libdeflate-backed
   parallel writer, whose deflate thread overlaps the main-thread
   read+clip even at one worker.

## Origin

Independent Rust reimplementation of `samtools ampliconclip` based on:

- The published method: Li H et al., *The Sequence Alignment/Map format
  and SAMtools*, Bioinformatics 25:2078 (2009),
  [doi:10.1093/bioinformatics/btp352].
- The SAM/BAM specification v1.6 (record layout, CIGAR/SEQ/QUAL fields)
  and the BED format.
- Reading the MIT-licensed samtools source `bam_ampliconclip.c`
  (`bam_clip`, `matching_clip_site`, `bam_trim_left`, `bam_trim_right`)
  to reproduce the exact clip-coordinate math, the BED→read-coordinate
  mapping with tolerance, the soft/hard CIGAR rewrite, the POS update on
  a 5' cut, the default NM/MD deletion (`del_tag`), and the `--unmap-len`
  rebuild.
- Black-box byte-exact testing against the `samtools ampliconclip` binary
  (v1.23+) across every implemented flag.

samtools is MIT-licensed, so reading and citing its source is permitted;
the clip math is reproduced from it, not reconstructed from memory.

License: MIT OR Apache-2.0.
Upstream credit: [samtools](https://github.com/samtools/samtools) (MIT).

### External-dep quadrant classification

- `noodles` (noodles-bam, noodles-sam) — Quadrant ① (pure Rust,
  spec-tracking); used for header parse/rewrite only.
- `rsomics-bamio` — Quadrant ① wrapper exposing libdeflate-backed BGZF
  plus a raw byte-passthrough record path.
- `rsomics-common`, `rsomics-help`, `clap`, `serde`, `serde_json` —
  Quadrant ④ (edge utilities).

No FFI wrappers (no Quadrant ②); no single-threaded-in-hot-path deps
(no Quadrant ③).

## Compatibility

`tests/compat.rs` asserts byte-exact equality of the alignment records
against `samtools ampliconclip` across every implemented flag and several
combinations (soft/hard, both-ends, strand, tolerance, the length
filters, keep-tag). The header `@PG`/`SO` lines legitimately differ
(samtools writes its own program name), so the contract is on the
alignment records, not the header. The test is version-gated to samtools
≥ 1.23, because the clip and tag-deletion semantics differ on older
releases (the main CI runs apt samtools 1.19.2) — gating avoids a
version-skew false-fail.

## Performance

Release contract: strictly faster wall-clock than `samtools ampliconclip`
on a representative coordinate-sorted BAM with a primer BED,
single-threaded (`-t1` vs `-@1`/single) and multi-threaded (`-t4` vs
`-@4`). The win is the raw-record clip path plus libdeflate BGZF.

[rsomics-bamio]: https://crates.io/crates/rsomics-bamio
[doi:10.1093/bioinformatics/btp352]: https://doi.org/10.1093/bioinformatics/btp352
