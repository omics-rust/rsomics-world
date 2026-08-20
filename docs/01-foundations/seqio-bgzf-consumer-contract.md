# BGZF raw-frame consumer contract

Status: two-consumer source audit complete. No public API is approved or
implemented by this document.

## Boundary decision

Do not create `rsomics-bgzf`. The only plausible shared boundary is a narrow,
format-neutral raw-frame facility inside the existing `rsomics-seqio` public
foundation. It may be promoted only after the current BAM and VCF code paths
pass their consumer tests and representative no-regression gates against the
same candidate implementation.

Normal BGZF decompression and compression continue to use `noodles-bgzf`.
Raw-frame access exists for workflows that must preserve already-compressed
record frames while replacing or removing a format header. That requirement is
different from ordinary sequence I/O and does not justify exposing BAM, VCF,
or BCF policy through `rsomics-seqio`.

## Audited source snapshot

| Repository | Revision | Worktree state | Relevant implementation |
|---|---|---|---|
| `rsomics-seqio` | `bf8c2c8eac4e` | clean | transparent gzip/BGZF FASTA and FASTQ input; `OutputEncoder` BGZF output |
| `rsomics-bam` | `6a8da2a25479` | clean | private `src/bgzf_rewrite.rs` used by `cat`, `reheader`, and `header_source` |
| `rsomics-vcf` | `682942cfa697` | dirty concat candidate | private `src/format/bgzf.rs` used by reheader and the uncommitted concat candidate |

The VCF observations include uncommitted user-owned candidate files atop the
listed revision. They are audit evidence, not an accepted or reproducible
release head, and must not be overwritten while the concat wave is repaired.

The dependency graph is not uniform: `rsomics-seqio 0.6.0` and
`rsomics-bam 0.29.0` use `noodles-bgzf 0.47`, while the current
`rsomics-vcf 0.6.0` worktree uses `noodles-bgzf 0.49`. A shared API must not
leak a version-specific noodles type, and dependency alignment is reviewed
separately from raw-frame extraction.

## Named consumers

### `rsomics-bam`

`src/bgzf_rewrite.rs` currently combines two different responsibilities:

- format-neutral frame layout, BC-extra-subfield parsing, exact-length reads,
  canonical EOF handling, raw frame copying, and optional frame inflation with
  CRC and ISIZE checks;
- BAM-specific header decoding, canonical SAM header rendering, discovery of
  the record boundary inside a decoded frame, and BAM error context.

The first group is a sharing candidate. The second remains private. Concrete
call sites are:

- `src/cat.rs`: preflight headers, write one merged BAM header, copy compressed
  record frames, and emit one EOF block;
- `src/reheader.rs`: replace the BAM header without recompressing record
  frames;
- `src/header_source.rs`: obtain the original raw BAM header text when needed.

The current fast copy path scans complete frames directly from a buffered file
and writes contiguous runs without allocating one `Vec` per frame. Any shared
implementation must retain that property or demonstrate a material compensating
benefit.

### `rsomics-vcf`

The dirty `src/format/bgzf.rs` supplies a frame reader, canonical EOF handling,
raw copy-through, and a structurally checked reader. Its concrete call sites
are:

- `src/reheader/vcf.rs`: inflate only header frames, rewrite the VCF header,
  then copy remaining frames through the canonical EOF;
- `src/reheader/bcf.rs`: place structural frame checks in front of the noodles
  BGZF decoder while rewriting BCF;
- `src/reheader.rs`: inspect initial decoded data while replaying the original
  raw prefix;
- dirty `src/concat/naive.rs`: inspect, validate, and splice compatible BGZF
  VCF or BCF streams;
- `src/index/build.rs`: check the canonical EOF marker before indexing.

The broader extra-subfield support is not yet end to end. The outer detector
in `src/reheader.rs` still recognizes BGZF only when `XLEN` is six and `BC` is
the first extra subfield, so reheader can reject a structurally valid stream
before the newer frame reader sees it. The consumer migration must begin with
a failing regression for this path rather than treating the dirty parser test
as product-level proof.

VCF and BCF header detection, typed-header comparison, dictionary policy,
record counting, concat ordering, and naive-mode compatibility remain private
to `rsomics-vcf`.

## Smallest shared semantics

A candidate shared facility may cover only these format-neutral contracts:

1. Parse one BGZF frame from an arbitrary `Read`, including short and
   interrupted reads.
2. Locate exactly one valid `BC` extra subfield while accepting unrelated gzip
   extra subfields in any order.
