# Survey: workflow and cross-cutting utilities

Status: refreshed 2026-07-31. This survey routes real operations into accepted
products; it does not create a generic utility crate or workflow engine.

## Accepted routing

| Upstream area | Accepted owner | Decision |
|---|---|---|
| bgzip/tabix | `rsomics-index` | compression, index construction, inspection, and region-query workflows |
| csvtk/datamash | `rsomics-table` | one coherent delimited-table product with subcommands |
| seqtk/SeqKit overlaps | `rsomics-seq` and `rsomics-fastq-preprocess` | route by sequence utility versus read-preprocessing semantics |
| sample manifests | the consuming Layer B product | schema and biological policy stay with the command that consumes them |
| MultiQC | external integration | emit stable structured outputs; do not rewrite a mature report aggregator without a distinct value case |
| Snakemake/Nextflow/Galaxy/CWL/WDL | external integration | wrappers and examples may be useful; no current `rsomics-workflow` product |
| generic process parallelism | operating system or workflow engine | out of rsomics product scope |

## Format and indexing utilities

The historical bgzip/tabix candidates are audited under `rsomics-index`.
`noodles-bgzf`, `noodles-tabix`, and related Rust libraries are implementation
choices rather than independently published rsomics foundations. The product
owns user-facing compression and query behavior; reusable APIs move public
only with two concrete consumers.

VCF indexing still belongs to the coherent `rsomics-vcf` user workflow where
format policy differs. Shared BGZF/index primitives are reviewed through real
call sites rather than duplicated or promoted speculatively.

## Delimited tables

csvtk and datamash expose related manipulation and aggregation operations.
They route to `rsomics-table`, not one crate for cut, join, group, transpose,
or summary.

The product dossier defines dialects, schemas, row/column identities,
streaming/materialization, numeric and missing-value behavior, compatibility
oracles, and performance gates. BED policy and product sample-sheet policy do
not enter the generic table product merely because their files are tabular.

## Sequence utilities

seqtk and SeqKit overlap heavily. Generic stats, selection, transformation,
subsequence, and sorting route to `rsomics-seq`; trimming, filtering, pairing,
merging, correction, UMI, and read deduplication route to
`rsomics-fastq-preprocess`.

No wrapper crate mirrors an upstream binary name. A niche operation enters one
of the existing products only when its semantics, oracle, and value gate are
complete.

## Workflow engines

Snakemake, Nextflow, WDL/Cromwell, CWL runners, Galaxy, and Toil are real
workflow products with mature languages, schedulers, executor ecosystems,
cache/resume behavior, environment/container integration, and provenance.

The [workflow boundary review](../10-products/workflow.md) rejects the current
`rsomics-workflow` name. Its sole historical candidate is a three-column
sample-path checker, not an engine. A Rust workflow runner would be a new
large product and requires an unmet user need plus a complete execution model;
it is not inferred from the existence of many rsomics binaries.

Rsomics instead makes every product orchestration-friendly through stable exit
codes, transactional outputs, machine-readable results, deterministic
profiles, resource controls, and small composable examples.

## Sample sheets and metadata

Illumina BCL Convert sample sheets, nf-core pipeline sheets, Snakemake JSON
Schema tables, PEP, SDRF, and product-local read manifests are not one format.
Validation belongs to the consuming product or the external workflow schema.

A policy-free parser, diagnostic type, or path resolver can become shared only
after two implemented products demonstrate the same contract. The historical
`rsomics-sample-sheet` does not establish that contract and is classified as a
rejected workflow-metadata capability asset.

## Visualization and reporting

MultiQC, IGV/JBrowse, and plotting ecosystems are generally consumers of
product outputs. The default rsomics strategy is interoperable structured
data, not a new plotting or dashboard crate. A future visualization product
requires its own recognizable workflow and evidence rather than accumulating
unrelated renderers.
