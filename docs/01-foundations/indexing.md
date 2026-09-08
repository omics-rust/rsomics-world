# Indexing ownership

Sequence-resource consumer recheck: 2026-09-09. Implemented operations and
future format coverage are distinguished below; no new API is approved here.

Random-access index formats are shared standards, but index construction is
not a standalone public foundation by default. The consuming product owns the
format policy, command contract, validation, and lifecycle of its indexes.
Codec and reader primitives remain external dependencies or narrow additions
to an existing format foundation.

## Product boundaries

| Index | Input model | Owning product | Current decision |
|---|---|---|---|
| BAI, alignment CSI, CRAI | SAM, BAM, CRAM alignments | `rsomics-bam` | `rsomics-bam index` |
| VCF CSI and TBI | VCF/BCF variation records | `rsomics-vcf` | `rsomics-vcf index` |
| GZI | BGZF-compressed resources | `rsomics-index` | implemented `bgzip` sidecar creation, rebuild and indexed byte-range reads |
| FAI and FASTQ index | reference and sequence files | `rsomics-index` | FASTA `faidx` is the next planned slice; FASTQ remains explicitly excluded from 0.1 |
| generic TBI/CSI | BGZF tabular records | `rsomics-index` | tabix-compatible workflow |

This split follows user workflow and record semantics. A BAI writer is not a
reason to publish `rsomics-bai`, and a reusable bin calculation is not enough
to create `rsomics-indexing`. A public item would still need two named product
consumers, consumer-side contract tests, and an API free of format policy.

## Alignment indexes

`rsomics-bam 0.6.0` builds BAI, CSI, and CRAI for coordinate-sorted BAM, BGZF
SAM, and CRAM. The command owns:

- BAI or CSI selection for BAM and BGZF SAM, and CRAI selection for CRAM;
- CSI minimum shift, custom output, multiple inputs, and worker selection;
- BGZF or CRAM EOF requirements and fail-loud malformed-input behavior;
- complete path-alias validation and transactional destination replacement;
- shared rsomics help, error, and JSON-summary presentation;
- compatibility and performance gates against samtools 1.24.

The construction backend is HTSlib through a product-private adapter. This is
an implementation dependency, not a public rsomics foundation API. The first
custom noodles builder was removed when measured evidence showed it was slower
and produced a larger BAI. Default indexing now selects up to four additional
workers, while `-@ 0` requests one-thread behavior.

`rsomics-bamio 0.8.4` owns the narrower shared read-side contract. Its indexed
alignment reader accepts BAI and CSI for BAM, CRAI for CRAM, CSI for BGZF SAM,
and the samtools-default appended BAI for BGZF SAM. The last case is covered by
a real region query, not only index parsing. No construction policy moved into
`bamio`.

## Variation and generic indexes

VCF/BCF index construction stays in `rsomics-vcf` because contig dictionaries,
record spans, TBI eligibility, and BCF policy are variation-specific. Generic
tabix, FAI, FASTQ-index, and GZI work remains grouped under `rsomics-index` as
one sequence and tabular indexing product. Deleted `rsomics-tabix`,
`rsomics-fasta-index`, `rsomics-bgzip`, and similar micro-crates remain source
or fixture assets and are not revived.

The external building blocks currently include:

| Format | Specification or implementation source | License note |
|---|---|---|
| BAI and CSI | SAM/BAM and CSI specifications; HTSlib; noodles-bam/noodles-csi | HTSlib MIT/BSD-style; noodles MIT |
| CRAI | CRAM and CRAI specifications; HTSlib; noodles-cram | permissive upstream licenses |
| TBI | tabix specification; HTSlib; noodles-tabix | permissive upstream licenses |
| FAI and GZI | HTSlib faidx/BGZF contracts; noodles-fasta/noodles-bgzf | permissive upstream licenses |

Upstream names and behavior remain attributed even though historical rsomics
code is team-owned.

## Current sequence-resource consumer evidence

