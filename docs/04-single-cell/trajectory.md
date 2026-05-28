# Trajectory inference and RNA velocity

> Pseudotime, lineage, and dynamical-modelling methods for inferring
> cell-state transitions from single-cell data.

## Scope

- Pseudotime / lineage tools that operate on a low-dimensional
  embedding (Monocle3, PAGA, Slingshot).
- RNA velocity methods that use spliced / unspliced ratios to infer
  derivatives (velocyto, scVelo).
- Aggregator frameworks (dynverse) and benchmarking suites.

Static clustering and neighbor graphs are in
[`analysis-core.md`](analysis-core.md).

## Design notes

- This is one of the most Python / R-centric corners of single-cell.
  Monocle3 (R), Slingshot (R), PAGA (Python/scanpy), scVelo (Python),
  dynverse (R). Almost nothing Rust.
- The numerical primitives needed are mostly already covered by
  `petgraph` (PAGA-style graph contraction), `linfa` (principal curves
  for Slingshot), and `ndarray-stats` (dynamic-EM for scVelo). What is
  missing is the wrapping logic and an `AnnData` ↔ Rust round-trip.
- scVelo's dynamical model uses ODE-likelihood fitting per gene — a
  good `rayon` parallelisation target if rewritten. `candle` or `burn`
  are not strictly needed.
- Pragmatic ordering: pseudotime tools wrap via `extendr` / pyo3 first;
  RNA velocity is the most promising native-Rust target because the
  preprocessing (counting spliced vs. unspliced) is heavy IO that
  `noodles-bam` does well.

## TODO

