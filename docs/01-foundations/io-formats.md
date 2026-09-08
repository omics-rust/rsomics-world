# I/O formats — product and contract survey

Updated 2026-09-09. This maps record and dataset formats to coherent products
and their existing foundations. It is not a parser-per-crate publication queue.
Compression framing and codecs belong in [compression.md](compression.md);
random-access formats belong in [indexing.md](indexing.md).

## Current ownership

| Format or capability | Product workflows | Boundary and evidence scope |
|---|---|---|
| FASTA/FASTQ | Sequence utilities, FASTQ preprocessing/QC, sketch input | Existing `rsomics-seqio` borrowed/owned records, readers and writers; products own operation policy |
| SAM/BAM/CRAM | Alignment-format operations and alignment-consuming calling/methylation workflows | Existing `rsomics-bamio` and selected format backends; pileup accumulation is a separate `rsomics-pileup` contract |
| VCF/BCF | `rsomics-vcf` view, conversion, normalization, statistics and related format workflows | Product-owned format modules and operation policy; a format name alone does not justify a variant-I/O foundation |
| GFF3/GTF | `rsomics-annotation` validation, conversion, transcript and sequence extraction | Product-private feature/transcript model over compatible noodles parsers; interval geometry uses `rsomics-intervals` |
| BED | `rsomics-bed` interval operations; `rsomics-annotation` BED conversion output | BED product preserves raw fields through its own parser/model; reusable interval geometry is already in `rsomics-intervals` |
| PAF | Planned complete `rsomics-minimap2 align` output contract | Engine/product-private mapping fields and tags; a PAF parser is not a separate product |
| AnnData / h5ad / Zarr | Planned `rsomics-sc` import, persisted analysis state and export | Product-private dataset I/O until a second matching consumer exists; array storage is not the scientific state model |
| UCSC MAF and mutation-annotation MAF | Distinct comparative-alignment and mutation-annotation use cases | Survey references only; neither the shared acronym nor a small parser establishes an accepted operation |
| htsget | Remote retrieval of reads or variants | Transport/client contract, not a record encoding; no standalone server or public foundation is implied |

The source snapshot includes clean seqio `bf8c2c8eac4e`, bamio `30459c78951f`,
BED `02b85a1a348c` and annotation `8e7beed4d51e`.
VCF is a dirty worktree atop `682942cfa697`: its uncommitted concat and related
changes are implementation assets, not a new accepted release. This survey
does not mutate them or refresh portfolio-wide publication counts.

## Source-led library choices

