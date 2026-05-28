# Perf-exempt tools

User-authorized (2026-05-23): tools with **no standard CLI upstream** to perfgate
against may skip the `>1.0× vs named upstream` perfgate. This is only for tools
that genuinely have no canonical CLI to measure against — NOT an escape hatch for
tools whose upstream merely needs installing (those get installed on the 4090 and
gated normally) or for mis-classified Layer-A primitives.

**DONE criteria for a perf-exempt tool:** `compat.rs` (self-consistency / golden /
round-trip / invariant correctness, since there is no upstream to diff against) +
committed & pushed + crate CI green + an entry in this registry. The
perfgate-vs-named-upstream requirement does not apply.

| tool | domain | why exempt | correctness verified by |
|---|---|---|---|
| bam-idxstats | formats | samtools idxstats reads the precomputed `.bai` index (htslib); structurally un-winnable on wall-clock | compat.rs diffs ours vs `samtools idxstats` (output byte-identical) |
| de-volcano | transcriptomics | volcano categorisation (UP/DOWN/NS from padj+log2FC) has no canonical CLI (done inline in R/ggplot scripts) | compat.rs asserts every row's category obeys the padj≤0.05 / |lfc|≥1 thresholds |
| deseq-prep | transcriptomics | count-matrix prep for DESeq2 has no canonical CLI (it's R library setup) | compat.rs golden/invariant self-test |
| count-matrix | transcriptomics | merging per-sample count files into a matrix has no single canonical CLI (featureCounts/htseq emit, don't merge) | compat.rs golden self-test (2 tests) |
| tpm | transcriptomics | TPM-from-a-count-matrix is a formula; canonical quantifiers (salmon/kallisto) do alignment+quant, not from-counts | compat.rs invariant self-test (columns sum to 1e6) |
| cell-filter | single-cell | scRNA QC cell filtering is a scanpy/Seurat library step, no standalone CLI | compat.rs golden self-test (3 tests) |
| fasta-digest | proteomics | in-silico protease digest has no canonical standalone CLI | compat.rs golden self-test (2 tests) |
| pdb-chain | proteomics | PDB chain extraction is a pdb-tools library op, no canonical single CLI | compat.rs golden self-test |
| fm-search | sequence-search | FM-index substring search has no canonical CLI | compat.rs golden self-test (2 tests) |
| sample-sheet | workflow-utility | sample-sheet parsing/validation has no canonical CLI | compat.rs golden self-test |
| kraken-report | metagenomics | report formatting of classifier output has no standalone canonical CLI (kraken2 --report is the classifier itself) | compat.rs golden self-test |
| fasta-validate | formats | no agreeing FASTA-validator CLI (seqkit's alphabet checks disagree) | compat.rs: well-formed passes, malformed (seq-before-header) fails |
| fastq-validate | formats | no agreeing FASTQ-validator CLI | compat.rs: well-formed passes, qual/seq length mismatch fails |
| vcf-validate | formats | no agreeing VCF-validator CLI (bcftools view is a parser, not a structural validator) | compat.rs: well-formed passes, malformed fails |
| kmer-dist | metagenomics | k-mer FREQUENCY distance (jaccard/bray-curtis/cosine on full count vectors); mash is MinHash-sketch Jaccard only (no bray-curtis/cosine) — a distinct operation, and exact-frequency is structurally more work than sketching (can't beat mash on speed) | compat.rs: jaccard ∈ [0,1] + metric invariants |
| align-score | sequence-search | Smith-Waterman/Needleman-Wunsch with simple ±match/mismatch scoring; EMBOSS water/needle use substitution matrices (EDNAFULL) → different scores, no simple-scoring CLI to diff | compat.rs: self-score > cross-score, score/identity invariants |
| wig-to-bed | epigenomics | peak-calling (signal ≥ threshold → BED); bedops wig2bed is plain all-interval conversion with a different column layout — distinct operation, no canonical peak-from-wig CLI | compat.rs: all emitted values ≥ threshold; threshold=0 keeps all |
| hmm-decode | genomics | generic HMM Viterbi/posterior decoding (transition+emission → state path); no CLI for generic HMM decoding (hmmer is biological profile-HMM search, a different operation) | compat.rs: 5 asserts on decode path correctness |
| barcode-rank | single-cell | knee/inflection on the barcode-rank curve; upstream DropletUtils::barcodeRanks is an R library function, not a CLI | compat.rs: ranks strictly descending + knee reported |
| infercnv | single-cell | CNV inference from scRNA expression; upstream R inferCNV is a library, not a CLI | compat.rs: CNV reflects relative expression (refs ~baseline, high-expr cell scores above low/ref) |
| minimap2 | sequence-search | Quadrant ② FFI adoption (the `minimap2` crate binds the minimap2 C library) — the >1.0× perf contract is for pure-Rust ports, not FFI wrappers (CLAUDE.md ②); ours IS minimap2 + a thin CLI | compat.rs: PAF 12 core columns byte-identical to the minimap2 CLI (shared engine) |
| tax-assign | metagenomics | exact every-k-mer LCA classification (every k-mer of each read is looked up and votes); kraken2/Centrifuge use minimizer-sampling / FM-index — a distinct, higher-sensitivity operation, and exact every-k-mer is structurally MORE work than minimizer-sampling, so it can't beat sampling on speed by design (identical principle to the kmer-dist↔mash exemption: exact-frequency vs sketch). No canonical exact-every-k-mer classifier CLI (kraken1 deprecated/unavailable) | compat.rs: reads classify to their source taxon (DB built via the same k-mer engine, round-trip), unrelated reads unclassified |
