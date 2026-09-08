# rsomics reconstruction roadmap

This roadmap replaces the operation-per-crate campaign. Phase completion is
recorded, but unattended work continues into the next unblocked phase.

Each accepted product has one primary workstream below. The target lists must
match the [30-product allowlist](docs/00-overview/registry-reset-keep.txt)
exactly; foundations are cross-cutting dependencies, not additional products.
The [dossier index](docs/10-products/README.md) records per-product scope and
release slices. A recorded first release does not complete the whole family,
and a roadmap status is not a fresh registry or performance verification.

## Phase 0 — namespace and control-plane reset

Status: in progress.

- [x] Reconstruct the old portfolio from local code and live registries.
- [x] Select 30 product families and nine public foundations.
- [x] Back up every retirement candidate to external storage.
- [x] Remove obsolete crates.io entries and GitHub repositories.
- [x] Remove the all-yanked orphan `rsomics-bam` entry.
- [x] Replace operation-per-crate architecture rules.
- [ ] Replace stale per-operation registry and TODO references with product
      dossiers.

Gate: the live namespace, registry, conventions, and unattended instructions
describe the same architecture.

## Phase 1 — product dossiers

Status: a dossier baseline is recorded for all 30 products. Source and
upstream-contract review continues before each implementation or release.

For all 30 products:

- confirm real upstream tools and packages;
- enumerate user-recognizable operations from current documentation and allowed
  sources;
- deduplicate overlap across upstreams;
- map historical rsomics code, tests, fixtures, and benchmarks;
- identify shared-foundation consumers;
- define explicit exclusions and release slices.

Gate: every retained product has a defensible scope and an implementation-asset
map. No public code is required yet.

## Phase 2 — low-state consolidation pilots

Status: initial sequence, preprocessing, QC, BED and annotation slices are
recorded in their dossiers; further consolidation and the index release gate
remain open.

### Sequence pilot

Target products:

- `rsomics-seq`
- `rsomics-fastq-preprocess`
- `rsomics-fastq-qc`

Foundations exercised:

- `rsomics-common`
- `rsomics-help`
- `rsomics-seqio`
- `rsomics-kmer`

This pilot internalizes or replaces the temporary `rsomics-igzip` boundary
after the new `rsomics-seqio` consumer contract passes its compatibility and
performance gates.

### Interval pilot

Target products:

- `rsomics-bed`
- `rsomics-annotation`
- `rsomics-index`

Foundations exercised:

- `rsomics-common`
- `rsomics-help`
- `rsomics-intervals`
- `rsomics-seqio`

Gate: at least two coherent products demonstrate the standard repository,
subcommand, compatibility, benchmark, and foundation-consumer pattern.

## Phase 3 — alignment and variation

Status: active. BAM, VCF, calling and copy-number dossiers record implemented
first slices; RNA-seq QC, signal and peak reconstruction remain open. The
[VCF concat repair](.autopilot/state/vcf-index-selection-2026-09-09.md) now has
native source-snapshot evidence for index selection. The complete operation
remains an uncommitted candidate with resource, compatibility and performance
gates open, not an advertised release operation.

Targets:

- `rsomics-bam`
- `rsomics-vcf`
- `rsomics-call`
- `rsomics-cnv`
- `rsomics-rnaseq-qc`
- `rsomics-signal`
- `rsomics-peak`

Foundations:

- `rsomics-bamio`
- `rsomics-pileup`
- `rsomics-intervals`
- `rsomics-stats`

Use the recovered historical BAM packages and the classified BAM/VCF source
assets without reviving micro-crate repositories. Keep format operations,
variant calling, copy-number inference, RNA-seq QC and signal workflows in
their accepted product boundaries.

Gate: BAM and VCF products cover their declared release slices with
compatibility and resource evidence.

## Phase 4 — stateful statistical workflows

Status: planned.

Targets:

- `rsomics-deseq`
- `rsomics-edger`
- `rsomics-limma`
- `rsomics-sc`

Treat each as a data-model workflow, not a bag of independent statistical
functions. Move only genuinely reusable numerical primitives into
`rsomics-stats`.
The rejected `rsomics-expression` utility boundary stays excluded: count-matrix
collation belongs to `rsomics-count`, and significance labels remain local to
the product producing the analysis results.

Gate: workflow state, transformations, model fitting, result objects, and
compatibility evidence form coherent product contracts.

## Phase 5 — domain products

Status: mixed. Dossiers record initial composition, phylogeny, metagenomics,
methylation and sketch slices; the remaining families and deferred operations
still require reconstruction and release evidence.

Targets:

- `rsomics-plink`
- `rsomics-popgen`
- `rsomics-ecology`
- `rsomics-composition`
- `rsomics-phylo`
- `rsomics-metagenomics`
- `rsomics-structure`
- `rsomics-methyl`
- `rsomics-sketch`

Evolve `rsomics-phylo-tree`, `rsomics-kmer`, and `rsomics-stats` through
concrete consumers. Remove generic graph, image, ML, and statistics product
status unless a real omics workflow needs them.

## Phase 6 — remaining anchors and workflow integration

Status: mixed. Count and liftover dossiers record initial releases; minimap2
still requires reconstruction, and table remains a held release candidate.

Targets:

- `rsomics-count`
- `rsomics-liftover`
- `rsomics-minimap2`
- `rsomics-table`

Table is an accepted product with its own dossier. Sample-sheet validation is
consumer-owned metadata handling, not the rejected `rsomics-workflow` product.
External workflow engines may compose product binaries without introducing
Layer B to Layer B library dependencies.

Gate: each live product justifies its namespace, ships a coherent binary, and
has a maintained compatibility and performance contract.
