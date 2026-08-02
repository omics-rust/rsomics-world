# Sequence and FASTQ product dossier

Status: source audit complete; `rsomics-seq 0.1.0`,
`rsomics-fastq-preprocess 0.1.1`, and `rsomics-fastq-qc 0.1.0` are published
and independently verified.

The source pool contains 47 relevant historical crates after routing
corrections: 34 for `rsomics-seq`, 12 for preprocessing, and one for QC. The
reuse unit is an algorithm module plus its evidence, not the old CLI shell.

## Shared design

The three products share `rsomics-seqio` for strict FASTA/FASTQ I/O.
`rsomics-common` supplies the execution and output contract used by the first
two implemented products, and `rsomics-help` supplies their shared CLI
presentation layer. `rsomics-kmer` currently serves sequence k-mer operations;
it is not a preprocessing dependency.

The broader cross-product integration gate is:

```mermaid
flowchart LR
    raw["raw single-end or paired FASTQ"] --> prep["fastq-preprocess run"]
    prep --> qc["fastq-qc report"]
    qc --> stats["seq stats"]
```

The QC handoff is now implemented without coupling the three product
boundaries. The same versioned inputs are checked against fastp, FastQC, and
SeqKit as appropriate. Compatibility, wall-clock distribution, peak memory,
compression mode, and thread count are recorded together.

`rsomics-seqio` must provide:

- strict FASTA and FASTQ readers and writers;
- plain, gzip, and BGZF detection by content;
- path and stdin/stdout boundaries;
- borrowed streaming records plus owned batches for parallel stages;
- paired-read semantics;
- a validated writer boundary; a specialized compression backend remains
  consumer-local until a second product needs the same contract.

The historical `rsomics-fqgz` writer and `rsomics-igzip` backend are migration
assets. New products do not depend on `rsomics-igzip` directly.

## `rsomics-seq`

### Boundary

One coherent FASTA/FASTQ utility product. Format-generic commands operate on
both formats when their semantics permit it; sequence-, codon-, and
protein-specific commands remain explicit subcommands.

### First release slice

| Subcommand | Canonical historical assets | Initial compatibility oracle |
|---|---|---|
| `stats` | `fasta-stats`, `fastq-stats`, N50 edge fixtures | SeqKit |
| `kmers` | `fasta-utils` k-mer operation | independent byte-window oracle; Jellyfish comparison before release |
| `grep` | `seq-grep`, selected utils fixtures | SeqKit |
| `convert` | `fastx-convert`, `fasta-fx2tab` | SeqKit and format goldens |
| `validate` | `fasta-validate`, `fastq-validate` tests | strict format behavior |

This slice deliberately exercises FASTA/FASTQ, compression, stdin/stdout,
multi-subcommand help, JSON, and error propagation before broader migration.
`kmers` moves forward from the later generic-analysis assets because it is the
smallest user-recognizable operation that makes the pilot a real joint
consumer of `rsomics-seqio` and `rsomics-kmer`. It retains exact counting,
skips ambiguity-bearing windows, constrains `k` to the foundation's checked
`1..=32` representation, and adds an explicit canonical mode. This does not
promote a new public crate or make k-mer counting product-independent policy.

The complete first slice is published from `omics-rust/rsomics-seq` revision
`81c1e03981e2`. Exact-head CI run `30724863360` passes on native Linux and
macOS for both `x86_64` and `aarch64`, including strict Clippy, rustdoc,
package verification, 45 tests, six live SeqKit 2.13.0 differentials, an
independent ordered k-mer oracle, five benchmark smoke tests, and a standard
stream composition across `grep`, `convert`, `validate`, and `stats`.

The representative Linux `x86_64` gate covers all five commands. On 6,282,141
compressed SRR341550 reads, `stats`, ID grep, double-strand sequence grep,
FASTQ-to-FASTA, and FASTQ normalization are byte-identical to SeqKit 2.13.0.
A 100,000-read subset contributes 8.1 million candidate 21-mer windows; all
104,521 emitted canonical count rows are byte-identical to Jellyfish 2.3.1.
Strict validation rejects truncated quality after a valid prefix and does not
commit its named output.

`stats` and double-strand sequence grep are respectively 1.32 and 1.82 times
faster than SeqKit on the measured host. Conversion is throughput-neutral or
12% slower but uses 68–91% less peak RSS. Exact k-mer counting is 1.52 times
slower than the matched Jellyfish count/dump/sort pipeline but uses 63% less
peak RSS. These are operation-specific throughput/resource decisions, not a
blanket replacement claim.

