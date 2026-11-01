# DNA methylation

> Bisulfite-sequencing alignment / extraction and long-read modified-base
> calling and summarisation.

Status: upstream operation survey. The [methyl product dossier](../10-products/methyl.md)
and [namespace allowlist](../00-overview/registry-reset-keep.txt) define the
accepted public boundary. Upstream implementation details below are inherited
survey observations, not fresh version, installation, or performance checks.

## Scope

Two regimes:

1. **Short-read bisulfite sequencing (WGBS, RRBS)** — Bismark, BWA-meth,
   MethylDackel for extraction, methylpy / methylKit for analysis.
2. **Long-read modified-base processing (Nanopore, PacBio)** — modkit
   (modBAM-based summarisation) and the legacy nanopolish call-methylation.

Differential methylation testing at the regional level (DMR / DML) is
covered here in the analysis tools (methylKit, methylpy). Single-cell
methylation lives in module 04.

`rsomics-methyl` owns bisulfite methylation extraction and bias QC, not
alignment, differential methylation, or long-read modified-base calling.

| MethylDackel operation | Owning product operation |
|---|---|
| `extract` | `rsomics-methyl extract` |
| `mbias` | `rsomics-methyl mbias` |
| `mergeContext` | `rsomics-methyl merge-context` |
| `perRead` | `rsomics-methyl per-read` |

## Design notes

- The long-read side includes **`modkit`** from Oxford Nanopore, an
  external Rust toolkit rather than a new rsomics product boundary.
- The short-read tools span alignment, extraction, and analysis:
  Bismark is Perl wrapping Bowtie2, BWA-meth is a Python wrapper around
  BWA, and MethylDackel is C. Bisulfite alignment uses nucleotide-converted
  references and reads (C→T forward, G→A reverse); alignment compatibility
  is separate from methylation extraction and has no approved public rsomics
  boundary here.
- Bisulfite calling accounts for an indexed reference, CpG / CHG / CHH context,
  available aligner tags, read orientation, and OT / OB / CTOT / CTOB strand
  interpretation. Overlapping
  mates must contribute at most one call per molecule and position. These
  policies stay in `rsomics-methyl`; validated alignment I/O and generic
  checked CIGAR / pileup / overlapping-mate evidence use `rsomics-bamio`
  and `rsomics-pileup`. `rsomics-common` supplies errors, execution reports,
  and single-output transactions; `rsomics-help` supplies the command tree.
- The SAM MM / ML tags encode modified-base evidence in modBAM. That evidence
  path is distinct from reference- and strand-dependent bisulfite calls;
  `rsomics-methyl` does not promise modBAM emission.
- `methylpy` and `methylKit` provide downstream DMR / DML analysis,
  smoothing, and statistical models such as beta-binomial likelihoods.
  Their analysis workflows are outside
  `rsomics-methyl`; no automatic R / Python bridge or performance gain is
  established by this survey.

## Upstream operation survey

