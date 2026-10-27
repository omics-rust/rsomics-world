# RNA-seq QC next-slice source recheck

Bounded parallel inventory while inferCNV remains the priority. No product
implementation, build, installation, deletion or release occurred. All21
historical clones are clean at the dossier-pinned revisions. No local
rsomics-rnaseq-qc clone or shared-scan candidate was found; live repository and
registry existence were not checked. Do not mistake a dossier for a product.

## Coherent next operation

Use the existing five-metric report contract: one BAM plus validated BED12,
mapping/strandedness/distribution/100-point coverage/per-transcript and sample
TIN, one shared traversal with metric-specific eligibility policies, and a
transactional JSON/TSV/manifest output set. Focused commands select the same
engine. Defer BigWig, Picard/refFlat/rRNA and other unfinished profiles rather
than inserting approximations. Coverage/TIN still need actual pileup oracles;
one traversal alone does not prove the metrics equivalent.

## Source-asset dispositions

All sources are under /Volumes/KIOXIA/Documents/omics-rust/.

| Asset and pinned commit | Reuse and necessary changes |
|---|---|
| `rsomics-bam-mapstat` `36a21482b2e76b03147d829469c4fd63b1e87aab` | Refactor driver; retain category counters/precedence and flag-diverse golden. Feed mapping before other metric exclusions. |
| `rsomics-bam-strandedness` `6e4e20b08fcd649d6578eb9c906433476d219c2f` | Refactor accumulator/finalizer; preserve first-N eligible gene-overlapping sampling and query-length overlap. |
| `rsomics-bam-read-dist` `2f39541572ebe2b795cba10275498f17333080ca` | Preferred midpoint/priority donor; replace readers, annotation parsing and scan wiring. Not yet accepted against the wheel. |
| `rsomics-read-distribution` `e169e6c9b5ab8f1b6af64ea633134931102d1e58` | Tests, fixtures and benchmarks only; do not merge its competing majority-overlap classifier. |
| `rsomics-genebody-coverage` `1813f1f6e7504be12c4dcd8e8d6f6e9d180ff482` | Retain percentile/minus-strand formulas and goldens; replace direct noodles reads and per-transcript indexed scans. |
| `rsomics-tin` `5e9fa05942c1ee999d87d475d3226638fa402c1a` | Refactor and reconcile entropy, summary eligibility, short-transcript sampling and background policy. Retain fixtures, not the entire driver. |
| `rsomics-rnaseq-metrics` `bb2f79492a3a5a4d8eec4f2d42542fd5f0235d51` | Retain Picard refFlat/rRNA/count/bias models and goldens for later complete profiles. Current metrics/bias passes are not a shared-scan report. |

The remaining14 donors stay in later junction/fragment/bias/duplication/
saturation dossier slices. No historical micro-crate is revived.

## Demonstrated source discrepancies

Root verified the current v5.05 GitHub tag resolves to commit
0d2e017723c5c4b41eb3b938e1631692d187de1f. This source inspection is not an
executed compatibility test of the separately pinned5.0.5 wheel.

