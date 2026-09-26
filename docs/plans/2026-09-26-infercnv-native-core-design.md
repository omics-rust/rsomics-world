# Native inferCNV pre-clustering core

## Intent and scope

Implement the real numerical core of the eventual `rsomics-sc cnv` workflow,
using the accepted oracle in
`../../.autopilot/state/infercnv-oracle-accepted-2026-09-26.md`. The user's
priority is a correct native inferCNV workflow, with parallel independent work
and no routine approval pauses. This slice is not the complete workflow or a
publishable substitute for inferCNV.

The bounded input is **already selected and genomically ordered counts at
upstream stage 1**. Implement stages 2, 3, 4, 8, 9, 10, 11, 12, and 14, retaining
identities and reporting the borrowed stage-1 state too. Creation from raw
files, AnnData adapters, persistent state, clustering, HMM, denoising, plots,
and a public CLI remain separate required work. Starting from stage 1 must not
be described as raw-input end-to-end compatibility.

Work belongs in the new local product repository
`/Volumes/KIOXIA/Documents/omics-rust/rsomics-sc`, with real core code and tests.
Set `publish = false`; do not create a public GitHub repository, publish a name,
or add empty modules for the other single-cell operations. Control-plane CI
can test a frozen source snapshot while local compilation is prohibited.

## Selected approach

Use a column-major, checked, dense CNV working matrix and direct triangular
smoothing first. Genomic neighbors within each cell are contiguous in memory.
This is an explicit dense numerical input, not silent densification of a sparse
dataset or a proposed replacement for the product's persisted annotated state.

A rolling/prefix implementation is a later measured optimization: current
evidence establishes the direct formula, not a safe universal error bound for
changed summation order. An R subprocess wrapper would not meet the native
goal; the historical approximation cannot supply the required semantics.

Reuse published `rsomics-common` 0.12.3 for `Result` and `RsomicsError`. Its live
registry version was verified, unyanked, with checksum
`d9a4246711f13a0fdd9398e7eab3a95b991ac555a52c98f6518252c037508401`.
No foundation API changes or Layer B dependencies are needed. This slice has
no binary or user-facing CLI; when the real CLI is implemented it must use
`rsomics-help`, not a private help/error presentation layer.

## Unpublished product-local interface

The library exposes `cnv` for integration tests and later product callers; its
types are not a stable released API or a new shared foundation. Internal
kernels stay private. Use these names consistently:

```rust
pub struct Gene { pub id: String, pub chromosome: String, pub start: u64, pub end: u64 }
pub struct Cell { pub id: String, pub group: String }
pub struct PreparedCounts { state: CnvState }
pub struct CnvState { genes: Vec<Gene>, cells: Vec<Cell>, values: Vec<f64> }
pub enum ReferenceMode {
    AllCells,
    GroupBounds(Vec<String>),
    GroupMean(Vec<String>),
}
pub struct PreprocessConfig {
    pub cutoff: f64,
    pub min_cells_per_gene: usize,
    pub window_length: usize,
    pub max_centered_threshold: f64,
    pub references: ReferenceMode,
    pub max_numeric_bytes: usize,
}
pub enum Stage { Input, Filtered, DepthNormalized, Log2, ReferenceCentered,
    Clamped, Smoothed, CellCentered, ReferenceRecentered, FoldChange }
```

`PreparedCounts::new(genes, cells, column_major_values)` returns the shared
`Result<Self>`. Validate nonempty shape, checked dimension multiplication,
exact value count, unique nonempty gene/cell IDs, nonempty cell groups,
finite nonnegative counts, and positive finite column totals. Gene records
require a nonempty chromosome, `start <= end`, and at least one nonzero
coordinate. Chromosome blocks must be contiguous and coordinates sorted by
`(start,end)` within each block. These are prepared-input requirements, not a
claim to reproduce every permissive upstream file parser.

`CnvState` offers read-only `genes()`, `cells()`, `values()` (column-major),
`gene_count()`, `cell_count()`, and `value(gene, cell)` accessors.
`PreparedCounts::state()` borrows its state. Avoid public mutable buffers.
`Stage::step() -> u8` returns the corresponding upstream checkpoint number.
Gene/cell records support `Clone`, `Debug`, `PartialEq`, and `Eq`; stage supports
`Copy`, `Clone`, `Debug`, `PartialEq`, and `Eq`; checked states support `Debug`.

