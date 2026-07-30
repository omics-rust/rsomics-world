# Differential-expression survey

Status: upstream survey. The authoritative portfolio and implementation
decisions are in
[`../10-products/bulk-expression.md`](../10-products/bulk-expression.md).

## Scope

Bulk differential-expression methods combine a count or log-expression matrix,
sample metadata, a design, fitted mean/variance state, hypothesis tests,
multiple-testing policy, diagnostics, and result tables.

Matrix construction from alignments belongs to
[`rsomics-count`](../10-products/count.md). Transcript abundance and
length-aware aggregation remain quantification concerns. Single-cell marker
testing and pseudobulk policy belong to the single-cell product review.

## Current product anchors

| Upstream | Accepted product | Core identity |
|---|---|---|
| [DESeq2](https://bioconductor.org/packages/release/bioc/html/DESeq2.html) | `rsomics-deseq` | negative-binomial GLM, size factors, dispersion shrinkage, Wald/LRT inference, effect shrinkage, and stabilized transforms |
| [edgeR](https://bioconductor.org/packages/release/bioc/html/edgeR.html) | `rsomics-edger` | DGE library state, expression filtering, normalization factors, NB and current quasi-likelihood inference |
| [limma](https://bioconductor.org/packages/release/bioc/html/limma.html) | `rsomics-limma` | linear models, empirical-Bayes moderation, voom precision weights, threshold tests, correlation, and gene-set workflows |

These are separate stateful products. They share possible policy-free
numerical kernels but do not become one umbrella binary with a method switch.
The rejected `rsomics-expression` name had no coherent statistical or
installation identity.

## Other method families

| Family | Distinct contract | Portfolio decision |
|---|---|---|
| sleuth and fishpond/Swish | inferential replicate or bootstrap-aware transcript analysis | outside the current allowlist; reconsider with a real quantification consumer |
| NOISeq | non-parametric noise modeling | excluded from the initial three products |
| EBSeq | hierarchical empirical-Bayes gene/isoform inference | excluded; do not hide it as a DESeq mode |
| DEXSeq | exon-usage NB models | review with transcript/splicing workflows |
| apeglm and ashr | adaptive effect-size shrinkage | method-specific dependencies or implementations under `rsomics-deseq`, not generic result formatting |
| tximport and tximeta | transcript-to-gene aggregation and provenance | quantification/input integration, not differential-expression fitting |

## Reconstruction principles

- Preserve a complete fitted analysis rather than flattening every upstream
  function into an independent CLI.
- Join count columns, sample metadata, and model terms by stable identity.
- Treat designs, contrasts, rank, filtering, convergence, and missing values as
  part of the public behavior.
- Compare against exact current upstream versions with field-specific
  tolerances and discrete-decision checks.
- Share numerical APIs only after two product consumers demonstrate the same
  contract.
- Require an end-to-end throughput or resource advantage for any declared
  replacement slice.

The historical source pool contains substantial Rust kernels and R goldens for
all three retained products. They are implementation and test assets, not proof
that the previous operation-sized package boundaries or compatibility claims
were correct.
