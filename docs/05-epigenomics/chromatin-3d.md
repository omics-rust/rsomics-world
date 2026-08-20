# 3D chromatin (Hi-C, Micro-C, HiChIP)

> From read pairs to contact matrices, normalised maps, and topological
> feature calls (TADs, loops, compartments).

## Scope

The full Hi-C analysis stack:

1. **Read pair extraction** — pairtools / distiller.
2. **Matrix storage and manipulation** — cooler (.cool / .mcool),
   Juicer (.hic).
3. **Pipelines** — HiC-Pro, distiller, HiCExplorer, Juicer.
4. **Feature calling** — loops (chromosight, mustache, hicFindPeaks),
   TADs / compartments (cooltools, HiCExplorer), differential analysis
   (FAN-C / fanc-tools).

scHi-C lives in module 04 (spatial / single-cell).

## Design notes

- The Hi-C stack is unusually well-specified: the `pairs` text format
  is a 4DN standard; the `.cool` format is HDF5 with a tiny schema;
  the `.hic` format is open. Rust IO crates for these are missing but
  small — a focused `cooler-rs` would unlock the rest.
- The bottleneck in real-world Hi-C is the **read-pair extraction**
  step (pairtools parse + dedup) on hundreds of millions of read
  pairs. Pure-Rust pairtools — `noodles-bam` reader + dedup hash +
  parallel sort — could be 5–10× faster than the Python original.
- Loop / TAD callers are mostly small image-processing-on-Hi-C-matrix
  algorithms (chromosight uses pattern matching; mustache uses
  scale-space blob detection). `ndarray` + `imageproc` style Rust
  crates cover the math.
- Juicer (Java) is fast but closed-feeling; the open ecosystem (cooler
  + cooltools) is what the field is converging on. Rust effort should
  follow that trajectory.

## TODO

