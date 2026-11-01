# Peak calling

> Identification of enriched genomic regions from ChIP-seq, ATAC-seq,
> CUT&RUN, and CUT&Tag data.

Status: upstream operation survey. The [peak product dossier](../10-products/peak.md)
and [namespace allowlist](../00-overview/registry-reset-keep.txt) define the
accepted public boundary. Upstream language, backend, version, and license
observations below are inherited survey records, not fresh upstream audits.

## Scope

Algorithms that take aligned reads (BAM / BED) and emit a list of
enriched intervals plus statistics. Downstream integration with
chromatin-state segmentation (ChromHMM, Segway) and full pipelines
(nf-core/chipseq) is in [`chip-atac-pipelines.md`](chip-atac-pipelines.md).
Hi-C "peak"/loop callers (chromosight, mustache, HiCCUPS) live in
[`chromatin-3d.md`](chromatin-3d.md).

`rsomics-peak` owns one chromatin-enrichment workflow, not a separate product
for every upstream caller. Its current private first slice is:

| Operation | Current contract |
|---|---|
| `call` | narrow MACS-compatible calling from BAM / BAMPE treatments and optional pooled controls |
| `call-sparse` | sparse positive-signal calling with an explicit compatibility profile; no public default selected |
| `annotate` | transcript-level GTF / GFF3 annotation with stable peak identity |
| `count` | identity-preserving multi-sample BAM counts in alignment units, not paired-fragment counts |

Broad peaks, FRAG barcode selection, track calling, HMMRATAC, model-only, and
refinement operations remain later dossier slices, not advertised commands.
HOMER, SICER, and the other surveyed models are not implicitly implemented
by a MACS-compatible caller.

No public release is established by this survey. The SEACR sparse-default
policy and registry-authorization holds remain separate. The
[current native state](../../.autopilot/state/peak-native-first-four-2026-11-01.md)
and [real-input gate](../../.autopilot/state/peak-real-performance-2026-11-01.md)
own the exact evidence: no whole-graph parallel speedup or exact aggregate
tree-memory advantage is established, and prior failed runs remain failed.
Historical annotation goldens are not current-default authority; the dossier
identifies the independently qualified source profiles.

## Design notes

- Compatibility is tied to a pinned caller, model, options, input, and output
  set. BED / narrowPeak / broadPeak formats do not by themselves establish
  identical calls or unchanged downstream behavior.
- Fragment reconstruction, extension, duplicate handling, local background,
  Poisson tails, q-values, broad-region rules, and sparse thresholds are
  caller-specific policies. Coverage construction is shared inside the product
  where the contracts agree; it does not justify a public coverage crate.
- `rsomics-bamio` supplies validated alignment readers and `rsomics-intervals`
  supplies checked geometry. `rsomics-common` supplies errors, execution
  reports, aliases, and transactions; `rsomics-help` supplies CLI help.
  Coverage indexes, transcript / strand models, MACS pileup and statistical
  policy, and SEACR thresholds stay private. Any shared-foundation expansion
  still needs two concrete consumers of the same policy-free contract.
- Potential chromosome or sample parallelism is not a measured whole-workflow
  speedup. Likewise, Go / Java / Cython / Rust implementation language alone
  establishes neither compatibility nor a resource-use advantage.
- SEACR 1.3 compatibility includes source quirks in first / final records,
  threshold populations, normalization, control exclusion, and merge gaps.
  The private legacy and corrected profiles remain distinct; corrected-view
  downstream checks do not establish end-to-end SEACR equivalence or choose
  the public default.

## Upstream operation survey