[noodles](https://github.com/zaeleus/noodles) supplies format-specific Rust
libraries and a feature-selected umbrella crate. Its format list is a useful
inventory, not proof of complete compatibility with every samtools/bcftools
operation or every malformed input. Optional native codec features also mean
that Rust-facing APIs do not establish an entirely Rust dependency graph.
Use the [HTS specifications](https://samtools.github.io/hts-specs/) and the
product's pinned command-line oracle to define behavior.

[needletail](https://github.com/onecodex/needletail) and
[seq_io](https://github.com/markschl/seq_io) are FASTA/FASTQ parser candidates.
Needletail also exposes sequence and k-mer operations; format detection is not
sequencing-adapter inference or trimming. The existing seqio contract does not
become obsolete because a parser advertises a faster scan. Compare complete
parsing, validation, decompression and consumer work under matching semantics
before changing its implementation. No unmeasured ranking against noodles is
retained.

[rust-htslib](https://github.com/rust-bio/rust-htslib) is an HTSlib binding,
not automatically technical debt to remove. The inspected bamio manifest
defaults to its `cram-htslib` feature, and
[CRAM record visitation](https://github.com/omics-rust/rsomics-bamio/blob/30459c78951fae406bd362854e7b80e42665a5c0/src/indexed.rs)
uses that backend. Replacing it requires the same reference handling, tags,
record semantics and platform coverage plus measured benefit. A goal of
making noodles a universal superset is not a prerequisite for this portfolio.

Version alignment follows actual type relationships. Annotation currently
pairs noodles-gff 0.55, noodles-gtf 0.50, noodles-core 0.19 and noodles-fasta
0.59; its private
[feature wrapper](https://github.com/omics-rust/rsomics-annotation/blob/8e7beed4d51efb78e839cddf24288e04bf93134a/src/annotation.rs)
uses the shared GFF record representation. VCF uses a different compatible
format graph. Review upgrades as graphs; neither arbitrary independent bumps
nor a portfolio-wide version pin is justified by this survey.

## Streaming and ownership contracts

The existing seqio
[reader](https://github.com/omics-rust/rsomics-seqio/blob/bf8c2c8eac4e8f44907587527bfd5f7f808de97e/src/reader.rs)
returns `Result<Option<Record<'_>>>` from `read_record(&mut self)`, borrowing
the reader's reusable storage. That is intentionally not an ordinary
`Iterator<Item = Record>`. Owned records, lending-style methods, visitors and
bounded batches are all valid when the consumer needs them. Preserve explicit
parse/I/O errors and distinguish them from normal exhaustion.

Describe allocation and lifetime guarantees precisely. A borrowed view is not
proof that decompression or wrapped-record assembly performs no copying.
Threaded stages need owned or otherwise lifetime-safe data and bounded queues.
Sorting, transcript assembly and matrix algorithms may need retained state;
choose bounded-memory or external-memory algorithms from their real workload,
not from a blanket ban on `Vec<Record>`.

Seqio's
[input entry points](https://github.com/omics-rust/rsomics-seqio/blob/bf8c2c8eac4e8f44907587527bfd5f7f808de97e/src/lib.rs)
detect gzip from content. Generic `Read` input decodes synchronously; the
file-path gzip path uses the producer thread described in the compression
survey. Their shared FASTA/FASTQ grammar does not imply identical execution
plumbing. Format detection, record parsing, format-specific validation and
product transformations should remain distinguishable.

## Format-specific compatibility obligations

| Family | Required distinctions before reuse or migration |
|---|---|
| FASTA/FASTQ | Names versus comments, wrapped records, quality absent for FASTA and required for FASTQ, sequence/quality length, accepted bytes, empty/truncated input and record lifetime |
| SAM/BAM/CRAM | Header dictionaries, reference identity, coordinates, CIGAR, flags, typed auxiliary tags, missing fields, long records and CRAM reference/codec behavior |
| VCF/BCF | Header-defined INFO/FORMAT types and cardinalities, missing/vector-end representations, allele/genotype mapping, sample order, numeric rendering and compressed/raw encodings |
| GFF3/GTF | Coordinate conversion, directives, escaping, multivalued attributes, dialect-specific quoting, parent/transcript relationships, strand and CDS phase |
| BED | Zero-based half-open coordinates, required versus operation-required fields, raw trailing columns, strand, empty intervals, BED12 blocks and upstream-specific accepted input |
| PAF | Query/target identities and lengths, intervals, strand, mapping quality and optional alignment tags; mapping summaries are not necessarily base-level alignments |
| Annotated matrices | Axis identity/order, dense versus CSR/CSC representation, data types, missing values, categories, aligned layers/graphs/embeddings and metadata preservation |

These are review obligations, not a claim that every listed case is already
supported. Each release declares its completed profile and explicit exclusions.
The [BED format reference](https://genome.ucsc.edu/FAQ/FAQformat.html#format1),
[GFF3 specification](https://github.com/The-Sequence-Ontology/Specifications/blob/master/gff3.md),
[Ensembl GTF description](https://www.ensembl.org/info/website/upload/gff.html)
and [minimap2 output specification](https://github.com/lh3/minimap2/blob/master/minimap2.1)
supply distinct syntax and coordinate contracts.

The current
[BED record model](https://github.com/omics-rust/rsomics-bed/blob/02b85a1a348c271e485cca629dc4e71fa075388a/src/bed.rs)
combines checked interval geometry with original row bytes. Replacing it with
noodles-bed solely to unify parser names could change observable columns or
bedtools compatibility. Share demonstrated mechanics, not product policy.
Likewise, the private VCF schema/rendering path is not equivalent to simply
calling a generic BCF reader.

## Annotated datasets and remote transport

[AnnData's on-disk specification](https://anndata.readthedocs.io/en/stable/fileformat-prose.html)
defines versioned elements on HDF5 or Zarr stores, including dense/sparse arrays,
data frames, categorical values and nested mappings. Reading `X` alone is not
an AnnData round trip. The [single-cell dossier](../10-products/sc.md) owns
the complete state and operation plan; unsupported elements must not be
silently dropped. Rust implementations such as
[anndata-rs](https://github.com/scverse/anndata-rs) are candidates to audit
against that declared profile, not evidence that all Rust libraries either
cover or fail the entire format.

[Zarr v3](https://zarr-specs.readthedocs.io/en/latest/v3/core/index.html) is a
chunked-array storage specification. Codec, data-type, metadata and storage
support must match the selected AnnData encoding and access pattern.
[zarrs](https://github.com/zarrs/zarrs) is an implementation candidate, not
automatic AnnData compatibility or a reason to create another foundation.
Keep scientific normalization, state alignment and provenance in the product.

[UCSC multiple-alignment MAF](https://genome.ucsc.edu/FAQ/FAQformat.html#format5)
and [GDC mutation-annotation MAF](https://docs.gdc.cancer.gov/Data/File_Formats/MAF_Format/)
need separate names and types. The former represents alignment blocks and
oriented sequences; the latter represents annotated variants. A generic
`MafRecord` or a claim that one reader supports both would hide incompatible
contracts. No new cancer or comparative-genomics product is created here.

[htsget](https://samtools.github.io/hts-specs/htsget.html) uses tickets describing
data blocks to retrieve and concatenate; it is not just a remote filename.
A future consumer must define authentication, request/coordinate semantics,
header/body assembly, URL handling, cancellation and failures before adopting
a client such as noodles-htsget. [htsget-rs](https://github.com/umccr/htsget-rs)
is a server reference, not an instruction to deploy a service or a current
product dependency.

## Adoption and extraction gate

Keep one coherent model inside each product. Add a Layer A item only after two
named consumers demonstrate the same policy-free contract, tests and resource
requirements. Existing `common`/`help` continue to own shared CLI reporting and
presentation; a format adapter must not build another UX layer.

For each migration, retain exact format/operation oracles, malformed-input and
write-failure tests, full-output or semantic round trips, and representative
timing/memory evidence. Record source revisions, actual dependency features,
native platforms, input identities and compatibility exclusions. Backend
language, broad SIMD/GPU labels, repository activity and a passing round trip
are not substitutes for that evidence. Review licenses and attribution for
the exact adopted sources and native dependencies.

No parser replacement, FFI-removal campaign, new public API, format expansion
or release approval follows from this survey alone.
