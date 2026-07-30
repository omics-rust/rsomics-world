# Interval, annotation, and index product dossier

Status: the BED first slice is verified. Annotation source assets and upstream
boundaries have been audited; annotation and index implementation has not
started.

Routing corrections move table aggregation to `rsomics-table`, SEACR to
`rsomics-peak`, FASTA masking to `rsomics-bed`, and FASTA indexing to
`rsomics-index`. The resulting source pool has 42 BED, two annotation, and four
index candidates.

## Shared design

`rsomics-intervals` owns coordinate-safe interval geometry and indexing.
`rsomics-bed` owns BED syntax and bedtools-style command policy.
`rsomics-annotation` owns GFF/GTF semantics. `rsomics-index` owns compression
and index workflows.

The public geometry layer does not absorb product-specific header, formatting,
or command behavior.

## `rsomics-bed`

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
current subtract implementation no longer builds an unused overlap tree and
must be remeasured on the representative fixture before publication. The
repository remains unpublished.

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

The source assets are `rsomics-gff-utils` and `rsomics-transcript-fasta`.
`rsomics-gff-utils` contains 27 small commands but reparses the same record in
most modules and has no shared typed record model. Its useful algorithms and
fixtures are assets, not the target structure. `rsomics-transcript-fasta` has
stronger gffread sequence goldens and a reusable transcript assembly path, but
its parser silently skips short records and swaps inverted coordinates.

The first implementation slice is:

- `validate`, with explicit GFF3 or GTF dialect selection and record-level
  failures carrying line numbers;
- `view`, preserving valid records while combining type, attribute, region,
  containment, and head limits;
- `to-bed`, converting 1-based inclusive features to 0-based half-open
  intervals at one checked boundary.

Transcript, CDS, and protein FASTA extraction follows on the same record and
hierarchy model. It is not advertised until the retained gffread goldens pass
through that model.

The old command inventory is consolidated as follows:

- count, head, feature/source/chromosome/strand summaries, and attribute
  inspection belong to `inspect`;
- feature extraction, grep, subset, and region selection belong to `view`;
- chromosome rename, sort, and split remain transformations inside this
  product, considered after the first slice;
- intron/UTR synthesis requires a validated transcript graph and is not copied
  from the old line-oriented modules;
- sequence extraction is refactored then merged from
  `rsomics-transcript-fasta`;
- duplicate count-only commands and permissive line splitters are discarded.

Current source behavior is not safe to merge unchanged:

- intron generation mixes 1-based inclusive annotation coordinates with
  0-based half-open BED coordinates;
- annotation length uses `end - start` where inclusive GFF/GTF length is
  `end - start + 1`;
- transcript extraction silently swaps reversed coordinates and skips short
  lines.

The target parser uses an explicit one-based inclusive feature span and fails
loud on malformed records. Conversion to `rsomics-intervals::Interval` occurs
only at the checked BED/region boundary. A one-base feature `1..=1` must become
`[0, 1)`, and coordinate zero or an inverted span is invalid.

`rsomics-annotation view --region` is a streaming predicate for ordinary
single-range selection. It must not build an annotation-wide interval tree
solely to create a second `IntervalIndex` consumer.

## `rsomics-index`

### Boundary

User-facing compression and indexing workflows:

- `bgzip`
- `tabix build`
- `tabix query`
- `tabix list`
- `fasta-index`

The source assets are `rsomics-bgzip`, `rsomics-tabix`,
`rsomics-fasta-index`, and `rsomics-fm-search`.

The first release slice is BGZF compression plus tabix build/query/list.
`fasta-index` follows the FASTA reader and random-access contract in
`rsomics-seqio`. FM search remains deferred until its product fit and second
consumer are concrete.

The current bgzip/tabix path uses bundled native libdeflate and is therefore an
FFI-backed dependency boundary. Product documentation and performance evidence
must say so explicitly.

Do not create `rsomics-hts-index` speculatively. Reconsider a neutral shared
index foundation only after both `rsomics-index` and a second product such as
`rsomics-vcf` require the same API.

## Foundation corrections before product use

`rsomics-intervals` revision `491b14c0d43b58371723488dd8a9482d55a16678`
provides checked index construction and queries around its `i32` backend. The
product manifest targets 0.3 and CI patches that exact unpublished revision.
`intersect` consumes the fallible boundary directly.

The coordinate model is justified by `rsomics-bed` and the planned
`rsomics-annotation` conversion boundary. That does not automatically justify
every public item in the crate. The checked `IntervalIndex` still has only one
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
