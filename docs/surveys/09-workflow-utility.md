# Survey: workflow / utility domain

Verified 2026-05-30 against noodles crates, csvtk/datamash/seqtk docs.

## bgzip / tabix → CLI gap (library exists)
noodles-bgzf 0.47 + noodles-tabix 0.62 are live Quadrant-① pure Rust. The **gap is only the
CLI binary**: no `rsomics-bgzip` (compress/decompress/-b/-s byte-range) and no generic
`rsomics-tabix` (index+query for BED/GFF). `rsomics-vcf-index` already covers VCF tabix via
noodles-tabix. **These are quick-win high-value crates** (used in nearly every pipeline).

## csvtk → `rsomics-csv-utils` multitool gap
~30 subcommands; we cover **2** (cut/rename → `rsomics-tsv-select`; join → `rsomics-tsv-join`),
both TSV-only (no CSV dialect). Big gaps: filter/filter2/grep, sort, uniq, freq, summary,
head/tail/sample, gather/spread/transpose, mutate, corr, split, concat. → a `rsomics-csv-utils`
multi-subcommand crate (analogous to how bed-utils bundles bed ops) + a `--csv` flag on the
existing tsv-select/tsv-join. (Note: a *bundled* utility crate is the right shape here — these
are thin awk-class column ops, not distinct algorithms.)

## datamash → `rsomics-tsv-stats` gap
sum/mean/median/min/max/stdev/var/count/unique after groupby; collapse/expand; transpose;
crosstab; pcov/ppearson/pspearman. Covered only as BED-domain analogues (`rsomics-bed-groupby`,
`rsomics-bed-expand`). → `rsomics-tsv-stats` (groupby + aggregations) covers most use.

## seqtk vs seqkit → already covered (same-op dedup)
seqtk and seqkit ~80% overlap. Canonical per-op already chosen: subseq→`rsomics-fasta-subseq`,
sample→`rsomics-fastq-sample` (cites seqtk), trimfq→`rsomics-fastq-trim`, comp→fasta/fastq-stats,
mergepe→`rsomics-fastq-pair`, kmer→`rsomics-kmer`, gc→fasta-stats/read-gc. **No `rsomics-seqtk`
wrapper.** Niche gaps: hety, telo, randbase (low priority).

## MultiQC → adopt-upstream
Python+JS HTML aggregator that *reads our tool outputs*. rsomics role: ensure every QC tool
emits structured `--json`. A future `rsomics-qc-report` (Tera/Askama) would consume those
feeds. Not a crate-building priority.

## parallel / xargs → out of scope (OS process mgmt).

## Gap summary
| gap | severity | action |
|---|---|---|
| rsomics-bgzip (CLI) | HIGH | new crate, wraps noodles-bgzf |
| rsomics-tabix (generic BED/GFF) | HIGH | new crate, wraps noodles-tabix |
| csvtk filter/sort/freq/summary/… | HIGH | rsomics-csv-utils multitool |
| CSV dialect in tsv-select/join | MEDIUM | --csv flag |
| datamash groupby+agg | MEDIUM | rsomics-tsv-stats |
| seqtk hety/telo/randbase | LOW | niche |

## Verification notes
noodles-bgzf 0.47 / noodles-tabix 0.62 confirmed via cargo search. csvtk subcommands from
bioinf.shenwei.me/csvtk docs; datamash from GNU man; seqtk from lh3/seqtk README. All HIGH
confidence.