`run(&PreparedCounts, &PreprocessConfig) -> Result<CnvState>` is the normal
entry. `run_with_observer` additionally accepts
`impl FnMut(Stage, &CnvState) -> Result<()>`; it reports all ten stages in order
and propagates observer errors immediately. Do not retain all stage matrices
by default. Raw input is immutable; returned identities reflect gene filtering.

Reject nonfinite/negative cutoff, zero detection minimum, zero/even smoothing
window, nonfinite/nonpositive clamp threshold, duplicate/empty/missing named
references, and an insufficient numerical workspace budget. `AllCells` is an
explicit proxy-reference choice, never a fallback for a misspelled group.

Before cloning/allocating numerical working buffers, check integer arithmetic
and a conservative bound of `3 * value_count + 4 * gene_count + cell_count`
f64 elements against `max_numeric_bytes`. Keep the implementation within that
bound, using one active matrix, bounded replacement/smoothing storage, and
reusable reduction scratch. This ceiling excludes caller-owned input, string
metadata, allocator overhead, and allocations made by the observer; it is not
a promise about process RSS. Use fallible reservations for large buffers.

## Numerical contract

1. Filter genes by mean over all cells `>= cutoff` and count of strictly
   positive cells `>= min_cells_per_gene`; preserve retained gene order.
   Reject an empty retained set and any zero/nonfinite post-filter cell total.
2. Normalize each cell to the median of the post-filter column totals.
3. Apply `log2(x + 1)`.
4. Subtract per-gene reference means. Bounds mode returns zero inside the
   min/max interval of group means and subtracts the nearest bound outside.
   Mean mode subtracts the equally weighted mean of group means. `AllCells`
   uses all cells as one proxy group.
5. Clamp to the explicit symmetric threshold.
6. Smooth each chromosome and cell independently with weights
   `(window_length + 1)/2 - distance`, truncating at chromosome ends and
   renormalizing retained weights. Copy singleton chromosomes unchanged.
7. Subtract each cell's median across all retained genes, not per chromosome.
8. Repeat reference subtraction using the same declared group policy.
9. Apply `2^x`, not `2^x - 1`.

Stop on nonfinite reduction or transformed results rather than emitting NaN
or infinity. In particular, zero depth after filtering is a deliberate
fail-loud divergence from the upstream NaN case already documented. Do not
silently repair values or change the chosen reference policy.

## Evidence and tests

Copy only accepted input/identity/expression TSVs needed by the tests, plus a
fixture manifest and provenance README, from run `36216764707` into product
test fixtures. Do not vendor R, RDS objects, or the entire upstream package.
The test fixture reader is test-only; no unsupported import command is exposed.

For each of the four profiles, construct prepared counts from the stage-1 TSV
and metadata, call the complete native pipeline once, and compare every stage
and identity to its corresponding saved upstream checkpoint. Numerical gate:
`abs(actual - expected) <= 1e-12 + 1e-12 * abs(expected)` for this bounded
fixture. Do not widen the gate to make a failing implementation pass. Report
observed maxima separately; the threshold is not a universal accuracy claim.

Add independent hand witnesses for `[1,3]` versus `[3,1]`, no-reference
reciprocal square-root signals, unequal reference-group weighting, post-filter
depth, cross-chromosome separation, global cell median, and the two-gene
window-101 result `[500/101,510/101]`. Explicitly test clipping below/at/above
both thresholds. Test singleton, 100/101/102-gene windows using constant and
linear input properties without calling the production kernel for expected
values. These supplement, but do not masquerade as additional executed R
goldens. Test invalid inputs/configuration, zero depth after filtering, budget
overflow/insufficiency, observer failure, stage sequence, and raw immutability.

Use Rust 1.91, edition 2024, rare invariant-only source comments, no unsafe code,
and no production unwrap except statically obvious invariants. Acceptance needs
tests in debug/release, formatting, strict Clippy, dependency/package metadata,
and native execution on Linux/macOS x86_64/aarch64. First capture a real red
test run before implementation. All local builds remain prohibited while the
Mac boot APFS is above 80%; snapshot compilation happens on hosted runners.

The eventual product still needs raw-state integration, downstream workflow
oracles, representative performance/RSS comparisons, unified CLI, and a fresh
release review. This core is a tested implementation slice, not that release.