- **`Bismark`** — Perl-orchestrated bisulfite aligner over Bowtie2.
  - Reference impl: `Perl` · [FelixKrueger/Bismark](https://github.com/FelixKrueger/Bismark) · `GPL-3.0`
  - Existing Rust: none verified
  - Existing Rust kind: `none`
  - Existing non-C alternatives: BWA-meth (Python+BWA)
  - Parallelism: upstream Perl wrapping Bowtie2 pthreads
  - SIMD: inherited from bowtie2
  - Quadrant: —
  - GPU-amenable: maybe — same SW kernel rationale as short-read aligners
  - Upstream license: `GPL-3.0`
  - Portfolio routing: surveyed alignment workflow, outside `rsomics-methyl`; no approved replacement product.
  - Notes: Compatibility includes nucleotide conversion, restored base identities, strand and aligner tags, and the Bismark BAM contract. A validated short-read aligner is a separate prerequisite, not a new shared-foundation promise.

- **`BWA-meth`** — Python wrapper around BWA for bisulfite alignment.
  - Reference impl: `Python + C` · [brentp/bwa-meth](https://github.com/brentp/bwa-meth) · `MIT`
  - Existing Rust: none verified
  - Existing Rust kind: `none`
  - Existing non-C alternatives: —
  - Parallelism: upstream BWA pthreads
  - SIMD: inherited from BWA
  - Quadrant: —
  - GPU-amenable: maybe — same as Bismark
  - Upstream license: `MIT`
  - Portfolio routing: surveyed alignment workflow, outside `rsomics-methyl`; no approved replacement product.
  - Notes: The BWA alignment backend and converted-reference workflow remain upstream behavior to study. This does not approve a public aligner or alignment foundation.

- **`methylKit`** — R/Bioconductor methylation analysis (DMR / DML).
  - Reference impl: `R / C++` · [Bioconductor methylKit](https://bioconductor.org/packages/release/bioc/html/methylKit.html) · `Artistic-2.0`
  - Existing Rust: none verified
  - Existing Rust kind: `none`
  - Existing non-C alternatives: —
  - Parallelism: R BiocParallel + C++ inner loops
  - SIMD: limited
  - Quadrant: —
  - GPU-amenable: maybe — DMR testing is per-region parallel
  - Upstream license: `Artistic-2.0`
  - Portfolio routing: downstream statistical workflow, excluded from `rsomics-methyl`.
  - Notes: Logistic-regression DMR testing and smoothing are analysis contracts, not extraction options. The dossier's methylKit-format extraction output does not implement the methylKit analysis package or establish a shared statistics API.

- **`methylpy`** — Python methylation extractor + DMR finder.
  - Reference impl: `Python / C++` · [yupenghe/methylpy](https://github.com/yupenghe/methylpy) · `MIT`
  - Existing Rust: none verified
  - Existing Rust kind: `none`
  - Existing non-C alternatives: —
  - Parallelism: Python multiprocessing
  - SIMD: limited
  - Quadrant: —
  - GPU-amenable: no — Python orchestration over per-region work
  - Upstream license: `MIT`
  - Portfolio routing: extraction is an upstream comparison candidate; DMR analysis is outside `rsomics-methyl`.
  - Notes: Its combined extraction / analysis workflow is not treated as interchangeable with MethylDackel, and this survey establishes neither methylpy compatibility nor a Python bridge.

- **`MethylDackel`** — methylation extraction and bias QC for bisulfite alignments.
  - Reference impl: `C` · [dpryan79/MethylDackel](https://github.com/dpryan79/MethylDackel) · `MIT`
  - Existing Rust: [`rsomics-methyl`](../10-products/methyl.md), with the four operations mapped above.
  - Historical source asset: retired `rsomics-methyldackel` at `9d32057f7ec5eb8bb241d53c115280f9d6acbdea`; algorithms, fixtures, and benchmark recipes are classified in the dossier, not offered as a standalone installation.
  - Existing non-C alternatives: —
  - Parallelism: upstream pthreads; product worker behavior and measurements belong to the dossier.
  - GPU-amenable: no — per-position binomial counting, memory-latency-bound
  - Upstream license: `MIT`
  - Portfolio routing: `rsomics-methyl`; no publication under the retired name.
  - Compatibility: the dossier distinguishes MethylDackel 0.6.1 from its pinned corrected primary oracle. Context, strand, overlap, trimming, reference, and output semantics need the appropriate profile, not an inherited blanket compatibility claim.
  - Historical measurement lead: this survey recorded `rsomics-methyldackel` 0.1.0 as 4.43× faster than MethylDackel 0.6.1 on a 50k-read bisulfite BAM (`mini_m2`, 2026-05-29), with CpG bedGraph output and a reported compatibility test. This is an asset-recovery lead, not current `rsomics-methyl` performance or representative release evidence.

- **`modkit`** — Oxford Nanopore modified-base toolkit (modBAM-based).
  - Reference impl: `Rust` · [nanoporetech/modkit](https://github.com/nanoporetech/modkit) · `MPL-2.0`
  - Existing Rust: the upstream toolkit itself; current installation details are not verified here.
  - Existing Rust kind: `rust-native`
  - Existing non-C alternatives: —
  - Parallelism: rayon
  - SIMD: inherits htslib's hand SIMD via `rust-htslib`
  - Quadrant: ②
  - GPU-amenable: no — bedMethyl summarisation is memory-bandwidth-bound
  - Upstream license: `MPL-2.0`
  - Portfolio routing: external tool for the distinct long-read modified-base workflow.
  - Notes: The inherited survey describes modBAM → bedMethyl summarisation and DMR/DML analysis, with Rust algorithms and `rust-htslib` / HTSlib BAM I/O (Quadrant ②). That dependency classification is historical, not a fresh audit of modkit or a reason to merge its evidence model into bisulfite extraction.

- **`nanopolish`** (call-methylation) — legacy HMM modified-base caller from signal-level data.
  - Reference impl: `C++` · [jts/nanopolish](https://github.com/jts/nanopolish) · `MIT`
  - Existing Rust: none verified
  - Existing Rust kind: `none`
  - Existing non-C alternatives: modkit (modBAM-based downstream processing, not the same signal-level input contract)
  - Parallelism: upstream pthreads
  - SIMD: upstream SSE
  - Quadrant: —
  - GPU-amenable: no — HMM signal-level, legacy
  - Upstream license: `MIT`
  - Portfolio routing: legacy upstream reference, not a porting target.
  - Notes: Retained for legacy R9 nanopore data analysis. Signal-level HMM calling and summarising already-called MM / ML evidence are distinct operations; neither is a `rsomics-methyl` command.
