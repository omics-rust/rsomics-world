# Interval, annotation, and index product dossier

Status: the BED and annotation initial slices are published and independently
verified. The complete BED family map is now maintained in `bed.md`. The BGZF
and tabix slice of `rsomics-index` is implemented and compatibility-verified;
publication still requires a current-head performance run and registry
credentials.

Routing corrections move table aggregation to `rsomics-table`, SEACR to
`rsomics-peak`, FASTA masking to `rsomics-bed`, and FASTA indexing plus
sequence-dictionary creation to `rsomics-index`. The resulting source pool has
42 BED, four annotation, and five index candidates.

## Shared design

`rsomics-intervals` owns coordinate-safe interval geometry and indexing.
`rsomics-bed` owns BED syntax and bedtools-style command policy.
`rsomics-annotation` owns GFF/GTF semantics. `rsomics-index` owns compression
and index workflows.

The public geometry layer does not absorb product-specific header, formatting,
or command behavior.

## `rsomics-bed`

The canonical full-family operation and asset dossier is `bed.md`. This
section retains the provenance of the five-operation 0.1 pilot.

### Boundary

One BED/interval-algebra product with recognizable operations as subcommands.
Region variants remain flags inside an operation.

### First release slice

- `sort`
- `merge`
- `intersect`
- `subtract`
- `complement`

These operations provide a narrow but complete test of coordinate semantics,
streaming records, interval indexing, multi-file behavior, and bedtools
compatibility.

The first slice is implemented at `omics-rust/rsomics-bed`; implementation
revision `ed415eeebd9d6a3bcb34cc9cf15bcfc5f7c587cd` is followed by evidence
revision `76d02dbc9c0fd549782f1e68e2b0ef5e64f13d45` and documentation revision
`97f5fe31662eb66aa7fad42dc4f62f3007783280`. The current boundary-refactor
head is `9f4ba8ee945c487b4157bf38eba7a6577fca5dfd`; exact-head CI run
`30570159631` builds the pinned bedtools 2.31.1 oracle and passes on native
Linux and macOS for both `x86_64` and `aarch64`. In addition to targeted and
exhaustive boundary fixtures, seeded differential testing passed 500 macOS and
1,000 Linux `x86_64` trials across all five operations.

The representative Linux gate uses one million primary records on ten
chromosomes, real intersect hits, repeated B intervals, emitted subtract
fragments, merge groups, and complement gaps. Complete outputs match bedtools
byte for byte before timing. All five operations retain strict throughput
advantages; see `bed-gate-2026-07-30.md`. The old empty-output intersect and
subtract figures are superseded rather than retained as release claims. The
current subtract implementation no longer builds an unused overlap tree. It
was remeasured on the representative fixture, and version 0.1.0 was published
and independently verified; see `bed-gate-2026-07-30.md`.

### Asset dispositions

- Direct merge seeds: `bed-sort`, `bed-merge`, `bed-intersect`,
  `bed-subtract`, and `bed-jaccard`.
- Refactor before merge: annotate, closest, cluster, complement, coverage,
  fisher, flank, genomecov, getfasta, map, maskfasta, multicov, multiinter,
  nuc, random, reldist, sample, shift, shuffle, slop, window, and unionbedg.
- Primarily evidence or small-operation consolidation: count, len, midpoint,
  stats, summary, total-bp, unique, validate, spacing, to-gff,
  bed12-to-bed6, makewindows, split, and overlap.

`rsomics-bed-expand` and `rsomics-bed-groupby` route to `rsomics-table`.
`rsomics-seacr` routes to `rsomics-peak`.

The existing `has_compat` inventory flag is not sufficient: only a subset runs
the real upstream binary. Every release-slice operation receives a pinned
bedtools invocation, adversarial boundary fixtures, and representative
performance inputs.

## `rsomics-annotation`

### Boundary

GFF/GTF parsing, validation, selection, transformation, and annotation-aware
sequence extraction.

The primary behavior sources are:

- [GFF3 1.26](https://github.com/The-Sequence-Ontology/Specifications/blob/master/gff3.md)
  for nine-column syntax, directives, percent encoding, feature coordinates,
  phase, and graph attributes;
- [Ensembl GFF/GTF format](https://www.ensembl.org/info/website/upload/gff.html?redirect=no)
  for the supported GTF2 dialect and its inclusive coordinates;
- [gffread 0.12.9](https://github.com/gpertea/gffread/tree/v0.12.9) for
  transcript selection, conversion, region filtering, and sequence extraction;
- [AGAT](https://github.com/NBISweden/AGAT) for the broader operation
  inventory and malformed real-world dialects, not as a byte-for-byte oracle.

The source assets are `rsomics-gff-utils`, `rsomics-transcript-fasta`,
`rsomics-vcf-csq`, and `rsomics-vcf-split-vep`.
`rsomics-gff-utils` at
`b8d74faf579168ad10cd833076e0f587ccb39521` contains 27 small commands but
reparses the same record in most modules and has no shared typed record model.
Its useful algorithms and fixtures are assets, not the target structure.
`rsomics-transcript-fasta` at
`486063bfd8c6b3ba3a059b8926d0416a714583e9` has stronger gffread sequence
goldens and a reusable transcript assembly path, but its parser silently skips
short records and swaps inverted coordinates.

The two consequence assets were formerly routed by their VCF input. Their user
workflow is functional annotation:

- `rsomics-vcf-csq` at
  `0cbbba412ee08b48ff83ff172e5aadb4b85555d4` is a refactor seed for a later
  `consequence` operation. Its current record-by-record consequence model does
  not implement the complete haplotype-aware bcftools `csq` contract.
- `rsomics-vcf-split-vep` at
  `e844a185391221549674415827afb8f35ff2674a` is a test and parser asset for
  later `consequence inspect` and `consequence extract` modes. Before merging,
  it needs the bcftools 1.24 SnpEff field support, configurable consequence
  field, and case-preserving severity behavior.

The initial implementation slice is:

- `validate`, with explicit GFF3 or GTF dialect selection and record-level
  failures carrying line numbers;
- `view`, preserving valid records while combining type, attribute, region,
  containment, and head limits;
- `to-bed`, converting 1-based inclusive features to 0-based half-open
  intervals at one checked boundary.

The initial slice is implemented at `omics-rust/rsomics-annotation` revision
`b8ad1eee786586fd1375e883c608e1feae0417d2`. Transcript, CDS, and protein
FASTA extraction follows at revision
`80920fb9e72b6d05c34de41eaa88bb971b1c48fe`; performance evidence is recorded
at `f089ec6a54bb985639828d367bb7d5ec25486d72`. Release revision
`8e7beed4d51e` consumes `rsomics-common 0.11.0`, removes the duplicate
single-output transaction and path-alias implementation, and retains the
product-specific coordinated extraction outputs. Exact-head CI run
`30725476586` passes on native Linux and macOS for both `x86_64` and `aarch64`.
The gate runs strict Clippy, rustdoc, debug and release suites, package
verification, and 41 tests. Three live-oracle cases build pinned gffread
0.12.9 and compare region selection, BED conversion, and all three sequence
outputs.

The implementation has one typed record stream for both dialects, explicit
dialect selection, transactional named outputs, line-numbered failures, and
the shared `rsomics-help` command presentation. Validation rejects inverted
coordinates, non-finite scores, invalid CDS phase use, unresolved GFF3
parents, incompatible repeated IDs, missing GTF hierarchy identifiers, and a
GFF3 version directive that is absent, repeated, non-version-3, or not the
first non-empty line.

Extraction reuses the same validated record stream and hierarchy model. It
supports both strands, multiple exons and transcripts, GFF3 and GTF, CDS
phase offsets, split GTF stop codons, multi-parent features, partial codons,
and internal stops. Plain FASTA can be indexed in memory without modifying the
input directory; compressed input requires its persistent random-access
indexes. Named outputs are staged together and become visible only after the
complete extraction succeeds.

All valid retained `rsomics-transcript-fasta` goldens remain byte-identical.
The old permissive parser is discarded: inverted coordinates, short records,
and GFF3 CDS records without a required phase now fail rather than being
repaired or skipped implicitly.

On the documented Apple M2 fixture of 80,000 three-exon transcripts,
`rsomics-annotation` and gffread produce byte-identical transcript, CDS, and
protein FASTA. The release-head verification uses five alternating warm-cache
trials with medians of 2.43 and 4.37 seconds respectively, a 1.80-times
throughput advantage. Peak RSS medians are 275,644,416 and 143,949,824 bytes,
so the implementation uses 1.91 times the memory. This passes the throughput
gate with an explicit memory tradeoff; it is not a universal performance
claim. Version 0.1.0 was published by run `30725542892`. The independently
downloaded archive has SHA-256
`4abbc46e44f4e98f5afa72bf3fba089f978d523aee739d33c592288d2e27b4ec`
and VCS revision `8e7beed4d51efb78e839cddf24288e04bf93134a`.

The old command inventory is consolidated as follows:

- count, head, feature/source/chromosome/strand summaries, and attribute
  inspection belong to `inspect`;
- feature extraction, grep, subset, and region selection belong to `view`;
- chromosome rename, sort, and split remain transformations inside this
  product, considered after the first slice;
- intron/UTR synthesis requires a validated transcript graph and is not copied
  from the old line-oriented modules;
- sequence extraction was refactored then merged from
  `rsomics-transcript-fasta`; its algorithms and valid goldens were retained,
  while the permissive parser and invalid missing-phase fixture were
  discarded;
- duplicate count-only commands and permissive line splitters are discarded.

Functional consequence annotation is a later complete slice:

- `consequence call` consumes the same validated transcript/CDS hierarchy and
  reference model as sequence extraction, then applies variants with explicit
  phasing, ploidy, overlapping-transcript, splice, symbolic-allele, and
  compound-event policy;
- `consequence inspect` lists and selects VEP `CSQ`, bcftools `BCSQ`, and
  SnpEff `ANN` fields;
- `consequence extract` renders selected consequence fields or promotes them
  to typed INFO tags.

The annotation product may parse VCF/BCF through an external standards-focused
library, but it does not depend on the Layer B `rsomics-vcf` product. Generic
VCF querying remains in `rsomics-vcf`; transcript-aware consequence semantics
remain here.

Historical source behavior was not safe to merge unchanged:

- intron generation mixes 1-based inclusive annotation coordinates with
  0-based half-open BED coordinates;
- annotation length uses `end - start` where inclusive GFF/GTF length is
  `end - start + 1`;
- the retired transcript parser silently swaps reversed coordinates and skips
  short lines; the merged implementation does neither.

The target parser uses an explicit one-based inclusive feature span and fails
loud on malformed records. Conversion to `rsomics-intervals::Interval` occurs
only at the checked BED/region boundary. A one-base feature `1..=1` must become
`[0, 1)`, and coordinate zero or an inverted span is invalid.

`rsomics-annotation view --region` is a streaming predicate for ordinary
single-range selection. It must not build an annotation-wide interval tree
solely to create a second `IntervalIndex` consumer.

## `rsomics-index`

### Boundary

One local genomic-resource indexing product with format-specific workflows as
subcommands:

- `bgzip`
- `tabix build`
- `tabix query`
- `tabix list`
- `faidx build`
- `faidx query`
- `dict`

The install identity is the resource preparation and random-access workflow,
not an individual index encoding. BGZF, GZI, TBI, CSI, FAI, and SAM sequence
dictionaries therefore do not become separate products.

### Upstream operation map

The compatibility sources are HTSlib 1.24 `bgzip` and `tabix`, samtools 1.24
`faidx` and `dict`, the SAMv1 BGZF and sequence-dictionary contracts, and the
TBI/CSI specifications. The stable 0.1 slice covers:

- BGZF compression, decompression, integrity testing, GZI creation and
  rebuilding, indexed byte-range reads, compression levels, text or binary
  block policy, and bounded worker counts;
- TBI or CSI construction with BED, GFF, SAM, and VCF presets or checked custom
  columns;
- inline, region-file, and streaming target queries, header modes, global
  deduplication, region separators, bounded BGZF cache, explicit indexes, and
  stored reference-name listing.

HTSlib 1.24 GAF indexing, reheadering, rebgzip layout reproduction, remote URI
and index discovery, metadata copying, implicit input deletion, and
multi-input invocation are explicit 0.1 exclusions. Unsupported operations
are absent from help rather than accepted as placeholders.

The next complete slice is FAI/GZI construction plus FASTA random-access
querying under `faidx`, followed by sequence-dictionary creation under `dict`.
FASTQ indexing, reverse-complement rendering, strand-label policy, alternate
locus dictionaries, and compressed-reference output indexing remain excluded
until their complete samtools contracts and fixtures are included.

### Implementation and evidence

The initial implementation runs from `acba3ec` through code head
`05960a4609a3b2acc388c0a149b5e023d53027f1`; repository head
`821d491042a92d35153efcdf160acebf381ca4ee` both withdraws the stale performance
claim and makes the formal harness build and authenticate its own exact-head
binary. Exact-code-head CI run `32331824268` and exact-repository-head run
`32340291429` pass on native Linux and macOS for both
`x86_64` and `aarch64`. The latter includes strict Clippy, debug and release
tests, rustdoc, package verification, and the Linux `x86_64` live compatibility
suite against pinned HTSlib 1.24. The always-run suite has 60 tests, with nine
additional HTSlib oracle tests.

The product uses `rsomics-help` for the complete nested command tree and
`rsomics-common` for structured success and failure output. Named output and
index pairs are staged before replacement. Malformed frames, CRC failures,
missing or duplicate EOF members, invalid GZI offsets, stale or corrupt tabix
indexes, unsorted records, malformed coordinates, write failures, and path
aliases fail nonzero without replacing an existing destination.

Compatibility covers cross-tool BGZF and GZI reads in both directions, TBI
and CSI structure and cross-reading, all four stable presets, custom columns,
header and query modes, large-coordinate boundaries, partial option defaults,
and semantic output hashes. A release review found and fixed one real drift:
when any tabix configuration flag is present, HTSlib starts from the GFF
defaults and overrides only supplied fields; requiring both sequence and begin
columns was incompatible.

The formal data at revision `df8089c8db89b5a3e064bb01d60414a47780f4d1`
is retained only as a historical optimization baseline. Later revisions
changed BGZF decompression and multi-region query algorithms, so those numbers
are not evidence for the current head. Publication requires a clean rebuild of
the exact head and a replacement 13-workload run with three warmups and ten
alternating measured pairs. No current performance claim is inherited from the
old micro-crates.

The replacement harness records and checks the clean Git head, release-binary
SHA-256, `Cargo.lock` SHA-256, toolchain, binary path, and build time before
correctness or timing work. It also rejects nonempty result directories, and
the summary command verifies the complete evidence manifest before rendering.

### Historical asset dispositions

The source assets are classified rather than revived:

- `rsomics-bgzip`: refactor then merge. Its basic stream and oracle fixtures
  informed the product, while input deletion, direct destination creation, and
  permissive lifecycle policy were discarded.
- `rsomics-tabix`: refactor then merge. Its presets, fixtures, binning work,
  and cross-tool evidence remain useful; the line-oriented query path and
  micro-crate CLI are replaced by the checked product model.
- `rsomics-fasta-index`: refactor then merge for future `faidx`; retain FAI and
  fetch goldens, but replace whole-file loading, lossy header conversion,
  non-transactional output, and the duplicate dictionary implementation.
- `rsomics-bam-dict`: refactor then merge for future `dict`; retain its
  streaming MD5 path, samtools option matrix, and goldens, then integrate them
  with the product's shared FASTA and output contracts.
- `rsomics-fm-search`: discard from this product. It rebuilds an in-memory FM
  index independently for every FASTA record and persists no index artifact.
  Exact sequence location belongs in `rsomics-seq` unless a real persistent
  search workflow and another consumer later justify a shared index API.
- `rsomics-fm-index`: internalize only if that later workflow proves it; its
  current single consumer does not justify a public foundation.

FAI and dictionary implementation will compare both historical sources instead
of selecting one wholesale. The shared parser must stream, preserve raw byte
offsets, reject duplicate or empty names, distinguish LF and CRLF line widths,
and make output replacement transactional before either operation is stable.

The current bgzip/tabix path uses bundled native libdeflate and is therefore an
FFI-backed dependency boundary. Product documentation and performance evidence
must say so explicitly.

Do not create `rsomics-hts-index` speculatively. BGZF frames and tabix policy
remain in this product until a second product demonstrates the same
format-neutral API with consumer tests. `rsomics-seqio` may receive shared
FASTA record or random-access contracts only when concrete `rsomics-index` and
another product call site, such as annotation extraction, agree on a
policy-free interface.

## Foundation corrections before product use

`rsomics-intervals` revision `491b14c0d43b58371723488dd8a9482d55a16678`
provides checked index construction and queries around its `i32` backend. The
product manifest targets 0.3 and CI patches that exact unpublished revision.
`intersect` consumes the fallible boundary directly.

The coordinate model is now exercised by both `rsomics-bed` and
`rsomics-annotation`: annotation constructs the shared half-open interval at
its checked BED conversion boundary. That does not automatically justify every
public item in the crate. The checked `IntervalIndex` still has only one
natural product consumer, `rsomics-bed`; it remains unpublished until another
product such as `rsomics-peak` or `rsomics-signal` demonstrates the same
policy-free query contract.

BED-specific sort, merge, and write helpers move into `rsomics-bed`.
The shared foundation retains:

- explicit half-open interval types;
- strand where geometrically relevant;
- reusable overlap primitives demonstrated by BED and annotation;
- checked interval indexing only after a second index consumer is concrete.

Consumer-contract tests define the public API item by item. An annotation
coordinate-conversion test does not count as an index-contract test.

## Recovery and provenance

Most source assets remain as local Git repositories. The absent local clones
`rsomics-bed-utils`, `rsomics-gff-head`, and `rsomics-gff-merge` are recoverable
from the verified Git bundles in the registry-reset archive.

Historical Rust code is team-owned and can be merged directly. Upstream
behavior, versions, fixtures, and third-party licenses still require explicit
provenance in the target products.
