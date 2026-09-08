# Compression — consumer and upstream survey

Updated 2026-09-09. This is an ownership and adoption map, not a list of
compression crates to publish. Record formats belong in
[io-formats.md](io-formats.md); coordinate and byte-offset indexes belong in
[indexing.md](indexing.md). Codec availability is not evidence of a completed
rsomics operation or a performance advantage.

## Current ownership

| Capability | Product workflow | Current implementation boundary |
|---|---|---|
| Plain/gzip/BGZF sequence input | Sequence utilities, FASTQ preprocessing/QC, sketch construction | `rsomics-seqio` probes gzip magic and replays the prefix; `MultiGzDecoder` runs synchronously for generic readers and on one producer thread for gzip file paths |
| Plain/BGZF byte output | For example, `rsomics-seq` compressed sequence output | Existing `rsomics-seqio::OutputEncoder`; the product selects encoding, level, destination and transaction policy |
| Thread-controlled gzip output | FASTQ preprocessing | Product-private writer; FASTA/FASTQ validation and serialization use `rsomics-seqio::Writer` |
| BAM BGZF output | Alignment-format operations | Existing `rsomics-bamio::RingBgzfWriter` over libdeflater; BAM record policy stays in the product |
| BGZF compression, integrity, GZI creation/rebuilding and indexed byte reads | `rsomics-index bgzip` | Product-owned workflow, frame reader/writer and GZI modules; ordered compression workers use libdeflater |
| Already-compressed frame copy/splice | BAM cat/reheader; VCF/BCF reheader and candidate concat | Private consumer implementations; a narrow shared seqio contract is under review, not approved by this survey |
| CRAM containers and codecs | CRAM paths in `rsomics-bam` and its format I/O foundation | CRAM-aware backend; not a BGZF wrapper or a generic sequence decoder |
| Other stream/archival codecs | Only a product that explicitly needs their format | External dependency or product-private adapter; no additional public foundation justified here |

