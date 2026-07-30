# 03 — Transcriptomics (bulk RNA-seq)

> Spliced alignment, transcript / gene quantification, isoform assembly,
> differential expression, and alternative splicing for **bulk** RNA-seq.

Single-cell RNA-seq lives in [`04-single-cell/`](../04-single-cell/). Most of
the upstream IO and short-read primitives we lean on are described in
[`01-foundations/`](../01-foundations/) and DNA short-read alignment lives in
[`02-genomics/`](../02-genomics/) — RNA-only tools shared with neither are the
focus here.

## Sub-topics

- [`alignment-spliced.md`](alignment-spliced.md) — STAR, HISAT2, TopHat2,
  Subjunc, MapSplice.
- [`quantification.md`](quantification.md) — Salmon, kallisto, RSEM,
  featureCounts, HTSeq-count, StringTie (eXpress-style abundance).
- [`assembly-isoform.md`](assembly-isoform.md) — StringTie, Cufflinks, Trinity,
  Bridger, SOAPdenovo-Trans, IsoQuant.
- [`differential-expression.md`](differential-expression.md) — DESeq2, edgeR,
  limma-voom, sleuth, NOISeq, EBSeq.
- [`splicing.md`](splicing.md) — rMATS, SUPPA2, LeafCutter, MAJIQ, Whippet,
  JUM, IsoformSwitchAnalyzeR.

## Cross-cutting design notes

- The transcriptomics stack spans format-heavy alignment/quantification and
  stateful statistical workflows. DESeq2, edgeR, and limma already have
  substantial historical Rust kernels, but only a complete fitted workflow
  with current upstream differentials can establish compatibility.
- COMBINE-lab has produced the most mature Rust output in this space:
  `alevin-fry`, `simpleaf`, `piscem` (Rust wrapper over a C++ index core),
  and `oarfish` (long-read transcript quantification). These define the
  current "adopt as-is" set.
- Splice-aware alignment is the single biggest open hole. No pure-Rust port
  of STAR or HISAT2 exists; both are large engineering projects with
  decade-old C / C++ codebases.
- Statistical-package reconstruction is consumer-driven: keep method policy in
  the product, promote only demonstrated policy-free kernels to
  `rsomics-stats`, and use R as a versioned behavior oracle rather than
  assuming either an FFI wrapper or an old Rust port is sufficient.
- We deliberately list legacy tools (TopHat2, Cufflinks) for reference
  even though their authors recommend successors — pipelines still
  invoke them, and benchmark suites compare against them.