The former shared `--threads` flag did not scale the current streaming
sequence operations and has been removed rather than justified with
speculative parallel code. The command tree now exposes only the shared JSON
output option. Exact distributions, RSS, commands, checksums, release
identities, and the remaining platform limitation are in the
[representative product gate](seq-gate-2026-07-30.md). The final API and
hot-path review removed the private output transaction and adopted the shared
alias and atomic-output contracts. Common, help, seqio, and kmer resolve from
published registry releases. Linux `aarch64` has native correctness CI but no
representative performance host.

### Later operation groups

| Group | Target operations | Primary source assets |
|---|---|---|
| generic transforms | sample, shuffle, sort, split, head, rename, revcomp, wrap, case conversion, subseq, interleave | `fasta-utils`, `fastq-utils`, `fastq-sample`, `fasta-subseq` |
| motif and feature search | locate, amplicon, palindrome, ORF | `fasta-locate`, `fasta-amplicon`, `palindrome`, `fasta-orf` |
| translation and coding | translate, codon usage, CAI, Nc, GC3 | `fasta-translate`, `cusp`, `cai`, `chips`, `gc123` |
| composition and physical properties | GC windows/skew, molecular weight, melting temperature, protein properties | `gc-windows`, `gc-skew`, `molecular-weight`, `tm-nn`, `prot-param` |
| alignment and consensus | global/local alignment, consensus | `align-score`, internalized `align-core`, consensus fixtures |

### Asset dispositions

- Direct merge after namespace and I/O adaptation: `aa-code`, `cai`, `chips`,
  `cusp`, `fasta-amplicon`, `fasta-fx2tab`, `fasta-locate`, `fasta-orf`,
  `fasta-sliding`, `fasta-stats`, `fasta-subseq`, `fasta-translate`,
  `fastq-stats`, `gc-skew`, `gc-windows`, `gc123`, `molecular-weight`,
  `palindrome`, `prot-param`, `seq-grep`, and the `tm-nn` algorithm.
- Refactor before merge: `align-score`, `fasta-digest`, both utils suites,
  both validators, `fastq-sample`, and `fastx-convert`.
- Evidence only or superseded: `fasta-n50`, `fastq-downsample`,
  `motif-scan`, and `seq-stats`.
- Rewrite: consensus semantics and traceback compatibility require new
  validation rather than inheriting current claims.

`rsomics-seqstats` is not retained as a public foundation. Statistics specific
to this product remain internal until a second product requires the same typed
contract.

## `rsomics-fastq-preprocess`

### Boundary

Read preprocessing over a shared single-end/paired-end chunk engine. Individual
operations remain callable, while `run` composes transforms in one
decompression/traversal/compression pass.

### First release slice

- `run`
- `trim`
- `filter`

The initial source assets are `rsomics-fastq-trim`,
`rsomics-fastq-filter`, `rsomics-fastq-quality`, and
`rsomics-fastq-complexity`.

### Internal architecture

```mermaid
flowchart LR
    reader["seqio reader"] --> chunks["SE/PE chunk engine"]
    chunks --> transforms["typed record transforms"]
    transforms --> writer["seqio writer"]
    transforms --> report["typed metrics and JSON"]
```

Trim, filter, complexity, UMI, and deduplication logic are pure transforms
where possible. Merge, pairing, correction, and reference-based
decontamination may own specialized stages but share the same record and I/O
contracts.

### Asset dispositions

- Direct merge after I/O extraction: `fastq-filter`, `fastq-merge`,
  `fastq-split`, `fastq-trim`, and `fastq-umi`.
- Refactor before merge: `bbduk`, `fastq-complexity`, `fastq-correct`,
  `fastq-dedup`, `fastq-pair`, and `fastq-quality`.
- Evidence only: the old `fastp` CLI, JSON, and fixtures. Its duplicate
  implementation is replaced by the stronger operation modules.

Correction and BBDuk-style filtering ship only after adversarial
compatibility and representative hot-path benchmarks. Existing small
subprocess benches are not release evidence.

The initial `run`, `trim`, and `filter` slice is published from
`omics-rust/rsomics-fastq-preprocess` revision `755cd715276b`. It combines
fixed, poly-G/poly-X, quality, N-content, length, and complexity transforms
over one ordered engine for single-end and paired-end reads. Exact-head CI run
`30726427849` passes on native Linux and macOS for both `x86_64` and
`aarch64`, including formatting, strict Clippy, rustdoc, clean package
verification, 52 debug and release tests, four live fastp 1.3.6
differentials, and a benchmark smoke test.

The product privately internalizes the useful `rsomics-fqgz` algorithm instead
of reviving that historical micro-foundation. `rsomics-seqio` still validates
and serializes every record; ordered 256 KiB gzip members are compressed by
libdeflate through the command's local Rayon pool and committed by the existing
two-output transaction.

