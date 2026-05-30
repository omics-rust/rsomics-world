# Survey: metagenomics / amplicon domain

Verified 2026-05-30 against kraken2 wiki, Bracken/MetaBAT2/CONCOCT/SemiBin READMEs,
vsearch.1 man page, MMseqs2 wiki, DADA2 (rdrr.io), vegan/phyloseq reference pages.

> **Domain status: largely GREENFIELD.** Only 5 partial crates exist (rsomics-kraken-report,
> rsomics-tax-assign, rsomics-taxonomy, rsomics-derep, rsomics-phylo-tree); none is a complete
> perfgated Layer-B tool. This is the biggest build-out opportunity after the gaps in genomics.

## Kraken2 family (MIT) → all gap

kraken2 classify (--paired/--confidence/--quick/--minimum-hit-groups/--use-mpa-style/
--report-minimizer-data/--memory-mapping) → `rsomics-kraken` (P0). kraken2-build (--standard/
--download-{taxonomy,library}/--add-to-library/--build/--protein/--special{greengenes,rdp,
silva,gtdb}) → `rsomics-kraken-build` (P0). kraken2-inspect → `rsomics-kraken-report` (partial;
note: report-*parsing* ≠ DB-*inspect* reading .k2d — two distinct tools).

## Bracken (GPL → clean-room) → all gap
bracken-build / bracken / est_abundance / generate_kmer_distribution / kmer2read_distr /
kreport2mpa → `rsomics-bracken` (P0, as `--mode bracken` subcommand of rsomics-kraken).

## Binning (MetaBAT2 BSD / MaxBin2 / CONCOCT / SemiBin) → all gap
metabat2 + jgi_summarize_bam_contig_depths → `rsomics-metabat` (P0). MaxBin2 (LOW confidence,
SourceForge 404), CONCOCT (cut_up_fasta/coverage_table/concoct/merge/extract), SemiBin
(single/multi_easy_bin, bin_long, feature-gen — PyTorch siamese, inference-only Rust via
candle/burn) → subcommands-of rsomics-metabat (P2).

## VSEARCH (BSD-2/GPL-3) — exhaustive man page

Chimera (uchime_denovo/2/3, uchime_ref) → `rsomics-chimera`. Clustering (cluster_size/fast/
smallmem/unoise) → `rsomics-otu-cluster`. Derep (derep_fulllength/prefix/fastx_uniques/id/
smallmem) → `rsomics-derep` (partial: full-length+prefix done; FASTQ/smallmem/id gap; add --uc
output for compat). Search (usearch_global/allpairs_global/search_exact, makeudb*) →
`rsomics-seq-search`. **Overlaps with existing fastx crates** (fastq_filter→fastq-filter,
fastq_mergepairs→fastq-merge, fastx_revcomp→fasta-utils, fastx_subsample→fastq-sample,
sortbysize/length→fastx-sort, fastx_mask→fasta-mask, sintax→tax-assign, fasta2fastq/
fastq_convert→fastx-convert) — these are *same-op*, route to the existing crate (verify
size= annotation + EE-metric modes supported).

## MMseqs2 (GPL → clean-room) → all gap
easy-cluster/linclust + cluster/linclust/clust/clusterupdate → `rsomics-mmseqs-cluster`.
easy-search/rbh + search/prefilter/align/convertalis/map/extractorfs/translatenucs →
`rsomics-mmseqs-search`. easy-taxonomy + taxonomy/taxonomyreport/addtaxonomy →
`rsomics-mmseqs-taxonomy`. createdb/createindex/createtaxdb/createtsv → `rsomics-mmseqs-build`.

## DADA2 (LGPL; clean-room from Callahan 2016 paper) → `rsomics-dada2` (P0)
filterAndTrim, derepFastq, learnErrors (Poisson EM), dada (ASV inference), mergePairs,
makeSequenceTable, removeBimeraDenovo, assignTaxonomy (kmer-bootstrap-Bayes), addSpecies +
~30 utility fns. Keep as ONE crate (error model tightly coupled to quality encoding;
derepFastq feeds dada()). Deblur (BSD) = `--mode deblur` subcommand. UNOISE3 lives in
`rsomics-otu-cluster` (vsearch --cluster_unoise), not separate.

## R packages (GPL → clean-room; deep-dive in 10-r-bioconductor.md)

- **phyloseq**: phyloseq()/otu_table/tax_table/sample_data containers; import_{biom,qiime,mothur};
  estimate_richness; distance() (44 methods inc. UniFrac); ordinate(); prune/filter/transform.
- **vegan** (HIGH conf, full reference): diversity/renyi/fisher.alpha/rarefy/specaccum/
  estimateR (Chao1/ACE) → `rsomics-diversity`; vegdist/designdist/betadiver/raupcrick →
  `rsomics-diversity`; rda/cca/capscale/metaMDS/decorana/procrustes/varpart → `rsomics-ordinate`;
  adonis2(PERMANOVA)/anosim/mrpp/mantel/simper/betadisper → `rsomics-permanova`;
  decostand/wisconsin (transforms).
- **microbiome** (LOW conf): core/prevalence, DMM typing, DOC, bimodality — lower priority.

## Cross-tool dedup signals (metagenomics)

| op | upstreams | decision |
|---|---|---|
| dereplication | vsearch derep + seqkit rmdup + DADA2 derepFastq | **same op** → `rsomics-derep` canonical (+ --uc) |
| clustering | vsearch (greedy) + mmseqs2 (cascaded) + DADA2 dada (denoise) | **distinct algorithms** → 3 crates: otu-cluster / mmseqs-cluster / dada2 (do NOT merge) |
| chimera | vsearch UCHIME + DADA2 bimera | related but distinct (pre- vs post-denoise) → `rsomics-chimera` + dada2-integrated |
| taxonomy | kraken2(kmer-LCA) + mmseqs2(aln-LCA) + DADA2(Bayes-kmer) + vsearch sintax + QIIME2 | **distinct algorithms** → kraken / mmseqs-taxonomy / dada2 / tax-assign(sintax) — do NOT merge |
| fastq filter / merge / sort | vsearch ≈ DADA2 ≈ existing fastq-* crates | same op → route to existing fastq-filter/merge/fastx-sort |

The dedup lesson here: a shared *user intent* (e.g. "assign taxonomy") often hides **distinct
algorithms** that legitimately warrant separate crates — the opposite of the bed-utils case.
Merge only when the algorithm is the same (dereplication, fastq filtering), not when only the
goal is shared.

## Gap summary (new crates needed)
P0: kraken, kraken-build, bracken, metabat, dada2, megahit, checkm, gtdbtk.
P1: chimera, otu-cluster, seq-search, mmseqs-{cluster,search,taxonomy,build}, diversity,
ordinate, permanova, drep, annotate, metaphlan, ganon, humann.
Complete partials: derep (FASTQ/smallmem/uc), kraken-report (compat+perfgate), tax-assign
(sintax), taxonomy (LCA vs NCBI taxdump).

## Verification notes
HIGH: kraken2/build/inspect wiki, Bracken README, MetaBAT2 (Bitbucket), CONCOCT, SemiBin,
vsearch.1 (most exhaustive), MMseqs2 wiki, DADA2 rdrr.io (41 fns), vegan reference (complete).
MODERATE: Deblur (only `workflow` documented), phyloseq (import HIGH, full fn list incomplete).
LOW: MaxBin2 (mirrors 404 — from literature), microbiome R (tutorial summary only).
