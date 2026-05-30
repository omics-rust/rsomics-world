# rsomics crate registry

Every crate is an independent repo under [omics-rust](https://github.com/omics-rust) and is published to crates.io. This file is the index; there is no submodule aggregation. Clone any crate flat under `/Volumes/KIOXIA/Documents/omics-rust/` to work on it.

_Generated 2026-05-30 — 231 crates._

| crate | description |
|---|---|
| [rsomics-align-core](https://github.com/omics-rust/rsomics-align-core) | Pairwise sequence alignment kernels (Smith-Waterman + Needleman-Wunsch, affine gap) for th |
| [rsomics-align-score](https://github.com/omics-rust/rsomics-align-score) | Pairwise sequence alignment — Needleman-Wunsch (global) or Smith-Waterman (local) |
| [rsomics-atac-shift](https://github.com/omics-rust/rsomics-atac-shift) | ATAC-seq Tn5 insertion-bias shift: +4/-5 bp coordinate correction and insertion-site BED o |
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
| [rsomics-bam-fasta](https://github.com/omics-rust/rsomics-bam-fasta) | Convert BAM to FASTA — Rust port of samtools fasta |
| [rsomics-bam-fingerprint](https://github.com/omics-rust/rsomics-bam-fingerprint) | ChIP-enrichment fingerprint (cumulative-coverage Lorenz curve) — Rust port of deeptools pl |
| [rsomics-bam-fixmate](https://github.com/omics-rust/rsomics-bam-fixmate) | Fill in mate coordinate, ISIZE and mate-related flags — Rust port of samtools fixmate |
| [rsomics-bam-flagstat](https://github.com/omics-rust/rsomics-bam-flagstat) | SAM/BAM/CRAM flag statistics — Rust port of samtools flagstat |
| [rsomics-bam-head](https://github.com/omics-rust/rsomics-bam-head) | Print the header and the first N alignment records of a BAM as SAM — Rust port of samtools |
| [rsomics-bam-idxstats](https://github.com/omics-rust/rsomics-bam-idxstats) | Per-reference read counts from a BAM index — Rust port of samtools idxstats |
| [rsomics-bam-import](https://github.com/omics-rust/rsomics-bam-import) | Convert FASTQ to unaligned BAM — Rust port of samtools import |
| [rsomics-bam-index](https://github.com/omics-rust/rsomics-bam-index) | Create BAI index for a coordinate-sorted BAM — Rust port of samtools index |
| [rsomics-bam-junctions](https://github.com/omics-rust/rsomics-bam-junctions) | Annotate splice junctions from spliced BAM reads vs BED12 gene model — Rust port of RSeQC  |
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
| [rsomics-bigwig-compare](https://github.com/omics-rust/rsomics-bigwig-compare) | Per-bin comparison of two bigWig files as a bedGraph track — Rust port of deeptools bigwig |
| [rsomics-cell-filter](https://github.com/omics-rust/rsomics-cell-filter) | Filter cells by QC metrics — min genes, min UMIs, max mito fraction from a barcode stats T |
| [rsomics-clipping-profile](https://github.com/omics-rust/rsomics-clipping-profile) | Per-position soft-clipping profile from a BAM — Rust port of RSeQC clipping_profile.py |
| [rsomics-common](https://github.com/omics-rust/rsomics-common) | Shared primitives for every rsomics-* crate (errors, CLI scaffold, runner, progress, exit  |
| [rsomics-compute-matrix](https://github.com/omics-rust/rsomics-compute-matrix) | bigWig signal → score matrix over BED regions — Rust port of deeptools computeMatrix |
| [rsomics-consensus](https://github.com/omics-rust/rsomics-consensus) | Compute consensus sequence from a multiple sequence alignment — majority-rule or threshold |
| [rsomics-count-matrix](https://github.com/omics-rust/rsomics-count-matrix) | Merge multiple featureCounts/htseq-count outputs into a single gene × sample count matrix |
| [rsomics-coverage-core](https://github.com/omics-rust/rsomics-coverage-core) | Genome-binned BAM read-coverage primitive (deeptools countReadsPerBin port): per-bin read  |
| [rsomics-de-volcano](https://github.com/omics-rust/rsomics-de-volcano) | Annotate differential expression results with significance categories for volcano plots |
| [rsomics-debruijn](https://github.com/omics-rust/rsomics-debruijn) | de Bruijn graph types + linear-path collapse + unitig extraction for the rsomics-* tool fa |
| [rsomics-deletion-profile](https://github.com/omics-rust/rsomics-deletion-profile) | Per-base CIGAR-deletion rate along aligned reads — Rust port of RSeQC deletion_profile.py |
| [rsomics-derep](https://github.com/omics-rust/rsomics-derep) | FASTA dereplication — port of vsearch --derep_fulllength / --derep_prefix |
| [rsomics-deseq-prep](https://github.com/omics-rust/rsomics-deseq-prep) | Filter low-count genes and normalize a count matrix for differential expression — pre-DESe |
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
| [rsomics-featurecounts](https://github.com/omics-rust/rsomics-featurecounts) | Count reads over genomic features (BAM + GFF) — Rust port of featureCounts (Subread) |
| [rsomics-fm-index](https://github.com/omics-rust/rsomics-fm-index) | FM-index over BWT + suffix array, with backward search / count / locate. Layer A primitive |
| [rsomics-fm-search](https://github.com/omics-rust/rsomics-fm-search) | Exact substring search in FASTA using FM-index — count or locate pattern occurrences |
| [rsomics-fpkm-count](https://github.com/omics-rust/rsomics-fpkm-count) | Compute per-gene FPKM from a BAM + BED12 model — Rust port of RSeQC FPKM_count |
| [rsomics-fqgz](https://github.com/omics-rust/rsomics-fqgz) | Chunked parallel-libdeflate gzip (or plain) FASTQ-record writer. Layer-A primitive shared  |
| [rsomics-fragment-size](https://github.com/omics-rust/rsomics-fragment-size) | Paired-end insert-size distribution from a BAM: histogram TSV + summary with ATAC nucleoso |
| [rsomics-gc-windows](https://github.com/omics-rust/rsomics-gc-windows) | Compute per-window GC content across a FASTA reference — BED output for CNV/WGS normalizat |
| [rsomics-genebody-coverage](https://github.com/omics-rust/rsomics-genebody-coverage) | Gene-body coverage profile (5'→3') for RNA-seq bias QC — Rust port of RSeQC geneBody_cover |
| [rsomics-gff-utils](https://github.com/omics-rust/rsomics-gff-utils) | GFF/GTF utility toolkit — count, filter, extract, sort, convert, and stats operations |
| [rsomics-help](https://github.com/omics-rust/rsomics-help) | Family-wide `--help` renderer for rsomics-* CLIs: figlet banner with gradient, section hel |
| [rsomics-hmm](https://github.com/omics-rust/rsomics-hmm) | Hidden Markov Model inference (Viterbi / forward / backward) for the rsomics-* tool family |
| [rsomics-hmm-decode](https://github.com/omics-rust/rsomics-hmm-decode) | Viterbi-decode observation sequences with a discrete HMM — chromatin state, gene finding,  |
| [rsomics-igzip](https://github.com/omics-rust/rsomics-igzip) | Minimal Quadrant-② FFI wrapper over Intel ISA-L igzip for fast gzip decompression. Isolate |
| [rsomics-infercnv](https://github.com/omics-rust/rsomics-infercnv) | Infer copy-number variations from single-cell RNA-seq expression — Rust port of inferCNV |
| [rsomics-inner-distance](https://github.com/omics-rust/rsomics-inner-distance) | mRNA-aware inner-distance distribution for paired-end RNA-seq — Rust port of RSeQC inner_d |
| [rsomics-insertion-profile](https://github.com/omics-rust/rsomics-insertion-profile) | Per-position CIGAR-insertion rate along the read — Rust port of RSeQC insertion_profile.py |
| [rsomics-intervals](https://github.com/omics-rust/rsomics-intervals) | BED algebra + interval index + GFF/GTF interval extraction for the rsomics-* tool family.  |
| [rsomics-junction-saturation](https://github.com/omics-rust/rsomics-junction-saturation) | Subsample-based splice-junction saturation analysis — Rust reimplementation of RSeQC junct |
| [rsomics-kmer](https://github.com/omics-rust/rsomics-kmer) | K-mer encoding, canonicalisation, ntHash rolling hash, MurmurHash3, k-mer counting for the |
| [rsomics-kmer-dist](https://github.com/omics-rust/rsomics-kmer-dist) | Pairwise k-mer frequency distance between FASTA/FASTQ samples — Jaccard/Bray-Curtis/cosine |
| [rsomics-kraken-report](https://github.com/omics-rust/rsomics-kraken-report) | Parse and summarize Kraken2 report files — top taxa, diversity stats |
| [rsomics-ld-matrix](https://github.com/omics-rust/rsomics-ld-matrix) | Compute pairwise linkage disequilibrium (r²) from a genotype matrix |
| [rsomics-methyldackel](https://github.com/omics-rust/rsomics-methyldackel) | Per-CpG methylation extraction from bisulfite-aligned BAM — Rust port of MethylDackel extr |
| [rsomics-minimap2](https://github.com/omics-rust/rsomics-minimap2) | Long/short-read aligner — CLI wrapper of minimap2 FFI bindings (Quadrant ②) |
| [rsomics-mismatch-profile](https://github.com/omics-rust/rsomics-mismatch-profile) | Per-base mismatch-rate profile from BAM MD tags — Rust port of RSeQC mismatch_profile.py |
| [rsomics-models](https://github.com/omics-rust/rsomics-models) | Pinned DL-model registry + per-OS cache + sha256-verify for the rsomics-* tool family. Lay |
| [rsomics-motif-scan](https://github.com/omics-rust/rsomics-motif-scan) | Scan FASTA sequences for IUPAC DNA motif occurrences — BED output of match positions |
| [rsomics-msa-trim](https://github.com/omics-rust/rsomics-msa-trim) | Trim MSA columns by gap fraction — Rust replacement for trimAl -gt |
| [rsomics-multibam-summary](https://github.com/omics-rust/rsomics-multibam-summary) | Multi-BAM per-bin / per-region read-count matrix — Rust port of deeptools multiBamSummary |
| [rsomics-multibigwig-summary](https://github.com/omics-rust/rsomics-multibigwig-summary) | Multi-bigWig per-bin / per-region mean-signal matrix — Rust port of deeptools multiBigwigS |
| [rsomics-nj-tree](https://github.com/omics-rust/rsomics-nj-tree) | Neighbor-joining tree construction from a distance matrix — outputs Newick |
| [rsomics-pdb-chain](https://github.com/omics-rust/rsomics-pdb-chain) | Extract or split PDB chains — list, extract single chain, or split all into separate files |
| [rsomics-peak-count](https://github.com/omics-rust/rsomics-peak-count) | Count BAM reads per BED peak region — ChIP-seq/ATAC-seq QC and quantification |
| [rsomics-pgen](https://github.com/omics-rust/rsomics-pgen) | PLINK1 .bed / .bim / .fam genotype-matrix reader + writer for the rsomics-* tool family. L |
| [rsomics-phylo-tree](https://github.com/omics-rust/rsomics-phylo-tree) | Phylogenetic tree type + Newick parser/emitter for the rsomics-* tool family. Layer A prim |
| [rsomics-pileup](https://github.com/omics-rust/rsomics-pileup) | Coordinate-sorted BAM pileup engine (htslib bam_plp port): per-position read columns with  |
| [rsomics-plink-assoc](https://github.com/omics-rust/rsomics-plink-assoc) | PLINK1 case/control association test (chi-squared + trend + linear regression) |
| [rsomics-plink-io](https://github.com/omics-rust/rsomics-plink-io) | PLINK1 binary .bed/.bim/.fam reader: allele-freq, missingness, HWE, VCF/012 export |
| [rsomics-plink-prune](https://github.com/omics-rust/rsomics-plink-prune) | LD pruning from PLINK1 binary filesets (--indep-pairwise) |
| [rsomics-popgen-core](https://github.com/omics-rust/rsomics-popgen-core) | Population-genetics primitives: π, Watterson's θ, Tajima's D, Hardy-Weinberg exact, LD r². |
| [rsomics-pvalue-adjust](https://github.com/omics-rust/rsomics-pvalue-adjust) | Multiple-testing correction for a column of p-values — the full R p.adjust method set (hol |
| [rsomics-read-duplication](https://github.com/omics-rust/rsomics-read-duplication) | Sequence-based and position-based read duplication rate — Rust port of RSeQC read_duplicat |
| [rsomics-read-gc](https://github.com/omics-rust/rsomics-read-gc) | Per-read GC% distribution from a BAM — Rust port of RSeQC read_GC.py |
| [rsomics-read-nvc](https://github.com/omics-rust/rsomics-read-nvc) | Per-cycle nucleotide composition (NVC) from a BAM — Rust port of RSeQC read_NVC.py |
| [rsomics-read-quality](https://github.com/omics-rust/rsomics-read-quality) | Per-base read-quality heatmap and boxplot from BAM — Rust port of RSeQC read_quality.py |
| [rsomics-rereplicate](https://github.com/omics-rust/rsomics-rereplicate) | Expand abundance-annotated FASTA back into individual reads — port of vsearch --rereplicat |
| [rsomics-rnaseq-metrics](https://github.com/omics-rust/rsomics-rnaseq-metrics) | RNA-seq QC metrics (region coverage fractions, strand bias, transcript-coverage bias) — Ru |
| [rsomics-rpkm-saturation](https://github.com/omics-rust/rsomics-rpkm-saturation) | Subsample-based RPKM saturation analysis — Rust reimplementation of RSeQC RPKM_saturation. |
| [rsomics-sam-to-bam](https://github.com/omics-rust/rsomics-sam-to-bam) | Convert SAM to BAM — Rust equivalent of samtools view -bS |
| [rsomics-sample-sheet](https://github.com/omics-rust/rsomics-sample-sheet) | Parse, validate, and convert sample sheets (Illumina/custom TSV) — check FASTQ paths, dete |
| [rsomics-seacr](https://github.com/omics-rust/rsomics-seacr) | CUT&RUN peak caller (bedGraph → BED peaks) — clean-room Rust port of SEACR |
| [rsomics-seq-stats](https://github.com/omics-rust/rsomics-seq-stats) | Quick stats for any FASTA/FASTQ — count, total bp, N50, GC%, min/max/mean length |
| [rsomics-seqio](https://github.com/omics-rust/rsomics-seqio) | Fast FASTQ reader for the rsomics-* tool family: decode-only producer thread + parallel pa |
| [rsomics-seqstats](https://github.com/omics-rust/rsomics-seqstats) | Format-agnostic sequence statistics primitives (length distribution: N50/L50/Nx + quartile |
| [rsomics-stats](https://github.com/omics-rust/rsomics-stats) | Statistical tests, FDR control, p-value combination for the rsomics-* tool family. Layer A |
| [rsomics-tajima-d](https://github.com/omics-rust/rsomics-tajima-d) | Compute Tajima's D from a site frequency spectrum (derived allele counts) |
| [rsomics-tax-assign](https://github.com/omics-rust/rsomics-tax-assign) | Lightweight taxonomic assignment from k-mer LCA — classify reads against a reference taxon |
| [rsomics-taxonomy](https://github.com/omics-rust/rsomics-taxonomy) | NCBI taxdump parser + LCA + lineage helpers for the rsomics-* tool family. Layer A primiti |
| [rsomics-tin](https://github.com/omics-rust/rsomics-tin) | Transcript Integrity Number (TIN) for RNA-seq QC — Rust port of RSeQC tin.py |
| [rsomics-tpm](https://github.com/omics-rust/rsomics-tpm) | TPM and FPKM normalization of gene count matrices given gene lengths |
| [rsomics-tsv-join](https://github.com/omics-rust/rsomics-tsv-join) | Join two TSV files by a shared key column — inner/left/outer join |
| [rsomics-tsv-select](https://github.com/omics-rust/rsomics-tsv-select) | Select, reorder, or rename columns from TSV files — cut + awk for bioinformatics pipelines |
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
| [rsomics-vcf-stats](https://github.com/omics-rust/rsomics-vcf-stats) | Basic VCF variant statistics — SNP/indel counts, Ti/Tv ratio |
| [rsomics-vcf-to-bed](https://github.com/omics-rust/rsomics-vcf-to-bed) | Convert VCF variant positions to BED intervals |
| [rsomics-vcf-utils](https://github.com/omics-rust/rsomics-vcf-utils) | VCF utility toolkit — view, filter, count, stats, and convert operations |
| [rsomics-vcf-validate](https://github.com/omics-rust/rsomics-vcf-validate) | Validate VCF format integrity |
| [rsomics-vcf-view](https://github.com/omics-rust/rsomics-vcf-view) | Subset and filter VCF records — Rust port of bcftools view |
| [rsomics-wig-to-bed](https://github.com/omics-rust/rsomics-wig-to-bed) | Convert WIG/bedGraph signal tracks to BED intervals above a threshold |
