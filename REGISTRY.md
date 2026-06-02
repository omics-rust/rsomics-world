# rsomics crate registry

Every crate is an independent repo under [omics-rust](https://github.com/omics-rust) and is published to crates.io. This file is the index; there is no submodule aggregation. Clone any crate flat under `/Volumes/KIOXIA/Documents/omics-rust/` to work on it.

_Generated 2026-05-30 — 231 crates._

| crate | description |
|---|---|
| [rsomics-aitchison-ops](https://github.com/omics-rust/rsomics-aitchison-ops) | Aitchison-simplex algebra (closure / centralize / perturb / power) on compositional data — scikit-bio CoDA port (value-exact, 7.08× -t1) |
| [rsomics-align-core](https://github.com/omics-rust/rsomics-align-core) | Pairwise sequence alignment kernels (Smith-Waterman + Needleman-Wunsch, affine gap) for th |
| [rsomics-align-score](https://github.com/omics-rust/rsomics-align-score) | Pairwise sequence alignment — Needleman-Wunsch (global) or Smith-Waterman (local) |
| [rsomics-alignment-sieve](https://github.com/omics-rust/rsomics-alignment-sieve) | Filter a BAM by mapq / fragment length / SAM flags / blacklist / duplicates — Rust port of deepTools alignmentSieve |
| [rsomics-alpha-diversity](https://github.com/omics-rust/rsomics-alpha-diversity) | Per-sample alpha-diversity metrics (Shannon/Simpson/Chao1/ACE/Pielou…) from a feature count table — Rust port of scikit-bio diversity.alpha |
| [rsomics-alr](https://github.com/omics-rust/rsomics-alr) | Additive log-ratio (ALR) transform of a compositional table — scikit-bio alr port (byte-exact, 5.3× CPU / 7.5× wall) |
| [rsomics-ancom](https://github.com/omics-rust/rsomics-ancom) | ANCOM differential-abundance test: per-feature W = significant log-ratio test count + percentile detection — Rust port of scikit-bio ancom (W integer-exact + decisions, 129× -t1, 49× lower RSS) |
| [rsomics-anosim](https://github.com/omics-rust/rsomics-anosim) | ANOSIM R-statistic test on a distance matrix (tie-averaged ranks + permutation p) — Rust port of scikit-bio anosim (R value-exact, 9.58× -t1) |
| [rsomics-atac-shift](https://github.com/omics-rust/rsomics-atac-shift) | ATAC-seq Tn5 insertion-bias shift: +4/-5 bp coordinate correction and insertion-site BED o |
| [rsomics-avelogcpm](https://github.com/omics-rust/rsomics-avelogcpm) | Per-gene average log2-CPM via edgeR's one-group negative-binomial fit — Rust port of edgeR aveLogCPM (byte-exact, 7.60× -t1) |
| [rsomics-bam-addreplacerg](https://github.com/omics-rust/rsomics-bam-addreplacerg) | Add or replace @RG header lines and RG:Z aux tags on BAM records — Rust port of samtools a |
| [rsomics-bam-ampliconclip](https://github.com/omics-rust/rsomics-bam-ampliconclip) | Clip amplicon primer regions off aligned reads given a BED — Rust port of samtools amplico |
| [rsomics-bam-ampliconstats](https://github.com/omics-rust/rsomics-bam-ampliconstats) | Amplicon sequencing statistics from primer BED + BAM — Rust port of samtools ampliconstats |
| [rsomics-bam-bedcov](https://github.com/omics-rust/rsomics-bam-bedcov) | Per-BED-region read depth — Rust port of samtools bedcov |
| [rsomics-bam-calmd](https://github.com/omics-rust/rsomics-bam-calmd) | Recompute the MD and NM aux tags against a reference FASTA — Rust port of samtools calmd |
| [rsomics-bam-cat](https://github.com/omics-rust/rsomics-bam-cat) | Concatenate BAM files by copying compressed BGZF blocks verbatim — Rust port of samtools c |
| [rsomics-bam-checksum](https://github.com/omics-rust/rsomics-bam-checksum) | Order-independent BAM checksum — Rust port of samtools checksum |
| [rsomics-bam-collate](https://github.com/omics-rust/rsomics-bam-collate) | Group BAM reads by QNAME so mates are adjacent — Rust port of samtools collate |
| [rsomics-bam-compare](https://github.com/omics-rust/rsomics-bam-compare) | Per-bin comparison of two BAMs as a bedGraph/bigWig track — Rust port of deeptools bamComp |
| [rsomics-bam-consensus](https://github.com/omics-rust/rsomics-bam-consensus) | FASTA/FASTQ/pileup consensus from a sorted BAM — Rust port of samtools consensus (simple m |
| [rsomics-bam-coverage](https://github.com/omics-rust/rsomics-bam-coverage) | Per-reference coverage histogram from BAM — Rust port of samtools coverage |
| [rsomics-bam-depad](https://github.com/omics-rust/rsomics-bam-depad) | Convert padded to unpadded BAM coordinates — Rust port of samtools depad |
| [rsomics-bam-depth](https://github.com/omics-rust/rsomics-bam-depth) | Per-base or per-region depth from BAM — Rust port of samtools depth |
| [rsomics-bam-dict](https://github.com/omics-rust/rsomics-bam-dict) | Generate SAM-format sequence dictionary from FASTA — Rust port of samtools dict |
| [rsomics-bam-divide](https://github.com/omics-rust/rsomics-bam-divide) | Randomly divide a BAM into N roughly-equal parts — Rust port of RSeQC divide_bam.py |
| [rsomics-bam-fasta](https://github.com/omics-rust/rsomics-bam-fasta) | Convert BAM to FASTA — Rust port of samtools fasta |
| [rsomics-bam-fingerprint](https://github.com/omics-rust/rsomics-bam-fingerprint) | ChIP-enrichment fingerprint (cumulative-coverage Lorenz curve) — Rust port of deeptools pl |
| [rsomics-bam-fixmate](https://github.com/omics-rust/rsomics-bam-fixmate) | Fill in mate coordinate, ISIZE and mate-related flags — Rust port of samtools fixmate |
| [rsomics-bam-flags](https://github.com/omics-rust/rsomics-bam-flags) | Convert between numeric and textual SAM FLAG representations — Rust port of samtools flags |
| [rsomics-bam-flagstat](https://github.com/omics-rust/rsomics-bam-flagstat) | SAM/BAM/CRAM flag statistics — Rust port of samtools flagstat |
| [rsomics-bam-head](https://github.com/omics-rust/rsomics-bam-head) | Print the header and the first N alignment records of a BAM as SAM — Rust port of samtools |
| [rsomics-bam-idxstats](https://github.com/omics-rust/rsomics-bam-idxstats) | Per-reference read counts from a BAM index — Rust port of samtools idxstats |
| [rsomics-bam-import](https://github.com/omics-rust/rsomics-bam-import) | Convert FASTQ to unaligned BAM — Rust port of samtools import |
| [rsomics-bam-index](https://github.com/omics-rust/rsomics-bam-index) | Create BAI index for a coordinate-sorted BAM — Rust port of samtools index |
| [rsomics-bam-junctions](https://github.com/omics-rust/rsomics-bam-junctions) | Annotate splice junctions from spliced BAM reads vs BED12 gene model — Rust port of RSeQC  |
| [rsomics-bam-mapstat](https://github.com/omics-rust/rsomics-bam-mapstat) | BAM mapping-statistics summary (splice / unique / proper-pair categories) — Rust port of RSeQC bam_stat.py |
| [rsomics-bam-markdup](https://github.com/omics-rust/rsomics-bam-markdup) | Mark or remove PCR/optical duplicates in sorted BAM — Rust port of samtools markdup |
| [rsomics-bam-merge](https://github.com/omics-rust/rsomics-bam-merge) | Merge multiple sorted BAM files — Rust port of samtools merge |
| [rsomics-bam-mpileup](https://github.com/omics-rust/rsomics-bam-mpileup) | Per-position text pileup of read bases, qualities and map qualities — Rust port of samtool |
| [rsomics-bam-phase](https://github.com/omics-rust/rsomics-bam-phase) | Heterozygote phasing of aligned reads — Rust port of samtools phase |
| [rsomics-bam-quickcheck](https://github.com/omics-rust/rsomics-bam-quickcheck) | Quickly validate a BAM file's BGZF framing and header magic — Rust port of samtools quickc |
| [rsomics-bam-read-dist](https://github.com/omics-rust/rsomics-bam-read-dist) | Classify mapped reads into genomic regions (CDS/UTR/intron/TSS/TES) from BAM + BED12 — Rus |
| [rsomics-bam-region](https://github.com/omics-rust/rsomics-bam-region) | Extract BAM reads overlapping a genomic region (chr:start-end) — indexed random access |
| [rsomics-bam-reheader](https://github.com/omics-rust/rsomics-bam-reheader) | Replace a BAM header, passing alignment blocks through verbatim — Rust port of samtools re |
| [rsomics-bam-reset](https://github.com/omics-rust/rsomics-bam-reset) | Revert aligner changes in BAM reads (flags, position, cigar, aux tags, orientation) — Rust |
| [rsomics-bam-samples](https://github.com/omics-rust/rsomics-bam-samples) | List @RG sample names from one or more BAM headers — Rust port of samtools samples |
| [rsomics-bam-signal](https://github.com/omics-rust/rsomics-bam-signal) | Binned BAM → bedGraph/bigWig signal track — Rust port of deeptools bamCoverage |
| [rsomics-bam-sort](https://github.com/omics-rust/rsomics-bam-sort) | BAM sorting by coordinate or read name — Rust port of samtools sort |
| [rsomics-bam-split](https://github.com/omics-rust/rsomics-bam-split) | Split BAM by read group — Rust port of samtools split |
| [rsomics-bam-split-gene](https://github.com/omics-rust/rsomics-bam-split-gene) | Split a BAM by a BED12 gene model into on-exon / off-gene / junk BAMs — Rust port of RSeQC split_bam.py |
| [rsomics-bam-split-pe](https://github.com/omics-rust/rsomics-bam-split-pe) | Split a paired-end BAM into read-1/read-2/unmapped BAMs — Rust port of RSeQC split_paired_bam.py |
| [rsomics-bam-stats](https://github.com/omics-rust/rsomics-bam-stats) | Comprehensive alignment statistics from BAM — Rust port of samtools stats |
| [rsomics-bam-strandedness](https://github.com/omics-rust/rsomics-bam-strandedness) | Infer RNA-seq library strand protocol from BAM + BED12 gene model — Rust port of RSeQC inf |
| [rsomics-bam-subsample](https://github.com/omics-rust/rsomics-bam-subsample) | Random downsampling of BAM/SAM records by fraction or target count |
| [rsomics-bam-targetcut](https://github.com/omics-rust/rsomics-bam-targetcut) | Identify target intervals from pileup depth — Rust port of samtools targetcut |
| [rsomics-bam-to-bed](https://github.com/omics-rust/rsomics-bam-to-bed) | Convert BAM alignments to BED6 format — Rust port of bedtools bamtobed |
| [rsomics-bam-to-fastq](https://github.com/omics-rust/rsomics-bam-to-fastq) | Extract FASTQ reads from BAM — Rust port of samtools fastq |
| [rsomics-bam-view](https://github.com/omics-rust/rsomics-bam-view) | View, filter, and convert SAM/BAM/CRAM alignments — Rust port of samtools view |
| [rsomics-bamio](https://github.com/omics-rust/rsomics-bamio) | Parallel-BGZF BAM reader/writer + raw-record edit shared by the rsomics-bam-* tool family. |
| [rsomics-barcode-rank](https://github.com/omics-rust/rsomics-barcode-rank) | Barcode rank statistics from a counts-per-barcode file — knee plot data for single-cell QC |
| [rsomics-bbduk](https://github.com/omics-rust/rsomics-bbduk) | K-mer-based contaminant removal + adapter/quality trimming for FASTQ — independent clean-r |
| [rsomics-bbi](https://github.com/omics-rust/rsomics-bbi) | Pure-Rust bigWig/BBI reader and writer: header, chromosome B-tree, R-tree interval search, |
| [rsomics-bed-annotate](https://github.com/omics-rust/rsomics-bed-annotate) | Annotate BED intervals with per-file overlap fractions from N annotation files — Rust port |
| [rsomics-bed-closest](https://github.com/omics-rust/rsomics-bed-closest) | Find the closest feature in B for each interval in A — bedtools closest equivalent |
| [rsomics-bed-cluster](https://github.com/omics-rust/rsomics-bed-cluster) | Cluster overlapping BED intervals and append a cluster ID — bedtools cluster equivalent |
| [rsomics-bed-complement](https://github.com/omics-rust/rsomics-bed-complement) | Compute the complement of a BED file — intervals not covered by any feature (bedtools comp |
| [rsomics-bed-count](https://github.com/omics-rust/rsomics-bed-count) | Count BED records (non-header, non-blank lines) |
| [rsomics-bed-coverage](https://github.com/omics-rust/rsomics-bed-coverage) | Per-interval coverage depth/breadth of B features onto A — Rust port of bedtools coverage |
| [rsomics-bed-expand](https://github.com/omics-rust/rsomics-bed-expand) | Replicate BED/TSV lines by expanding comma-separated column values — Rust port of bedtools |
| [rsomics-bed-fisher](https://github.com/omics-rust/rsomics-bed-fisher) | Fisher's exact test for overlap significance between two BED interval sets — bedtools fish |
| [rsomics-bed-flank](https://github.com/omics-rust/rsomics-bed-flank) | Create flanking BED intervals for each feature — bedtools flank equivalent |
| [rsomics-bed-genomecov](https://github.com/omics-rust/rsomics-bed-genomecov) | Genome-wide coverage from BED intervals — bedgraph, per-base depth, and histogram modes |
| [rsomics-bed-getfasta](https://github.com/omics-rust/rsomics-bed-getfasta) | Extract FASTA sequences for BED intervals — bedtools getfasta equivalent |
| [rsomics-bed-groupby](https://github.com/omics-rust/rsomics-bed-groupby) | Group tab-delimited rows by key columns and aggregate value columns — bedtools groupby equ |
| [rsomics-bed-intersect](https://github.com/omics-rust/rsomics-bed-intersect) | Intersect BED intervals — bedtools intersect equivalent |
| [rsomics-bed-jaccard](https://github.com/omics-rust/rsomics-bed-jaccard) | Compute Jaccard similarity statistic between two BED files — bedtools jaccard equivalent |
| [rsomics-bed-len](https://github.com/omics-rust/rsomics-bed-len) | Append interval length (end - start) as a new column to BED records |
| [rsomics-bed-makewindows](https://github.com/omics-rust/rsomics-bed-makewindows) | Tile a genome into fixed-size BED windows — bedtools makewindows equivalent |
| [rsomics-bed-map](https://github.com/omics-rust/rsomics-bed-map) | Aggregate column values from B intervals overlapping each A interval — bedtools map equiva |
| [rsomics-bed-maskfasta](https://github.com/omics-rust/rsomics-bed-maskfasta) | Mask FASTA bases overlapping BED intervals — bedtools maskfasta equivalent |
| [rsomics-bed-merge](https://github.com/omics-rust/rsomics-bed-merge) | Merge overlapping BED intervals — bedtools merge equivalent |
| [rsomics-bed-midpoint](https://github.com/omics-rust/rsomics-bed-midpoint) | Collapse BED intervals to their midpoints — outputs a 1-bp BED record at floor((start+end) |
| [rsomics-bed-multicov](https://github.com/omics-rust/rsomics-bed-multicov) | Count reads from multiple BAM files overlapping BED intervals — bedtools multicov equivale |
| [rsomics-bed-multiinter](https://github.com/omics-rust/rsomics-bed-multiinter) | Multi-file interval intersection depth — bedtools multiinter equivalent |
| [rsomics-bed-nuc](https://github.com/omics-rust/rsomics-bed-nuc) | Per-interval nucleotide composition from BED + FASTA — Rust port of bedtools nuc |
| [rsomics-bed-overlap](https://github.com/omics-rust/rsomics-bed-overlap) | Compute overlap or distance between two interval columns — bedtools overlap equivalent |
| [rsomics-bed-random](https://github.com/omics-rust/rsomics-bed-random) | Generate random BED intervals from a genome sizes file |
| [rsomics-bed-reldist](https://github.com/omics-rust/rsomics-bed-reldist) | Compute relative distances between two BED interval sets — bedtools reldist equivalent |
| [rsomics-bed-sample](https://github.com/omics-rust/rsomics-bed-sample) | Sample random BED records using reservoir sampling — bedtools sample equivalent |
| [rsomics-bed-shift](https://github.com/omics-rust/rsomics-bed-shift) | Shift BED coordinates by a fixed offset — bedtools shift equivalent |
| [rsomics-bed-shuffle](https://github.com/omics-rust/rsomics-bed-shuffle) | Randomly relocate BED intervals within a genome — bedtools shuffle equivalent |
| [rsomics-bed-slop](https://github.com/omics-rust/rsomics-bed-slop) | Extend BED intervals by N bp on each side, clamping to chromosome bounds — bedtools slop e |
| [rsomics-bed-sort](https://github.com/omics-rust/rsomics-bed-sort) | Sort BED intervals by chromosome and start — bedtools sort equivalent |
| [rsomics-bed-spacing](https://github.com/omics-rust/rsomics-bed-spacing) | Report gap lengths between consecutive BED intervals — bedtools spacing equivalent |
| [rsomics-bed-split](https://github.com/omics-rust/rsomics-bed-split) | Split a BED file into N equal-base-pair or equal-record parts — Rust port of bedtools spli |
| [rsomics-bed-stats](https://github.com/omics-rust/rsomics-bed-stats) | Summary statistics (count, total bp, min/max/mean/median length) for BED intervals |
| [rsomics-bed-subtract](https://github.com/omics-rust/rsomics-bed-subtract) | Subtract BED intervals — bedtools subtract equivalent |
| [rsomics-bed-summary](https://github.com/omics-rust/rsomics-bed-summary) | Statistical summary of BED intervals per chromosome — Rust port of bedtools summary |
| [rsomics-bed-to-gff](https://github.com/omics-rust/rsomics-bed-to-gff) | Convert BED intervals to GFF3 format |
| [rsomics-bed-total-bp](https://github.com/omics-rust/rsomics-bed-total-bp) | Count total base-pairs covered by BED intervals (sum of end - start) |
| [rsomics-bed-unionbedg](https://github.com/omics-rust/rsomics-bed-unionbedg) | Combine multiple sorted BedGraph files into one — bedtools unionbedg equivalent |
| [rsomics-bed-unique](https://github.com/omics-rust/rsomics-bed-unique) | Remove duplicate BED intervals (same chrom, start, end) |
| [rsomics-bed-validate](https://github.com/omics-rust/rsomics-bed-validate) | Validate BED file format: check field counts, coordinate ordering, and integer parsing |
| [rsomics-bed-window](https://github.com/omics-rust/rsomics-bed-window) | Find BED features within a window of A intervals — bedtools window equivalent |
| [rsomics-bed12-to-bed6](https://github.com/omics-rust/rsomics-bed12-to-bed6) | Break BED12 block annotations into discrete BED6 intervals — Rust port of bedtools bed12to |
| [rsomics-beta-diversity](https://github.com/omics-rust/rsomics-beta-diversity) | Pairwise between-sample beta-diversity distance matrix (braycurtis/jaccard/euclidean/canberra/cityblock) — scikit-bio-compatible, byte-exact |
| [rsomics-bgzip](https://github.com/omics-rust/rsomics-bgzip) | Block-compress or decompress a file in BGZF — Rust port of htslib bgzip (2.07× compress vs libdeflate bgzip) |
| [rsomics-bigwig-average](https://github.com/omics-rust/rsomics-bigwig-average) | Per-bin average of N bigWig files as a bedGraph track — Rust port of deepTools bigwigAverage |
| [rsomics-bigwig-compare](https://github.com/omics-rust/rsomics-bigwig-compare) | Per-bin comparison of two bigWig files as a bedGraph track — Rust port of deeptools bigwig |
| [rsomics-bioenv](https://github.com/omics-rust/rsomics-bioenv) | BIO-ENV/BEST: best environmental-variable subset maximizing Spearman correlation with a community distance matrix — Rust port of scikit-bio bioenv (value-exact, 3.82× -t1, 36× lower RSS) |
| [rsomics-cca](https://github.com/omics-rust/rsomics-cca) | Canonical Correspondence Analysis (CCA): constrained unimodal ordination of a community count matrix on environmental constraints — scikit-bio cca port (faer, eigenvalues value-exact ~1e-9, 3.98× -t1) |
| [rsomics-cell-filter](https://github.com/omics-rust/rsomics-cell-filter) | Filter cells by QC metrics — min genes, min UMIs, max mito fraction from a barcode stats T |
| [rsomics-clipping-profile](https://github.com/omics-rust/rsomics-clipping-profile) | Per-position soft-clipping profile from a BAM — Rust port of RSeQC clipping_profile.py |
| [rsomics-clr](https://github.com/omics-rust/rsomics-clr) | Centered log-ratio (CLR) compositional transform of a feature table — Rust port of scikit-bio clr (value-exact 2.7e-15, 12.3× -t1) |
| [rsomics-common](https://github.com/omics-rust/rsomics-common) | Shared primitives for every rsomics-* crate (errors, CLI scaffold, runner, progress, exit  |
| [rsomics-compute-gc-bias](https://github.com/omics-rust/rsomics-compute-gc-bias) | deeptools computeGCBias: observed-vs-expected read counts per GC bin over a 2bit genome (pure-Rust 2bit reader + scipy-exact poisson outlier cap) — Rust port of deeptools 3.5.6 (byte-identical, 19.04× -t1) |
| [rsomics-compute-matrix](https://github.com/omics-rust/rsomics-compute-matrix) | bigWig signal → score matrix over BED regions — Rust port of deeptools computeMatrix |
| [rsomics-consensus](https://github.com/omics-rust/rsomics-consensus) | Compute consensus sequence from a multiple sequence alignment — majority-rule or threshold |
| [rsomics-cophenet](https://github.com/omics-rust/rsomics-cophenet) | Cophenetic distances + cophenetic correlation coefficient from a hierarchical linkage matrix — scipy cophenet port (byte-exact, 5.13× -t1, companion to rsomics-upgma) |
| [rsomics-correct-gc-bias](https://github.com/omics-rust/rsomics-correct-gc-bias) | deeptools correctGCBias GC-bias correction (deterministic bedGraph path: per-fragment 1/R_gc binned coverage + binom outlier cap) — Rust port of deeptools 3.5.6 (byte-identical, 37.5× -t1) |
| [rsomics-correspondence-analysis](https://github.com/omics-rust/rsomics-correspondence-analysis) | Correspondence Analysis of a feature table (chi-square transform + SVD → eigenvalues + sample/feature scores) — Rust port of scikit-bio ca via faer (eigenvalues to ~14 digits, 3.31× -t1) |
| [rsomics-count-matrix](https://github.com/omics-rust/rsomics-count-matrix) | Merge multiple featureCounts/htseq-count outputs into a single gene × sample count matrix |
| [rsomics-coverage-core](https://github.com/omics-rust/rsomics-coverage-core) | Genome-binned BAM read-coverage primitive (deeptools countReadsPerBin port): per-bin read  |
| [rsomics-cpm](https://github.com/omics-rust/rsomics-cpm) | Counts-per-million / log2-CPM normalization of a gene count matrix — edgeR cpm-compatible (5.01× -t1) |
| [rsomics-de-volcano](https://github.com/omics-rust/rsomics-de-volcano) | Annotate differential expression results with significance categories for volcano plots |
| [rsomics-debruijn](https://github.com/omics-rust/rsomics-debruijn) | de Bruijn graph types + linear-path collapse + unitig extraction for the rsomics-* tool fa |
| [rsomics-deletion-profile](https://github.com/omics-rust/rsomics-deletion-profile) | Per-base CIGAR-deletion rate along aligned reads — Rust port of RSeQC deletion_profile.py |
| [rsomics-derep](https://github.com/omics-rust/rsomics-derep) | FASTA dereplication — port of vsearch --derep_fulllength / --derep_prefix |
| [rsomics-deseq-estimate-dispersions](https://github.com/omics-rust/rsomics-deseq-estimate-dispersions) | DESeq2 estimateDispersions: per-gene Cox-Reid MLE → parametric trend fit → empirical-Bayes MAP shrinkage (dispGeneEst/dispFit/dispMAP) — Rust port of DESeq2 (dispMAP value-exact ~2.6e-6, 19.4× -t1, 63× lower RSS) |
| [rsomics-deseq-fpkm](https://github.com/omics-rust/rsomics-deseq-fpkm) | DESeq2 fpkm(robust=TRUE): median-of-ratios robust-normalized FPKM of a gene count matrix given per-gene basepairs — Rust port of DESeq2 (byte-exact, 80.4× -t1) |
| [rsomics-deseq-lfc-shrink](https://github.com/omics-rust/rsomics-deseq-lfc-shrink) | DESeq2 lfcShrink(type=normal) zero-centered normal-prior log2FC shrinkage on a Wald fit — Rust port of DESeq2 (value-exact, 26.75× -t1) |
| [rsomics-deseq-lrt](https://github.com/omics-rust/rsomics-deseq-lrt) | DESeq2 likelihood-ratio test (full vs nested reduced design): median-of-ratios norm, MAP dispersion, NB-GLM fits, χ² LRT + BH — Rust port of DESeq2 (value-exact, 4.13× -t1) |
| [rsomics-deseq-norm-transform](https://github.com/omics-rust/rsomics-deseq-norm-transform) | DESeq2 normTransform: median-of-ratios size factors then log2(count/sf + 1) — Rust port of DESeq2 (byte-identical, 132× end-to-end / 54× compute) |
| [rsomics-deseq-prep](https://github.com/omics-rust/rsomics-deseq-prep) | Filter low-count genes and normalize a count matrix for differential expression — pre-DESe |
| [rsomics-deseq-results](https://github.com/omics-rust/rsomics-deseq-results) | DESeq2 nbinomWaldTest + results(): median-of-ratios norm, MAP dispersion, NB-GLM Wald test of a two-group contrast + Cook's outlier removal + independent filtering + BH — Rust port of DESeq2 (value-exact, 23.92× -t1) |
| [rsomics-deseq-rlog](https://github.com/omics-rust/rsomics-deseq-rlog) | DESeq2 rlog regularized-log transform (ridge-penalized NB-GLM, variance-stabilizing) of a count matrix — Rust port of DESeq2 (value-exact 1e-6, 20.1× -t1, 32× lower RSS) |
| [rsomics-deseq-sizefactors](https://github.com/omics-rust/rsomics-deseq-sizefactors) | DESeq2 median-of-ratios size factors per sample from a count matrix — Rust port of DESeq2 estimateSizeFactors (10.93× -t1) |
| [rsomics-deseq-vst](https://github.com/omics-rust/rsomics-deseq-vst) | DESeq2 blind variance-stabilizing transform of a count matrix (median-of-ratios size factors → Cox-Reid parametric dispersion → closed-form VST) — Rust port of DESeq2 varianceStabilizingTransformation (value-exact, 5.81× -t1) |
| [rsomics-dssp](https://github.com/omics-rust/rsomics-dssp) | Per-residue protein secondary-structure assignment from a PDB — Kabsch-Sander/DSSP-4 (99.96% vs mkdssp 4.5, 22× perf) |
| [rsomics-edger-camera](https://github.com/omics-rust/rsomics-edger-camera) | limma camera competitive gene-set test with inter-gene-correlation VIF (fixed or estimated) — clean-room Rust port of limma (value-exact both modes, 22.64× -t1, 5× lower RSS) |
| [rsomics-edger-cpm-by-group](https://github.com/omics-rust/rsomics-edger-cpm-by-group) | edgeR cpmByGroup per-group (log-)CPM via per-gene×group NB one-group GLM (not naive sum) — clean-room Rust port of edgeR (value-exact 0.0 reldev, 27× -t1, 11× lower RSS) |
| [rsomics-edger-diff-splice](https://github.com/omics-rust/rsomics-edger-diff-splice) | edgeR diffSpliceDGE/topSpliceDGE NB-GLM differential exon usage (per-exon LR + gene-level Simes) — clean-room Rust port of edgeR (value-exact, 1.94× -t1, 10.8× lower RSS) |
| [rsomics-edger-estimate-disp](https://github.com/omics-rust/rsomics-edger-estimate-disp) | edgeR NB dispersion estimation (common/trended/tagwise) via Cox-Reid APL + weighted-likelihood empirical Bayes — clean-room Rust port of edgeR estimateDisp (value-exact, 1.77× -t1) |
| [rsomics-edger-exact-test](https://github.com/omics-rust/rsomics-edger-exact-test) | Two-group negative-binomial exact test for differential expression (logFC/logCPM/PValue/FDR) — clean-room Rust port of edgeR exactTest (logFC/logCPM byte-exact, 2.08× -t1, 15.8× lower RSS) |
| [rsomics-edger-glm-lrt](https://github.com/omics-rust/rsomics-edger-glm-lrt) | edgeR NB-GLM fit + likelihood-ratio test of a coefficient/contrast (glmFit + glmLRT) — clean-room Rust port of edgeR (value-exact, 1.24× -t1 / 3.8× -t8) |
| [rsomics-edger-glm-qlf](https://github.com/omics-rust/rsomics-edger-glm-qlf) | edgeR quasi-likelihood F-test for DE (glmQLFit legacy + glmQLFTest) — clean-room Rust port of edgeR (value-exact, 1.47× -t1, 3.8× lower RSS) |
| [rsomics-edger-glm-treat](https://github.com/omics-rust/rsomics-edger-glm-treat) | edgeR glmTreat: NB-GLM test of whether log2-fold-change exceeds a threshold (count analog of limma treat) — clean-room Rust port of edgeR (value-exact among callable genes, 1.88× -t1, 24× lower RSS) |
| [rsomics-edger-goodturing](https://github.com/omics-rust/rsomics-edger-goodturing) | Good-Turing proportion estimation per library (edgeR goodTuringProportions) — clean-room Rust port of edgeR (value-exact ~5e-11, 5.34× -t1) |
| [rsomics-faith-pd](https://github.com/omics-rust/rsomics-faith-pd) | Per-sample Faith's phylogenetic diversity (PD) from a feature count table + a rooted Newick tree — scikit-bio faith_pd equivalent (value-exact, 16.83× -t1) |
| [rsomics-fasta-amplicon](https://github.com/omics-rust/rsomics-fasta-amplicon) | Extract amplicon regions from FASTA/FASTQ by primer pair (IUPAC-aware, mismatches, region/flanking, BED) — byte-exact Rust port of seqkit amplicon (1.55–2.12× -t1) |
| [rsomics-fasta-digest](https://github.com/omics-rust/rsomics-fasta-digest) | In-silico protein digestion — trypsin/LysC/other enzymes, missed cleavages, peptide mass f |
| [rsomics-fasta-index](https://github.com/omics-rust/rsomics-fasta-index) | FASTA index (.fai) creation, random-access fetch, and sequence dictionary — Rust port of s |
| [rsomics-fasta-locate](https://github.com/omics-rust/rsomics-fasta-locate) | Locate subsequences/motifs in FASTA files — seqkit locate port |
| [rsomics-fasta-mask](https://github.com/omics-rust/rsomics-fasta-mask) | Mask FASTA sequences by BED regions — soft-mask (lowercase) or hard-mask (N) |
| [rsomics-fasta-n50](https://github.com/omics-rust/rsomics-fasta-n50) | Compute N50, L50, and assembly statistics from FASTA |
| [rsomics-fasta-sliding](https://github.com/omics-rust/rsomics-fasta-sliding) | Sliding-window subsequence extraction from FASTA — seqkit sliding port |
| [rsomics-fasta-stats](https://github.com/omics-rust/rsomics-fasta-stats) | Per-record statistics for FASTA files (num_seqs, sum_len, GC%, N50, …) — Rust port of `seq |
| [rsomics-fasta-subseq](https://github.com/omics-rust/rsomics-fasta-subseq) | Extract FASTA subsequences by 1-based region — seqkit subseq port |
| [rsomics-fasta-translate](https://github.com/omics-rust/rsomics-fasta-translate) | Translate DNA/RNA FASTA to protein sequences (six-frame) |
| [rsomics-fasta-utils](https://github.com/omics-rust/rsomics-fasta-utils) | FASTA utility toolkit — count, chroms, len, revcomp, rename, tab, wrap, unique, convert, a |
| [rsomics-fasta-validate](https://github.com/omics-rust/rsomics-fasta-validate) | Validate FASTA format integrity |
| [rsomics-fastp](https://github.com/omics-rust/rsomics-fastp) | Fast FASTQ quality control and preprocessing |
| [rsomics-fastq-complexity](https://github.com/omics-rust/rsomics-fastq-complexity) | FASTQ low-complexity filter — discard reads whose per-base change fraction falls below a t |
| [rsomics-fastq-correct](https://github.com/omics-rust/rsomics-fastq-correct) | FASTQ k-mer-spectrum substitution-error correction. Independent Rust port of BFC (lh3): tr |
| [rsomics-fastq-dedup](https://github.com/omics-rust/rsomics-fastq-dedup) | Sequence-based FASTQ deduplication (kmer-bin or full-sequence hash). Rust port consolidati |
| [rsomics-fastq-downsample](https://github.com/omics-rust/rsomics-fastq-downsample) | Deterministic random downsampling of FASTQ to a target read count or fraction |
| [rsomics-fastq-filter](https://github.com/omics-rust/rsomics-fastq-filter) | FASTQ per-read quality + length filter. Rust port of fastp's quality/length filter (pass/f |
| [rsomics-fastq-merge](https://github.com/omics-rust/rsomics-fastq-merge) | Merge overlapping paired-end FASTQ reads into consensus reads — Rust port of fastp's overl |
| [rsomics-fastq-pair](https://github.com/omics-rust/rsomics-fastq-pair) | Re-pair shuffled paired-end FASTQ reads by name |
| [rsomics-fastq-quality](https://github.com/omics-rust/rsomics-fastq-quality) | FASTQ sliding-window and leading/trailing quality trimmer — Rust port of fastp/Trimmomatic |
| [rsomics-fastq-sample](https://github.com/omics-rust/rsomics-fastq-sample) | Random subsample FASTQ records by fraction or exact count — seqkit sample / seqtk sample e |
| [rsomics-fastq-split](https://github.com/omics-rust/rsomics-fastq-split) | Split a FASTQ into N files or by line count. Rust port of fastp's split (deterministic --s |
| [rsomics-fastq-stats](https://github.com/omics-rust/rsomics-fastq-stats) | Per-file statistics for FASTQ files (num_seqs, sum_len, N50, GC%, Q20/Q30%, AvgQual, …) —  |
| [rsomics-fastq-trim](https://github.com/omics-rust/rsomics-fastq-trim) | FASTQ adapter / poly-G / poly-X / fixed-length trimming. Rust port of fastp's trim hot pat |
| [rsomics-fastq-umi](https://github.com/omics-rust/rsomics-fastq-umi) | FASTQ inline-UMI extract + stamp. Rust port of fastp's UMI processing — full --umi_loc set |
| [rsomics-fastq-utils](https://github.com/omics-rust/rsomics-fastq-utils) | FASTQ utility toolkit — lightweight subcommands for counting, filtering, converting, and i |
| [rsomics-fastq-validate](https://github.com/omics-rust/rsomics-fastq-validate) | Validate FASTQ format integrity (line counts, quality encoding) |
| [rsomics-fastqc](https://github.com/omics-rust/rsomics-fastqc) | Per-file FASTQ quality-control report (FastQC-equivalent: per-base quality, GC, N, length, |
| [rsomics-fastx-sort](https://github.com/omics-rust/rsomics-fastx-sort) | Deterministic FASTA sorting by abundance or length — port of vsearch --sortbysize / --sort |
| [rsomics-fcluster](https://github.com/omics-rust/rsomics-fcluster) | Form flat clusters from a hierarchical linkage matrix (distance/maxclust/inconsistent/monocrit) — scipy fcluster port, cluster labels byte-identical (companion to rsomics-upgma) |
| [rsomics-featurecounts](https://github.com/omics-rust/rsomics-featurecounts) | Count reads over genomic features (BAM + GFF) — Rust port of featureCounts (Subread) |
| [rsomics-filter-by-expr](https://github.com/omics-rust/rsomics-filter-by-expr) | Boolean per-gene low-expression filter (CPM-cutoff + total-count keep rules, group-aware) — Rust port of edgeR filterByExpr (byte-exact, 8.51× -t1) |
| [rsomics-fm-index](https://github.com/omics-rust/rsomics-fm-index) | FM-index over BWT + suffix array, with backward search / count / locate. Layer A primitive |
| [rsomics-fm-search](https://github.com/omics-rust/rsomics-fm-search) | Exact substring search in FASTA using FM-index — count or locate pattern occurrences |
| [rsomics-fpkm-count](https://github.com/omics-rust/rsomics-fpkm-count) | Compute per-gene FPKM from a BAM + BED12 model — Rust port of RSeQC FPKM_count |
| [rsomics-fqgz](https://github.com/omics-rust/rsomics-fqgz) | Chunked parallel-libdeflate gzip (or plain) FASTQ-record writer. Layer-A primitive shared  |
| [rsomics-fragment-size](https://github.com/omics-rust/rsomics-fragment-size) | Paired-end insert-size distribution from a BAM: histogram TSV + summary with ATAC nucleoso |
| [rsomics-freesasa](https://github.com/omics-rust/rsomics-freesasa) | Solvent-accessible surface area from a PDB via the Lee-Richards algorithm (ProtOr radii, 1.4 Å probe) — freesasa-compatible Rust rewrite |
| [rsomics-gc-windows](https://github.com/omics-rust/rsomics-gc-windows) | Compute per-window GC content across a FASTA reference — BED output for CNV/WGS normalizat |
| [rsomics-genebody-coverage](https://github.com/omics-rust/rsomics-genebody-coverage) | Gene-body coverage profile (5'→3') for RNA-seq bias QC — Rust port of RSeQC geneBody_cover |
| [rsomics-gff-utils](https://github.com/omics-rust/rsomics-gff-utils) | GFF/GTF utility toolkit — count, filter, extract, sort, convert, and stats operations |
| [rsomics-help](https://github.com/omics-rust/rsomics-help) | Family-wide `--help` renderer for rsomics-* CLIs: figlet banner with gradient, section hel |
| [rsomics-hmm](https://github.com/omics-rust/rsomics-hmm) | Hidden Markov Model inference (Viterbi / forward / backward) for the rsomics-* tool family |
| [rsomics-hmm-decode](https://github.com/omics-rust/rsomics-hmm-decode) | Viterbi-decode observation sequences with a discrete HMM — chromatin state, gene finding,  |
| [rsomics-hommola](https://github.com/omics-rust/rsomics-hommola) | Hommola et al. host–parasite cospeciation test (correlation of host vs parasite distances over interactions + permutation p) — scikit-bio port (statistic value-exact, 7.51× -t1) |
| [rsomics-igzip](https://github.com/omics-rust/rsomics-igzip) | Minimal Quadrant-② FFI wrapper over Intel ISA-L igzip for fast gzip decompression. Isolate |
| [rsomics-ilr](https://github.com/omics-rust/rsomics-ilr) | Isometric log-ratio (ILR) compositional transform via the default Gram-Schmidt SBP basis (O(D) prefix-sum) — Rust port of scikit-bio ilr (value-exact 1e-9, 12.2× -t1) |
| [rsomics-ilr-basis](https://github.com/omics-rust/rsomics-ilr-basis) | Isometric log-ratio (ILR) transform with a user-supplied SBP/tree contrast basis — scikit-bio ilr(basis=) port (value-exact, 5.20× -t1) |
| [rsomics-infercnv](https://github.com/omics-rust/rsomics-infercnv) | Infer copy-number variations from single-cell RNA-seq expression — Rust port of inferCNV |
| [rsomics-inner-distance](https://github.com/omics-rust/rsomics-inner-distance) | mRNA-aware inner-distance distribution for paired-end RNA-seq — Rust port of RSeQC inner_d |
| [rsomics-insertion-profile](https://github.com/omics-rust/rsomics-insertion-profile) | Per-position CIGAR-insertion rate along the read — Rust port of RSeQC insertion_profile.py |
| [rsomics-intervals](https://github.com/omics-rust/rsomics-intervals) | BED algebra + interval index + GFF/GTF interval extraction for the rsomics-* tool family.  |
| [rsomics-junction-saturation](https://github.com/omics-rust/rsomics-junction-saturation) | Subsample-based splice-junction saturation analysis — Rust reimplementation of RSeQC junct |
| [rsomics-kinship](https://github.com/omics-rust/rsomics-kinship) | KING-robust pairwise kinship coefficients from PLINK genotypes — clean-room Rust port of plink2 --make-king-table (Manichaikul 2010), bitplane transpose + runtime AVX2 popcount (byte-identical .kin0, 1.13× -t1) |
| [rsomics-kmer](https://github.com/omics-rust/rsomics-kmer) | K-mer encoding, canonicalisation, ntHash rolling hash, MurmurHash3, k-mer counting for the |
| [rsomics-kmer-dist](https://github.com/omics-rust/rsomics-kmer-dist) | Pairwise k-mer frequency distance between FASTA/FASTQ samples — Jaccard/Bray-Curtis/cosine |
| [rsomics-kraken-report](https://github.com/omics-rust/rsomics-kraken-report) | Parse and summarize Kraken2 report files — top taxa, diversity stats |
| [rsomics-ld-matrix](https://github.com/omics-rust/rsomics-ld-matrix) | Compute pairwise linkage disequilibrium (r²) from a genotype matrix |
| [rsomics-liftover](https://github.com/omics-rust/rsomics-liftover) | Lift BED interval coordinates between assemblies via a UCSC chain file — Rust port of UCSC liftOver |
| [rsomics-limma-array-weights](https://github.com/omics-rust/rsomics-limma-array-weights) | limma arrayWeights: REML per-sample quality weights from a log-expression matrix + design — clean-room Rust port of limma (matches to ~0.1%, REML prior.n residual documented; 9.35× -t1) |
| [rsomics-limma-decide-tests](https://github.com/omics-rust/rsomics-limma-decide-tests) | limma decideTests up/down/notsig classification (separate/global BH/holm/… adjust + p-cutoff + lfc filter) — clean-room Rust port of limma (integer-exact, 21.2× -t1) |
| [rsomics-limma-diff-splice](https://github.com/omics-rust/rsomics-limma-diff-splice) | limma diffSplice/topSplice differential exon usage (per-exon moderated-t + gene-level Simes/F) — clean-room Rust port of limma legacy=TRUE (value-exact 1e-9, 11.06× -t1) |
| [rsomics-limma-duplicate-correlation](https://github.com/omics-rust/rsomics-limma-duplicate-correlation) | limma duplicateCorrelation: REML consensus intra-block (technical-replicate) correlation — clean-room Rust port of limma (consensus value-exact, 114× -t1) |
| [rsomics-limma-ebayes](https://github.com/omics-rust/rsomics-limma-ebayes) | Per-gene linear-model fit + empirical-Bayes moderated t-statistics (lmFit+eBayes+topTable) for a log-expression matrix — clean-room Rust port of limma (value-exact, 49.5× -t1, 10× lower RSS) |
| [rsomics-limma-proptruenull](https://github.com/omics-rust/rsomics-limma-proptruenull) | Proportion of true null hypotheses from a p-value vector (limma propTrueNull/convest) — clean-room Rust port of limma (value-exact, 20.3× -t1) |
| [rsomics-limma-treat](https://github.com/omics-rust/rsomics-limma-treat) | Moderated t-test against a log-fold-change threshold (limma treat + topTreat) for a log-expression matrix — clean-room Rust port of limma (value-exact, 48× -t1) |
| [rsomics-limma-vooma](https://github.com/omics-rust/rsomics-limma-vooma) | limma vooma mean-variance precision weights for log-expression (lowess trend → 1/SD^4) — clean-room Rust port of limma (value-exact 4e-10, 18.97× -t1) |
| [rsomics-mantel](https://github.com/omics-rust/rsomics-mantel) | Mantel test (pearson/spearman) between two distance matrices + permutation p — Rust port of scikit-bio mantel (r value-exact, 1.24× -t1) |
| [rsomics-methyldackel](https://github.com/omics-rust/rsomics-methyldackel) | Per-CpG methylation extraction from bisulfite-aligned BAM — Rust port of MethylDackel extr |
| [rsomics-minimap2](https://github.com/omics-rust/rsomics-minimap2) | Long/short-read aligner — CLI wrapper of minimap2 FFI bindings (Quadrant ②) |
| [rsomics-mismatch-profile](https://github.com/omics-rust/rsomics-mismatch-profile) | Per-base mismatch-rate profile from BAM MD tags — Rust port of RSeQC mismatch_profile.py |
| [rsomics-models](https://github.com/omics-rust/rsomics-models) | Pinned DL-model registry + per-OS cache + sha256-verify for the rsomics-* tool family. Lay |
| [rsomics-motif-scan](https://github.com/omics-rust/rsomics-motif-scan) | Scan FASTA sequences for IUPAC DNA motif occurrences — BED output of match positions |
| [rsomics-msa-trim](https://github.com/omics-rust/rsomics-msa-trim) | Trim MSA columns by gap fraction — Rust replacement for trimAl -gt |
| [rsomics-multi-replace](https://github.com/omics-rust/rsomics-multi-replace) | Multiplicative zero-replacement of a compositional table (small δ for zeros, rest rescaled to preserve closure) — scikit-bio multiplicative_replacement port (value-exact, 5.95× -t1) |
| [rsomics-multibam-summary](https://github.com/omics-rust/rsomics-multibam-summary) | Multi-BAM per-bin / per-region read-count matrix — Rust port of deeptools multiBamSummary |
| [rsomics-multibigwig-summary](https://github.com/omics-rust/rsomics-multibigwig-summary) | Multi-bigWig per-bin / per-region mean-signal matrix — Rust port of deeptools multiBigwigS |
| [rsomics-nj-tree](https://github.com/omics-rust/rsomics-nj-tree) | Neighbor-joining tree construction from a distance matrix — outputs Newick |
| [rsomics-pcoa](https://github.com/omics-rust/rsomics-pcoa) | Principal Coordinates Analysis (PCoA) of a distance matrix — scikit-bio-compatible Rust port (2.75× -t1, faer eigh) |
| [rsomics-pdb-chain](https://github.com/omics-rust/rsomics-pdb-chain) | Extract or split PDB chains — list, extract single chain, or split all into separate files |
| [rsomics-peak-count](https://github.com/omics-rust/rsomics-peak-count) | Count BAM reads per BED peak region — ChIP-seq/ATAC-seq QC and quantification |
| [rsomics-permanova](https://github.com/omics-rust/rsomics-permanova) | PERMANOVA pseudo-F test on a distance matrix + permutation p — Rust port of scikit-bio permanova (pseudo-F value-exact, 2.05× -t1) |
| [rsomics-permdisp](https://github.com/omics-rust/rsomics-permdisp) | PERMDISP test of homogeneity of multivariate dispersions (centroid/spatial-median, faer PCoA + permutation p) — scikit-bio permdisp port (F value-exact ~1e-9, 2.16× median / 10.46× centroid -t1) |
| [rsomics-pgen](https://github.com/omics-rust/rsomics-pgen) | PLINK1 .bed / .bim / .fam genotype-matrix reader + writer for the rsomics-* tool family. L |
| [rsomics-phylo-tree](https://github.com/omics-rust/rsomics-phylo-tree) | Phylogenetic tree type + Newick parser/emitter for the rsomics-* tool family. Layer A prim |
| [rsomics-pileup](https://github.com/omics-rust/rsomics-pileup) | Coordinate-sorted BAM pileup engine (htslib bam_plp port): per-position read columns with  |
| [rsomics-plink-assoc](https://github.com/omics-rust/rsomics-plink-assoc) | PLINK1 case/control association test (chi-squared + trend + linear regression) |
| [rsomics-plink-check-sex](https://github.com/omics-rust/rsomics-plink-check-sex) | PLINK1 --check-sex: per-sample X-chromosome inbreeding F + sex imputation/discrepancy on non-PAR X markers — clean-room Rust port of PLINK 1.9 (byte-identical .sexcheck, 11.4× CPU / 21.25× wall -t1) |
| [rsomics-plink-cluster](https://github.com/omics-rust/rsomics-plink-cluster) | IBS-based complete-linkage agglomerative clustering of samples (plink --cluster) — clean-room Rust port of PLINK 1.9 (cluster membership byte-identical; AVX2 2-plane IBS kernel, 1.18× CPU -t1) |
| [rsomics-plink-epistasis](https://github.com/omics-rust/rsomics-plink-epistasis) | PLINK --epistasis SNP×SNP logistic-regression interaction scan (9-cell grouped-binomial IRLS → OR_INT/STAT/P) — clean-room Rust port of PLINK 1.9 (value-exact, 3.74× -t1) |
| [rsomics-plink-flip-scan](https://github.com/omics-rust/rsomics-plink-flip-scan) | PLINK1 --flip-scan: LD-sign strand-inconsistency QC scan (per-SNP positive/negative correlation matches between cases and controls) — clean-room Rust port of PLINK 1.9 (field-exact, 1.07× SNP-heavy / 1.48× sample-heavy -t1) |
| [rsomics-plink-freq](https://github.com/omics-rust/rsomics-plink-freq) | Per-variant allele frequencies (.frq) — Rust port of PLINK --freq (1.63× -t1, mmap+popcount) |
| [rsomics-plink-freqx](https://github.com/omics-rust/rsomics-plink-freqx) | Per-variant genotype-class counts (.frqx) — Rust port of PLINK --freqx (1.14× -t1, byte-exact, SWAR popcount) |
| [rsomics-plink-grm](https://github.com/omics-rust/rsomics-plink-grm) | GCTA standardized-genotype Genetic Relationship Matrix from PLINK genotypes (blocked faer GEMM + AND-popcount denom) — Rust port of plink2 --make-rel (byte-identical, 1.38× -t1) |
| [rsomics-plink-het](https://github.com/omics-rust/rsomics-plink-het) | Per-sample inbreeding coefficient F (autosomes) — Rust port of PLINK --het (1.14× -t1) |
| [rsomics-plink-homozyg](https://github.com/omics-rust/rsomics-plink-homozyg) | Runs of homozygosity (.hom) — Rust port of PLINK --homozyg (field-exact, 1.26× -t1) |
| [rsomics-plink-ibc](https://github.com/omics-rust/rsomics-plink-ibc) | Per-sample inbreeding F-hat estimators (Fhat1/2/3) — Rust port of PLINK --ibc (2.09× -t1) |
| [rsomics-plink-io](https://github.com/omics-rust/rsomics-plink-io) | PLINK1 binary .bed/.bim/.fam reader: allele-freq, missingness, HWE, VCF/012 export |
| [rsomics-plink-ld](https://github.com/omics-rust/rsomics-plink-ld) | Pairwise LD (r²) computation and LD matrix export from PLINK1 binary filesets |
| [rsomics-plink-missing](https://github.com/omics-rust/rsomics-plink-missing) | Per-sample (.imiss) and per-variant (.lmiss) genotype missingness — Rust port of PLINK --missing (1.28× CPU -t1 / 2.13× -t8) |
| [rsomics-plink-model](https://github.com/omics-rust/rsomics-plink-model) | Per-variant genotypic association — GENO/ALLELIC/DOM/REC/TREND — Rust port of PLINK --model (byte-identical; perf 0.96× -t1, output-bound near-miss, optimization pending #142) |
| [rsomics-plink-pca](https://github.com/omics-rust/rsomics-plink-pca) | PCA and GRM computation from PLINK1 binary filesets using faer EVD |
| [rsomics-plink-prune](https://github.com/omics-rust/rsomics-plink-prune) | LD pruning from PLINK1 binary filesets (--indep-pairwise) |
| [rsomics-plink-recode](https://github.com/omics-rust/rsomics-plink-recode) | Additive 0/1/2 genotype-dosage matrix (.raw) — Rust port of PLINK --recode A (byte-exact, 1.36× -t1) |
| [rsomics-plink-score](https://github.com/omics-rust/rsomics-plink-score) | Polygenic score from a weights file (.profile) — Rust port of PLINK --score (1.37× -t1) |
| [rsomics-plink-tdt](https://github.com/omics-rust/rsomics-plink-tdt) | Transmission disequilibrium test for trios (.tdt) — Rust port of PLINK --tdt (1.44× -t1) |
| [rsomics-plot-coverage](https://github.com/omics-rust/rsomics-plot-coverage) | deeptools plotCoverage coverage distribution (deterministic strided sampling, per-bin raw counts + per-sample summary) — Rust port of deeptools 3.5.6 (byte-identical, 79.5× -t1, 8× lower RSS) |
| [rsomics-popgen-core](https://github.com/omics-rust/rsomics-popgen-core) | Population-genetics primitives: π, Watterson's θ, Tajima's D, Hardy-Weinberg exact, LD r². |
| [rsomics-pvalue-adjust](https://github.com/omics-rust/rsomics-pvalue-adjust) | Multiple-testing correction for a column of p-values — the full R p.adjust method set (hol |
| [rsomics-pwmantel](https://github.com/omics-rust/rsomics-pwmantel) | Pairwise Mantel test across N distance matrices (pearson/spearman r + permutation p, results table) — scikit-bio pwmantel port (statistic value-exact ~5e-13, 1.32–1.37× -t1) |
| [rsomics-quantile-norm](https://github.com/omics-rust/rsomics-quantile-norm) | Quantile normalization of a gene × sample matrix — Rust port of limma normalizeQuantiles (8.92× -t1) |
| [rsomics-rda](https://github.com/omics-rust/rsomics-rda) | Redundancy Analysis (RDA): constrained linear ordination of a response matrix on environmental constraints — scikit-bio rda port (faer SVD, eigenvalues value-exact ~1e-9, 7.36× -t1) |
| [rsomics-read-distribution](https://github.com/omics-rust/rsomics-read-distribution) | Distribution of reads over genomic features (CDS/UTR/intron/TSS/TES) — Rust port of RSeQC read_distribution.py |
| [rsomics-read-duplication](https://github.com/omics-rust/rsomics-read-duplication) | Sequence-based and position-based read duplication rate — Rust port of RSeQC read_duplicat |
| [rsomics-read-gc](https://github.com/omics-rust/rsomics-read-gc) | Per-read GC% distribution from a BAM — Rust port of RSeQC read_GC.py |
| [rsomics-read-nvc](https://github.com/omics-rust/rsomics-read-nvc) | Per-cycle nucleotide composition (NVC) from a BAM — Rust port of RSeQC read_NVC.py |
| [rsomics-read-quality](https://github.com/omics-rust/rsomics-read-quality) | Per-base read-quality heatmap and boxplot from BAM — Rust port of RSeQC read_quality.py |
| [rsomics-remove-batch-effect](https://github.com/omics-rust/rsomics-remove-batch-effect) | Regress out a batch factor from a log-expression gene × sample matrix — Rust reimplementation of limma removeBatchEffect (field-exact, 7.20× -t1) |
| [rsomics-rereplicate](https://github.com/omics-rust/rsomics-rereplicate) | Expand abundance-annotated FASTA back into individual reads — port of vsearch --rereplicat |
| [rsomics-rna-fragment-size](https://github.com/omics-rust/rsomics-rna-fragment-size) | Per-transcript mRNA fragment-size distribution for paired RNA-seq — Rust port of RSeQC RNA_fragment_size.py |
| [rsomics-rnaseq-metrics](https://github.com/omics-rust/rsomics-rnaseq-metrics) | RNA-seq QC metrics (region coverage fractions, strand bias, transcript-coverage bias) — Ru |
| [rsomics-rpkm-saturation](https://github.com/omics-rust/rsomics-rpkm-saturation) | Subsample-based RPKM saturation analysis — Rust reimplementation of RSeQC RPKM_saturation. |
| [rsomics-sam-to-bam](https://github.com/omics-rust/rsomics-sam-to-bam) | Convert SAM to BAM — Rust equivalent of samtools view -bS |
| [rsomics-sample-sheet](https://github.com/omics-rust/rsomics-sample-sheet) | Parse, validate, and convert sample sheets (Illumina/custom TSV) — check FASTQ paths, dete |
| [rsomics-sc-cell-cycle](https://github.com/omics-rust/rsomics-sc-cell-cycle) | Cell-cycle phase scoring (S/G2M scores + phase call) of a single-cell matrix — scanpy score_genes_cell_cycle-compatible, bit-identical (2.97× CPU -t1) |
| [rsomics-sc-combat](https://github.com/omics-rust/rsomics-sc-combat) | Per-gene ComBat empirical-Bayes batch-effect correction of a single-cell matrix — Rust port of scanpy pp.combat (value-exact, 8.09× -t1, 5.4× lower RSS) |
| [rsomics-sc-downsample](https://github.com/omics-rust/rsomics-sc-downsample) | Downsample a single-cell count matrix so each cell has at most N counts (without replacement) — scanpy pp.downsample_counts-compatible, integer-exact via bit-exact numba MT19937 RNG (2.45× -t1, 5.6× less memory) |
| [rsomics-sc-filter](https://github.com/omics-rust/rsomics-sc-filter) | Filter cells & genes of a 10x matrix (min/max genes/counts/cells) — scanpy filter_cells/filter_genes-compatible (2.48× -t1) |
| [rsomics-sc-hvg](https://github.com/omics-rust/rsomics-sc-hvg) | Highly-variable-gene selection (seurat flavor) from a 10x matrix — scanpy highly_variable_genes-compatible (1.78× -t1) |
| [rsomics-sc-marker-overlap](https://github.com/omics-rust/rsomics-sc-marker-overlap) | Overlap of per-cluster ranked marker genes against a reference marker-set panel → reference×cluster matrix — scanpy marker-genes-compatible |
| [rsomics-sc-normalize](https://github.com/omics-rust/rsomics-sc-normalize) | Library-size normalization + log1p of a 10x single-cell matrix — scanpy normalize_total/log1p-compatible (3.06× -t1, 4× less memory) |
| [rsomics-sc-pseudobulk](https://github.com/omics-rust/rsomics-sc-pseudobulk) | Pseudobulk aggregation (sum/mean counts per group) of a 10x matrix — scanpy get.aggregate-compatible (2.41× -t1) |
| [rsomics-sc-qc-metrics](https://github.com/omics-rust/rsomics-sc-qc-metrics) | Per-cell & per-gene QC metrics from a 10x matrix — scanpy calculate_qc_metrics-compatible (2.28× -t1) |
| [rsomics-sc-rank-genes](https://github.com/omics-rust/rsomics-sc-rank-genes) | Per-group marker-gene ranking by Welch t-test from a single-cell matrix — Rust port of scanpy rank_genes_groups (value-exact, 13.37× -t1) |
| [rsomics-sc-regress-out](https://github.com/omics-rust/rsomics-sc-regress-out) | Regress out per-cell covariates from a single-cell matrix via per-gene OLS residuals — Rust port of scanpy pp.regress_out (value-exact, 18.34× -t1) |
| [rsomics-sc-scale](https://github.com/omics-rust/rsomics-sc-scale) | Per-gene z-score scaling of a single-cell matrix (zero-center, ddof=1 std, symmetric clip) — Rust port of scanpy pp.scale (value-exact, 15.78× -t1) |
| [rsomics-sc-score-genes](https://github.com/omics-rust/rsomics-sc-score-genes) | Per-cell gene-set score (set mean − bin-matched control mean) from a single-cell matrix — Rust port of scanpy tl.score_genes, bit-exact numpy-RNG control sampling (value-exact, 3.73× -t1) |
| [rsomics-seacr](https://github.com/omics-rust/rsomics-seacr) | CUT&RUN peak caller (bedGraph → BED peaks) — clean-room Rust port of SEACR |
| [rsomics-seq-stats](https://github.com/omics-rust/rsomics-seq-stats) | Quick stats for any FASTA/FASTQ — count, total bp, N50, GC%, min/max/mean length |
| [rsomics-seqio](https://github.com/omics-rust/rsomics-seqio) | Fast FASTQ reader for the rsomics-* tool family: decode-only producer thread + parallel pa |
| [rsomics-seqstats](https://github.com/omics-rust/rsomics-seqstats) | Format-agnostic sequence statistics primitives (length distribution: N50/L50/Nx + quartile |
| [rsomics-stats](https://github.com/omics-rust/rsomics-stats) | Statistical tests, FDR control, p-value combination for the rsomics-* tool family. Layer A |
| [rsomics-subsample-counts](https://github.com/omics-rust/rsomics-subsample-counts) | scikit-bio subsample_counts without-replacement rarefaction draw (numpy default_rng/PCG64 reproduced bit-exact) — Rust port of scikit-bio 0.7.2 (integer-exact, 1.35× algo / 9.76× end-to-end) |
| [rsomics-tabix](https://github.com/omics-rust/rsomics-tabix) | Coordinate index (.tbi/.csi) for bgzipped position-sorted files + region query — Rust port of htslib tabix (byte-identical index, 1.10× build / 1.49× query) |
| [rsomics-tajima-d](https://github.com/omics-rust/rsomics-tajima-d) | Compute Tajima's D from a site frequency spectrum (derived allele counts) |
| [rsomics-tax-assign](https://github.com/omics-rust/rsomics-tax-assign) | Lightweight taxonomic assignment from k-mer LCA — classify reads against a reference taxon |
| [rsomics-taxonomy](https://github.com/omics-rust/rsomics-taxonomy) | NCBI taxdump parser + LCA + lineage helpers for the rsomics-* tool family. Layer A primiti |
| [rsomics-tin](https://github.com/omics-rust/rsomics-tin) | Transcript Integrity Number (TIN) for RNA-seq QC — Rust port of RSeQC tin.py |
| [rsomics-tm-align](https://github.com/omics-rust/rsomics-tm-align) | Pairwise protein structural alignment + TM-score — clean-room Rust impl of TM-align (10.7× -t1) |
| [rsomics-tmm-norm](https://github.com/omics-rust/rsomics-tmm-norm) | TMM (trimmed mean of M-values) per-sample normalization factors — Rust port of edgeR calcNormFactors(method=TMM) (6.04× -t1) |
| [rsomics-tpm](https://github.com/omics-rust/rsomics-tpm) | TPM, FPKM, and upper-quartile (FPKM-UQ) normalization of gene count matrices given gene lengths |
| [rsomics-tsv-crosstab](https://github.com/omics-rust/rsomics-tsv-crosstab) | Cross-tabulation (long→wide pivot) of a TSV — GNU datamash crosstab-compatible (byte-exact, 1.81× -t1) |
| [rsomics-tsv-join](https://github.com/omics-rust/rsomics-tsv-join) | Join two TSV files by a shared key column — inner/left/outer join |
| [rsomics-tsv-select](https://github.com/omics-rust/rsomics-tsv-select) | Select, reorder, or rename columns from TSV files — cut + awk for bioinformatics pipelines |
| [rsomics-tsv-stats](https://github.com/omics-rust/rsomics-tsv-stats) | Per-column and grouped summary statistics of delimited files — GNU datamash-compatible (1.48-2.10× -t1) |
| [rsomics-tsv-transpose](https://github.com/omics-rust/rsomics-tsv-transpose) | Transpose a TSV (rows↔columns) — GNU datamash transpose-compatible (byte-exact, 3.42× -t1) |
| [rsomics-unifrac](https://github.com/omics-rust/rsomics-unifrac) | UniFrac phylogenetic beta-diversity distance matrix (unweighted / weighted / weighted-normalized) from a feature count table + rooted Newick tree — scikit-bio-compatible Rust port (value-exact ~1e-14, 3.5×/13.6×/8.4× -t1) |
| [rsomics-upgma](https://github.com/omics-rust/rsomics-upgma) | UPGMA average-linkage hierarchical-clustering tree from a distance matrix → Newick — value-exact vs scipy linkage(average) (cophenetic 7e-18; 3.90× end-to-end / 1.05× pure-core -t1) |
| [rsomics-uq-norm](https://github.com/omics-rust/rsomics-uq-norm) | Upper-quartile per-sample normalization factors — Rust port of edgeR calcNormFactors(method=upperquartile) (byte-exact, 4.55× -t1) |
| [rsomics-vcf-annotate](https://github.com/omics-rust/rsomics-vcf-annotate) | Annotate VCF variants with labels from a BED/TSV file |
| [rsomics-vcf-call](https://github.com/omics-rust/rsomics-vcf-call) | Bayesian SNP/indel calling from mpileup likelihoods — Rust port of bcftools call -c |
| [rsomics-vcf-cnv](https://github.com/omics-rust/rsomics-vcf-cnv) | HMM-based CNV caller from BAF + LRR in a single-sample VCF — Rust port of bcftools cnv |
| [rsomics-vcf-concat](https://github.com/omics-rust/rsomics-vcf-concat) | Concatenate VCFs (same samples) — Rust port of bcftools concat |
| [rsomics-vcf-consensus](https://github.com/omics-rust/rsomics-vcf-consensus) | Apply VCF variants to a reference FASTA — Rust port of bcftools consensus |
| [rsomics-vcf-convert](https://github.com/omics-rust/rsomics-vcf-convert) | Convert between VCF text, bgzipped VCF, and HAP/LEGEND/SAMPLE — Rust port of bcftools conv |
| [rsomics-vcf-csq](https://github.com/omics-rust/rsomics-vcf-csq) | Annotate VCF variants with functional consequences (missense, frameshift, splice, …) using |
| [rsomics-vcf-expr](https://github.com/omics-rust/rsomics-vcf-expr) | bcftools-style VCF filter-expression parser and per-sample evaluator |
| [rsomics-vcf-fill-tags](https://github.com/omics-rust/rsomics-vcf-fill-tags) | Recompute VCF INFO tags (AN, AC, AF, MAF, NS, AC_Hom, AC_Het, AC_Hemi, HWE, ExcHet) from F |
| [rsomics-vcf-filter](https://github.com/omics-rust/rsomics-vcf-filter) | VCF/BCF record filtering by region, quality, INFO/FORMAT fields — Rust port of bcftools vi |
| [rsomics-vcf-fixref](https://github.com/omics-rust/rsomics-vcf-fixref) | Check/fix VCF REF allele and strand against a reference FASTA — Rust port of bcftools +fix |
| [rsomics-vcf-gtcheck](https://github.com/omics-rust/rsomics-vcf-gtcheck) | Sample concordance / discordance estimator — Rust port of bcftools gtcheck |
| [rsomics-vcf-head](https://github.com/omics-rust/rsomics-vcf-head) | Print the VCF header and the first N records — Rust port of bcftools head |
| [rsomics-vcf-index](https://github.com/omics-rust/rsomics-vcf-index) | Index a bgzipped VCF (.csi/.tbi) — Rust port of bcftools index |
| [rsomics-vcf-isec](https://github.com/omics-rust/rsomics-vcf-isec) | VCF intersection — find shared variants between two VCFs (bcftools isec) |
| [rsomics-vcf-merge](https://github.com/omics-rust/rsomics-vcf-merge) | Merge multi-sample VCFs by position — bcftools merge equivalent |
| [rsomics-vcf-mpileup](https://github.com/omics-rust/rsomics-vcf-mpileup) | VCF-emitting pileup (genotype likelihoods) from BAM — Rust port of bcftools mpileup single |
| [rsomics-vcf-norm](https://github.com/omics-rust/rsomics-vcf-norm) | Left-align and normalize VCF indels — Rust port of bcftools norm |
| [rsomics-vcf-polysomy](https://github.com/omics-rust/rsomics-vcf-polysomy) | Estimate per-chromosome copy number from BAF distributions — Rust port of bcftools polysom |
| [rsomics-vcf-popgen](https://github.com/omics-rust/rsomics-vcf-popgen) | Population-genetics statistics from VCF: allele-freq, pi, Tajima-D, Fst, het, HWE, missing |
| [rsomics-vcf-query](https://github.com/omics-rust/rsomics-vcf-query) | Extract fields from VCF records — Rust port of bcftools query |
| [rsomics-vcf-reheader](https://github.com/omics-rust/rsomics-vcf-reheader) | Replace a VCF header or rename samples — Rust port of bcftools reheader |
| [rsomics-vcf-roh](https://github.com/omics-rust/rsomics-vcf-roh) | Runs-of-homozygosity detector — Rust port of bcftools roh |
| [rsomics-vcf-sample](https://github.com/omics-rust/rsomics-vcf-sample) | Random subsample VCF variants by fraction or exact count — bcftools view subsample equival |
| [rsomics-vcf-setgt](https://github.com/omics-rust/rsomics-vcf-setgt) | Conditionally rewrite VCF genotypes — Rust port of bcftools +setGT |
| [rsomics-vcf-sort](https://github.com/omics-rust/rsomics-vcf-sort) | Sort a VCF by chromosome and position — Rust port of bcftools sort |
| [rsomics-vcf-split](https://github.com/omics-rust/rsomics-vcf-split) | Split VCF by chromosome into per-chromosome files |
| [rsomics-vcf-split-vep](https://github.com/omics-rust/rsomics-vcf-split-vep) | Query and extract VEP/bcftools-csq CSQ/BCSQ annotations — Rust port of bcftools +split-vep |
| [rsomics-vcf-stats](https://github.com/omics-rust/rsomics-vcf-stats) | Basic VCF variant statistics — SNP/indel counts, Ti/Tv ratio |
| [rsomics-vcf-to-bed](https://github.com/omics-rust/rsomics-vcf-to-bed) | Convert VCF variant positions to BED intervals |
| [rsomics-vcf-utils](https://github.com/omics-rust/rsomics-vcf-utils) | VCF utility toolkit — view, filter, count, stats, and convert operations |
| [rsomics-vcf-validate](https://github.com/omics-rust/rsomics-vcf-validate) | Validate VCF format integrity |
| [rsomics-vcf-view](https://github.com/omics-rust/rsomics-vcf-view) | Subset and filter VCF records — Rust port of bcftools view |
| [rsomics-vlr](https://github.com/omics-rust/rsomics-vlr) | Variation log-ratio proportionality matrix (and single-pair) of a compositional feature table — scikit-bio vlr/pairwise_vlr port (value-exact ~4e-15, 8.40× -t1, faer triangular GEMM) |
| [rsomics-voom](https://github.com/omics-rust/rsomics-voom) | voom log2-CPM transform + mean-variance precision weights for RNA-seq — Rust reimplementation of limma voom (field-exact, 2.86× -t1) |
| [rsomics-voom-quality-weights](https://github.com/omics-rust/rsomics-voom-quality-weights) | limma voomWithQualityWeights: voom precision weights composed with arrayWeights sample-quality weights — clean-room Rust port of limma (value-exact, 13.55× -t1) |
| [rsomics-wig-to-bed](https://github.com/omics-rust/rsomics-wig-to-bed) | Convert WIG/bedGraph signal tracks to BED intervals above a threshold |
