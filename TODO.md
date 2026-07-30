# rsomics current work

This is the executable control-plane checklist. Product-specific operation
maps belong in their dossiers under `docs/`.

## P0.1 — finish the architecture reset

- [x] Freeze the 30-product and nine-foundation allowlist.
- [x] Back up all crates.io and GitHub retirement candidates.
- [x] Delete 594 dependency-ordered crates.io reset candidates.
- [x] Delete 596 GitHub reset candidates.
- [x] Back up and delete the all-yanked orphan `rsomics-bam`.
- [x] Replace the operation-per-crate instructions and conventions.
- [x] Regenerate `REGISTRY.md` from live GitHub, crates.io, and the allowlist.
- [ ] Convert stale domain documents from crate queues into product-operation
      surveys.
- [x] Record the final reset gate and exact live counts.

## P0.2 — product dossiers

Each dossier must include upstream scope, operations, deduplication, historical
assets, target modules, shared consumers, compatibility, performance, and
exclusions.

- [x] `rsomics-annotation`
- [ ] `rsomics-bam`
- [x] `rsomics-bed`
- [ ] `rsomics-composition`
- [ ] `rsomics-count`
- [ ] `rsomics-deseq`
- [ ] `rsomics-ecology`
- [ ] `rsomics-edger`
- [ ] `rsomics-expression`
- [x] `rsomics-fastq-preprocess`
- [x] `rsomics-fastq-qc`
- [x] `rsomics-index`
- [ ] `rsomics-liftover`
- [ ] `rsomics-limma`
- [ ] `rsomics-metagenomics`
- [ ] `rsomics-methyl`
- [ ] `rsomics-minimap2`
- [ ] `rsomics-peak`
- [ ] `rsomics-phylo`
- [ ] `rsomics-plink`
- [ ] `rsomics-popgen`
- [ ] `rsomics-rnaseq-qc`
- [ ] `rsomics-sc`
- [x] `rsomics-seq`
- [ ] `rsomics-signal`
- [ ] `rsomics-sketch`
- [ ] `rsomics-structure`
- [x] `rsomics-table`
- [ ] `rsomics-vcf`
- [ ] `rsomics-workflow`

## P0.3 — public-foundation audit

- [x] Complete the initial source and API audit for all nine retained
      foundations.
- [x] `rsomics-common`: narrow the runtime through real product consumers.
- [x] `rsomics-help`: replace duplicate CLI models with the shared Clap layer.
- [x] `rsomics-seqio`: derive strict stream contracts from sequence and
      preprocessing.
- [ ] `rsomics-kmer`: add a second product consumer and four-native-target
      release gate.
- [ ] `rsomics-intervals`: review public items individually and demonstrate a
      second consumer for the checked index contract.
- [ ] Reconstruct `rsomics-bamio` and `rsomics-pileup` through BAM and VCF.
- [ ] Reconstruct `rsomics-stats` through at least two stateful workflows.
- [ ] Reconstruct `rsomics-phylo-tree` through phylogenetics and ecology.
- [ ] Internalize or replace `rsomics-igzip`, then remove its public registry
      boundary when immutable dependencies allow.
- [ ] Correct repository metadata in future foundation releases.

## P1 — sequence pilot

- [x] Complete the `seq/fastq` historical asset classification.
- [x] Define the `rsomics-seq` subcommand surface.
- [x] Define preprocessing versus QC boundaries for FASTQ.
- [x] Create or recover target product repositories on KIOXIA.
- [x] Merge the first consumer slice without reviving micro-crate dependencies.
- [x] Consolidate golden and compatibility tests for the implemented operations.
- [ ] Add end-to-end tests composing multiple subcommands.
- [x] Benchmark implemented slices against the real upstream tools.
- [x] Review `common` and `seqio` public API changes through the `seq` and
      `fastq-preprocess` consumers.
- [x] Review `help` through sequence, preprocessing, BED, and annotation.
- [ ] Review `kmer` through its required second product consumer.

## P1 — interval pilot

- [x] Complete the `bed/annotation/index` historical asset classification.
- [x] Define the `rsomics-bed` subcommand surface.
- [x] Separate annotation semantics from generic interval algebra.
- [x] Separate indexing/compression concerns from BED operations.
- [ ] Recover `rsomics-bed-utils`, `rsomics-gff-head`, and `rsomics-gff-merge`
      from verified Git bundles where local clones are absent.
- [x] Merge the first release slice.
- [x] Consolidate compatibility fixtures and benchmarks.
- [x] Review `common`, `help`, and the interval coordinate model through BED
      and annotation.
- [ ] Review sequence-index I/O through the `rsomics-index` consumer.

## P2 — alignment and variation

- [ ] Recover the archived `rsomics-bam 0.1.0/0.2.0` packages as source assets.
- [ ] Classify all BAM and VCF historical implementations.
- [ ] Define BAM/VCF format-operation boundaries versus RNA-seq QC and signal.
- [ ] Review `bamio`, `pileup`, and `intervals` APIs.
- [ ] Reconstruct coherent `rsomics-bam` and `rsomics-vcf` products.

## Durable evidence

- Namespace allowlist:
  `docs/00-overview/registry-reset-keep.txt`
- Source routing ledger:
  `docs/00-overview/portfolio-inventory.tsv`
- Reconstruction rationale:
  `docs/00-overview/portfolio-reconstruction.md`
- Registry reset archives:
  `/Volumes/KIOXIA/Documents/omics-rust/_retired/`
- Machine progress journals:
  `.autopilot/state/`