- [read_distribution.py](https://github.com/liguowang/RSeQC/blob/0d2e017723c5c4b41eb3b938e1631692d187de1f/scripts/read_distribution.py#L281)
  assigns blocks by midpoint and priority. Actual donors disagree at
  bam-read-dist/src/driver.rs:74 and read-distribution/src/lib.rs:407. Both
  contain the same basic soft-clip/=/X projection quirks. Keep any required
  legacy projection separate from canonical alignment blocks.
- [geneBody_coverage2.py](https://github.com/liguowang/RSeQC/blob/0d2e017723c5c4b41eb3b938e1631692d187de1f/scripts/geneBody_coverage2.py#L29)
  consumes BigWig. The survey's multi-input description was wrong; ordinary
  geneBody_coverage supports multiple BAM inputs.
- [tin.py](https://github.com/liguowang/RSeQC/blob/0d2e017723c5c4b41eb3b938e1631692d187de1f/scripts/tin.py#L397)
  returns100*exp(entropy)/sampled_length for nonempty positive coverage,
  including entropy zero. The donor score.rs:263 instead returns zero.
  Upstream process_bam includes eligible zero scores in its summary;
  donor stats.rs:9 excludes every zero. Background and sampled-length policy
  also need actual wheel reconciliation. Misleading donor comments are not
  compatibility evidence.

## Foundations and fixture boundary

Existing foundations suffice: `bamio 0.8.4` at30459c78951fae406bd362854e7b80e42665a5c0,
`common 0.12.3` at77ff148b6b66089260be6a2896980185bdbcd4d2,
`help 0.4.0` at61dd6f2ce0cef6d9b4e349af5f96f96a7c95a013,
`intervals 0.4.0` atcda29c88b67882b63e1710a22aa480e905d4423f. Concrete reader/CIGAR
consumers include count/src/alignment.rs:110/163 and bam/src/to_bed/mod.rs:227;
transactional output is used in count/src/count.rs:217. Current count/BAM CLI
use rsomics_help::parse. Transcript/category/filter policy remains internal.
No new public API or foundation is justified by this inventory.

Previously documented HDD100k performance fixtures were checked absent, not
proven deleted. Existing KIOXIA/Developments/rsomics-fixtures assets are
`bench_3M.bam` (170,283,804bytes) and `gene_model_chr12.bed12` (5,450,292bytes). Historical
TIN speed notes include conda startup; Picard notes admit output differences.
Neither establishes shared-report correctness or representative speed/RSS.
The next implementation must pass one complete workflow gate and one fresh
whole-feature review, not a series of per-helper campaigns.

## Existing oracle availability

Bounded metadata/script/header lookup, not analysis execution: local
`/Volumes/KIOXIA/Developments/conda/envs/rs-rseqc` is RSeQC5.0.4,
Python3.10.20/pysam0.24.0/NumPy2.2.6. Existing4090
`/home/liangjy/miniforge3/envs/rs-up` is also RSeQC5.0.4,
Python3.14.5/pysam0.24.0/NumPy2.4.6. Both have the five scripts, all declaring
5.0.4. They recover historical context, not pinned5.0.5 acceptance. No5.0.5
wheel/source was found in the targeted external scratch/fixture lookup;
other caches and environments remain unverified. Do not blindly reinstall,
overwrite, or treat the current GitHub tag as the installed wheel.

Pinned Picard3.4.0 exists at
`/Volumes/KIOXIA/Developments/conda/envs/rs-up-x64/bin/picard`, verified from
its JAR manifest. Bare Homebrew picard is3.5.0, so use the explicit pinned path.
No Picard executable was present at the checked remote rs-up/bin/picard path;
other remote environments were not searched.

Local samtools/htslib are1.24;4090 is samtools1.21/htslib1.21-1-g4dc620e5.
The retained BAM header declares coordinate sorting and chr1/chr2 lengths
300,000,000. Its BAI is316,608bytes. BED12 has29,900 records per contig within
that dictionary. This is a structurally sensible synthetic stress pair, not a
representative RNA-seq cohort; actual sorting, splice/pair/overlap/coverage
behavior remains unverified. No full BAM scan, test, install or build occurred.

## Current release delta and oracle choice

Primary PyPI metadata independently confirms5.0.5 released2026-07-27 and
5.0.6 released2026-10-19. The5.0.5 wheel SHA is
a1fca6d455886b19bc2d58aa1e13b233977a2c835f676ef93bd0c6f4099b8452;
sdist c3f757e5f384004678c10cbc830355e841f5d5db850fc5b131c2e872afe9d516.
The5.0.6 wheel SHA is
b88ff92dece63619a86522710b633fb6f957bf2420d91a7e42b72e7f980ce461;
sdist b361a45ef4a9bb90e926d19abc7801b840897ccad4627f41750972d8cf18cad7.
See [5.0.5](https://pypi.org/project/RSeQC/5.0.5/) and
[5.0.6](https://pypi.org/project/RSeQC/5.0.6/). Neither release has a PyPI
provenance attestation; no wheel was downloaded or inspected in this lookup.

GitHub annotated tag v5.0.6 has object425109e505547617287d523d1066e91d5745c5dd,
resolving to da6e8687f64373fea75fc8a8f79adec9e202acc2. The
[exact source comparison](https://github.com/liguowang/RSeQC/compare/0d2e017723c5c4b41eb3b938e1631692d187de1f...da6e8687f64373fea75fc8a8f79adec9e202acc2)
does not change the five metric algorithms or bam_cigar projection. It changes
getBamFiles discovery used by coverage/TIN: both BAI naming forms, user-path
expansion, list-relative fallback, sorted discovery, string deduplication and
reported list errors. Symlink-directory equivalence is not established.
Packaging adds pandas/matplotlib; version strings now depend on installed
distribution metadata and cannot prove a separately supplied script's commit.

Ruling: retain the declared5.0.5 numerical-oracle baseline for the first
single-BAM report, with wheel/dependency acquisition still pending on hosted
compute. Record5.0.6's discovery behavior before implementing multi-input
profiles, not as a reason to silently change the dossier or compare against
installed5.0.4. Cost if wrong: a later explicit discovery-profile update; no
metric algorithm fix in this bounded comparison requires changing the current
numerical target. Wheel-to-repository byte equivalence remains unverified.
