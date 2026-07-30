# `rsomics-help` consumer contract

Status: 0.4 foundation implementation committed and exact-head CI verified;
three-product migration remains local-only and nothing has been published.

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

## Prototype evidence

The prototype was tested on 2026-07-30 with local Cargo patches only:

| Product | Command shape | Verified evidence |
|---|---|---|
| `rsomics-seq` | five sequence subcommands | strict Clippy; 7 library, 4 binary, 25 CLI, one independent k-mer oracle, and six live SeqKit tests; five benchmark smokes |
| `rsomics-fastq-preprocess` | three subcommands with nested input, trim, filter, length, thread, and output groups | strict Clippy; 18 library, three binary, 21 CLI, and four live fastp tests; benchmark smoke |
| `rsomics-bed` | five interval subcommands with positional and required named inputs | strict Clippy; 38 library, 12 CLI, and three live bedtools/golden tests; full benchmark smoke |

The foundation itself passes strict Clippy, package verification, and six unit
tests covering nested help, generated help navigation, suggestions, normal
positional `help`, and derived-type construction. Commit `c615aa8b8522` passes
exact-head CI on native Linux and macOS for both `x86_64` and `aarch64`.

## Coordinated dependency boundary

The prototype also demonstrated that `rsomics-common` cannot be upgraded only
at a Layer-B leaf when another foundation exposes its error types. Temporary
0.7 alignment produced one common version in each tested graph:

```text
seq -> help 0.4
seq -> common 0.7 <- seqio

fastq-preprocess -> help 0.4
fastq-preprocess -> common 0.7 <- seqio

bed -> help 0.4
bed -> common 0.7 <- intervals
```

`rsomics-seqio` passed strict Clippy, 45 unit tests, five compatibility tests,
and its benchmark smoke after a dependency-only common version change.
`rsomics-intervals` passed strict Clippy, 48 unit tests, and six property tests
after the same change.

## Release order

1. Review and commit the `rsomics-help` 0.4 API and `rsomics-common` 0.7 API as
   separate concerns.
2. Align `rsomics-seqio` and `rsomics-intervals` with common 0.7 and verify
   their exact-head four-native-target CI.
3. Publish the foundations only after package review and registry credentials
   are explicitly available.
4. Migrate `seq`, `fastq-preprocess`, and `bed` without committed path patches;
   keep their command-tree, help, error, compatibility, and benchmark tests.
5. Use these consumer contracts as the default CLI baseline for later product
   reconstruction.
