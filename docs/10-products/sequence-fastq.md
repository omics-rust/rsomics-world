# Sequence and FASTQ product dossier

Status: source audit complete; first release slices selected; `rsomics-seq`
and `rsomics-fastq-preprocess` have verified public repositories.

The source pool contains 47 relevant historical crates after routing
corrections: 34 for `rsomics-seq`, 12 for preprocessing, and one for QC. The
reuse unit is an algorithm module plus its evidence, not the old CLI shell.

## Shared design

The three products share `rsomics-seqio` for strict FASTA/FASTQ I/O.
`rsomics-common` supplies the execution and output contract used by the first
two implemented products. `rsomics-kmer` currently serves sequence k-mer
operations; it is not a preprocessing dependency.

The first end-to-end gate is:

```mermaid
flowchart LR
    raw["raw single-end or paired FASTQ"] --> prep["fastq-preprocess run"]
    prep --> qc["fastq-qc report"]
    qc --> stats["seq stats"]
```

The same versioned inputs are compared against fastp, FastQC, and SeqKit.
Compatibility, wall-clock distribution, peak memory, compression mode, and
thread count are recorded together.

`rsomics-seqio` must provide:

- strict FASTA and FASTQ readers and writers;
- plain, gzip, and BGZF detection by content;
- path and stdin/stdout boundaries;
- borrowed streaming records plus owned batches for parallel stages;
- paired-read semantics;
- a private compression backend.

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

The complete first slice is implemented at `omics-rust/rsomics-seq` revision
`02f8268931b0`. Its exact-head CI passes on native Linux and macOS for both
`x86_64` and `aarch64`, including strict Clippy, 43 tests, six live SeqKit
2.13.0 differentials, an independent ordered k-mer oracle, and five benchmark
smoke tests.

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

The shared `--threads` flag does not scale the current streaming sequence
operations; compressed input uses a fixed decompressor/consumer pipeline.
That misleading CLI surface is a release API gate and is not being hidden with
speculative parallel code. Exact distributions, RSS, commands, checksums, and
remaining gates are in the
[representative product gate](seq-gate-2026-07-30.md). Publication remains
blocked on that CLI decision, unpublished foundation revisions, final API and
hot-path review, and unavailable native Linux `aarch64` performance evidence.

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

The initial `run`, `trim`, and `filter` slice is implemented at
`omics-rust/rsomics-fastq-preprocess` revision `8e483fc95556`. It combines
fixed, poly-G/poly-X, quality, N-content, length, and complexity transforms
over one ordered chunk engine for single-end and paired-end reads. Its
exact-head CI passes on native Linux and macOS for both `x86_64` and
`aarch64`, including strict Clippy, 42 internal/CLI tests, four live fastp
1.3.6 differentials, and a benchmark smoke test.

The product privately internalizes the useful `rsomics-fqgz` algorithm instead
of reviving that historical micro-foundation. `rsomics-seqio` still validates
and serializes every record; ordered 256 KiB gzip members are compressed by
libdeflate through the existing Rayon pool and committed by the existing
two-output transaction.

On provenance-checked SRR341550 paired input, decompressed output is
byte-identical to the aligned fastp slice. On Ubuntu 22.04 / Linux 6.8
`x86_64`, four-thread paired output measures `10.863 ± 0.298 s` and 31.5 MiB
peak RSS versus fastp's `13.891 ± 0.447 s` and 101.9 MiB. One-thread paired
output measures `22.308 ± 0.610 s` and 31.5 MiB versus `39.091 ± 0.894 s`
and 88.7 MiB. Single-end output is not a throughput win on that host:
four-thread time is `5.360 ± 0.075 s` versus fastp's `4.937 ± 0.721 s`, but
peak RSS is 19.6 MiB versus 52.9 MiB. The compressed files are about 0.07%
larger than fastp's, pass `gzip -t`, and are consumed by SeqKit and fastp.

This is a strong product checkpoint, not publication approval: the
unpublished foundation revisions, end-to-end QC handoff, final API review, and
release-level performance decision remain. Exact machine, fixture, causal
control, interoperability, and raw-result checksums are recorded in the
[parallel-gzip product gate](fastq-preprocess-gate-2026-07-30.md).

## `rsomics-fastq-qc`

### Boundary

FastQC-style diagnostics and reports. Generic counts and length statistics
belong to `rsomics-seq stats`; preprocessing reports belong to
`rsomics-fastq-preprocess`.

The historical `rsomics-fastqc` implementation is a refactor seed, not a
finished compatibility claim. Its twelve analyzers are retained, but the
product requires:

- a typed analyzer pipeline without per-read trait-object dispatch;
- one or more fixtures that trigger PASS, WARN, and FAIL for every module;
- full module-data comparison, not summary status alone;
- documented long-read binning and duplication-curve behavior;
- complete text/HTML report compatibility rules;
- single-thread and multi-thread performance evidence.

The first slice is `report --format fastqc`, integrated with the new
`rsomics-seqio`.

## Origin and reuse

The historical Rust implementations are team-owned and may be merged directly.
That does not promote inherited README claims to verified facts. Target
products retain exact upstream names, versions, black-box commands, public
format specifications, papers where relevant, and third-party dependency
licenses.

In particular, FastQC “exact” claims, incomplete digest enzyme semantics, and
the old FASTQ validator's parser behavior must be re-established by tests
before documentation or release.
