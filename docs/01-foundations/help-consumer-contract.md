# `rsomics-help` consumer contract

Status: help 0.4.0, common 0.8.0, intervals 0.3.0, and seqio 0.3.0 are
published and verified from downloaded crates.io archives. Product command
trees consume the registry versions established by their exact lockfiles.

## Role

`rsomics-help` is the family-wide CLI interaction and presentation layer. It
owns:

- help and version presentation;
- terminal and `NO_COLOR` behavior;
- argument-error styling, usage, and suggestions;
- top-level and nested help navigation;
- the visual grammar for headings, literals, placeholders, defaults, and
  invalid values.

`rsomics-common` owns typed runtime errors, exit-code mapping, JSON result
envelopes, and the product runner. Product crates own their domain arguments,
semantic help text, meaningful argument groups, and execution policy.

## Single source of truth

The product's `clap::Command` tree is authoritative for names, subcommands,
arguments, aliases, defaults, requirements, conflicts, value hints, usage, and
descriptions. `rsomics-help` decorates and parses that tree recursively.

The 0.3 `HelpSpec`, `FlagSpec`, manual argv interception, figlet renderer, and
parallel JSON help model are removed. They duplicated the parser and could not
preserve nested-command semantics.

The normal Layer-B call is:

```rust
let cli = rsomics_help::parse::<Cli>();
```

The product continues to use normal Clap derive or builder APIs. No adapter
type, duplicated flag table, registration macro, or product-specific renderer
is required.

## Presentation contract

- Interactive terminals use one restrained rsomics palette.
- Redirected help and `NO_COLOR` output contain no ANSI escapes.
- Help and version retain successful Clap exits.
- Invalid arguments retain exit status 2, contextual usage, and suggestions.
- A `help` subcommand navigates the same tree used for parsing.
- Ordinary positional values equal to `help` are not intercepted.
- Help width is terminal-aware and capped for readability.

Simple products may use the standard `Arguments`, `Options`, `Commands`, and
`Global options` sections. Complex products group their real Clap argument
types by user task. For example, the FASTQ pilot declares `Input/output`,
`Trimming`, `Filtering`, and `Length filtering` headings on its existing
flattened argument structs. The foundation does not infer domain groupings.

Examples and longer explanations use Clap `long_about` and `after_help`.
Structured command export is deferred until a concrete consumer needs it; any
future representation must be derived from the same `Command` tree.

## Product evidence

Each product keeps registry-compatible manifests. Common and help resolve from
crates.io; only still-unpublished domain foundations use exact CI patches:

| Product | Command shape | Verified evidence |
|---|---|---|
| `rsomics-seq` `bf00b71477b8` | five sequence subcommands | strict Clippy; 7 library, 4 binary, 26 CLI, one independent k-mer oracle, and six live SeqKit tests; five benchmark smokes; CI `30598213179` |
| `rsomics-fastq-preprocess` `a56519d9d6c0` | three subcommands with nested input, trim, filter, length, thread, and output groups | strict Clippy; 18 library, 4 binary, 21 CLI, and four live fastp tests; benchmark smoke; CI `30598213737` |
| `rsomics-bed` `989894f2dad5` | five interval subcommands with positional and required named inputs | strict Clippy; 40 library, 12 CLI, and three live bedtools/golden tests; full benchmark smoke and representative million-record gate; CI `30621067404` |
| `rsomics-vcf` `84e27f734911` | nested `head` and `query` operations with unified global output | strict Clippy; typed VCF/BGZF/BCF tests; pinned bcftools 1.24 command oracles; benchmark smoke; CI `30622684140` |

The foundation itself passes strict Clippy, package verification, and six unit
tests covering nested help, generated help navigation, suggestions, normal
positional `help`, and derived-type construction. Commit `61dd6f2ce0ce` passes
exact-head CI `30596121607` on native Linux and macOS for both `x86_64` and
`aarch64`. The downloaded 0.4.0 archive passes the same six tests.

## Coordinated dependency boundary

The prototype demonstrated that `rsomics-common` cannot be upgraded only at a
Layer-B leaf when another foundation exposes its error types. Seqio and its
sequence consumers therefore remain on their tested common 0.7 contract.
Intervals exposes its own narrow construction error and does not depend on
common. BED and VCF independently moved to common 0.8 for the shared
transactional output contract:

```text
seq -> help 0.4
seq -> common 0.7 <- seqio

fastq-preprocess -> help 0.4
fastq-preprocess -> common 0.7 <- seqio

bed -> help 0.4
bed -> common 0.8
bed -> intervals 0.3

vcf -> help 0.4
vcf -> common 0.8
```

`rsomics-seqio` commit `7b5b1c68f52e` passes strict Clippy, 45 unit tests, five
compatibility tests, benchmark smoke, package verification, and exact-head CI
`30598214929` on all four native targets. Published `rsomics-intervals 0.3.0`
contains only the validated generic coordinate model and passes eight unit
tests plus exact-head CI `30597681539`.

## Release order

1. Help 0.4.0, common 0.8.0, intervals 0.3.0, and seqio 0.3.0 are published
   and verified.
2. The `seq`, `fastq-preprocess`, `bed`, and `annotation` lockfiles resolve
   their reviewed versions from crates.io and retain their exact-head gates.
3. VCF resolves help 0.4 and common 0.8 from crates.io; kmer remains behind a
   second product consumer.
4. Use these consumer contracts as the default CLI baseline for later product
   reconstruction.
