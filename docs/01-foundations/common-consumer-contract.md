# `rsomics-common` consumer contract

Status: common 0.9.0, help 0.4.0, intervals 0.3.0, and seqio 0.3.0 are
published and verified from downloaded crates.io archives. BED resolves
common 0.8 and VCF resolves common 0.9 from the registry; the sequence pilots
remain on their tested common 0.7 contract through seqio.

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

- `rsomics-common` `10a8b4708d36`;
- `rsomics-seq` `d4c840be2e37`;
- `rsomics-fastq-preprocess` `442c202908d1`;
- `rsomics-bed` `989894f2dad5`;
- `rsomics-vcf` `bbc09be7ed38`;
- `rsomics-seqio` `d7e1c33bb600`;
- `rsomics-intervals` `6783f67614ae`.

| Current item | Concrete retained consumers | Finding |
|---|---|---|
| `RsomicsError`, `Result`, `Context` | `seq`, `fastq-preprocess`, `bed`, `annotation`, `seqio` | keep; multiple real call sites and stable error categories |
| `ExitCode`, JSON envelopes, `ToolMeta`, `run()` | `seq`, `fastq-preprocess`, `bed` | keep after removing unrelated capability initialization |
| `Validation<T>`, `run_validation()` | implemented `vcf validate`; planned `bam validate` | keep the format-neutral valid/invalid report and exit contract; validator policy stays in products |
| `OutputArgs::json` | `seq`, `fastq-preprocess`, `bed`, `vcf` | retained as the one demonstrated shared CLI control |
| former `CommonFlags::threads` and global Rayon setup | effective only in `fastq-preprocess` | removed from common; preprocessing owns a local Rayon pool |
| former `CommonFlags::seed` | none of the current consumers | removed; no RNG work was manufactured to justify it |
| former `CommonFlags::quiet` / `verbose` | no current product emits common info/debug messages | removed until two products establish a logging contract |
| `StderrLog` | no direct product consumer | remove; its write errors are also currently discarded |
| `Tool` trait | only unreconstructed `liftover` and `minimap2` repositories | do not let inherited wrappers freeze the pilot-product API |
| former path/stdin and truncating path/stdout helpers | no retained consumer | removed; they did not provide a safe shared contract |
| `write_atomic` | `bed`, `vcf` | keep; both products use same-directory temporary output, atomic persist, cleanup on failure, destination-mode preservation, and parent-directory sync |
| `format_g6` | no retained current product consumer | remove until two typed numeric-output contracts require it |
| `test-support` / `tier2` | no current retained consumer | remove; the fixture macro assumes the deleted monorepo layout |
| `flate2` dependency | no common source call site | remove |

The 562 historical micro-crate manifests that mention `rsomics-common` are
implementation assets, not public-API consumers. Counting them would recreate
the topology being retired.

## Implemented 0.9 boundary

The 0.8 implementation established:

- the typed error categories and contextual I/O conversion;
- stable process exit-code mapping;
- the versioned JSON success and error envelope;
- `ToolMeta`;
- a minimal shared JSON/output-mode flag;
- a runner that maps the body result to JSON/plain diagnostics and an exit
  code without initializing unrelated global state;
- `write_atomic`, added only after BED and VCF demonstrated the same named-file
  transaction contract.

Common 0.9 adds `Validation<T>` and `run_validation`. A product can return a
complete invalid report, emit it inside the shared JSON error envelope, and
still use the shared invalid-input exit code. The common crate does not define
diagnostics, record models, validity rules, or output summaries for VCF, BAM,
or any other format.

`write_atomic` creates its temporary file beside the destination, removes it
when the producer fails, flushes and syncs before persist, preserves the mode
of an existing destination, honors the process umask for a new destination,
and syncs the parent directory on Unix. Consumer tests cover preservation and
failure paths rather than assuming an atomic rename is sufficient.

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
of this migration. Revision `989894f2dad5` removes its private transactional
writer and consumes common 0.8; exact-head CI run `30621067404` passes all four
native target classes and its pinned bedtools 2.31.1 oracle.

### `rsomics-vcf`

The `query` and `view` commands write named projections transactionally while
stdout remains a direct stream. The `index` command likewise builds a complete
CSI or TBI before replacing its destination and retains the previous index on
a parse, compatibility, allocation, write, flush, or sync failure. The
`validate` command consumes common 0.9 so invalid JSON output retains the full
structured report instead of reducing it to a message. Revision
`bbc09be7ed38` exercises all four contracts and passes exact-head CI run
`30633237582` on all four native target classes. Published VCF 0.1.0 was
downloaded from crates.io, reinstalled, and tested from its registry archive.

### Other retained repositories

`rsomics-liftover` and `rsomics-minimap2` currently implement the inherited
`Tool` trait, but neither has passed its reconstruction dossier and product
gate. They remain on the published 0.6 API until reconstructed, then migrate to
the same explicit runner contract. Historical micro-crates are not updated.

## Remaining unpublished-foundation workaround

Common, help, intervals, and seqio no longer use a CI path patch. Their
consumers lock the published crates.io checksums and pass on four native
targets. `rsomics-seq` still patches kmer 0.2.1 until a second product
consumer exists.

## Verification and release sequence

1. Common 0.9.0, help 0.4.0, intervals 0.3.0, and seqio 0.3.0 are published,
   archive-checksum verified, and consumed from crates.io.
2. Every migrated product regenerates its lockfile and passes exact-head CI
   without a path dependency for those releases.
3. Kmer remains unpublished until a second product demonstrates its contract.

Common 0.9 exact-head CI run `30626561250` and publish run `30626628961`
completed successfully. The downloaded registry archive and crates.io API
agree on SHA-256
`00ecf388c7cccd2792438260c90e121f6de758c519f76ed64ff24b61f2e4ac78`.