3. Enforce checked frame lengths and the 65,536-byte BGZF frame ceiling.
4. Distinguish a data frame from the canonical 28-byte EOF marker without
   interpreting the contained BAM, VCF, or BCF bytes.
5. Provide a strict stream operation that requires one terminal canonical EOF,
   rejects bytes after it, and propagates read and write failures.
6. Permit raw-copy and boundary-splice paths without mandatory decompression or
   recompression.
7. Offer decoding as an explicit stronger operation that validates DEFLATE,
   CRC32, and ISIZE.

Structural framing and payload validation are different guarantees. A frame
whose header, BC field, length, and trailer layout are structurally readable
can still contain invalid DEFLATE data, CRC, or ISIZE. Names and documentation
must not call the structural path fully validated.

The API shape, ownership model, iterator vocabulary, error type, buffering,
and visibility remain undecided until the consumer tests and hot-path
measurements are runnable. In particular, the VCF candidate's owned
`Vec<u8>`-per-frame interface must not silently replace BAM's contiguous
buffered-copy path.

## Product-private exclusions

The shared module must not own:

- BAM, VCF, or BCF header parsing and serialization;
- format detection from decompressed payload bytes;
- header equality, string-map, sample, or reference-sequence policy;
- record validation or record counts;
- CLI flags, thread counts, compression-level policy, output naming, or JSON;
- transaction and atomic-output policy;
- BAI, CSI, TBI, GZI, virtual offsets, or index freshness;
- user-facing product error context;
- CRAM, which is not a BGZF container.

The canonical-EOF requirement is currently shared by both consumers, but the
low-level frame parser should only report frames. Strict terminal-EOF policy
belongs in a separate stream operation so a future consumer is not forced into
product policy accidentally.

## Required evidence before extraction

### Foundation tests

- canonical EOF, missing EOF, repeated EOF, and trailing bytes;
- arbitrary extra subfields before and after `BC`;
- missing, duplicate, malformed, truncated, and overflowing extra subfields;
- partial fixed headers, extra fields, payloads, and trailers;
- minimum and maximum legal frame lengths;
- one-byte and interrupting readers;
- read and write error propagation;
- raw byte preservation across copy and splice operations;
- correct payload, malformed DEFLATE, CRC mismatch, and ISIZE mismatch for the
  explicit decoding path;
- noncanonical empty data frames followed by the canonical EOF marker.

### BAM consumer tests

- `cat` and `reheader` preserve compressed record bytes and record order;
- headers ending within one frame and spanning several frames;
- arbitrary BGZF extra subfields;
- malformed, truncated, missing-EOF, repeated-EOF, trailing-data, and output
  failure cases remain fail-loud and atomic;
- the existing samtools 1.24 compatibility surface remains unchanged.

### VCF consumer tests

- VCF and BCF reheader paths with single-frame and multi-frame headers;
- naive concat for both encodings, including a header boundary inside a frame;
- exactly one output EOF marker;
- malformed structural frames and malformed DEFLATE, CRC, and ISIZE failures;
- arbitrary extra subfields;
- named-output failures remain atomic;
- the completed bcftools 1.24 reheader and concat compatibility surfaces remain
  unchanged.

### Performance and resource gates

Measure the old product-private path and candidate shared path on the same
machines and inputs. At minimum cover:

- BAM `cat` and `reheader` on a representative multi-gigabyte BAM;
- VCF and BCF naive concat on many files and many small BGZF frames;
- wall time, throughput, peak RSS, allocation behavior, and output hash;
- a deliberately short-read source to expose buffering regressions.

The shared path must not regress the BAM contiguous-copy hot path or make VCF
naive concat slower or more memory-hungry without another material measured
benefit.

## Implementation order

1. Complete and stabilize the VCF concat candidate in product-private code,
   including its index-resolution and bounded-concurrency repairs.
2. Restore a permitted build environment and run the existing BAM and VCF
   baselines before editing either implementation.
3. Add the smallest internal candidate to `rsomics-seqio`, beginning with the
   consumer-derived tests above; do not expose noodles types.
4. Exercise both products against the local packaged candidate without adding
   tracked path dependencies.
5. Migrate VCF and BAM in separate product commits, keeping format policy in
   each product and preserving their existing error context.
6. Run strict formatting, Clippy, tests, rustdoc, package checks, compatibility,
   and representative performance on all three repositories.
7. Publish `rsomics-seqio` only after both consumer suites pass against the
   exact candidate; then switch both consumers to the registry release and
   repeat their exact-head gates.

Until these gates pass, duplication is preferable to freezing an uncertain
public API or degrading an established hot path.
