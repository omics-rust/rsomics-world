# rsomics reconstruction roadmap

This roadmap replaces the operation-per-crate campaign. Phase completion is
recorded, but unattended work continues into the next unblocked phase.

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

Status: active.

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

Status: planned.

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

Status: planned.

Targets:

- `rsomics-bam`
- `rsomics-vcf`
- `rsomics-rnaseq-qc`
- `rsomics-signal`
- `rsomics-peak`

Foundations:

- `rsomics-bamio`
- `rsomics-pileup`
- `rsomics-intervals`
- `rsomics-stats`

Recover the yanked historical `rsomics-bam` package and all BAM/VCF
operation-sized source assets. Separate format operations from RNA-seq QC and
signal workflows.

Gate: BAM and VCF products cover their declared release slices with
compatibility and resource evidence.

## Phase 4 — stateful statistical workflows

Status: planned.

Targets:

- `rsomics-deseq`
- `rsomics-edger`
- `rsomics-limma`
- `rsomics-expression`
- `rsomics-sc`

Treat each as a data-model workflow, not a bag of independent statistical
functions. Move only genuinely reusable numerical primitives into
`rsomics-stats`.

Gate: workflow state, transformations, model fitting, result objects, and
compatibility evidence form coherent product contracts.

## Phase 5 — domain products

Status: planned.

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

Status: planned.

Targets:

- `rsomics-count`
- `rsomics-liftover`
- `rsomics-minimap2`
- `rsomics-table`
- `rsomics-workflow`

Review whether generic table and workflow functionality belongs in standalone
rsomics products or should remain internal support.

Gate: each live product justifies its namespace, ships a coherent binary, and
has a maintained compatibility and performance contract.
