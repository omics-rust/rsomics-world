# Product reconstruction dossiers

This directory is the product-level map between the retained historical source
pool and the public rsomics portfolio.

A dossier is an implementation contract, not a promise that every historical
operation will ship. It records:

- upstream scope and user-recognizable operations;
- overlapping implementations and the selected canonical operation;
- historical code, test, fixture, and benchmark assets;
- target subcommands and internal modules;
- public-foundation consumers;
- compatibility and performance gates;
- explicit exclusions and staged release slices.

## Portfolio map

| Product | Source candidates | Dossier state |
|---|---:|---|
| `rsomics-annotation` | 4 | [audited](interval-annotation-index.md#rsomics-annotation) |
| `rsomics-bam` | 41 | [audited](bam.md) |
| `rsomics-bed` | 42 | [audited](interval-annotation-index.md#rsomics-bed) |
| `rsomics-call` | 2 | [audited](variant.md#rsomics-call) |
| `rsomics-cnv` | 2 | [audited](variant.md#rsomics-cnv) |
| `rsomics-composition` | 10 | [audited](composition.md) |
| `rsomics-count` | 4 | [audited](count.md) |
| `rsomics-deseq` | 12 | [audited](bulk-expression.md#rsomics-deseq) |
| `rsomics-ecology` | 19 | [audited](ecology.md) |
| `rsomics-edger` | 17 | [audited](bulk-expression.md#rsomics-edger) |
| `rsomics-fastq-preprocess` | 12 | [audited](sequence-fastq.md#rsomics-fastq-preprocess) |
| `rsomics-fastq-qc` | 1 | [released 0.1.0](fastq-qc-gate-2026-08-02.md) |
| `rsomics-index` | 5 | [audited](interval-annotation-index.md#rsomics-index) |
| `rsomics-liftover` | 1 | [released 0.1.0](liftover.md) |
| `rsomics-limma` | 16 | [audited](bulk-expression.md#rsomics-limma) |
| `rsomics-metagenomics` | 5 | [audited](metagenomics-sketch.md#rsomics-metagenomics) |
| `rsomics-methyl` | 1 | [audited](methyl.md) |
| `rsomics-minimap2` | 1 | [audited](minimap2.md); legacy release requires reconstruction |
| `rsomics-peak` | 5 | [audited](peak.md); four workflow assets and one discarded generic candidate |
| `rsomics-phylo` | 11 | [audited](phylo.md) |
| `rsomics-plink` | 42 | [audited](genotype-popgen.md#rsomics-plink) |
| `rsomics-popgen` | 14 | [audited](genotype-popgen.md#rsomics-popgen) |
| `rsomics-rnaseq-qc` | 21 | [audited](rnaseq-qc-signal.md#rsomics-rnaseq-qc) |
| `rsomics-sc` | 29 | [audited](sc.md) |
| `rsomics-seq` | 34 | [audited](sequence-fastq.md#rsomics-seq) |
| `rsomics-signal` | 15 | [audited](rnaseq-qc-signal.md#rsomics-signal) |
| `rsomics-sketch` | 1 | [audited](metagenomics-sketch.md#rsomics-sketch) |
| `rsomics-structure` | 9 | [audited](structure.md) |
| `rsomics-table` | 16 | [audited](table.md) |
| `rsomics-vcf` | 30 | [audited](variant.md#rsomics-vcf) |

Counts are generated from
[`portfolio-inventory.tsv`](../00-overview/portfolio-inventory.tsv). They
describe recoverable inputs, not planned subcommand counts.

All 30 accepted product boundaries now have a source and upstream-operation
dossier. An audited dossier authorizes reconstruction work; it does not mean
the product or every listed operation is implemented or release-ready.

## Rejected public boundary

| Former candidate | Source candidates | Decision |
|---|---:|---|
| `rsomics-workflow` | 1 | [rejected](workflow.md); the sample-sheet asset is consumer-specific metadata validation, not a workflow product |
| `rsomics-expression` | 2 | [rejected](bulk-expression.md#rejected-rsomics-expression-boundary); count-matrix collation belongs to `rsomics-count`, and significance labels are product-local reporting |

## Relationship map

```mermaid
flowchart TB
    subgraph sequence["Sequence and FASTQ pilot"]
        seq["rsomics-seq"]
        prep["rsomics-fastq-preprocess"]
        qc["rsomics-fastq-qc"]
    end

    subgraph interval["Interval and annotation pilot"]
        bed["rsomics-bed"]
        annotation["rsomics-annotation"]
        index["rsomics-index"]
        liftover["rsomics-liftover"]
    end

    common["rsomics-common"] --> seq
    common --> prep
    common --> bed
    common --> annotation
    common --> liftover
    common --> qc
    common -. planned .-> index
    help["rsomics-help"] --> seq
    help --> prep
    help --> bed
    help --> annotation
    help --> liftover
    help --> qc
    help -. planned .-> index
    seqio["rsomics-seqio"] --> seq
    seqio --> prep
    seqio --> qc
    seqio -. planned .-> index
    kmer["rsomics-kmer"] --> seq
    intervals["rsomics-intervals"] --> bed
    intervals --> annotation
    intervals --> liftover
```

Solid arrows are current consumer contracts. Dotted arrows are planned
consumers and do not justify a public API by themselves. None implies that a
foundation API is already stable.
