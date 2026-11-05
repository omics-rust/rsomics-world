# Metagenomics database/classification implementation plan

> Execute inline with `superpowers:executing-plans`, one whole-feature review
> and one necessary fix pass. Existing unattended authority supersedes a new
> approval checkpoint. No per-module reviewer ladder or empty public commands.

**Goal:** deliver one complete nucleotide database build/inspect, single-end
classification and standard-report lifecycle inside `rsomics-metagenomics`,
without replacing or regressing its published amplicon operations.

**Architecture:** a product-local checked taxonomy, exact upstream minimizer
and compact-cell semantics, and a portable native database file. Ordered
bounded read batches share a local worker pool; report aggregation shares the
taxonomy, not a separate parser/model. No new Layer A API is presumed.

**Tech stack:** Rust1.91, existing common0.12/help0.4/seqio0.5; Rayon for the
actual parallel hot path; SHA-256 for durable database/source identity. Keep
`forbid(unsafe_code)` and avoid a duplicate whole-table byte buffer on load.

**Spec:** `docs/10-products/metagenomics-sketch.md`, owning architecture and
`.autopilot/state/metagenomics-classification-frontier-2026-11-05.md`.
The latter pins official Kraken2.17.1 source, actual outputs and collision
witnesses. Historical tax-assign is negative evidence, not production input.

## Global constraints and selected first profile

- Owning repository is `/Volumes/KIOXIA/Documents/omics-rust/rsomics-metagenomics`,
  starting clean main `34e60b53de0ba1f5937e28fd4f68abb4a1442435`.
- Use the AGENTS external Cargo/rustup/target/scratch/fixture paths and actual
  mounted-device/resolved-target guard before every tool execution that writes
  build, cache, download or scratch output. No dependency on another Layer B.
- Public commands: `database build`, `database inspect`, `classify`, `report`.
  Keep existing amplicon grammar/output unchanged. All use rsomics-help.
- Build consumes local NCBI nodes/names, optional merged/deleted dumps and
  ordered FASTA libraries with explicit `kraken:taxid` IDs or a checked ID map.
  No taxonomy/reference download manager, protein search, masking pipeline,
  confidence/quick/paired/multidatabase expansion or minimizer-data/HLL report
  is advertised in this first slice. Standard six-column report is complete.
- Native file is identified as rsomics, never a plain TSV or claimed `.k2d`
  interchange. Preserve probabilistic compact fingerprints and collision LCA.
- Default k35/l31/spaces7, fixed upstream toggle/reverse-complement1/Murmur
  finalizer/linear probing. Capacity uses the upstream sampled estimator:
  low ten hash bits select four sets, `(sample_distinct+1)*256`, then add8192
  and divide by0.7. An explicit capacity supports controlled source witnesses.
- Canonical native build uses fixed block/subblock16384, independent of worker
  count; allow explicit parameters for exact oracle profiles. Kraken's wrapper
  otherwise derives subblock size from threads, and actual source demonstrates
  partition-dependent keys. Compare upstream with identical explicit settings;
  document this profile rather than claiming identical automatic defaults.
- Validate configuration and file/model boundaries once; do not repeat checks
  already established by typed records. Errors propagate nonzero, named outputs
  use common transactions/alias handling, no overwrite or partial database.
- Sparse comments explain only stable non-obvious compatibility invariants.
  Do not publish or expose incomplete behavior merely to reserve scope.

## File and interface map

Add the following product modules only as their implementations land:

- `src/classifier/taxonomy.rs`: checked NCBI loading, merged/deleted resolution,
  deterministic pruned BFS internal IDs and LCA; no publicly mutable maps.
- `src/classifier/minimizer.rs`: source-equivalent rolling scanner, ambiguity,
  interval partitioning and sampled capacity calculation.
- `src/classifier/compact.rs`: compact u32 cells, widths, fixed hash/probe,
  deterministic zone/key insertion and LCA merging; no exact-map substitution.
- `src/classifier/database.rs`: portable fixed-width LE header/sections,
  decoded parameters, ordered input hashes, taxonomy/cells, SHA-256 integrity,
  bounded direct decoding and atomic commit.
- `src/classifier/assign.rs`: ordered minimizer hits/groups and read assignment,
  five-column status/identity/taxon/length/hit-list output; minimum-hit-groups2
  matches the actual wrapper, not the backend's unused default0.
- `src/classifier/report.rs`: direct/clade counts, ranks, hierarchy, percentages,
  source ordering and optional zero rows; shared by inspect and report.
- `src/classifier/mod.rs`: narrow public workflow entry points/configuration.
  Internal algorithm/data representation stays private to the product.
- `src/cli/classifier.rs` and existing `src/cli.rs`: nested typed Clap options,
  common output/thread arguments, actual worker pool and workflow dispatch.

Workflow interfaces produced together in Task1:

```rust
pub fn build_database(config: &BuildConfig, pool: &rayon::ThreadPool) -> Result<DatabaseManifest>;
pub fn inspect_database(config: &InspectConfig) -> Result<DatabaseSummary>;
pub fn classify(config: &ClassifyConfig, pool: &rayon::ThreadPool) -> Result<ClassifySummary>;
pub fn report(config: &ReportConfig) -> Result<ReportSummary>;
```

Configs own the corresponding paths and validated operation settings; summaries
are serializable product reports in the existing common envelope. Do not add
generic plugin, trait hierarchy, public matrix or taxonomy foundation APIs.

## Task1: implement the connected lifecycle

- [ ] Import the small team-owned taxonomy/reference/read fixtures and actual
  output goldens from the two accepted source packets, with exact pins and
  upstream attribution. No runtime test relies on an absolute scratch path.
- [ ] Add `tests/classification_lifecycle.rs` with one connected
  `database_classify_report_matches_pinned_kraken` workflow: build, inspect,
  classify and report; assert retained read identities, complete five/six-column
  bytes, LCA/collision behavior,1/4-worker identity and unchanged amplicon route.
  Run it against the old CLI first: expected FAIL at missing `database`, not
  a missing-oracle skip or a compilation typo.
- [ ] Implement the modules/interfaces above. Internalize only audited taxonomy
  and report assets; port minimizer/compact logic from the pinned allowed source.
  Preserve copyright/license originals where code is derived. Bound per-worker
  sequence/output buffers; use sorted deduplicated zone vectors, not duplicate
  whole-reference collections or an unordered-map approximation of the table.
- [ ] Add one combined failed-output lifecycle: invalid taxonomy, corrupted
  database and a late malformed query must fail without committing the affected
  named output or altering an existing destination. Checked model construction,
  not speculative defensive branches, owns the underlying invariants.
- [ ] Run guarded offline formatting, strict Clippy, docs, full debug/release
  suites and required live pinned VSEARCH plus Kraken comparisons. Expected:
  all existing amplicon tests and connected new tests pass without oracle skips.
  Do not add an unrelated test matrix or split one workflow into microcrates.

## Task2: qualify the actual hot paths

Consumes Task1's complete workflows and portable database.

- [ ] Select/version an existing or public reference collection with at least
  ten million bases plus at least one million nontrivial mixed query reads;
  record accession/source, hashes, taxonomy, ordering and compression. Pin the
  manifest before timing; no seed/input changes to erase a negative result.
- [ ] Run both build and full classify/report at1/4 workers against the actual
  pinned source with identical capacity and explicit partition parameters.
  Check decoded database/table/taxonomy equivalence and complete classification/
  report outputs before timing. Include retained unknown/ambiguous reads.
- [ ] One warmup plus five alternating paired trials, no competing build or
  benchmark. Retain all argv, inputs/outputs, versions, binary/source identities,
  wall/user/system CPU, RSS, database/scratch sizes and timing distributions.
  Require a strict hot-path throughput or resource advantage; report failed
  scaling, I/O or wall results without broad replacement claims.
- [ ] One fresh whole-feature API/hot-path review, covering all five classes
  below; resolve important findings in one RED→GREEN fix pass plus full suite.
  No repeated unchanged review or performance campaigns.

## Task3: deliver only the qualified complete slice

- [ ] Update product scope, provenance, license notices and a0.2 candidate only
  after the full workflow is real. Preserve original amplicon evidence and
  distinguish native-format/profile choices from upstream defaults.
- [ ] Commit one concern per commit on owning main, push through the available
  authenticated transport and wait for exact-head native Linux/macOS x86_64/
  aarch64 tests with mandatory real oracles. Package and install from the clean
  source; exercise the complete installed workflow on each native class.
- [ ] Adopt original CI/source/license/binary receipts and public installation
  before GitHub distribution; crates.io publication additionally needs restored
  valid registry authentication. Never retry an unchanged known-invalid secret.
- [ ] Update control-plane dossier/state with exact delivered identities,
  residual exclusions and failed gates; then advance the broader portfolio.

## Whole-feature review focus

1. Taxdump/reference identity resolution and source ordering must not silently
   relabel a genome or change internal IDs/collision outcomes.
2. Canonical/spaced minimizers, ambiguity resets and subblock bounds must match
   the real oracle, including repeated hit counts and group qualification.
3. Fingerprint collisions, occupancy and deterministic insertion must retain
   source semantics at meaningful load, not only on a sparse toy database.
4. Truncated/mixed/corrupted database and late query errors must preserve output
   transactions; database allocation/decoding must not duplicate the table.
5. 1/4-worker scheduling, status/length/hit-list bytes and direct/clade reporting
   must preserve record order and the connected lifecycle's totals.

Each class is covered by Task1's integrated normal/collision/failure workflows;
the review checks interactions and production hot paths, not a new test ladder.