Source snapshot: clean `rsomics-seqio` at `bf8c2c8eac4e8f44907587527bfd5f7f808de97e`,
`rsomics-bamio` at `30459c78951fae406bd362854e7b80e42665a5c0`, and
`rsomics-index` at `41b161a7dac7eb3700208f025a6be6005c917002`.
Relevant anchors are seqio's
[input probe](https://github.com/omics-rust/rsomics-seqio/blob/bf8c2c8eac4e8f44907587527bfd5f7f808de97e/src/detect.rs),
[gzip producer](https://github.com/omics-rust/rsomics-seqio/blob/bf8c2c8eac4e8f44907587527bfd5f7f808de97e/src/reader_gz.rs),
[output encoder](https://github.com/omics-rust/rsomics-seqio/blob/bf8c2c8eac4e8f44907587527bfd5f7f808de97e/src/output_writer.rs),
bamio's [ring writer](https://github.com/omics-rust/rsomics-bamio/blob/30459c78951fae406bd362854e7b80e42665a5c0/src/ring_writer.rs),
and index's [compression workers](https://github.com/omics-rust/rsomics-index/blob/41b161a7dac7eb3700208f025a6be6005c917002/src/bgzip/writer.rs).
These are source observations, not fresh execution or registry-release claims.

`rsomics-igzip` remains a temporary namespace exception for immutable historical
registry dependencies. The inspected seqio manifest no longer depends on it;
neither this fact nor a faster codec candidate authorizes deleting an archive
required by a published version.

## DEFLATE, gzip and BGZF are different contracts

[flate2](https://docs.rs/flate2/1.1.10/flate2/) supports raw DEFLATE, zlib and
gzip wrappers with selectable backends. `miniz_oxide` and `zlib-rs` are Rust
implementations; `zlib-ng` is a C backend, not pure Rust. The inspected seqio
manifest explicitly disables defaults and requests `zlib-rs`. Backend features
can unify transitively, so record the actual compiled feature set when comparing
products. Do not infer a codec from the direct manifest alone.

Whole-stream input must consume all gzip members: a single-member decoder can
silently return only the first member, whereas flate2's `MultiGzDecoder`
processes concatenated members and rejects trailing non-gzip data. Reading BGZF
as gzip supplies decoded bytes; it does not retain virtual positions or prove
a strict canonical-EOF/frame-copy contract.

[libdeflate](https://github.com/ebiggers/libdeflate) supplies whole-buffer
DEFLATE/zlib/gzip operations, exposed to Rust by libdeflater. It is not a
streaming replacement for an arbitrary large gzip member. Bounded BGZF blocks
are a concrete use case; backend adoption still needs the consumer's output
size, checksum, failure and resource evidence. No unconditional two-fold
speed claim is retained.

The [HTSlib BGZF description](https://www.htslib.org/doc/bgzip.html#BGZF_FORMAT)
defines concatenated gzip members with a `BC` extra subfield and a 64 KiB
compressed and uncompressed block ceiling. Shared code must separate structural
framing from DEFLATE, CRC and ISIZE validation. The
[BAM/VCF raw-frame contract](seqio-bgzf-consumer-contract.md) names the two
consumers, copy-through requirements and extraction gates. Header semantics,
record ordering, index freshness and CLI policy stay with their owners.

The inspected `noodles-bgzf 0.47.0` source already exports both
`MultithreadedReader` and `MultithreadedWriter` alongside synchronous I/O.
The old claim that its reader is necessarily single-threaded is withdrawn.
An available upstream API is not a reason to replace the current decoder or
introduce `rsomics-bgzf` without a measured consumer requirement.

## BGZF workflow, not a general-purpose compression product

The [HTSlib bgzip manual](https://www.htslib.org/doc/bgzip.html) covers
compression/decompression, integrity testing, GZI creation/rebuilding,
uncompressed-offset reads, text/binary block placement and worker selection.
GZI maps compressed block offsets to uncompressed stream offsets; it is not
a genomic-coordinate index. The accepted
[index product dossier](../10-products/interval-annotation-index.md#rsomics-index)
owns this operation map and its explicit exclusions, including rebgzip layout
reproduction, implicit input deletion and multi-input invocation for 0.1.
The command's help does not imply every upstream flag is implemented.

[pigz](https://zlib.net/pigz/) and [crabz](https://github.com/sstadick/crabz)
already serve general parallel gzip workflows. [gzp](https://docs.rs/gzp/2.0.4/gzp/)
provides worker-backed writers for gzip, BGZF and related encodings. These are
implementation references, not new install identities or proof that rsomics
matches their speed. The current index writer is product-local, not a gzp or
crabz adoption. Any later scheduler sharing must demonstrate matching consumers
and exclude product-specific block placement, worker budget and output policy.

The index repository's
[current performance record](https://github.com/omics-rust/rsomics-index/blob/41b161a7dac7eb3700208f025a6be6005c917002/PERFORMANCE.md)
holds publication because the original raw evidence cannot currently be
reverified. Historical tables do not close that gate.

## Other codecs and format detection

| Survey topic | Reference and adoption boundary |
|---|---|
| zstd | [Zstandard](https://github.com/facebook/zstd), Rust `zstd` bindings and the `ruzstd` decoder are candidates only where a product explicitly supports that encoding. Neither gzip nor BGZF permits replacing DEFLATE with zstd while retaining format compatibility. |
| LZ4 | [lz4_flex](https://github.com/PSeitz/lz4_flex) exposes block and frame formats. The distinction, framing, limits and durability contract must be explicit if used for product-local scratch or interchange; no numerical ranking against C is established here. |
| xz / liblzma | [XZ Utils](https://tukaani.org/xz/) provides the xz format/library. Rust bindings and decoder alternatives require version-specific compatibility and dependency review. This is not intrinsically an SRA ingest or standalone rsomics workflow. |
| niffler | [niffler](https://docs.rs/niffler/3.0.1/niffler/) detects compression from bytes and exposes feature-selected readers/writers. It is not an open-by-extension helper and is not a new portfolio-wide default. Existing seqio consumers should retain their tested shared contract. |

CRAM 3.1 does not introduce zstd: its
[format specification, sections 8 and 14](https://samtools.github.io/hts-specs/CRAMv3.pdf)
adds rANS4x16, adaptive arithmetic coding, fqzcomp and read-name tokenisation
to the earlier codec set. CRAM 3.0 already supports LZMA encapsulated in xz.
These are CRAM block codecs, not an instruction to open a CRAM file as an xz
stream. The old zstd and archival-only descriptions are withdrawn.
[NCBI's fasterq-dump guide](https://github.com/ncbi/sra-tools/wiki/HowTo:-fasterq-dump)
also distinguishes FASTQ generation from subsequent explicit compression;
it does not establish the old claimed xz output workflow.

License and attribution review belongs to the exact selected source version,
including native bindings, bundled codec code and optional features. The
previous approximate license/version labels are not an adoption approval.

## Evidence before changing a shared path

1. Name two actual product consumers and their matching byte-stream contract
   before adding a public item; otherwise keep the adapter private.
2. Test content detection independently of extensions, concatenated members,
   truncated input, checksum failures, short/interrupted I/O, finalization
   failures and cancellation. Add BGZF frame/EOF/index cases only where that
   stronger guarantee is promised.
3. Retain both consumer oracles and the downstream record semantics, not just
   a codec round trip. Standard-output piping and named-output transactions
   are distinct product contracts.
4. Measure complete representative workflows with exact binaries, backend
   features, inputs, levels, thread budgets, output sizes, timing distributions
   and peak memory on the relevant native platforms.
5. Make an explicit correctness/performance decision. A successful benchmark
   collection, dependency's advertised speed or generic SIMD/GPU label does
   not constitute a release gate.

No new public crate, backend switch or parallel-compression API is approved by
this survey. The next shared compression work remains driven by the existing
consumer contracts, not by the old checked adoption list.