Clean index head `41b161a7dac7eb3700208f025a6be6005c917002` exposes only
`bgzip` and `tabix build/query/list`. The
[product dossier](../10-products/interval-annotation-index.md#rsomics-index)
is complete; `faidx` and `dict` are planned operations, not missing dossiers
or released commands. The manifest, lockfile and production source contain no
`rsomics-seqio` dependency or call site. BGZF/GZI implementations and tabix
record/selection policy currently remain product-local.

Exact-head [CI 34247470721](https://github.com/omics-rust/rsomics-index/actions/runs/34247470721)
was re-read successful on all four native targets. Pinned HTSlib 1.24 live
oracles run on Linux x86_64; that is not four-platform oracle coverage.
The [release state](../../.autopilot/state/index-0.1-release-gate-2026-08-20.md)
still holds publication because bounded recovery checks have not located the
historical raw performance bundle. Its recorded directory was again absent;
no benchmark was rerun and the checked-in performance table is not fresh evidence.

Two different interfaces must not be conflated:

- Seqio `bf8c2c8eac4e` exposes borrowed `id`, assembled `seq`, and optional
  `qual`. Its public record and reader do not expose physical line widths or
  record byte offsets; this is a sequence-record contract, not an FAI builder.
- Annotation `8e7beed4d51e` privately wraps noodles-fasta's indexed reader in
  [`Genome`](https://github.com/omics-rust/rsomics-annotation/blob/8e7beed4d51efb78e839cddf24288e04bf93134a/src/genome.rs).
  It queries a complete named reference and retains the current chromosome
  for transcript extraction. Plain FASTA without a sidecar gets an in-memory
  index; compressed input requires persistent indexes. This is an actual
  reference-access consumer, but not a demonstrated consumer of a shared
  rsomics random-access API or an arbitrary interval-query interface.

The [FAI specification](https://www.htslib.org/doc/faidx.html) records physical
base offsets and bases/bytes per line; FASTQ adds a quality offset. A normalized
sequence alone cannot reconstruct those values. The future
[samtools faidx](https://www.htslib.org/doc/samtools-faidx.html) contract must
therefore test LF/CRLF layout, wrapping, names, region order, bounds and BGZF/GZI
interaction before selecting a parser. Duplicate-name rejection in the dossier
is an intentional strictness difference from samtools's warning and first-name
retrieval, not exact compatibility with that input.

Keep FAI construction and dictionary policy inside index first. The future
[`dict`](https://www.htslib.org/doc/samtools-dict.html) operation shares sequence
identity/streaming needs but does not turn dictionary tags or CLI choices into
a parser contract. Index and annotation are named potential consumers for a
narrow reference reader only after both have concrete adapter tests, matching
lifetime/coordinate requirements and representative resource evidence. Prefer
existing external readers when they meet that contract; no new foundation,
seqio API expansion or Layer B-to-Layer B dependency follows this audit.

## Required evidence

An index operation is stable only when all relevant gates pass:

1. malformed, truncated, unsorted, out-of-range, and aliased inputs fail
   non-zero without replacing an existing destination;
2. the real upstream tool reads the generated index and returns identical
   region or statistics output;
3. rsomics readers consume the index through an actual query;
4. index kind, minimum shift, metadata, empty references, unplaced records,
   and alternative filenames have fixtures where relevant;
5. a representative non-trivial benchmark records tool versions, machine,
   input and binary checksums, flags, timing distribution, CPU, peak RSS, and
   output identity;
6. the established-tool hot path has a strict measured throughput or resource
   advantage, or another material user benefit.

Byte identity is required when the upstream format and backend make it stable.
Otherwise compatibility is established through independent readers and
query-output equality, with any byte-level difference explained.

## Open work

- Recover or reproduce the complete BGZF/tabix performance evidence before
  reopening the current index release gate.
- Implement the dossier's complete FASTA `faidx` slice, then `dict`; keep FASTQ
  indexing and other explicit exclusions out of the advertised 0.1 surface.
- Revisit the shared sequence-index I/O gate only with real index/annotation
  consumer adapters and tests. The source audit alone does not close it.
- Retain format-specific construction inside its product until a second
  concrete consumer proves a shared public API.
- Benchmark CSI, BGZF SAM, and CRAM separately before making performance claims
  beyond the default BAM/BAI gate.
