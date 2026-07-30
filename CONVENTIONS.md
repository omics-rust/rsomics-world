# rsomics conventions

## Public boundary

An rsomics product is a coherent bioinformatics tool or workflow family. A
crate is not created for every function, flag, statistical primitive, or
upstream subcommand.

Use these tests when deciding whether operations belong together:

1. Do they share a primary data model or file format?
2. Would users expect one installation and one documentation surface?
3. Do they share parsing, filtering, execution, and output policy?
4. Are they commonly composed in one workflow?
5. Can their names remain clear as subcommands?

If most answers are yes, they belong in one product crate.

## Repository model

Every public crate has its own flat GitHub repository under `omics-rust`.
`rsomics-world` is documentation and planning only.

There is no cross-repository Cargo workspace, submodule tree, or path
dependency. Published crates depend on versioned crates.io releases.

## Layers

### Layer A

Layer A is a library-only public foundation. It has no product binary and must
serve at least two named Layer B products.

Names describe a stable technical domain:

- `rsomics-common`
- `rsomics-intervals`
- `rsomics-seqio`
- `rsomics-bamio`

### Layer B

Layer B is an installable product family. It normally contains:

```text
src/
├── lib.rs
├── main.rs
├── cli.rs
├── operations/
│   ├── mod.rs
│   ├── view.rs
│   ├── sort.rs
│   └── stats.rs
├── model/
└── io/
```

The binary and package use the same name, such as `rsomics-bed`. Operations
appear as subcommands, such as `rsomics-bed intersect`.

## Dependency direction

```text
Layer B → Layer A → external crates
```

- Layer A must not depend on Layer B.
- Layer B must not depend directly on another Layer B product.
- Product-specific policy stays in the product.
- Code starts internal and is promoted only after a second product consumer is
  implemented or concretely scheduled.

## Naming

Package and repository:

```text
rsomics-<product>
```

Rust library:

```text
rsomics_<product>
```

CLI:

```text
rsomics-<product> <subcommand>
```

Prefer established user vocabulary over internal algorithm names. Avoid names
that expose implementation details unless that algorithm is itself the
recognized product.

## CLI behavior

- `--help` succeeds without requiring input files.
- Errors go to stderr and return non-zero.
- Data output goes to stdout unless an explicit output path is supplied.
- Diagnostics never corrupt machine-readable stdout.
- Thread count, output compression, region syntax, and format selection are
  consistent within a product.
- JSON output is a stable documented contract where provided.
- Defaults are deterministic unless the operation is explicitly stochastic.
- Stochastic operations expose and report a seed.

Use `rsomics-common` for shared execution and output primitives and
`rsomics-help` for family-wide help rendering. Do not duplicate their behavior
inside every product.

## Error handling

- Return structured errors from libraries.
- Add context at I/O and command boundaries.
- Do not replace invalid biological data with defaults.
- Do not silently skip malformed records unless the user selected an explicit
  permissive mode.
- Validate user input once at the boundary.
- Preserve the first useful cause in the displayed error chain.

## Module design

- One module may implement one operation without becoming a crate.
- Shared record types live under `model`.
- Shared format adapters live under `io`.
- Product-wide filters and options use typed structures.
- Hot loops avoid trait objects, per-record allocation, and repeated parsing
  unless measurement justifies them.
- Public APIs expose biological and format concepts, not CLI parser types.

## Historical code migration

Old operation-sized repositories are source assets.

For every migrated operation record:

- source repository and commit;
- target product/module/subcommand;
- retained tests, fixtures, and benchmarks;
- behavior changes;
- removed duplication;
- public-foundation calls introduced.

Direct code reuse is allowed because the user confirms the historical
implementations are team-owned. Refactor names, ownership, and error flow to fit
the target product instead of preserving obsolete crate boundaries.

## Tests

Products use four levels:

1. unit tests for parsers, models, and algorithms;
2. golden tests for CLI and format output;
3. compatibility tests against upstream behavior;
4. integration tests composing multiple subcommands.

Each migrated operation must preserve its strongest prior evidence. Tests from
multiple old repositories may be consolidated into one operation-oriented test
module.

Foundations require consumer-contract tests. A foundation API is not complete
merely because its own unit tests pass.

## Compatibility

Compatibility is defined per operation, not per old crate:

- byte exact when output is canonically defined;
- field exact when ordering or formatting is intentionally flexible;
- tolerance based only for documented floating-point behavior;
- explicit divergence where rsomics deliberately fixes an upstream bug or
  changes a default.

Record the upstream version and command used as the oracle.

## Performance

Benchmarks must report:

- rsomics commit;
- upstream version;
- machine and operating system;
- thread count;
- fixture identity and checksum;
- compression and output mode;
- timing distribution;
- peak memory where material.

Profile before optimizing. Evaluate allocation, cache behavior, syscalls,
parallel scaling, and output cost in addition to CPU time.

An operation intended to replace an established implementation must provide a
strict throughput or resource-use advantage on its relevant hot path. Equal
performance is not a release pass merely because the binary is easier to
install. Performance claims without provenance are not release evidence.

## External dependency classification

Record hot-path dependencies as:

1. pure Rust with explicit parallelism or vectorization;
2. FFI wrapper over native code;
3. pure Rust but single-threaded in a hot path;
4. non-hot utility.

FFI is permitted when documented. It does not count as a pure-Rust rewrite.
Product documentation must state the boundary clearly.

## Documentation

Every product README includes:

- purpose and upstream scope;
- subcommand list;
- install and usage examples;
- format and compatibility notes;
- threading and memory behavior;
- origin and attribution;
- current limitations.

Do not describe phase history or deleted micro-crate topology as current
architecture.

## Release

Before publishing:

```bash
cargo fmt --check
cargo clippy --all-targets --all-features -- -D warnings
cargo test --all-features
```

Then run the product's compatibility and performance gates, push, and verify
exact-head CI on Linux and macOS for both `x86_64` and `aarch64`.

Versioning is independent per repository. A product release may add
subcommands without creating additional packages. Breaking shared-library APIs
require a semver-major change and coordinated consumer updates.

## Status vocabulary

Control-plane registry states:

- `planned` — accepted boundary, no public repository or release;
- `source-pool` — historical implementations exist for consolidation;
- `pilot` — active consolidation with an unpublished interface;
- `live` — public repository and at least one non-yanked release;
- `repo-only` — public repository exists but no release;
- `temporary` — public only because migration or immutable registry history
  prevents immediate removal;
- `retired` — removed from live GitHub and crates.io surfaces.
