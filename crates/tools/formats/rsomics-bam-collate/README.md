# rsomics-bam-collate

Group BAM reads by QNAME so mates are adjacent — a Rust reimplementation
of `samtools collate`. The output is *not* coordinate-sorted and *not*
name-sorted; the only guarantee is that all records sharing a QNAME are
contiguous, which is exactly what a downstream `fixmate` / `markdup`
pipeline needs.

## Install

```
cargo install rsomics-bam-collate
```

Single binary.

## Usage

```
rsomics-bam-collate input.bam -o collated.bam
rsomics-bam-collate input.bam -u -o - | samtools fixmate - out.bam
```

- `-o FILE` output BAM (default stdout).
- `-u` write uncompressed BAM (skip deflate; useful when piping into
  another tool).
- `-t N` inflate/deflate worker threads.

## Why it can be faster

`samtools collate` (default mode) hashes every record's QNAME to one of
N temporary files **on disk**, then reads each temp file back and shuffles
its records by hash key — a full disk round-trip of the entire input.
That cost is justified when the input is larger than RAM. For an input
that fits in memory it is pure overhead: `rsomics-bam-collate` groups
entirely in memory in one read pass and emits in one write pass, with no
temp files. The BGZF inflate/deflate uses libdeflate via [rsomics-bamio].

The inter-group order therefore differs from samtools' hash order: ours
emits groups in first-seen-QNAME order. This is itself a valid collation
(every same-QNAME run contiguous) and is deterministic — the same input
produces byte-identical output across runs.

## Origin

Independent Rust reimplementation of `samtools collate` based on:

- The published method: Li H et al., *The Sequence Alignment/Map format
  and SAMtools*, Bioinformatics 25:2078 (2009),
  [doi:10.1093/bioinformatics/btp352].
- The SAM/BAM specification v1.6 (record layout, QNAME field).
- Black-box behaviour testing against `samtools collate` (v1.19+).

samtools is MIT-licensed; its `bamshuf.c` (the `collate`/`main_bamshuf`
path) was consulted to establish the contract — that collation only
requires same-QNAME contiguity and that the inter-group order is an
implementation choice (samtools uses hash order, ours uses first-seen
order). Record bytes pass through unchanged via [rsomics-bamio]'s raw
path (seq/qual/cigar/name never decoded).

License: MIT OR Apache-2.0.
Upstream credit: [samtools](https://github.com/samtools/samtools) (MIT).

### External-dep quadrant classification

- `noodles` (noodles-bam, noodles-sam, noodles-bgzf) — Quadrant ①
  (pure Rust, spec-tracking).
- `rsomics-bamio` — Quadrant ① wrapper exposing libdeflate-backed BGZF
  plus a raw byte-passthrough record path.
- `rsomics-common`, `rsomics-help`, `clap`, `serde`, `serde_json` —
  Quadrant ④ (edge utilities).

No FFI wrappers (no Quadrant ②); no single-threaded-in-hot-path deps
(no Quadrant ③).

## Compatibility

Because collation has no canonical inter-group order, the compat test
(`tests/compat.rs`) asserts the *invariants* rather than byte-equality to
samtools:

1. the output's record multiset equals the input's (every record present
   exactly once);
2. all records sharing a QNAME are contiguous;
3. ours is deterministic (byte-identical across runs).

The same input run through `samtools collate` is cross-checked to satisfy
invariants 1 and 2, confirming the invariants are the shared spec.

## Performance

Release contract: strictly faster wall-clock than `samtools collate` on a
RAM-fitting fixture, single-threaded and multi-threaded. The win is the
eliminated temp-file disk round-trip plus libdeflate BGZF.

[rsomics-bamio]: https://crates.io/crates/rsomics-bamio
[doi:10.1093/bioinformatics/btp352]: https://doi.org/10.1093/bioinformatics/btp352
