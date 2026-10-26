# Larger real-input scaling source

Update: the fixture-only converter scripts/prepare_infercnv_10x_fixture.py
is implemented and the real conversion completed. One integrated witness
passes, including exact
row/column count placement, version-normalized gene coordinates, unchanged
barcodes and explicit unclassified annotations. The existing coherent
controller suite passes 440 tests in 48.205s, with two native-only skips.
The matching GENCODE v32 GTF and derived data remain on the fixture HDD.
This does not add native
10x ingestion or inferred normal-cell identities to the product.

The real conversion validates all 12,410,405 positive integer entries and
preserves all 36,601 rows/4,103 barcodes, total 54,032,413 UMIs. Every gene
matches the version-normalized GENCODE v32 coordinates; unmatched count zero.
Exported gene totals, barcode totals and nonzero cardinality agree with the
original sparse source. No counts are filtered or normalized by conversion.
prepared-v1/ contains counts.tsv, positions.tsv, annotations.tsv, manifest.json
and the retained little-endian row-major u32 intermediate. CLI chromosome/
expression filters remain separate and must be reported during execution.
Counts SHA ffa286fe314011bf0e4998fe9cbc631d08e0cc19457da2e68d4e6580e400b4f8.
Positions SHA 3a6fb0f3cebf38350917c08e91b781b3b1752fc51e3e88a044480d3ce524a145.
Annotations SHA 372845bcf2512bf37db5cff70d1ddbf07754fed96de92c86c58b44c8e71fbb17.
All annotations are explicitly gbm_unclassified. CLI execution and actual
large-cohort timing/RSS are still unmeasured, not inferred from conversion.

The existing samples measurement driver now has an explicit gbm-4103 case,
not a weakened canonical expectation. It uses all-cells reference, cutoff0.1,
4GiB requested numeric budget, one unclassified observations tree covering
all 4,103 cells and no invented expected post-filter gene count. All retained
genes must be selected, outlier map null; reports must remain equal across
workers/repetitions. It reuses the same owned child/timing/RSS mechanism.
The five driver tests pass, including this distinct fixture contract.
Final coherent controller suite: 441 tests pass in 47.452s with the two
existing native-only skips. No local Rust/R build was run.

Remote 4090 is reachable: native Linux x86_64, glibc2.35, Python3.10.12,
112 affinity-visible CPUs and about959GiB available RAM. Its root NVMe has
296GiB free (84% used); no compiler/install/dependency download is requested.
Private scratch was created as /tmp/rsomics-cnv-20261026.U4Wbro and must be
retained, not cleaned destructively. Only public source data, the retained
green-21 Linux binary and existing Python controllers are being transferred.
The measured selection will be 1/4 workers on four affinity-pinned CPUs;
this is a different host, never a direct timing ratio against hosted CI.
Remote preparation completed with identical counts/positions/annotations
SHA-256 to the external local preparation. Existing native binary executes
on glibc2.35 and its SHA agrees with the retained CI binary. World controller
40310284331 passed at 236641bd4a2568bc06aca898e6073d30fb5c1964.
Actual measurement is now running in the original remote scratch under
gbm-cli-benchmark/, using taskset CPUs0-3, five alternating measured pairs
after two warmups, 1800s per-child timeout. Owner is the Python command
measure_infercnv_samples_cli.py --case gbm-4103; do not launch a duplicate.
The observed owner PID is2320170 (verify argv/ownership before signaling).
First actual serial warmup succeeds: whole CLI130.64s, maximum RSS
2,335,039,488 bytes, all4,103 cells pass the report contract. This is a warmup,
not one of the five measured serial trials. The driver found the host busy:
load219.45/236.64/237.07 at start; a later2s sample shows pinned CPU0-3 busy
100/93/92/95%. Other jobs are untouched. Keep the original predeclared affinity
and alternating trials, retain this as loaded-host computational evidence,
not a dedicated/idle-host or release performance advantage. Do not tune away
slow samples, compare these times to hosted CI, or infer CNV validity.
The successful report actually retains9,465 genes/4,103 cells, selects all
9,465 genes and returns one all_observations tree with4,102 merge rows.
Four-worker warmup succeeds52.68s at the same peak RSS. Initial measured
pair1 is131.53/53.93s; these are partial observations, not final medians.
Output fingerprints remain identical so far. Do not summarize until all
five predeclared pairs complete.

Upstream-comparison preflight:4090 has R4.6.1 and Docker, but its default R
libraries lack infercnv/fastcluster/parallelDist (Matrix1.7.5 is installed).
No installation or existing library was changed. Existing source-pinned
oracle/measurement CI remains available for the equivalent upstream work;
do not interpret a missing package query as a portfolio blocker or restart
the already accepted parser/kernel campaigns.
Results are pending. Preserve any failure/partial logs, never fabricate a
complete summary. Collect the original remote records to external fixtures
after completion; remote scratch is not the sole durable copy of inputs.

Original GTF URL (HTTPS variant of the official build-notes URL):
https://ftp.ebi.ac.uk/pub/databases/gencode/Gencode_human/release_32/gencode.v32.primary_assembly.annotation.gtf.gz
GTF SHA 0c36c37abb0c88637b04a8e401c93f0199a65bbdd0702c9e13bdfbadc053fe1e.
An initial timeout/restart was replaced by a resumable transfer; no partial
download was used for conversion and no source/intermediate was deleted.

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

Next: execute the existing strict-TSV CLI with the explicitly documented
all-cells/no-inferred-reference scaling configuration and a numeric budget
large enough for the actual dense matrices. The previous184-cell benchmark
driver remains canonical-only; do not feed this cohort through its fixed
assertions or invent expected outputs. Full larger-input execution and
equivalent upstream comparison remain open.