- **`MACS2`** — model-based ChIP-seq peak caller (legacy).
  - Reference impl: `Python / C` · [macs3-project/MACS](https://github.com/macs3-project/MACS) · `BSD-3-Clause`
  - Existing Rust: none verified
  - Existing Rust kind: `none`
  - Existing non-C alternatives: GoPeaks (Go), epic2 (Cython)
  - Parallelism: upstream Python multiprocessing
  - SIMD: none
  - Quadrant: —
  - GPU-amenable: maybe — coverage pileup is memory-bandwidth-bound
  - Upstream license: `BSD-3-Clause`
  - Portfolio routing: historical compatibility reference within the peak workflow, not a separate public crate or an implemented MACS2 flag.
  - Notes: Retained for reproducibility of older studies. The current first slice uses the dossier's pinned MACS3 profile; MACS2 parity is not inferred from related command names or formats.

- **`MACS3`** — MACS peak-calling family with ATAC-seq, scATAC, and broad-peak operations.
  - Reference impl: `Python / Cython` · [macs3-project/MACS](https://github.com/macs3-project/MACS) · `BSD-3-Clause`
  - Existing Rust: the private `rsomics-peak` first slice above; retired `rsomics-macs` is an implementation asset, not an installable product.
  - Existing non-C alternatives: GoPeaks (Go), epic2 (Cython)
  - Parallelism: worker behavior and whole-workflow evidence belong to the current dossier and gate, not this old chromosome-parallelism proposal.
  - SIMD: auto-vectorize on pileup loops
  - Quadrant: —
  - GPU-amenable: maybe — coverage pileup SIMT-trivial; the peak-shape model less so
  - Upstream license: `BSD-3-Clause`
  - Portfolio routing: `rsomics-peak call`; broader MACS operations follow the dossier's explicit product-operation routing and later-slice gates.
  - Historical source asset: `rsomics-macs` at `f914cdcad4c1ef63ac9930d13257518fd23531e3`, refactor then merge selected single-end no-control algorithms, model, goldens, and measurements.
  - Notes: The dossier distinguishes its recorded MACS3 3.1.0 primary profile from historical 3.0.4 evidence. Treatment pooling, controls, model / no-model paths, duplicates, cutoffs, and output sets are explicit contracts; neither broad-peak parity nor a universal default byte-identity claim follows from the narrow slice.

- **`HOMER findPeaks`** — tag-directory-based peak caller with GRO-seq / 4C / etc. modes.
  - Reference impl: `Perl / C++` · [HOMER](http://homer.ucsd.edu/homer/) · terms unverified here
  - Existing Rust: none verified
  - Existing Rust kind: `none`
  - Existing non-C alternatives: —
  - Parallelism: upstream Perl serial
  - SIMD: none
  - Quadrant: —
  - GPU-amenable: maybe — coverage pileup family
  - Upstream license: the old survey said “unspecified open source”; that is not a verified reuse license.
  - Portfolio routing: surveyed peak-workflow model, not an implemented operation or a new HOMER / tag-directory crate plan.
  - Notes: The tag-directory contract matters to downstream HOMER tools; matching only peak BED rows would not establish compatibility. Actual upstream terms and behavior require review before any implementation is selected.

- **`SEACR`** — Sparse Enrichment Analysis for CUT&RUN.
  - Reference impl: `R / Bash` · [FredHutch/SEACR](https://github.com/FredHutch/SEACR) · see dossier's pinned license record
  - Existing Rust: private `rsomics-peak call-sparse` with a mandatory explicit profile; no public sparse default is selected.
  - Existing non-C alternatives: —
  - Parallelism: upstream R serial + shell
  - SIMD: none
  - Quadrant: —
  - GPU-amenable: no — small algorithm
  - Upstream license: the dossier records `GPL-2.0` for the pinned source and corrects the historical `GPL-3.0` label. This survey performs no new license audit or reuse authorization.
  - Portfolio routing: `rsomics-peak call-sparse`, separate from MACS statistical policy.
  - Historical source asset: `rsomics-seacr` at `d6ed2f0890989fa2de2007a1763bed3d51e365b2`; selected threshold algorithms and the retained input pair are assets, not a current six-mode receipt from the skippable old harness.
  - Notes: Contiguous sparse-signal blocks, control-derived versus numeric fractions, normalization, and stringent / relaxed thresholds remain distinct choices. Preserve the pending default-policy decision; source omissions and corrected handling must not be silently interchanged. Historical startup-heavy comparisons do not prove a current hot-path advantage.

- **`SICER` / `SICER2`** — spatial-clustering peak caller for broad histone marks.
  - Reference impl: `Python / C` · [zanglab/SICER2](https://github.com/zanglab/SICER2) · `MIT`
  - Existing Rust: none verified
  - Existing Rust kind: `none`
  - Existing non-C alternatives: `epic2` (Cython)
  - Parallelism: upstream Python multiprocessing
  - SIMD: none
  - Quadrant: —
  - GPU-amenable: maybe — clustering parallelises
  - Upstream license: `MIT`
  - Portfolio routing: surveyed broad-histone model, not an implemented MACS mode or a new crate queue.
  - Notes: Spatial clustering is not interchangeable with MACS broad-region policy. The old survey quoted a 30× epic2 speedup without input, host, or version provenance; retain it only as a source-research lead, not a current rsomics performance claim.

- **`PeakSeq`** — original ENCODE-era ChIP-seq peak caller.
  - Reference impl: `C / Perl` · [gersteinlab/PeakSeq](https://github.com/gersteinlab/PeakSeq) · recorded `GPL-2`
  - Existing Rust: none verified
  - Existing Rust kind: `none`
  - Existing non-C alternatives: —
  - Parallelism: upstream serial
  - SIMD: none
  - Quadrant: —
  - GPU-amenable: no — historical
  - Upstream license: `GPL-2`
  - Portfolio routing: historical reference for ENCODE reproducibility, not a porting target.
  - Notes: The old survey recorded a last update of 2023-05; this is not a current activity check or an active aliveness-policy gate.

- **`GoPeaks`** — Go peak caller for CUT&Tag.
  - Reference impl: `Go` · [maxsonBraunLab/gopeaks](https://github.com/maxsonBraunLab/gopeaks) · `MIT`
  - Existing Rust: none verified
  - Existing Rust kind: `none`
  - Existing non-C alternatives: GoPeaks itself (Go)
  - Parallelism: Go goroutines
  - SIMD: limited
  - Quadrant: —
  - GPU-amenable: no — same family as MACS
  - Upstream license: `MIT`
  - Portfolio routing: external caller and comparison context, not a new public product.
  - Notes: Its CUT&Tag model and output contract need independent comparison; sharing the peak-workflow domain does not establish equivalence to MACS or SEACR.

- **`GEM`** — Java peak caller with motif-aware refinement.
  - Reference impl: `Java` · [gem](http://groups.csail.mit.edu/cgs/gem/) · `GPL-3.0`
  - Existing Rust: none verified
  - Existing Rust kind: `none`
  - Existing non-C alternatives: GEM itself (Java)
  - Parallelism: JVM
  - SIMD: none
  - Quadrant: —
  - GPU-amenable: maybe — motif scanning is SIMT-friendly
  - Upstream license: `GPL-3.0`
  - Portfolio routing: external motif-aware caller reference, not a porting target.
  - Notes: Motif-coupled peak refinement is a distinct model, not an implied feature of the current narrow-calling slice.