- [ ] **`Monocle3`** — graph-abstraction-based pseudotime.
  - Reference impl: `R` · [cole-trapnell-lab/monocle3](https://github.com/cole-trapnell-lab/monocle3) · `MIT`
  - Existing Rust: none verified
  - Existing Rust kind: `none`
  - Existing non-C alternatives: —
  - Parallelism: R BiocParallel + C++ inner loops
  - SIMD: limited
  - Quadrant: —
  - GPU-amenable: maybe — principal-graph learner has dense linear-algebra subroutines
  - Upstream license: `MIT`
  - Priority: `P1`
  - Layer: `subcommand-of-rsomics-sc` (trajectory subcommand inside the umbrella)
  - Consumes primitives: `anndata-rs`, `linfa`, `petgraph`, `extendr`-bridge, future `rsomics-stats`
  - Notes: Builds on UMAP + a custom graph-clustering principal-graph learner. The principal-graph step is the algorithmically novel piece; Rust port is feasible with `petgraph` + `linfa` but is non-trivial. First step: wrap via `extendr` and ship a Rust AnnData ↔ Monocle3 bridge.

- [ ] **`PAGA`** — Partition-based graph abstraction (Scanpy).
  - Reference impl: `Python` · [scverse/scanpy](https://scanpy.readthedocs.io/en/stable/api/scanpy.tl.paga.html) · `BSD-3-Clause`
  - Existing Rust: none verified
  - Existing Rust kind: `none`
  - Existing non-C alternatives: —
  - Parallelism: trivial parallel over cluster pairs
  - SIMD: none
  - Quadrant: —
  - GPU-amenable: no — graph aggregation latency-bound
  - Upstream license: `BSD-3-Clause`
  - Priority: `P1`
  - Layer: `subcommand-of-rsomics-sc`
  - Consumes primitives: `petgraph`, `statrs`, future `rsomics-stats`
  - Notes: Algorithmically tiny — given a clustering and a neighbor graph, aggregate edges between clusters and test connectivity. `petgraph` + statistical-test crate covers it. Good early Rust rewrite candidate inside `rsomics-sc`.

- [ ] **`Slingshot`** — principal-curves pseudotime over a clustering.
  - Reference impl: `R` · [kstreet13/slingshot](https://github.com/kstreet13/slingshot) · `Artistic-2.0`
  - Existing Rust: none verified. Python port [`pyslingshot`](https://github.com/mossjacob/pyslingshot) exists
  - Existing Rust kind: `none`
  - Existing non-C alternatives: pyslingshot (Python)
  - Parallelism: R BiocParallel
  - SIMD: none
  - Quadrant: —
  - GPU-amenable: no — principal-curve fitting is per-lineage and small
  - Upstream license: `Artistic-2.0`
  - Priority: `P1`
  - Layer: `subcommand-of-rsomics-sc`
  - Consumes primitives: `linfa` (principal curves primitive needed), `petgraph` (MST), future `rsomics-stats`
  - Notes: Lineage tree from cluster MST + smoothed principal curves. Algorithm is compact; if `linfa` gets a principal-curve primitive this rewrites into a few hundred lines.

- [ ] **`scVelo`** — dynamical RNA velocity model with latent time.
  - Reference impl: `Python` · [theislab/scvelo](https://github.com/theislab/scvelo) · `BSD-3-Clause`
  - Existing Rust: none verified
  - Existing Rust kind: `none`
  - Existing non-C alternatives: —
  - Parallelism: Python multiprocessing
  - SIMD: none
  - Quadrant: —
  - GPU-amenable: maybe — per-gene ODE EM fits parallelise; could be GPU SIMT
  - Upstream license: `BSD-3-Clause`
  - Priority: `P1`
  - Layer: `B` (tool — `rsomics-velocity`)
  - Consumes primitives: `anndata-rs`, `ndarray-stats`, `rayon`, future `rsomics-stats`
  - Notes: scVelo only supports Python by design — the EM over the splicing-kinetics ODE per gene is the bottleneck and an obvious `rayon` win. Pair with a Rust spliced/unspliced counter (see `velocyto` below). Long-term scVelo replacement is a strong Phase-3 target.

- [ ] **`velocyto`** — per-cell spliced / unspliced counting from BAM.
  - Reference impl: `Python + C++` · [velocyto-team/velocyto.py](https://github.com/velocyto-team/velocyto.py) · `BSD-2-Clause`
  - Existing Rust: none direct, but [`alevin-fry`](https://github.com/COMBINE-lab/alevin-fry) USA-mode quantification already emits spliced/unspliced/ambiguous counts per cell — the modern recommendation
  - Existing Rust kind: `none` (for velocyto itself; alevin-fry covers the use case differently)
  - Existing non-C alternatives: alevin-fry USA mode (Rust, `rust-native`)
  - Parallelism: upstream Python multiprocessing
  - SIMD: none
  - Quadrant: —
  - GPU-amenable: no — BAM IO is the bottleneck
  - Upstream license: `BSD-2-Clause`
  - Priority: `P1`
  - Layer: `subcommand-of-rsomics-velocity` (a `count` subcommand that wraps alevin-fry USA mode)
  - Consumes primitives: `alevin-fry`, `noodles-bam`, `anndata-rs`, `polars`
  - Notes: Mark `velocyto` itself as legacy; the rsomics pipeline routes through alevin-fry USA. Add a thin `polars` exporter so scVelo / a future Rust velocity tool can consume it directly.

- [ ] **`dynverse`** — meta-framework benchmarking 70+ trajectory methods.
  - Reference impl: `R` · [dynverse](https://github.com/dynverse/dynverse) · `MIT`
  - Existing Rust: none verified
  - Existing Rust kind: `none`
  - Existing non-C alternatives: —
  - Parallelism: R BiocParallel
  - SIMD: none
  - Quadrant: —
  - GPU-amenable: no — benchmarking harness
  - Upstream license: `MIT`
  - Priority: `P2`
  - Layer: —
  - Consumes primitives: —
  - Notes: dynverse's value is the benchmarking harness, not the methods. If rsomics ships its own trajectory tools, a small `rsomics-bench-trajectory` crate replicating dynverse's metric suite (TI evaluation) is a useful Phase-4 deliverable.

- [ ] **`CellRank 2`** — RNA-velocity + multi-kernel cell-fate mapping.
  - Reference impl: `Python` · [theislab/cellrank](https://github.com/theislab/cellrank) · `BSD-3-Clause`
  - Existing Rust: none verified
  - Existing Rust kind: `none`
  - Existing non-C alternatives: —
  - Parallelism: Python + scipy sparse
  - SIMD: via numpy BLAS
  - Quadrant: —
  - GPU-amenable: maybe — Markov chain operations on transition matrix
  - Upstream license: `BSD-3-Clause`
  - Priority: `P1`
  - Layer: `B` (tool — `rsomics-cellrank`)
  - Consumes primitives: `anndata-rs`, `ndarray`, `rayon`, `petgraph`, future `rsomics-stats`
  - Notes: Canonical trajectory-fate mapper combining RNA velocity, pseudotime, CytoTRACE, and custom kernels into a Markov transition matrix. Absorption probabilities reveal terminal fates. Nature Methods 2021+2024 updates. BSD-3-Clause allows source reading.

- [ ] **`CytoTRACE` / `CytoTRACE 2`** — developmental potential / stemness scoring.
  - Reference impl: `R / Python` · Gulati et al., Science 2020 · `custom academic`
  - Existing Rust: none verified
  - Existing Rust kind: `none`
  - Existing non-C alternatives: —
  - Parallelism: minimal
  - SIMD: none
  - Quadrant: —
  - GPU-amenable: no — per-cell gene-count correlation scoring
  - Upstream license: varies (CytoTRACE 2 license requires verification)
  - Priority: `P2`
  - Layer: `subcommand-of-rsomics-cellrank` (stemness-scoring kernel)
  - Consumes primitives: `anndata-rs`, `ndarray`, `rayon`
  - Notes: Core: correlate gene counts with gene expression diversity per cell → stemness score. v2 adds a neural-net classifier. Simple scoring algorithm, good Rust target.

- [ ] **`Palantir`** — Markov-chain pseudotime with branch fate probabilities.
  - Reference impl: `Python` · [dpeerlab/Palantir](https://github.com/dpeerlab/Palantir) · `BSD-3-Clause`
  - Existing Rust: none verified
  - Existing Rust kind: `none`
  - Existing non-C alternatives: CellRank (Python)
  - Parallelism: Python + scipy
  - SIMD: via numpy BLAS
  - Quadrant: —
  - GPU-amenable: maybe — diffusion-component eigendecomposition
  - Upstream license: `BSD-3-Clause`
  - Priority: `P2`
  - Layer: `subcommand-of-rsomics-cellrank` (diffusion-pseudotime kernel)
  - Consumes primitives: `anndata-rs`, `ndarray`, `rayon`
  - Notes: Diffusion maps + Markov chain for pseudotime and branch probabilities. Complementary to CellRank (CellRank can use Palantir as a kernel). BSD-3 allows source reading.
