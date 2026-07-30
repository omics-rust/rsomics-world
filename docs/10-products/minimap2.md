# Minimap2 product dossier

Status: source and upstream-operation audit complete. Published version 0.1.0
is a legacy proof of FFI linkage, not the implementation baseline for a new
release.

## Boundary

`rsomics-minimap2` is one sequence-alignment product embedding the upstream
minimap2 engine. The recognized installation identity is minimap2-compatible
genomic, spliced, overlap, and assembly alignment. Index construction and
alignment are its two user operations.

The primary behavior sources are:

- [minimap2 2.31](https://github.com/lh3/minimap2/releases/tag/v2.31), its
  [user guide](https://github.com/lh3/minimap2#users-guide), and man page;
- the [minimap2 method](https://doi.org/10.1093/bioinformatics/bty191);
- [minimap2-rs 0.1.31](https://docs.rs/minimap2/0.1.31+minimap2.2.30/minimap2/)
  and its statically linked minimap2 2.30 engine;
- FASTA, FASTQ, MMI, PAF, SAM, BAM, CRAM, BED12, and relevant tag
  specifications.

This product is not a commitment to reimplement minimap2's algorithms in pure
Rust. The embedded MIT-licensed engine remains the algorithm owner. The
rsomics layer is justified only if it preserves minimap2 behavior and adds a
material installation or composition benefit without material overhead.

## Operation map

| Target operation | minimap2 operation | Decision |
|---|---|---|
| `index` | `-d <index.mmi> <reference>` | build a reusable MMI with explicit preset and index parameters |
| `align` | default mapping and base-level alignment | map one or two query streams against FASTA or MMI and emit PAF, SAM, or direct alignment formats |

Read-overlap, assembly-to-assembly, short-read, and spliced modes are presets of
`align`, not separate public crates or subcommands.

### Preset surface

The stable surface covers the pinned engine's complete documented preset
families:

- `map-pb`, `map-ont`, `map-hifi`, `lr:hq`, and `map-iclr`;
- `sr`;
- `splice`, `splice:hq`, and `splice:sr`;
- `ava-pb` and `ava-ont`;
- `asm5`, `asm10`, and `asm20`.

Unknown or engine-incompatible presets fail. They never fall back to
`map-ont`.

### Option groups

The command model groups minimap2 controls by stable meaning:

- index and seeding: k-mer, window, homopolymer compression, batch size, and
  split indexes;
- chaining and mapping: occurrence filters, bandwidth, gap and chain scores,
  secondary alignment policy, and seed;
- base alignment: scoring, end bonus, z-drop, CIGAR, MD, and `cs` tags;
- splicing: orientation, intron length, splice flank, junction BED, two-pass
  junctions, and junction bonus;
- inputs: FASTA/FASTQ, gzip, stdin, paired files, interleaved reads, reference
  FASTA, and MMI;
- outputs: PAF, PAF with CIGAR, SAM, direct BAM where justified, header and
  read-group metadata, long CIGAR, and named or standard output;
- execution: mapping threads, index threads, I/O buffering, and verbosity.

Low-level options are not copied blindly into a second parser. Each exposed
item must map exactly to the pinned engine and have a differential. An explicit
compatibility escape hatch may pass documented minimap2 arguments only if it
does not bypass rsomics validation or make help inaccurate.

`paftools.js`, `mappy`, variant calling, assembly graph construction, and
post-alignment sorting or indexing are outside this product.

## Product value gate

A same-engine wrapper is not performance-exempt. Before another release,
`rsomics-minimap2` must demonstrate at least one material benefit while keeping
mapping results compatible:

- a reproducible static Cargo installation across all four native target
  classes;
- direct transactional BAM output through `rsomics-bamio` that improves the
  composed `minimap2 | samtools view` workflow;
- or a bounded streaming Rust API used by a concrete rsomics product without
  intermediate sequence or alignment files.

Unified help alone does not justify maintaining a less capable second CLI. If
none of these benefits survives compatibility, throughput, memory, and
maintenance review, the correct decision is to keep upstream minimap2 as an
external dependency and retire the rsomics product after a separate public
boundary decision.

## Execution model

- Reference FASTA and MMI paths are distinguished before engine construction.
  Index parameters stored in MMI cannot be silently overridden while mapping.
- Query FASTA and FASTQ records stream through `rsomics-seqio`, including gzip,
  stdin, wrapped records, qualities, paired files, and interleaved input.
- Record names and comments preserve the exact minimap2 contract. Sequence
  parse, encoding, pairing, and output failures propagate.
- The engine version, binding version, compile features, SIMD backend, preset,
  index parameters, and thread counts are available in the execution report.
- PAF contains the complete required fields and requested optional tags. SAM
  contains a valid header, flags, CIGAR, sequence, qualities, and requested
  tags.
- Standard output remains streamable. Named SAM, BAM, and other file outputs
  use transactional replacement and reject aliases.
- Mapping order and deterministic differences across thread counts are
  documented and tested according to upstream behavior.
- SIMD and compile-time features are selected explicitly for x86-64 and
  aarch64; an accidental portable or SSE2-only build cannot define a release
  performance result.

## Target structure

```text
src/
├── cli.rs
├── engine/
│   ├── version.rs
│   ├── index.rs
│   ├── preset.rs
│   └── options.rs
├── input/
│   ├── reference.rs
│   └── query.rs
├── align.rs
├── output/
│   ├── paf.rs
│   ├── sam.rs
│   └── bam.rs
└── report.rs
```

`rsomics-common` owns errors, exit mapping, output modes, reports, and
transactions. `rsomics-help` owns the authoritative command tree.
`rsomics-seqio` supplies strict streaming query records.

Direct BAM output makes `rsomics-minimap2` a concrete planned consumer of
validated writer contracts in `rsomics-bamio`. Minimap2 scoring, indexing,
mapping options, PAF formatting, and engine ownership do not enter any
foundation. `rsomics-kmer` is not used to replace the embedded minimizer
engine.

## Historical asset disposition

The one routed source candidate is the clean `rsomics-minimap2` repository at
`1a47a36f2af0ab51f87e7881cfc122c48e898af5`.

| Asset | Disposition |
|---|---|
| minimap2-rs engine construction | dependency and build recipe only; repin after 2.31/API review |
| four-preset conversion | discard; replace with an exhaustive typed preset map and fatal unknown values |
| handwritten FASTA loader | discard; use `rsomics-seqio` streaming FASTA/FASTQ |
| PAF first-12-field writer | tests and fixture asset only; use a complete fallible formatter |
| CLI and duplicated `HelpSpec` | discard through the current common/help layer |
| tiny FASTA fixtures | smoke and differential seed only |
| live PAF12 comparison | retain as one baseline, then expand to full records and SAM |
| Criterion subprocess benchmark | discard as a performance gate; it measures only tiny process startup |

The published crate resolves `minimap2`
`0.1.31+minimap2.2.30`, while the current upstream release is minimap2 2.31.
A loose `minimap2 = "0.1"` dependency is insufficient release provenance.

## Existing implementation gaps

The current 195-line wrapper is much narrower than its name and metadata imply.

- Only `sr`, `map-ont`, `map-hifi`, and `map-pb` are recognized. Every other
  string silently selects `map-ont`.
- Query input is read wholly as UTF-8 and split with a handwritten FASTA-only
  parser. FASTQ, gzip, stdin, pairs, qualities, comments, malformed records,
  and bounded memory are absent.
- Every run rebuilds an index from FASTA. MMI input and index output are absent.
- Threads from the inherited common flags are unused.
- Only the first 12 PAF fields are emitted. Optional minimap2 tags, base-level
  PAF, SAM, paired flags, read groups, and direct BAM are absent.
- Formatting errors are discarded through `.ok()`.
- Reference path conversion can become an empty string on non-UTF-8 paths.
- Query names, lengths, wrapping, and sequence symbols are not validated
  against upstream parsing.
- The differential uses one tiny `map-ont` fixture, sorts only 12 PAF fields,
  and skips when minimap2 is absent.
- No representative performance or memory record exists. The historical FFI
  exemption is rejected by the current product rules.
- CI covers only Linux `x86_64`; metadata still points to the old control plane;
  no README or user contract is present.

## Retained evidence

The only useful existing evidence is a tiny live differential showing that
sorted first-12-field PAF records can match the minimap2 CLI for one
`map-ont` fixture. Because both tools use the same underlying engine, this
establishes neither complete compatibility nor product value.

There is no representative timing, CPU, RSS, index, streaming, paired-read,
spliced, SAM, or multi-platform record. The previous `EXEMPT` verdict is not a
release gate.

## First release slice

The first releasable slice contains:

- `index` for FASTA to MMI;
- `align` from FASTA or MMI with streaming FASTA/FASTQ, gzip, stdin, one or two
  query streams, and the complete preset set supported by the pinned engine;
- complete PAF, PAF+CIGAR, and SAM;
- direct BAM only if its composed-workflow value gate passes;
- mapping and indexing threads that reach the engine;
- the most used seed, chaining, alignment, secondary, splicing, junction,
  read-group, CIGAR, MD, and `cs` controls;
- exact engine and build provenance, unified help, strict failure, and
  transactional named output.

Unimplemented minimap2 options stay absent. There is no generic placeholder or
claim of full CLI parity until the option ledger and differentials are
complete.

## Compatibility gates

- Pin upstream minimap2 2.31 and an embedding binding that contains the same
  engine revision. Record the vendored source commit and build features.
- Differential-test every stable preset with official minimap2 on ONT, PacBio
  CLR, HiFi, ICLR, short single and paired reads, long and short spliced reads,
  read overlaps, and assembly pairs.
- Compare complete PAF records and optional tags. Compare SAM headers, flags,
  coordinates, CIGAR, mate fields, sequence, quality, and tags after
  normalizing only tool-identification lines and documented thread ordering.
- Exercise FASTA, FASTQ, gzip, stdin, wrapped records, comments, empty input,
  ambiguous bases, malformed records, paired-name rules, interleaving, and
  output write failure.
- Exercise fresh and split indexes, MMI reuse, incompatible index parameters,
  junction BED, two-pass splicing, long CIGAR, secondary and supplementary
  alignments, and all exposed scoring controls.
- Verify x86-64 SIMD, aarch64 NEON or SIMDe, and exact-head native CI on Linux
  and macOS for both architectures.
- Run ABI and sanitizer checks around the FFI boundary on supported native
  targets.

## Performance gates

- Compare the exact embedded engine against the matching official minimap2
  build, not a different release or SIMD configuration.
- Measure reference indexing, MMI loading, query parsing, mapping, formatting,
  and total execution separately where possible.
- Use representative human-scale reference and ONT, HiFi, paired short-read,
  spliced-read, overlap, and assembly workloads.
- Compare equal mapping and I/O thread counts and record wall distribution,
  total CPU, peak RSS, output size and hashes, and index size.
- For PAF and SAM, quantify wrapper overhead and require no material regression
  together with an independently demonstrated product benefit.
- For direct BAM, compare against the complete
  `minimap2 | samtools view -b` pipeline with identical records, compression,
  threads, and output.
- Record exact revisions, binary hashes, build flags, commands, fixture hashes,
  warmups, machine, and SIMD features.
- Do not publish a same-engine wrapper on the expectation that it is
  automatically fast enough.

## License and attribution

Minimap2 is MIT licensed. minimap2-rs is MIT OR Apache-2.0 and statically links
the upstream engine. `rsomics-minimap2` remains MIT OR Apache-2.0 while
retaining all required notices, upstream version and commit, binding version,
paper citation, and compile-feature provenance in source and release
artifacts.

## Explicit exclusions

- No pure-Rust minimap2 rewrite as a prerequisite.
- No unknown-preset fallback.
- No handwritten whole-file FASTA parser.
- No PAF12-only claim of minimap2 compatibility.
- No stale engine or loosely resolved FFI dependency in a release.
- No FFI-based performance exemption.
- No `paftools.js`, mappy, variant calling, sorting, indexing of alignment
  output, or assembly graph workflow.
- No next release until the material product-value gate passes.
