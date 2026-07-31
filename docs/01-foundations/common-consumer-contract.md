# `rsomics-common` consumer contract

Status: common 0.7.0, help 0.4.0, and intervals 0.3.0 are published and
verified from downloaded crates.io archives. Pilot products resolve those
versions from the registry. Seqio 0.3 remains unpublished.

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

- `rsomics-common` `5a46f8ee5888`;
- `rsomics-seq` `bf00b71477b8`;
- `rsomics-fastq-preprocess` `a56519d9d6c0`;
- `rsomics-bed` `e8898dbcb0db`;
- `rsomics-seqio` `7b5b1c68f52e`;
- `rsomics-intervals` `6783f67614ae`.

| Current item | Concrete retained consumers | Finding |
|---|---|---|
| `RsomicsError`, `Result`, `Context` | `seq`, `fastq-preprocess`, `bed`, `annotation`, `seqio` | keep; multiple real call sites and stable error categories |
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

## Remaining unpublished-foundation workaround

Common, help, and intervals no longer use a CI path patch. Their consumers
lock the published crates.io checksums and pass on four native targets.
`rsomics-seq` and `rsomics-fastq-preprocess` still patch exact seqio 0.3
revisions inside CI because that foundation has not yet passed its
representative performance decision. `rsomics-seq` likewise patches kmer
0.2.1 until a second product consumer exists.

## Verification and release sequence

1. Common 0.7.0, help 0.4.0, and intervals 0.3.0 are published, archive
   checksum verified, and consumed from crates.io.
2. Every migrated product regenerates its lockfile and passes exact-head CI
   without a path dependency for those releases.
3. Seqio 0.3 is published only after its representative compatibility,
   throughput, memory, public-API, and exact-head gates are complete.
4. Kmer remains unpublished until a second product demonstrates its contract.
