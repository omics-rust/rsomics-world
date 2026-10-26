# Real GBM CLI worker scaling

Status: complete count-preserving preparation and actual whole-CLI scaling
measurement. The five predeclared alternating pairs all completed. This is
loaded-host computational evidence, not R-comparison or biological validation.
The fixture-only converter has one integrated sparse-to-TSV witness; matching
GENCODE v32 data and derived files remain on the fixture HDD. No native10x
product support or normal-cell identity is inferred from this conversion.

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
All annotations are explicitly gbm_unclassified. Actual CLI execution retains
9,465 genes/4,103 cells, selects all9,465 genes and produces a single
all_observations tree with4,102 merge rows, without reference outliers.

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
green-21 Linux binary and existing Python controllers were transferred.
The measured selection was1/4 workers on four affinity-pinned CPUs;
this is a different host, never a direct timing ratio against hosted CI.
Remote preparation completed with identical counts/positions/annotations
SHA-256 to the external local preparation. Existing native binary executes
on glibc2.35 and its SHA agrees with the retained CI binary. World controller
40310284331 passed at 236641bd4a2568bc06aca898e6073d30fb5c1964.
Measurement completed in gbm-cli-benchmark/, using taskset CPUs0-3, five
alternating measured pairs after two warmups,1800s per-child timeout. The
former owner2320170 has exited; it is not a live process handle. Do not
duplicate this completed campaign or signal an old PID without ownership.

Actual measured one-worker wall times:131.53,131.72,131.03,132.51,131.50s.
Four-worker times:53.93,52.86,53.32,54.60,52.61s. Medians131.53/53.32s,
ratio2.466804; complete-range separation holds (parallel max54.60 < serial
min131.03). Median max RSS is2,335,035,392bytes for both selections (2.17GiB).
Median user CPU129.04/135.04s and system2.50/2.26s. Four workers reduce wall
time while using slightly more CPU work, not less total computation.
Warmups130.64/52.68s are retained but excluded from measured statistics.

All12 distinct owned child records exit zero with empty process groups.
Retained actual time files, report hashes/fingerprints, source scripts and
binary hash agree; root recomputed statistics from the ten measured records.
Every report has fingerprint
320f476a87363a1d1e88caa8f741980f9752b69e228c335bdb4cd38dcc0c86a8.
The measured retained binary SHA is
50dd80358e80f4e9b6d10a4828cdc12303f7d95d7ca09240ae9482c9da178320.
The controller SHA is
bb8febe17ef125808d09fce883953b4de98b4ecf2ec35b72c71a971f452f2331.
Original reports, stdout/stderr, time/process records, provenance, summary,
host CPU output, preparation manifest, executed scripts and binary are
copied externally to evidence/infercnv-gbm-scaling-2026-10-26/4090-U4Wbro/.
Its71-file collected-evidence.sha256 has SHA
7ba7b32e17bba1438c53f2af1eac427b8e2e62f2a21a92dfefb735795f18e5c0.

Host: Intel Xeon Gold6238R, two28-core sockets/112 logical CPUs, Linux6.8,
glibc2.35, Python3.10.12, GNU time. Quota unavailable, not proven unlimited.
The host was busy: load219.45/236.64/237.07 before,262.28/240.53/237.70 after;
a2s sample showed pinned CPUs0-3 busy100/93/92/95%. Other jobs were untouched.
Predeclared affinity/trials were not changed and no slow sample was removed.
This establishes useful scaling on this loaded host, not a dedicated/idle-host
or upstream replacement-speed gate. Never ratio these times against hosted
CI or infer CNV validity from successful computation.

Upstream-comparison preflight:4090 has R4.6.1 and Docker, but its default R
libraries lack infercnv/fastcluster/parallelDist (Matrix1.7.5 is installed).
No installation or existing library was changed. Existing source-pinned
oracle/measurement CI remains available for the equivalent upstream work;
do not interpret a missing package query as a portfolio blocker or restart
the already accepted parser/kernel campaigns.
No registry publication, deletion, Mac compilation or system-library change
occurred. Both original inputs and completed outputs have external copies.

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
The converter subsequently validates all entries and count conservation.

The retained web-summary.html SHA is
eafccc013cc014992d74b8e3e702b99158e1fd83fe8feb8f105cc9aed6f0ca05.
The portal's direct HTML request returned429 and produced no file; its public
dataset metadata was read through the web source, and the published CDN
matrix/web-summary requests succeeded. No account or access control was bypassed.

Next: equivalent upstream whole-workflow comparison and the remaining
inferCNV denoise/HMM/reporting work, then the rest of the portfolio. Keep
the existing canonical case assertions and the distinct gbm-4103 contract.
Do not rerun accepted per-helper oracle or artifact campaigns.
