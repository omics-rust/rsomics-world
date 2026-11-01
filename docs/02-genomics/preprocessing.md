# Preprocessing & QC

> Quality trimming, adapter removal, FASTQ wrangling, and read-level QC
> before alignment.

Status: upstream operation survey. The [sequence and FASTQ product dossier](../10-products/sequence-fastq.md)
and [namespace allowlist](../00-overview/registry-reset-keep.txt) define the
accepted public boundaries. Upstream versions, backends, installation details,
and license labels below are inherited survey observations, not fresh audits.

## Scope

Everything that touches raw FASTQ before the aligner sees it: trimmers
(fastp, Trimmomatic, cutadapt, BBDuk, Trim Galore, AdapterRemoval), QC
reporters (FastQC, MultiQC), and Swiss-Army-knife sequence-manipulation
tools (seqkit, seqtk). Basecalling (Guppy/Dorado), long-read graphics
(NanoPlot), and post-alignment QC (samtools stats, mosdepth) are context,
not operations of the FASTQ products described here.

| Workflow | Owning product and current surface |
|---|---|
| Read preprocessing | `rsomics-fastq-preprocess run`, `trim`, `filter`: fixed, poly-G/poly-X, quality, N-content, length, and complexity transforms |
| FastQC-style diagnostics | `rsomics-fastq-qc report`: per-input data, status summary, and self-contained HTML, with multi-file parallelism |
| Generic FASTA/FASTQ utilities | `rsomics-seq stats`, `kmers`, `grep`, `convert`, `validate`; further transforms remain dossier plans |

These are not full replacements for every upstream feature below. UMI,
adapter alignment, paired-read merging, ancient-DNA collapse, correction,
and reference-based decontamination need their own compatibility contracts
before becoming advertised operations.

The products use `rsomics-seqio` for strict record I/O, `rsomics-common` for
execution, errors, and output contracts, and `rsomics-help` for CLI help.
The useful historical `rsomics-fqgz` algorithm is private to preprocessing,
not a public dependency to revive. `rsomics-kmer` has sequence and sketch
consumers; it is not a current preprocessing dependency. This survey approves
no new shared statistics API.

## Design notes

- `fastp` combines trimming, filtering, QC, UMI handling, and JSON / HTML
  reporting. That breadth is upstream context, not a parity claim for the
  current preprocessing slice or its JSON schema.
- One-pass composition avoids repeated decompression, traversal, and
  compression. `rsomics-fastq-preprocess run` composes its supported
  transforms over one ordered single-end / paired-end chunk engine; separate
  QC and generic sequence products retain their own output contracts.
- RastQC, fastqc-rs, fasterp, seqtk-rs, faster, and chopper are external
  implementation or comparison candidates, not rsomics product dependencies
  approved by this survey. MultiQC remains an external report aggregator.
- Generic counts, searches, conversions, and later sequence transforms belong
  to `rsomics-seq`, not new seqkit- or seqtk-named public crates.
- AdapterRemoval's ancient-DNA collapse, BBDuk's reference-based contaminant
  matching, and cutadapt's adapter alignment have distinct semantics; the
  current trim / filter slice does not establish those contracts.

## Upstream operation survey