On provenance-checked SRR341550 input, decompressed single-end and paired
outputs are byte-identical to the aligned fastp slice. On Ubuntu 22.04 /
Linux 6.8 `x86_64`, four-thread paired output measures
`10.914 ± 0.493 s` and 31.5 MiB peak RSS versus fastp's
`14.690 ± 0.715 s` and 99.2 MiB: 1.35 times the throughput with 68% less
peak memory. Single-end output is not a throughput win on that host:
`5.969 ± 0.431 s` versus `5.503 ± 0.862 s`, but peak RSS is 18.0 MiB versus
51.1 MiB.

The final API review made public record transforms fallible on malformed
caller input, tied pipeline operation labels to typed constructors, retained a
validation-free parsed hot path, and verified `trim | filter` stream
composition against `run`. Common, help, and seqio resolve from their
published registry releases.

Crates.io publication run `30726551865` produced non-yanked 0.1.0 archive
checksum `d12cb432e56fdeb151e91c80804301089a8b5716e07dd922b347024b9c82c016`
with VCS identity `755cd715276b`. An independent registry download,
`cargo install --locked`, help check, identity stream, and malformed-input
smoke all passed. Exact machine, fixture, compatibility, API, performance, and
raw-result evidence is recorded in the
[product gate](fastq-preprocess-gate-2026-07-30.md).

Patch release 0.1.1 at `89d4f534f90e` replaced the product-local duplicate
thread argument with `rsomics-common::ThreadArgs` while retaining the local
Rayon pool and product scheduling policy. Exact-head CI run `30731874951` and
publish run `30732312288` passed; no preprocessing behavior or public product
boundary changed.

## `rsomics-fastq-qc`

### Boundary

FastQC-style diagnostics and reports. Generic counts and length statistics
belong to `rsomics-seq stats`; preprocessing reports belong to
`rsomics-fastq-preprocess`.

The released `report` command accepts multiple plain, gzip, or BGZF FASTQ
inputs and writes one report directory per input. Each directory contains
FastQC/MultiQC-compatible `fastqc_data.txt`, a status summary, and a
self-contained rsomics HTML report. BAM/SAM, Casava grouping, Nanopore
`fast5`, custom limits, custom adapter and contaminant lists, and FastQC ZIP
packaging remain explicit exclusions.

The historical `rsomics-fastqc` repository was refactored rather than revived.
Its analyzers and fixtures were classified module by module; the per-read
trait-object dispatch, parallel mutable module state, JSON-only output,
placeholder thread flag, and inherited compatibility claims were discarded.
The product now uses a typed analyzer pipeline, real multi-file parallelism,
strict `rsomics-seqio` parsing, shared `rsomics-common` thread and result
contracts, and `rsomics-help` for the CLI presentation layer.

Full module data is byte-identical to FastQC 0.12.1 on both 6,282,141-read
SRR341550 mates. Controlled edge grids additionally freeze status thresholds,
length grouping, the 100,000-unique-sequence duplication sampler, numeric
formatting, and ordering. Exact-head CI run `30733371110` passes lint,
documentation, package verification, debug/release tests, the live FastQC
oracle, and benchmark smoke on native Linux and macOS for both `x86_64` and
`aarch64`.

The recorded Apple M2 end-to-end gate measures `16.563 ± 0.063 s` and
43.0 MiB peak RSS versus FastQC's `25.003 ± 0.888 s` and 624.5 MiB. The
rsomics workflow is 1.51 times faster with 93.1% less peak resident memory on
that host. Report packaging differs: FastQC writes separate image assets,
whereas rsomics embeds SVG charts in its HTML.

Crates.io publication run `30733480364` produced non-yanked 0.1.0 archive
checksum `979cf2d2340c8d4b6db2eab342cdefe526191a3a01a908a2b7554e9646eeb08b`
with VCS identity `f24f0e1766d1`. An independent registry download,
`cargo install --locked`, help check, and report-generation smoke passed.
Complete identities, compatibility, transaction, performance, and publication
evidence are recorded in the
[product gate](fastq-qc-gate-2026-08-02.md).

## Origin and reuse

The historical Rust implementations are team-owned and may be merged directly.
That does not promote inherited README claims to verified facts. Target
products retain exact upstream names, versions, black-box commands, public
format specifications, papers where relevant, and third-party dependency
licenses.

In particular, FastQC “exact” claims, incomplete digest enzyme semantics, and
the old FASTQ validator's parser behavior must be re-established by tests
before documentation or release.
