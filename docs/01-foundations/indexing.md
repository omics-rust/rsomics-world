# Indexing ownership

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
| FAI, FASTQ index, GZI | reference and sequence files | `rsomics-index` | sequence-index utility workflow |
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

- Complete the `rsomics-index` dossier before implementing FAI, FASTQ index,
  GZI, and generic tabix operations.
- Retain format-specific construction inside its product until a second
  concrete consumer proves a shared public API.
- Benchmark CSI, BGZF SAM, and CRAM separately before making performance claims
  beyond the default BAM/BAI gate.
