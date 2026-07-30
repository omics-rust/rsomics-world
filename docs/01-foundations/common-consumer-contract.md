# `rsomics-common` consumer contract

Status: common 0.7, help 0.4, seqio 0.3, intervals 0.3, and all three pilot
product migrations are committed and exact-head CI verified. Nothing has been
published.

## Why this audit exists

The former crate presented one `CommonFlags` bundle containing threads, JSON,
quiet, verbose, and seed controls. `run()` always attempts to initialize the
global Rayon pool when the default feature is enabled. This makes every
consumer advertise capabilities it may not implement.

The retained architecture requires the opposite direction: products establish
real contracts first, and common receives only items shared by at least two of
them.

## Current consumer matrix

The table is based on live source at these revisions:

- `rsomics-common` `9f11f37c0fa4`;
- `rsomics-seq` `2727daa3bf4f`;
- `rsomics-fastq-preprocess` `f217fc4902b2`;
- `rsomics-bed` `9f4ba8ee945c`;
- `rsomics-seqio` `b23cf8ad29fd`;
- `rsomics-intervals` `491b14c0d43b`.

| Current item | Concrete retained consumers | Finding |
|---|---|---|
| `RsomicsError`, `Result`, `Context` | `seq`, `fastq-preprocess`, `bed`, `seqio`, `intervals` | keep; multiple real call sites and stable error categories |
| `ExitCode`, JSON envelopes, `ToolMeta`, `run()` | `seq`, `fastq-preprocess`, `bed` | keep after removing unrelated capability initialization |
| `OutputArgs::json` | `seq`, `fastq-preprocess`, `bed` | retained as the one demonstrated shared CLI control |
| former `CommonFlags::threads` and global Rayon setup | effective only in `fastq-preprocess` | removed from common; preprocessing owns a local Rayon pool |
| former `CommonFlags::seed` | none of the three current products | removed; no RNG work was manufactured to justify it |
| former `CommonFlags::quiet` / `verbose` | no current product emits common info/debug messages | removed until two products establish a logging contract |
| `StderrLog` | no direct product consumer | remove; its write errors are also currently discarded |
| `Tool` trait | only unreconstructed `liftover` and `minimap2` repositories | do not let inherited wrappers freeze the pilot-product API |
| path/stdin and path/stdout helpers | no current product consumer | remove; current output helper truncates directly and is weaker than product transaction contracts |
| `format_g6` | no retained current product consumer | remove until two typed numeric-output contracts require it |
| `test-support` / `tier2` | no current retained consumer | remove; the fixture macro assumes the deleted monorepo layout |
| `flate2` dependency | no common source call site | remove |

The 562 historical micro-crate manifests that mention `rsomics-common` are
implementation assets, not public-API consumers. Counting them would recreate
the topology being retired.

## Implemented 0.7 boundary

The 0.7 implementation contains:

- the typed error categories and contextual I/O conversion;
- stable process exit-code mapping;
- the versioned JSON success and error envelope;
- `ToolMeta`;
- a minimal shared JSON/output-mode flag;
- a runner that maps the body result to JSON/plain diagnostics and an exit
  code without initializing unrelated global state.

`rsomics-help` is the companion presentation layer. It decorates and parses
the authoritative Clap tree; common does not render help, style argument
errors, or maintain command metadata.

It should not contain thread pools, RNG policy, progress/log verbosity, generic
file truncation, bioinformatics format I/O, numeric rendering copied from one
tool, or old-workspace fixture discovery.

The JSON-only structure is named `OutputArgs`, avoiding the misleading
`CommonFlags` abstraction. Products flatten it only when they support the
shared JSON envelope.

## Product migrations

### `rsomics-fastq-preprocess`

Owns a private `--threads` argument and validates a positive count. Each
command execution builds a local Rayon pool, avoiding process-global
one-time initialization. Consumer tests require byte-identical single-end and
paired-end output at one and four threads.

### `rsomics-seq`

Removes `--threads`, `--seed`, `--quiet`, and `--verbose`. Its representative
controls show no thread scaling, and no current operation uses an RNG or common
logging.

### `rsomics-bed`

Removes the same four inapplicable flags. The five operations remain
single-process interval algorithms; adding speculative parallelism is not part
of this migration.

### Other retained repositories

`rsomics-liftover` and `rsomics-minimap2` currently implement the inherited
`Tool` trait, but neither has passed its reconstruction dossier and product
gate. They remain on the published 0.6 API until reconstructed, then migrate to
the same explicit runner contract. Historical micro-crates are not updated.

## Current unpublished-foundation workaround

The three product repositories and the unpublished foundation
releases make CI check out exact foundation revisions and generate a
`[patch.crates-io]` path table inside the job. This proves the coordinated
implementations without committing path dependencies, but it is temporary.
Once the reviewed foundations are published, the checkout/patch steps are
removed, lockfiles are regenerated against registry sources, and clean
registry resolution is tested.

## Verification and release sequence

1. Keep common `9f11f37c0fa4` and help `c615aa8b8522` as the reviewed
   exact-head baselines.
2. Keep seqio `b23cf8ad29fd` and intervals `491b14c0d43b` as the aligned
   common-0.7 baselines; both have exact-head four-native-target CI.
3. Publish common, help, and seqio only after final package review and an
   explicit new credential path; the previous crates.io token was revoked.
   Intervals additionally remains behind its second-consumer and BED-policy
   gates.
4. Update and gate each product independently against published versions;
   never commit a path dependency.

Until publication, product CI checks out and verifies the exact unpublished
revisions. The irrelevant flags are already absent from all three command
trees.
