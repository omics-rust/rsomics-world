# Survey: metagenomics and sequence sketches

Status: refreshed 2026-07-31. This is an upstream landscape and routing
summary. The implementation contract is the
[metagenomics/sketch product dossier](../10-products/metagenomics-sketch.md).

## Accepted public boundaries

| Product | User-recognizable workflow | Primary upstream anchors |
|---|---|---|
| `rsomics-metagenomics` | abundance-aware amplicon processing, taxonomic database construction, read classification, taxonomy, and reports | VSEARCH 2.31.0, Kraken 2 2.17.1, NCBI Taxonomy |
| `rsomics-sketch` | persistent sketches, comparison, similarity/containment search, indexing, and mixture decomposition | sourmash 4.9.4, Mash 2.3 |

These products remain separate. A bounded persistent sketch and searchable
signature collection are a different durable model and installation identity
from exact amplicon abundance records or a taxonomy-labelled read-classifier
database.

The historical five metagenomics candidates and one sketch candidate are
implementation inputs, not the planned number of operations. No operation-sized
repository is revived.

## Operation routing

| User operation | Target owner | Decision |
|---|---|---|
| exact full/prefix dereplication with `;size=N` | `rsomics-metagenomics` | one `dereplicate` subcommand with named VSEARCH profiles |
| abundance sorting and rereplication | `rsomics-metagenomics` | amplicon abundance lifecycle |
| generic length sort, sampling, shuffle, conversion, validation | `rsomics-seq` | generic FASTA/FASTQ utilities |
| generic trimming, quality filtering, merging, correction, UMI, read deduplication | `rsomics-fastq-preprocess` | shared read-preprocessing pipeline |
| OTU/ASV clustering, chimera detection, amplicon reference search | `rsomics-metagenomics` | later complete amplicon slices |
| Kraken-style database build, inspect, classify, and report | `rsomics-metagenomics` | one tested database/classifier/report slice |
| NCBI taxonomy parsing, lineage, and LCA for classification | internal to `rsomics-metagenomics` | no one-consumer public foundation |
| FracMinHash/MinHash sketch, compare, search, index, gather | `rsomics-sketch` | persistent sketch workflow |
| taxonomic aggregation of versioned gather results | `rsomics-metagenomics` | file interoperability, no Layer B dependency |
| community diversity, dissimilarity, ordination, and permutation tests | `rsomics-ecology` | community-table analysis, not classification |

Shared user intent does not imply a shared algorithm. Kraken minimizer LCA,
alignment-based LCA, marker-gene profiling, Bayesian amplicon assignment, and
sketch containment require separate behavior profiles even when they all emit
taxonomic names. They remain modules or release slices inside a coherent
product unless a later portfolio review finds a genuinely distinct
installation identity.

## Current historical evidence

- `rsomics-derep`, the abundance half of `rsomics-fastx-sort`, and
  `rsomics-rereplicate` are meaningful VSEARCH migration assets, but they
  duplicate FASTA, header, output, CLI, and error handling.
- `rsomics-kraken-report` is a small parser seed. It does not yet model the
  standard hierarchy or the eight-column minimizer report.
- `rsomics-taxonomy` contains useful taxdump, lineage, and LCA seeds but exposes
  mutable invalid state and silently caps cycles. It is refactored and
  internalized.
- `rsomics-tax-assign` is discarded as a production classifier. It uses an
  unversioned exact-k-mer TSV, never performs taxonomy LCA, silently drops
  invalid windows, and emits a non-Kraken four-column format.
- `rsomics-kmer-dist` is an exact full k-mer profile, not a bounded sketch. It
  is retained only for formula fixtures and memory-baseline evidence.

The current code pool therefore supports a credible first amplicon slice but
does not support publishing a classifier or sketch placeholder.

## Broader domain survey

The metagenomics domain also contains:

- DADA2, Deblur, UNOISE, mothur, and QIIME 2 amplicon workflows;
- Centrifuge, Kaiju, MMseqs2, MetaPhlAn, mOTUs, ganon, and long-read
  classifiers;
- Bracken and other abundance re-estimation methods;
- HUMAnN and gene/pathway functional profiling;
- MEGAHIT/metaSPAdes assembly, MetaBAT/CONCOCT/SemiBin/VAMB binning,
  DAS Tool refinement, CheckM quality control, GTDB-Tk taxonomy, and dRep
  genome dereplication.

These are real upstream areas, but this survey does not mint one public crate
per upstream binary or algorithm. Each enters the accepted portfolio only
after an operation map answers:

1. whether it extends an existing metagenomics workflow or has a distinct
   installation identity;
2. what durable data model it shares with neighboring operations;
3. which historical or new implementation assets exist;
4. which oracle, database, license, fixture, and performance evidence is
   feasible;
5. whether a public foundation has two concrete product consumers.

Assembly/MAG recovery and functional profiling remain notable portfolio gaps,
not hidden promises in the first `rsomics-metagenomics` release.

## Compatibility and performance priorities

- Required VSEARCH, Kraken 2, sourmash, and Mash jobs install the pinned oracle
  and fail when it is unavailable.
- Classifier evidence covers database construction, integrity, taxonomy,
  single/paired output, confidence boundaries, reports, throughput, database
  bytes, and peak RSS together.
- Sketch evidence proves retained hashes, persistent-format interoperability,
  comparison/search results, deterministic platform behavior, and memory
  bounded by sketch parameters.
- Established-tool replacements require a strict throughput or material
  resource-use advantage on the relevant hot path.
- Database and reference-content licenses are reviewed independently from the
  upstream software license.