- **`fastp`** — all-in-one FASTQ preprocessor.
  - Reference impl: `C/C++` · [OpenGene/fastp](https://github.com/OpenGene/fastp) · `MIT`
  - Existing Rust: the survey recorded [`fasterp`](https://crates.io/crates/fasterp) `0.2.1`, claiming fastp-compatible JSON; its recorded default `native` feature used `libdeflater` + `flate2/zlib-ng` FFI codecs, with AWS SDK + tokio for cloud streaming.
  - Existing Rust kind: `pure-port`
  - Existing non-C alternatives: —
  - Parallelism: rayon over read pairs
  - SIMD: auto-vectorize on quality + base counting; explicit candidates for `std::simd` in adapter overlap scan; codec SIMD inherited from `libdeflater` / `zlib-ng`
  - Quadrant: ①+②
  - GPU-amenable: maybe — read-level parallelism is SIMT-trivial; engineering cost vs. CPU performance is marginal
  - Upstream license: `MIT`
  - Portfolio routing: supported transforms belong to `rsomics-fastq-preprocess`; the retired `rsomics-fastp` CLI, JSON, and fixtures are evidence-only assets.
  - Notes: UMI, adapter detection, overlap correction, and output splitting are separate feature contracts, not implied by `run`. The historical fasterp codec and cloud-dependency observations are comparison leads, not a fresh adoption decision or a new public crate plan. Compatibility is limited to the aligned operations and outputs recorded in the dossier.

- **`Trimmomatic`** — adapter + quality trimming (Java).
  - Reference impl: `Java` · [usadellab/Trimmomatic](https://github.com/usadellab/Trimmomatic) · `GPL-3.0`
  - Existing Rust: none verified
  - Existing Rust kind: `none`
  - Existing non-C alternatives: —
  - Parallelism: JVM threading
  - SIMD: none
  - Quadrant: —
  - GPU-amenable: no — same constraints as fastp
  - Upstream license: `GPL-3.0`
  - Portfolio routing: upstream reference and legacy-pipeline context, not a new product queue.
  - Notes: Its adapter and quality policies remain distinct upstream behavior; the current preprocessing slice is not advertised as Trimmomatic-compatible.

- **`cutadapt`** — adapter-trimming tool.
  - Reference impl: `Python / C (Cython)` · [marcelm/cutadapt](https://github.com/marcelm/cutadapt) · `MIT`
  - Existing Rust: none verified
  - Existing Rust kind: `none`
  - Existing non-C alternatives: —
  - Parallelism: Python multiprocessing
  - SIMD: upstream's Cython adapter alignment uses SSE
  - Quadrant: —
  - GPU-amenable: maybe — adapter alignment is SW; per-read SIMT-trivial
  - Upstream license: `MIT`
  - Portfolio routing: adapter-alignment semantics are a future preprocessing contract, not an implemented compatibility mode.
  - Notes: `fastp` and `fasterp` use overlap-based adapter handling, unlike cutadapt's adapter alignment. Small-RNA and single-cell linker protocols need those distinctions preserved; the current trim / filter slice does not establish cutadapt parity.

- **`BBDuk`** — k-mer-based contaminant removal + trimmer.
  - Reference impl: `Java` · [BBTools (JGI)](https://jgi.doe.gov/data-and-tools/software-tools/bbtools/) · terms unverified here
  - Historical Rust asset: retired `rsomics-bbduk`; refactor then merge into preprocessing, not an installable product or current stable command.
  - Existing non-C alternatives: —
  - Parallelism: JVM threading
  - SIMD: none significant
  - Quadrant: —
  - GPU-amenable: maybe — k-mer hashing is SIMT-trivial
  - Upstream license: the old survey recorded “proprietary-but-free”; this is not a verified legal classification. Review the actual upstream terms before porting.
  - Portfolio routing: later reference-based decontamination inside `rsomics-fastq-preprocess`, subject to compatibility and representative resource gates.
  - Notes: Reference k-mer matching (phiX, rRNA, host) differs from simple length / quality filtering. Historical `fastbloom` / ntHash choices are asset details, not proof of a current kmer dependency or a performance advantage.

- **`BFC`** — k-mer-spectrum substitution-error corrector for Illumina reads.
  - Reference impl: `C` · [lh3/bfc](https://github.com/lh3/bfc) · `MIT`
  - Historical Rust asset: retired `rsomics-fastq-correct`; refactor then merge into preprocessing, not a current stable correction command.
  - Existing non-C alternatives: `Lighter` ([mourisl/Lighter](https://github.com/mourisl/Lighter), C++, recorded `GPL-3.0` — Bloom-cardinality corrector); `Karect`, `Musket`, `Bloocoo`
  - Parallelism: BFC pthreads over reads; the historical Rust asset used rayon over read batches.
  - SIMD: k-mer roll (ntHash) auto-vectorises; correction-decision path is branchy
  - Quadrant: the historical asset was classified ① (Rust + rayon, ntHash, and a `fastbloom` blocked filter).
  - GPU-amenable: maybe — k-mer counting/lookup is SIMT-trivial; the greedy correction walk is branch-divergent, CPU is the right first target
  - Upstream license: recorded `MIT`; `bfc.c` is the behavior and compatibility reference.
  - Portfolio routing: later correction stage inside `rsomics-fastq-preprocess`, subject to adversarial compatibility and representative hot-path benchmarks.
  - Notes: BFC builds a k-mer coverage histogram, selects a trusted / untrusted threshold from the spectrum, then greedily rewrites untrusted bases along a supported k-mer path. A correction contract must cover threshold selection, the correction walk, `-k`, `-s`, worker behavior, and drop-versus-correct policy; existing small subprocess benches do not establish release readiness.

- **`Trim Galore`** — cutadapt + FastQC wrapper (Perl).
  - Reference impl: `Perl` · [FelixKrueger/TrimGalore](https://github.com/FelixKrueger/TrimGalore) · `GPL-3.0`
  - Existing Rust: none verified
  - Existing Rust kind: `none`
  - Existing non-C alternatives: —
  - Parallelism: Perl shell-level
  - SIMD: none
  - Quadrant: —
  - GPU-amenable: no — orchestration wrapper
  - Upstream license: `GPL-3.0`
  - Portfolio routing: external workflow reference, not a new wrapper crate.
  - Notes: Its cutadapt-plus-FastQC composition is useful context; a shared workflow name does not prove either tool's compatibility.

- **`AdapterRemoval`** — paired-end + ancient-DNA trimmer.
  - Reference impl: `C++` · [MikkelSchubert/adapterremoval](https://github.com/MikkelSchubert/adapterremoval) · `GPL-3.0`
  - Existing Rust: none verified
  - Existing Rust kind: `none`
  - Existing non-C alternatives: —
  - Parallelism: upstream pthreads
  - SIMD: upstream SSE
  - Quadrant: —
  - GPU-amenable: no — same constraints as fastp
  - Upstream license: `GPL-3.0`
  - Portfolio routing: ancient-DNA and paired-read collapse are future preprocessing contracts, not current trim / filter behavior.
  - Notes: Short fragments, adapter overlap, collapse decisions, and resulting sequence / quality semantics need their own fixtures and oracle. No separate public trimmer is approved here.

- **`FastQC`** — per-base QC report generator.
  - Reference impl: `Java` · [s-andrews/FastQC](https://github.com/s-andrews/FastQC) · `GPL-3.0`
  - Existing Rust: [`rsomics-fastq-qc report`](../10-products/sequence-fastq.md#rsomics-fastq-qc); retired `rsomics-fastqc` analyzers and fixtures were refactored rather than revived.
  - Existing non-C alternatives: `falco` (C++ port); `seqkit stats` (Go)
  - Parallelism: `rsomics-fastq-qc` uses multi-file parallelism; the old survey also considered fastqc-rs per-chunk work and RastQC streaming.
  - SIMD: auto-vectorize on quality histograms
  - Quadrant: the old candidate survey recorded ①; current product backend and performance evidence belong to the dossier.
  - GPU-amenable: no — histogram accumulation is memory-bandwidth-bound
  - Upstream license: `GPL-3.0`
  - Portfolio routing: `rsomics-fastq-qc report`; generic counts and length statistics belong to `rsomics-seq stats`.
  - Notes: The product writes FastQC / MultiQC-compatible `fastqc_data.txt`, a status summary, and self-contained HTML for multiple plain, gzip, or BGZF FASTQ inputs. BAM/SAM, Casava grouping, `fast5`, custom limits / adapter / contaminant lists, and FastQC ZIP packaging remain excluded. Preprocessing metrics stay with preprocessing.

- **`MultiQC`** — report aggregator.
  - Reference impl: `Python` · [MultiQC/MultiQC](https://github.com/MultiQC/MultiQC) · `GPL-3.0`
  - Existing Rust: none verified
  - Existing Rust kind: `none`
  - Existing non-C alternatives: —
  - Parallelism: Python serial
  - SIMD: none
  - Quadrant: —
  - GPU-amenable: no — report aggregation
  - Upstream license: `GPL-3.0`
  - Portfolio routing: external report aggregation, not a new rsomics product.
  - Notes: Interoperation depends on the actual input schema of the relevant MultiQC module. FastQC-compatible data and fastp-style JSON are different contracts; arbitrary JSON output is not enough, and not every Rust tool needs a generic MultiQC JSON format.

- **`seqkit`** — Swiss-Army-knife FASTA/FASTQ tool (Go).
  - Reference impl: `Go` · [shenwei356/seqkit](https://github.com/shenwei356/seqkit) · `MIT`
  - Existing Rust: the survey recorded [`faster`](https://github.com/angelovangel/faster) and [`seqtk-rs`](https://crates.io/crates/seqtk-rs) `0.2.0` (seqtk-like subset); current installation details are not verified here.
  - Existing Rust kind: `partial-port`
  - Existing non-C alternatives: `seqkit` itself (Go); `pyfastx` (Python)
  - Parallelism: rayon (faster); single-threaded (seqtk-rs)
  - SIMD: auto-vectorize
  - Quadrant: ①
  - GPU-amenable: no — text-based sequence manipulation
  - Upstream license: `MIT`
  - Portfolio routing: `rsomics-seq`, not a separate seqkit-named public product.
  - Notes: The dossier's five-command first slice is `stats`, `kmers`, `grep`, `convert`, and `validate`. Compatibility and performance are operation-specific; broader transforms remain plans, not a percentage-of-SeqKit parity promise.

- **`seqtk`** — lh3's minimalist sequence toolkit.
  - Reference impl: `C` · [lh3/seqtk](https://github.com/lh3/seqtk) · `MIT`
  - Existing Rust: the survey recorded [`seqtk-rs`](https://crates.io/crates/seqtk-rs) `0.2.0`.
  - Existing Rust kind: `partial-port`
  - Existing non-C alternatives: `seqkit` (Go, superset functionality)
  - Parallelism: single-threaded in the inherited survey
  - SIMD: auto-vectorize
  - Quadrant: ①
  - GPU-amenable: no — text-based sequence manipulation
  - Upstream license: `MIT`
  - Portfolio routing: generic sequence operations belong to `rsomics-seq`.
  - Notes: Upstream overlap is a deduplication lead, not evidence that all seqtk commands exist in the current product or an adoption decision for seqtk-rs.

- **`Dorado`** — ONT's GPU-accelerated neural-network basecaller (successor to Guppy).
  - Reference impl: `C++ / CUDA` · [nanoporetech/dorado](https://github.com/nanoporetech/dorado) · `Apache-2.0`
  - Existing Rust: none verified
  - Existing Rust kind: `none`
  - Existing non-C alternatives: —
  - Parallelism: multi-GPU (C++/CUDA)
  - SIMD: via libtorch / CUDA
  - Quadrant: —
  - GPU-amenable: yes — neural-net basecalling is GPU-native
  - Upstream license: `Apache-2.0`
  - Portfolio routing: external basecalling context, excluded from the FASTQ product surfaces above.
  - Notes: GPU model inference precedes the read-level workflows surveyed here. No Dorado wrapper crate or implemented invocation / normalization workflow is approved or claimed here.

- **`NanoPlot` / `NanoPack2`** — quality visualization and filtering for long-read ONT data.
  - Reference impl: `Python + Rust (chopper)` · [wdecoster/NanoPlot](https://github.com/wdecoster/NanoPlot) · `MIT`
  - Existing Rust: the survey recorded [`chopper`](https://crates.io/crates/chopper) as NanoPack2's Rust read-length / quality filter.
  - Existing Rust kind: `partial-port` (chopper only; NanoPlot itself is Python)
  - Existing non-C alternatives: —
  - Parallelism: Python multiprocessing (NanoPlot)
  - SIMD: auto-vectorize (chopper)
  - Quadrant: ① (chopper)
  - GPU-amenable: no
  - Upstream license: `MIT`
  - Portfolio routing: external filtering / visualization context; no NanoPlot-named rsomics product.
  - Notes: Read-length / quality filtering, generic sequence statistics, and long-read graphics are distinct contracts. The chopper profile is an external comparison lead, not a new Layer B dependency; NanoPlot graphics are not an implemented FASTQ-QC feature.
