# Larger real-input scaling source

Selected a public single-donor GBM count matrix for computational scaling,
not clinical validation or a source of inferred normal-cell labels.

Primary dataset and attribution: [10x Genomics, Human Glioblastoma Multiforme
whole-transcriptome dataset](https://www.10xgenomics.com/datasets/human-glioblastoma-multiforme-5-v-1-whole-transcriptome-analysis-1-standard-4-0-0),
CC BY4.0, Cell Ranger4.0.0. The original web summary identifies the actual
transcriptome as GRCh38-2020-A. Official [10x reference build notes](https://www.10xgenomics.com/support/software/cell-ranger/downloads/cr-ref-build-steps)
identify that reference's annotation as GENCODE32/Ensembl98; do not silently
join it to a current or differently assembled gene annotation.

Original archive URL:
https://cf.10xgenomics.com/samples/cell-vdj/4.0.0/Parent_SC5v1_Human_Glioblastoma/Parent_SC5v1_Human_Glioblastoma_filtered_feature_bc_matrix.tar.gz

External location: rsomics-fixtures/datasets/10x-gbm-4103-20261026/.
Archive46,963,441bytes,SHA
03c0552bb6e71fcb93483414baf24ccd435d82f775fe6d06b3e8036f858fe709.
Actual barcode/feature/header inspection establishes36,601 features,
4,103 cells and12,410,405 nonzero integer entries. Features use Ensembl IDs
and Gene Expression type; no matrix replication or synthetic cells are used.
This is header/cardinality inspection, not yet complete count validation.

The retained web-summary.html SHA is
eafccc013cc014992d74b8e3e702b99158e1fd83fe8feb8f105cc9aed6f0ca05.
The portal's direct HTML request returned429 and produced no file; its public
dataset metadata was read through the web source, and the published CDN
matrix/web-summary requests succeeded. No account or access control was bypassed.

Not ready as a benchmark yet: obtain/join the matching gene coordinates,
convert the sparse source to the currently supported strict TSV without
changing counts or claiming native10x support, and explicitly document the
all-cells/no-inferred-reference scaling configuration. Keep identity/row join
statistics and realistic resource limits. The previous184-cell benchmark
driver remains canonical-only; do not feed this cohort through its fixed
assertions or invent expected outputs. Full larger-input execution and
equivalent upstream comparison remain open.
