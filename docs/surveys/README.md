# Upstream function survey — the granularity bedrock

The prerequisite to every granularity / dedup / "is this crate right" decision:
a verified inventory of **every upstream tool we reimplement**, its complete
function set, and where functions overlap across tools. Without this, "one op
one crate" and "dedup duplicates" are guesswork.

## Method (reliability is the point)

For each tool, the operation list is captured from the **most authoritative
source available**, in this priority:

1. **The actual binary** — `<tool> --help` / subcommand dump from the installed
   version. Ground truth; we record the exact version.
2. **Upstream source** — the repo's command registry / man pages, for tools not
   installed or to confirm completeness (a `--help` can hide subcommands).
3. **Official docs** — for semantics, flags, and edge behaviour.
4. **Web** — only to locate the above, never as the primary claim.

Every table records its source + version. A table is "verified" only when the
op list is cross-checked against at least source *or* docs (not just `--help`).

## What each survey records

Per operation: `operation | upstream tool(s) that provide it | our crate |
status | notes`. The cross-tool view (an operation provided by 2+ tools) is the
**dedup signal** — one canonical implementation, others depend on it; we do not
reimplement the same operation twice (see the `cross-tool-functional-dedup`
principle).

## Domains (mirrors docs/01..09)

- `01-formats-alignment.md` — samtools, bcftools, bedtools, seqkit, fastp,
  cutadapt, trimmomatic, fastqc, bbduk, vcftools
- `02-genomics.md` — bwa, bowtie2, minimap2, plink/plink2, gatk, sv callers
- `03-transcriptomics.md` — STAR, HISAT2, salmon, kallisto, featureCounts/Subread,
  RSeQC, Picard
- `04-single-cell.md`
- `05-epigenomics.md` — deeptools, MACS, SEACR, MethylDackel, Bismark
- `06-metagenomics.md` — kraken2, bracken, metabat2, dada2, vsearch, mmseqs2
- `07-proteomics-structure.md`
- `08-phylogenetics-popgen.md` — iqtree, raxml, mafft, muscle, vcftools popgen
- `09-workflow-utility.md`

Status legend: ✓ canonical crate · ⊃ covered by a multitool (dedup target) ·
gap (no crate yet) · adopt (use upstream Rust crate, don't rebuild).