- [ ] **`cooler`** — HDF5-backed Hi-C contact matrix format and library.
  - Reference impl: `Python` · [open2c/cooler](https://github.com/open2c/cooler) · `BSD-3-Clause`
  - Existing Rust: none verified. `hdf5-metno` provides the IO primitives
  - Existing Rust kind: `none`
  - Existing non-C alternatives: —
  - Parallelism: Python multiprocessing
  - SIMD: BLAS via numpy
  - Quadrant: —
  - GPU-amenable: maybe — sparse matrix ops are GPU-friendly
  - Upstream license: `BSD-3-Clause`
  - Priority: `P0`
  - Layer: adopt inside the first concrete 3D-chromatin product
  - Consumes primitives: `hdf5-metno`, `ndarray`, `nalgebra-sparse`
  - Notes: A concrete 3D-chromatin product would first need private `.cool` / `.mcool` I/O because the rest of the workflow depends on it. The small schema does not make it a standalone foundation. The initial path is Quadrant ② through HDF5 FFI; reassess public extraction only after a second product consumer exists.

- [ ] **`cooltools`** — analysis on top of cooler (eigenvector compartments, insulation, dot calling).
  - Reference impl: `Python` · [open2c/cooltools](https://github.com/open2c/cooltools) · `MIT`
  - Existing Rust: none verified
  - Existing Rust kind: `none`
  - Existing non-C alternatives: —
  - Parallelism: Python multiprocessing
  - SIMD: BLAS
  - Quadrant: —
  - GPU-amenable: maybe — dense linear algebra on contact matrices
  - Upstream license: `MIT`
  - Priority: `P1`
  - Layer: survey candidate; no accepted product boundary
  - Consumes primitives: product-private Cooler I/O, `ndarray-linalg`, `nalgebra-sparse`, future `rsomics-stats`
  - Notes: Keep Cooler I/O and the selected cooltools analyses together in the first concrete product. Insulation, compartment eigendecomposition, and expected-decay routines do not establish separate product or foundation boundaries.

- [ ] **`HiC-Pro`** — established Hi-C processing pipeline (mapping + filtering + matrix construction).
  - Reference impl: `Python / Bash` · [nservant/HiC-Pro](https://github.com/nservant/HiC-Pro) · `BSD-3-Clause`
  - Existing Rust: none verified
  - Existing Rust kind: `none`
  - Existing non-C alternatives: distiller (Nextflow)
  - Parallelism: shell-level
  - SIMD: inherited from invoked binaries
  - Quadrant: —
  - GPU-amenable: no — orchestration layer
  - Upstream license: `BSD-3-Clause`
  - Priority: `P1`
  - Layer: —
  - Consumes primitives: future rsomics-* binaries
  - Notes: Pipeline orchestration is out of scope. Revisit component implementation only after a coherent 3D-chromatin product boundary enters the allowlist.

- [ ] **`HiCExplorer`** — Hi-C visualisation and analysis suite.
  - Reference impl: `Python` · [deeptools/HiCExplorer](https://github.com/deeptools/HiCExplorer) · `GPL-3.0`
  - Existing Rust: none verified
  - Existing Rust kind: `none`
  - Existing non-C alternatives: —
  - Parallelism: Python multiprocessing
  - SIMD: BLAS
  - Quadrant: —
  - GPU-amenable: maybe — visualisation rendering is GPU-friendly
  - Upstream license: `GPL-3.0`
  - Priority: `P2`
  - Layer: product-private analysis operation if a 3D-chromatin product is accepted
  - Consumes primitives: product-private Cooler I/O, `ndarray-linalg`
  - Notes: User-facing tool; not a priority for Rust rewrite.

- [ ] **`Juicer` / `JuicerTools`** — Aiden lab pipeline + `.hic` toolkit.
  - Reference impl: `Java + C++` · [aidenlab/juicer](https://github.com/aidenlab/juicer) · `MIT`
  - Existing Rust: none verified. `hicstraw`-style readers exist but only in Python / C++
  - Existing Rust kind: `none`
  - Existing non-C alternatives: cooler / cooltools is the open counterpart
  - Parallelism: JVM threading + C++ inner loops
  - SIMD: BLAS
  - Quadrant: —
  - GPU-amenable: maybe — dense linear algebra
  - Upstream license: `MIT`
  - Priority: `P2`
  - Layer: product-private legacy-format compatibility if a 3D-chromatin product is accepted
  - Consumes primitives: `noodles-bam`, a product-private `.hic` parser
  - Notes: A Rust `.hic` reader/writer would be useful for legacy compatibility, but the field is converging on `.cool`.

- [ ] **`pairtools`** — extract `.pairs` from SAM/BAM, dedup, filter.
  - Reference impl: `Python / Cython` · [open2c/pairtools](https://github.com/open2c/pairtools) · `MIT`
  - Existing Rust: none verified
  - Existing Rust kind: `none`
  - Existing non-C alternatives: —
  - Parallelism: Python multiprocessing + Cython
  - SIMD: none
  - Quadrant: —
  - GPU-amenable: no — dedup hash + sort, latency-bound
  - Upstream license: `MIT`
  - Priority: `P0`
  - Layer: survey candidate; no accepted product boundary
  - Consumes primitives: `noodles-bam`, `rsomics-intervals`, `rayon`, `xxhash`-style hashing
  - Notes: **High-impact Rust target.** Read-pair extraction is the pipeline bottleneck. `noodles-bam` reader + parallel dedup (xxhash3 → sharded hashmap) + parallel sort → `.pairs.gz` output. Match pairtools' format exactly so downstream cooler can read it.

- [ ] **`distiller`** — Nextflow Hi-C pipeline maintained by Open2C.
  - Reference impl: `Nextflow / Python` · [open2c/distiller-nf](https://github.com/open2c/distiller-nf) · `MIT`
  - Existing Rust: none verified
  - Existing Rust kind: `none`
  - Existing non-C alternatives: HiC-Pro
  - Parallelism: Nextflow workflow engine
  - SIMD: inherited
  - Quadrant: —
  - GPU-amenable: no — orchestration layer
  - Upstream license: `MIT`
  - Priority: `P2`
  - Layer: —
  - Consumes primitives: future rsomics-* binaries
  - Notes: Pipeline orchestration; out of scope. Components above serve as drop-in replacements.

- [ ] **`hicFindPeaks`** (HOMER) — Hi-C loop caller within HOMER.
  - Reference impl: `Perl / C++` · part of HOMER · unspecified
  - Existing Rust: none verified
  - Existing Rust kind: `none`
  - Existing non-C alternatives: —
  - Parallelism: upstream Perl serial
  - SIMD: none
  - Quadrant: —
  - GPU-amenable: no — niche, no upside
  - Upstream license: unspecified open source
  - Priority: `P2`
  - Layer: —
  - Consumes primitives: —
  - Notes: Niche; users now prefer chromosight or mustache. Listed for completeness.

- [ ] **`FAN-C` / `fanc-tools`** — multi-resolution Hi-C analysis suite.
  - Reference impl: `Python` · [vaquerizaslab/fanc](https://github.com/vaquerizaslab/fanc) · `GPL-3.0`
  - Existing Rust: none verified
  - Existing Rust kind: `none`
  - Existing non-C alternatives: —
  - Parallelism: Python multiprocessing
  - SIMD: BLAS
  - Quadrant: —
  - GPU-amenable: maybe — same as cooltools
  - Upstream license: `GPL-3.0`
  - Priority: `P2`
  - Layer: product-private analysis operation if a 3D-chromatin product is accepted
  - Consumes primitives: product-private Cooler I/O, `ndarray-linalg`
  - Notes: Niche; useful for differential Hi-C analysis. Not a porting target.

- [ ] **`chromosight`** — pattern-matching loop / stripe caller.
  - Reference impl: `Python` · [koszullab/chromosight](https://github.com/koszullab/chromosight) · `MIT`
  - Existing Rust: none verified
  - Existing Rust kind: `none`
  - Existing non-C alternatives: —
  - Parallelism: Python multiprocessing
  - SIMD: BLAS
  - Quadrant: —
  - GPU-amenable: yes — 2D convolution / template matching is GPU-trivial
  - Upstream license: `MIT`
  - Priority: `P1`
  - Layer: product-private loop-calling operation if a 3D-chromatin product is accepted
  - Consumes primitives: product-private Cooler I/O, `ndarray`, FFT crate, `imageproc`-style primitives
  - Notes: Algorithm is a 2D convolution / template matching on the contact map. `ndarray` + an FFT crate covers it; small focused Rust port worth ~100× speedups on dense matrices.

- [ ] **`mustache`** — scale-space chromatin loop caller.
  - Reference impl: `Python` · [ay-lab/mustache](https://github.com/ay-lab/mustache) · `MIT`
  - Existing Rust: none verified
  - Existing Rust kind: `none`
  - Existing non-C alternatives: —
  - Parallelism: Python multiprocessing
  - SIMD: BLAS
  - Quadrant: —
  - GPU-amenable: yes — Gaussian pyramids are GPU-friendly
  - Upstream license: `MIT`
  - Priority: `P1`
  - Layer: product-private loop-calling operation if a 3D-chromatin product is accepted
  - Consumes primitives: product-private Cooler I/O, `ndarray`, Gaussian-pyramid Rust crate
  - Notes: Scale-space blob detection on the contact matrix. Like chromosight, well suited to `ndarray` + Gaussian-pyramid Rust crates.
