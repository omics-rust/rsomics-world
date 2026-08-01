# rsomics public namespace

This file indexes intended public product and foundation boundaries. It is not
an index of every historical implementation module.

Live state is verified against crates.io and the
[omics-rust organization](https://github.com/omics-rust). Historical
operation-sized code is indexed separately in
[`docs/00-overview/portfolio-inventory.tsv`](docs/00-overview/portfolio-inventory.tsv).

Status meanings are defined in [CONVENTIONS.md](CONVENTIONS.md).

## Product families

| Product | Status | Initial scope / upstream anchors |
|---|---|---|
| `rsomics-annotation` | repo-only | GFF/GTF inspection, selection, transformation, sequence extraction |
| `rsomics-bam` | planned | gate-complete SAM/BAM/CRAM release candidate; name cooldown ends 2026-08-02 19:56:02 UTC |
| `rsomics-bed` | live | BED/interval algebra and bedtools-like operations |
| `rsomics-call` | live | alignment pileup, genotype likelihoods, and lightweight small-variant calling |
| `rsomics-cnv` | planned | BAF/LRR copy-number HMM and chromosome-level polysomy analysis |
| `rsomics-composition` | planned | Aitchison geometry, log-ratio transforms, zero handling, proportionality, and composition-native inference |
| `rsomics-count` | planned | feature and read counting workflows |
| `rsomics-deseq` | planned | DESeq2-style differential-expression workflow |
| `rsomics-ecology` | planned | community-table diversity, ecological dissimilarity, ordination, metadata association, and permutation analysis |
| `rsomics-edger` | planned | edgeR-style differential-expression workflow |
| `rsomics-fastq-preprocess` | repo-only | trimming and filtering (initial fastp-compatible subset); later correction, UMI, merge, deduplication |
| `rsomics-fastq-qc` | pilot | FASTQ quality metrics and reports |
| `rsomics-index` | planned | bgzip/tabix and reusable sequence-index workflows |
| `rsomics-liftover` | repo-only | UCSC chain coordinate liftOver |
| `rsomics-limma` | planned | limma/voom workflow |
| `rsomics-metagenomics` | planned | abundance-aware amplicon processing, taxonomic databases, read classification, and reports |
| `rsomics-methyl` | planned | methylation extraction and analysis |
| `rsomics-minimap2` | live | published legacy FFI wrapper; reconstruction required |
| `rsomics-peak` | planned | chromatin peak calling, annotation, and quantification |
| `rsomics-phylo` | planned | alignment trimming, evolutionary distance, tree inference, comparison, measures, and later likelihood/species-tree/placement workflows |
| `rsomics-plink` | planned | PLINK-style genotype analysis |
| `rsomics-popgen` | planned | population-genetic statistics outside the PLINK workflow |
| `rsomics-rnaseq-qc` | planned | RSeQC/Picard-like RNA-seq quality control |
| `rsomics-sc` | planned | stateful single-cell and spatial analysis workflow |
| `rsomics-seq` | repo-only | coherent FASTA/FASTQ sequence utility suite |
| `rsomics-signal` | planned | deepTools/bigWig signal workflows |
| `rsomics-sketch` | planned | persistent sequence sketches, comparison, search, indexing, and mixture decomposition |
| `rsomics-structure` | planned | protein structure and PDB analysis |
| `rsomics-table` | planned | bioinformatics-oriented table manipulation |
| `rsomics-vcf` | live | VCF/BCF inspection, transformation, filtering, indexing, and format statistics |

Product status summary:

| Status | Count |
|---|---:|
| live | 4 |
| repo-only | 4 |
| pilot | 1 |
| planned | 21 |
| **Total** | **30** |

`pilot` means the boundary and source audit are active. It does not imply that
a public repository or installable release exists. `repo-only` likewise does
not imply that the crate has passed its publication gate.

## Public foundations

| Foundation | crates.io | GitHub | Pilot drivers and later consumers |
|---|---|---|---|
| `rsomics-common` | live | live | all 30 accepted products |
| `rsomics-help` | live | live | all 30 accepted CLI products |
| `rsomics-bamio` | live | live | BAM, variant calling, RNA-seq QC, signal |
| `rsomics-intervals` | live | live | BED, annotation, peak, signal |
| `rsomics-kmer` | live | live | pilot: sequence; later: metagenomics and sketch |
| `rsomics-seqio` | live | live | current: sequence and preprocessing; planned: FASTQ QC and indexing |
| `rsomics-stats` | live | live | composition, expression workflows, single-cell, ecology, population genetics |
| `rsomics-phylo-tree` | live | live | composition balance bases, phylogenetics, ecology |
| `rsomics-pileup` | live | live | BAM, variant calling, methylation |

## Temporary public dependency

| Crate | Status | Reason |
|---|---|---|
| `rsomics-igzip` | temporary | non-yanked `rsomics-seqio 0.1.1` depends on it on Linux |

The goal is to internalize or replace the igzip boundary. Do not build new
products directly against it.

## Control plane

| Repository | Purpose |
|---|---|
| `rsomics-world` | product dossiers, architecture, registry, audit scripts, durable state |
| `.github` | organization metadata |

## Registry rules

- A planned name is not published until its product dossier and first release
  slice are implemented and verified.
- A repository is created only for a product or justified public foundation.
- Historical micro-crate names remain retired.
- Live counts and metadata are refreshed before every release or registry
  decision.
