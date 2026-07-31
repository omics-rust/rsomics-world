# rsomics-world — unattended operating manual

## Mission

`rsomics-world` is the control plane for a family of high-performance Rust
bioinformatics products under the `omics-rust` GitHub organization.

The public unit is a coherent product or workflow family that a user would
recognize and install. It is not an individual function, flag, or upstream
subcommand. Operations within a product are Rust modules and CLI subcommands.

The quality target remains:

- fail fast and fail loud;
- one easy-to-install binary per product family;
- clear code and explicit data models;
- compatibility with the relevant upstream behavior;
- measured CPU, memory, and I/O performance;
- reusable public libraries only where multiple products need them.

## Repository architecture

The project uses flat independent repositories, not a Cargo monorepo,
submodules, or path dependencies.

```text
omics-rust/
├── rsomics-world/          control plane; documentation and audit scripts
├── rsomics-common/         public foundation library
├── rsomics-intervals/      public foundation library
├── rsomics-seqio/          public foundation library
├── rsomics-seq/            coherent sequence-utility product
├── rsomics-bed/            coherent interval-tool product
├── rsomics-bam/            coherent alignment-format product
└── ...
```

Local layout:

| Purpose | Path |
|---|---|
| Control plane | `/Volumes/Zane's HDD/Documents/rsomics-world` |
| Active and historical crate clones | `/Volumes/KIOXIA/Documents/omics-rust/` |
| Cargo home | `/Volumes/KIOXIA/Developments/cargo-home` |
| Cargo target | `/Volumes/KIOXIA/Developments/cargo-target` |
| Scratch | `/Volumes/KIOXIA/Developments/tmp` |
| Large fixtures | `/Volumes/Zane's HDD/rsomics-fixtures/` |

Never compile, download project data, or create project scratch on the Mac mini
boot disk.

## Two layers

### Layer A — public foundations

Layer A crates are library-only shared components. A public foundation must
have at least two named target-product consumers with concrete call sites or
near-term implementation plans.

Current long-term foundations:

- `rsomics-common`
- `rsomics-help`
- `rsomics-bamio`
- `rsomics-intervals`
- `rsomics-kmer`
- `rsomics-seqio`
- `rsomics-stats`
- `rsomics-phylo-tree`
- `rsomics-pileup`

`rsomics-igzip` is temporary. It remains public only because an immutable
published `rsomics-seqio` version depends on it. Its code should be internalized
or replaced when the registry dependency can be removed safely.

### Layer B — products

Layer B crates may contain a library and a single product binary. A Layer B
crate represents one coherent product family and may expose many subcommands.

Examples:

- `rsomics-bed intersect`
- `rsomics-bed merge`
- `rsomics-bed coverage`
- `rsomics-vcf view`
- `rsomics-vcf norm`
- `rsomics-vcf stats`

Region filtering is an option of `view`, not a separate crate. An upstream
binary name is not automatically the right boundary; group by shared data
model, user workflow, and installation identity.

Dependency direction is:

```text
Layer B product → Layer A foundation → external dependency
```

Layer A never depends on Layer B. Layer B products do not depend directly on
other Layer B products.

## Product-family allowlist

The current namespace allowlist is
`docs/00-overview/registry-reset-keep.txt`: 30 product-family names and nine
public foundations. It is a planning baseline, not an instruction to publish
empty crates. `rsomics-workflow` and `rsomics-expression` were rejected after
their candidates proved consumer-local. `rsomics-call` and `rsomics-cnv`
replace those speculative boundaries with real variant-calling and copy-number
workflows recovered during the VCF review.

Do not create a new public crate merely because a source module is reusable or
an upstream exposes another operation. First place the code inside the
consuming product. Promote it only after a second product consumer is concrete.

## Historical implementation source pool

The operation-sized repositories deleted from GitHub and crates.io remain on
external storage as implementation assets:

- local clones under `/Volumes/KIOXIA/Documents/omics-rust/`;
- crates.io archives and Git bundles under
  `/Volumes/KIOXIA/Documents/omics-rust/_retired/`;
- the generated routing ledger at
  `docs/00-overview/portfolio-inventory.tsv`.

The user has confirmed that the historical implementation code is team-owned
and may be reused directly. Upstream names still need attribution and behavior
provenance, but GPL contamination is not presumed.

For each target product, classify each old implementation as:

1. direct merge;
2. refactor then merge;
3. test, fixture, or benchmark asset only;
4. discard.

Do not revive deleted micro-crate repositories. Move selected code into modules
of the target product and preserve useful Git provenance in the merge record.

## Required product dossier

Before implementing a product family, its dossier must record:

- real upstream tools and packages in scope;
- every user-recognizable operation;
- overlaps and deduplication across upstreams;
- old rsomics implementation assets;
- target module and subcommand names;
- shared-foundation requirements and named consumers;
- format semantics and compatibility oracle;
- fixture and benchmark coverage;
- license and attribution notes;
- explicit exclusions.

