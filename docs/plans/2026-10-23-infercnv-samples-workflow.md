# Functioning samples/Ward workflow

Status2026-10-26: integrated correctness accepted in actual native
green20/run39820518598 on all four platform classes; product f8a10a0 is clean.
Each debug/release profile passes114 tests and canonical1/4-worker equality.
Performance, larger cohorts and remaining inferCNV operations stay open.

Implement a real unpublished rsomics-sc slice, not another public micro-crate:
strict raw count/position/annotation TSV (plain or gzip) to ordered typed input,
existing preprocessing through step14, temporary reference-gene filtering,
grouped or pooled Ward.D2 clustering, and one transactional JSON report.
The mandatory CLI entry is rsomics-help; common owns errors, worker-count
arguments, result envelopes, alias protection and transactional output.

The user authorized routine implementation choices and requested workflow-first
execution, useful parallelism and less exhaustive per-component testing.
Keep existing evidence, use a few meaningful integrated checks and one overall
review. Do not start new helper-specific oracle/RED/review campaigns.

## Product contract

- `rsomics-sc cnv samples` is the only candidate command. No HMM, denoising,
  plots or unfinished single-cell operations are advertised.
- Keep CreatedInput alive so the stage14 state uses its actual ordered group
  maps; never reconstruct maps from sorted cell labels.
- Observation groups precede requested-order references. Pooled observations
  concatenate observation maps in their existing order. Reference-only inputs
  omit an empty generated observation group.
- At most two members bypass Ward without bypassing group/identity validation.
  Larger groups retain input-local leaves and output global members in tree
  order. The report states zero-based global/local indices and signed R merges.
- Positive filter enables the source's fixed0.8 threshold; inactive NULL,
  active empty outliers and zero selected genes remain distinct. Selection
  does not mutate expression, genes or maps.
- Pooled references named all_observations fail explicitly, the user-selected
  known incompatibility rather than the upstream duplicated-observation bug.
- Aggregate the additional requested numeric/index buffers in samples ownership;
  the CLI's per-stage limit does not claim to bound metadata, retained earlier
  stages, thread stacks, allocator capacity or RSS.
- Parallelize independent condensed-distance chunks with scoped workers;
  preserve the existing even/odd accumulation inside each pair. Do not claim
  a speed advantage from thread count alone.
- One named JSON report includes identities, input maps, filtering and trees;
  optional expression carries the original column-major step14 values. Use
  common's transactional writer and reject aliases to all three inputs.
  Stdout report and --json result envelope are mutually exclusive.

## Verification and next work

The original standalone filter is frozen in green17/run39815910744. It is not
integrated workflow evidence. The integrated candidate retains those11 filter
contracts and52 original route/152 computed-tree comparisons, adds8 group
contracts, one serial/parallel distance check and actual CLI tests. One
canonical raw CLI case exercises8508 retained genes/184 cells, originalR
selection counts6187/2321, all three pooled groups and identical1/4-worker
reports. This is not an assertion of exact current-Rust-to-R tree heights.

Run the existing native candidate workflow in debug/release on Linux/macOS
x86_64/aarch64 with the existing original witness validators, formatting and
strict Clippy. The controller now accepts either the prior private library
or exactly one private rsomics-sc binary; publish=false remains required.

The one fresh-context overall review found no Critical/Important issue.
Its minor temporary-fixture leak is repaired before the integrated snapshot.
Compilation/runtime acceptance now comes from actual green20, not that review.
Rulings on review limits: arbitrary-input R precision, exact current-Rust-to-R
tree equality, full inferCNV, clinical meaning and release readiness remain
unclaimed. Requested-buffer limits intentionally exclude RSS/thread stacks.
These do not become silently inferred claims when CI passes.

After functional validation, measure the complete slice and its distance hot
path on the shipped example and a representative larger cohort. Record actual
throughput/memory and serial/parallel behavior before making performance or
replacement claims. Continue remaining inferCNV work and the broader portfolio;
the active goal does not terminate at this slice or at inferCNV.
