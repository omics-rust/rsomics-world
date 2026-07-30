# `rsomics-common` consumer contract

Status: live-consumer audit complete; 0.7 boundary proposed; no API change or
publication performed.

## Why this audit exists

The current crate presents one `CommonFlags` bundle containing threads, JSON,
quiet, verbose, and seed controls. `run()` always attempts to initialize the
global Rayon pool when the default feature is enabled. This makes every
consumer advertise capabilities it may not implement.

The retained architecture requires the opposite direction: products establish
real contracts first, and common receives only items shared by at least two of
them.

## Current consumer matrix

The table is based on live source at these revisions:

- `rsomics-common` `1c51f7d0b356`;
- `rsomics-seq` `02f8268931b0`;
- `rsomics-fastq-preprocess` `8e483fc95556`;
- `rsomics-bed` `97f5fe31662e`;
- `rsomics-seqio` `ce9c5514c235`;
- `rsomics-intervals` `c13cb75c318`.

| Current item | Concrete retained consumers | Finding |
|---|---|---|
| `RsomicsError`, `Result`, `Context` | `seq`, `fastq-preprocess`, `bed`, `seqio`, `intervals` | keep; multiple real call sites and stable error categories |
| `ExitCode`, JSON envelopes, `ToolMeta`, `run()` | `seq`, `fastq-preprocess`, `bed` | keep after removing unrelated capability initialization |
| `CommonFlags::json` | `seq`, `fastq-preprocess`, `bed` | keep as the one demonstrated shared CLI control |
| `CommonFlags::threads` and global Rayon setup | effective only in `fastq-preprocess` | move product-private; one consumer does not justify a foundation item |
| `CommonFlags::seed` | none of the three current products | remove; do not manufacture RNG work to justify it |
| `CommonFlags::quiet` / `verbose` | no current product emits common info/debug messages | remove until two products establish a logging contract |
| `StderrLog` | no direct product consumer | remove; its write errors are also currently discarded |
| `Tool` trait | only unreconstructed `liftover` and `minimap2` repositories | do not let inherited wrappers freeze the pilot-product API |
| path/stdin and path/stdout helpers | no current product consumer | remove; current output helper truncates directly and is weaker than product transaction contracts |
| `format_g6` | no retained current product consumer | remove until two typed numeric-output contracts require it |
| `test-support` / `tier2` | no current retained consumer | remove; the fixture macro assumes the deleted monorepo layout |
| `flate2` dependency | no common source call site | remove |

The 562 historical micro-crate manifests that mention `rsomics-common` are
implementation assets, not public-API consumers. Counting them would recreate
the topology being retired.

## Proposed 0.7 boundary

The next common release should contain:

- the typed error categories and contextual I/O conversion;
- stable process exit-code mapping;
- the versioned JSON success and error envelope;
- `ToolMeta`;
- a minimal shared JSON/output-mode flag;
- a runner that maps the body result to JSON/plain diagnostics and an exit
  code without initializing unrelated global state.

It should not contain thread pools, RNG policy, progress/log verbosity, generic
file truncation, bioinformatics format I/O, numeric rendering copied from one
tool, or old-workspace fixture discovery.

Keeping the name `CommonFlags` for a JSON-only structure would minimize source
edits but preserve a misleading abstraction. The preferred API is an explicit
name such as `OutputMode`, flattened only by products that support the shared
JSON envelope. This is a deliberate 0.7 breaking boundary rather than another
field-by-field compatibility layer.

## Product migrations

### `rsomics-fastq-preprocess`

Owns a private `--threads` argument and validates a positive count. It installs
the Rayon pool before executing the pipeline, with consumer tests requiring
byte-identical single-end and paired-end output at one and four threads.

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

`rsomics-seq` and `rsomics-fastq-preprocess` currently make their CI check out
exact common/seqio/kmer revisions and generate a `[patch.crates-io]` path table
inside the job. This proved the coordinated implementations, but it is not the
target flat-repository dependency model and must not become permanent
infrastructure. The product manifests remain version dependencies; once the
reviewed foundations are published, the CI checkout/patch steps are removed
and clean registry resolution is tested.

## Verification and release sequence

1. Implement the narrow common boundary with unit tests for plain, JSON,
   serialization-failure, write-failure, and exit-code behavior.
2. Use temporary local Cargo patches to compile and test `seq`,
   `fastq-preprocess`, and `bed` without adding path dependencies to their
   manifests.
3. Review the three resulting command trees and consumer tests together.
4. Commit common as one API concern and obtain exact-head native CI on Linux
   and macOS for `x86_64` and `aarch64`.
5. Publish only after the common package metadata and public API pass final
   review. The previous crates.io token was revoked, so publication requires a
   new explicit credential path.
6. Update and gate each product independently against the published version;
   never commit a path dependency.

Until this sequence is complete, products keep their current published common
dependency and record the irrelevant flags as release blockers.