Read actual public documentation, format specifications, papers, and allowed
source before reconstructing non-trivial behavior. Do not invent upstream
operations from memory.

## Implementation rules

- Production errors propagate to the top-level command and exit non-zero.
- Do not swallow parse, I/O, compatibility, or invariant failures.
- Avoid redundant defensive checks where the boundary or type already enforces
  the invariant.
- Keep operation logic in modules with narrow typed interfaces.
- Share parsers, record types, and execution plumbing inside the product before
  considering a public foundation.
- Prefer names, types, and narrow functions over comments. Keep source comments
  rare and consistent.
- Comments explain stable invariants and non-obvious reasons, not phases,
  audits, history, or code narration. Public API docs and CLI help describe
  user contracts rather than implementation steps.
- Production `unwrap()` is allowed only for statically obvious invariants.
- Tests may unwrap freely.
- Use Mermaid or D2 for diagrams.

## Compatibility and performance

Every migrated operation must retain or improve the strongest useful evidence
from its source asset:

- unit tests for internal invariants;
- golden fixtures for format and CLI behavior;
- compatibility tests against the real upstream oracle where appropriate;
- benchmarks on a representative non-trivial input;
- version, machine, input, flags, timing distribution, and memory provenance.

Publish only when all operations declared stable for that release are correct.
An unfinished subcommand stays undocumented or feature-gated; do not advertise
placeholder behavior.

For a replacement of an established tool, the relevant hot path must show a
strict throughput or resource-use advantage. Equal performance without another
material benefit is insufficient.

## Public-foundation API rule

Foundation work proceeds alongside product work, not as an independent
speculative phase.

A new public item requires:

1. two named product consumers;
2. consumer-side tests demonstrating the shared contract;
3. an API review that excludes product-specific policy;
4. compatibility and performance checks where the item is on a hot path.

If only one product needs the code, keep it internal.

## Git and CI

- Before any build, test, benchmark, or dependency download, verify that
  `CARGO_HOME`, Cargo's resolved target directory, and `TMPDIR` point to the
  allowed external-disk locations. Also check free space on `/` and KIOXIA.
  Stop before compiling if the boot disk is at or above 80% usage or any build
  path resolves to it.
- Work in the owning repository under
  `/Volumes/KIOXIA/Documents/omics-rust/rsomics-<name>`.
- Direct commits to `main`; no pull requests unless the user requests one.
- One concern per commit.
- Never add `Co-Authored-By`.
- Do not stage unrelated or inherited worktree changes.
- After each push, wait for the exact-head GitHub Actions run to pass.
- Control-plane changes are committed in `rsomics-world`.
- First-class CI targets are Linux and macOS on both `x86_64` and `aarch64`.
  A release gate must exercise all four target classes; cross-compilation alone
  does not replace native tests for platform-specific behavior.

Commit prefixes:

- `docs(<area>):`
- `feat(<product>):`
- `fix(<product>):`
- `refactor(<product>):`
- `test(<product>):`
- `chore:`
- `ci:`

## Publication

Crates publish independently from their own repositories. Before publication:

1. confirm the target product or foundation boundary;
2. run formatting, strict Clippy, tests, compatibility, and benchmarks;
3. verify exact-head CI;
4. verify repository and documentation metadata;
5. record the performance decision;
6. perform a fresh review of public API and production hot paths.

Do not publish a name merely to reserve it. Deleted names may be republished
after the registry cooldown when the product is real.

## Unattended execution

The user authorizes unattended progress within this architecture.

Durable state belongs in tracked control-plane documents or `.autopilot/state`;
do not rely on conversational memory. Work should be idempotent and resumable.
If one family is blocked, record it and advance another unblocked family.

Do not create proactive reminders, cron jobs, external notifications, or
heartbeat automations unless the user explicitly asks.

Parallel agents may perform bounded read-only inventories, differentiated
reviews, and isolated implementation tasks. They must receive the external-disk
paths and may not delete, publish, or stage unrelated files.

## Stop conditions

Stop the affected action and request a user decision when:

- two product boundaries are equally defensible and materially change the
  public interface;
- a destructive target lacks a complete verified backup;
- live registry, GitHub, source, and local evidence materially disagree;
- a public API choice would lock several products into an uncertain model;
- correctness or compatibility evidence contradicts the intended behavior;
- the boot disk exceeds 80% usage or build output appears on it.

Do not stop merely at a phase boundary, because work is difficult, or because
another product remains. Record the gate and continue with an unblocked task.

## Current order of work

1. Align the control-plane documents with this product-family architecture.
2. Complete upstream-operation and source-asset dossiers for all 30 products.
3. Consolidate `rsomics-seq` and `rsomics-bed` as pilot products.
4. Evolve `common`, `help`, `seqio`, `kmer`, and `intervals` through those
   concrete consumers.
5. Consolidate BAM and variant calling with their `bamio`/`pileup`
   foundations, then consolidate VCF/BCF format workflows.
6. Consolidate stateful statistical and domain workflows.
