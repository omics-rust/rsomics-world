# Liftover product dossier

Status: `rsomics-liftover 0.1.0` published and independently verified. The
production hot path was measured at `db576170a48b3b5762fd128890303298c6b2fa3e`;
the published VCS identity is `597220ee2ed1102360bc3ab15646e2c9067a1a84`.

## Boundary

`rsomics-liftover` is one product for translating genomic records between
assemblies through a UCSC chain. The installation identity is coordinate
conversion, while each format remains a separate subcommand because its
mapping, rejection, reference, and output contracts differ materially.

The primary behavior sources are:

- the [UCSC chain format](https://genome.ucsc.edu/goldenPath/help/chain.html)
  and the `liftOver` command-line program for interval compatibility;
- [CrossMap 0.7.4](https://crossmap.readthedocs.io/en/latest/) for the
  multi-format product surface;
- [Picard LiftoverVcf](https://broadinstitute.github.io/picard/command-line-overview.html#LiftoverVcf)
  and the
  [BCFtools/Liftover method](https://doi.org/10.1093/bioinformatics/btae038)
  for variant-specific allele semantics.

The product does not treat every record as a three-column interval. BED,
variants, alignments, continuous signal, and hierarchical annotation require
different correctness models even though they share the chain parser and
coordinate mapper.

## Upstream operation map

| Target operation | Upstream operations | Decision |
|---|---|---|
| `bed` | UCSC `liftOver` default mode; CrossMap `bed` and `region` | BED3-6 and BED12 conversion with explicit contiguous, split, block, and multiple-mapping policies |
| `position` | UCSC `-positions`; UCSC web position input | compact single-position and region text input without pretending it is BED |
| `vcf` | CrossMap `vcf` and `gvcf`; Picard `LiftoverVcf`; BCFtools/Liftover | one variant-aware command with target-reference validation, allele transformation, normalization, and rejected VCF |
| `alignment` | CrossMap `bam` for SAM, BAM, and CRAM | one alignment command that updates headers, mates, flags, and template lengths |
| `signal` | CrossMap `wig` and `bigwig` | Wiggle, bedGraph, and BigWig conversion with explicit overlap aggregation |
| `annotation` | UCSC `-gff`; CrossMap `gff` | hierarchy-aware GFF3/GTF conversion, not independent line lifting |
| `maf` | CrossMap `maf` | Mutation Annotation Format conversion; not UCSC multiple-alignment MAF |
| `chain validate` | chain specification | strict syntax, bounds, block-total, strand, identifier, and ordering validation |
| `chain inspect` | CrossMap `viewchain` | normalized block-to-block table and chain summary |

UCSC `-genePred`, `-sample`, `-pslT`, `-bedPlus`, and `-hasBin` are
compatibility formats, not public crate boundaries. They remain deferred until
a real rsomics workflow needs them. `-chainTable` depends on a UCSC database
deployment and is excluded.

CrossMap's `region` does not become a second generic interval command. Its
whole-region behavior is a named mapping policy of `bed`. CrossMap `gvcf` is a
mode of `vcf`, and its Wiggle and BigWig operations share `signal`.

## Chain and mapping model

- Chain target coordinates describe the old assembly and query coordinates
  describe the new assembly.
- Coordinates are checked 0-based half-open integers. Query minus-strand
  blocks are projected through the declared query size.
- The parser accepts plain or gzip chain streams and rejects malformed headers,
  block rows, unexpected strands, arithmetic overflow, out-of-bounds spans,
  incorrect terminal totals, duplicate identifiers, and impossible ordering.
- A chain index is built by source sequence and source span. File order is
  retained only where a pinned compatibility rule requires it.
- Mapping returns typed candidates and rejection causes. BED, VCF, alignment,
  signal, and annotation policy is not embedded in the chain index.
- Minimum-match comparison, multiple mappings, target gaps, query gaps,
  reverse mappings, ties, and chain-size filters receive exact boundary tests.
- Output aliases are rejected before work begins. Mapped and rejected outputs
  are staged and committed as one transaction.
- Chain direction remains an explicit user input. The crate does not download
  or redistribute assembly chain files.

## Target structure

```text
src/
├── bed.rs
├── bed12.rs
├── chain.rs
├── cli.rs
├── io.rs
├── lib.rs
├── main.rs
├── mapping.rs
└── transaction.rs
```

Format modules are added only with a complete release slice. Empty modules and
advertised placeholder subcommands are forbidden.

The chain parser, index, and mapping engine remain product-private.
`rsomics-common` owns errors, exit mapping, input aliases, execution reports,
and single-output plumbing. `rsomics-help` owns CLI parsing and presentation.
`rsomics-intervals` supplies the checked half-open interval type used by the
BED paths. The liftover-specific two-output commit, chain ordering, and
candidate policy remain private because they have no second product consumer.

Future alignment support is a named consumer of `rsomics-bamio` alongside
`rsomics-bam`. A public variant, signal, or annotation foundation is not
created speculatively.

## Historical asset disposition

The one routed source candidate is the clean `rsomics-liftover` repository at
`d7948c5f8753e27ae7aa770945ce33e307c09983`.

| Asset | Disposition |
|---|---|
| plus- and minus-strand block projection | refactor then merge behind checked coordinate types |
| best-chain traversal | algorithm asset only; replace tie, multiple-map, span, and rejection policy |
| chain parser | discard and rewrite; it can panic, ignores malformed tokens, omits required validation, and materializes the whole file as UTF-8 |
| BED reader and writer | discard and rewrite; malformed records are silently skipped and BED12 semantics are absent |
| CLI and output creation | discard and rewrite through `rsomics-help` and transactional `rsomics-common` APIs |
| small synthetic chain and BED fixture | retain as a live-oracle golden seed |
| Criterion subprocess benchmark | benchmark recipe only; replace the tiny fixture and record direct upstream measurements |

No retired micro-crate needs revival. The target repository remains the owner
of the selected code and history.

## First-release implementation

The historical 302-line path was replaced rather than extended in place. The
current implementation provides:

- strict streaming plain/gzip chain parsing with checked header, block,
  identifier, span, terminal-total, UTF-8, and overflow failures;
- typed chain candidates and UCSC-compatible rejection causes;
- BED3-6 and BED12 conversion, including reverse chains, gaps, zero-length
  coordinates, minimum-match and minimum-block boundaries, multiple mappings,
  serial control, chain-size filters, name preservation, and thick bounds;
- buffered mapped and rejected output with alias rejection and paired durable
  commit behavior;
- `rsomics-help` CLI presentation and `rsomics-common` errors, reports, and
  exit mapping;
- internal invariants, committed goldens, malformed-input CLI tests, and live
  UCSC black-box differentials.

Production paths contain no inherited narrative comments or unchecked parser
operations. Future formats remain absent rather than exposed as placeholders.

## Retained evidence

The small committed goldens and live differentials are byte-identical to the
separately downloaded official macOS binaries. Exact-head CI pins macOS arm64
SHA-256
`0604b6ef4a0ae5dd56847950469fe000df7e206f235a05b3d1332e226823969d`
and macOS x86_64 SHA-256
`464a2850020a25b79d8931f0e9f7091b180333ec0579cdec88da69558ddff0f2`.

The superseded historical performance note reports 0.49 seconds for rsomics
and 1.12 seconds for UCSC on 300,000 intervals and a synthetic 2,000-block chain.
Neither the large fixture nor its generator is retained, and the note does not
record output hashes or timing distributions. The result is therefore a useful
recipe, not a release claim.

## First release slice

The implemented first release contains:

- `chain validate`;
- `chain inspect`;
- `bed` for strict BED3-6 and BED12;
- `--min-match`, `--min-blocks`, `--multiple`, `--preserve-input`, chain-size
  filters, and BED12 thick-bound handling where the pinned oracle supports
  them;
- plain and gzip chain input;
- mapped and rejected transactional outputs;
- unified help, error, and execution-report behavior.

`bed` must preserve all fields whose semantics are unchanged, update strand
and BED12 geometry correctly, and expose split behavior explicitly. It does
not silently choose CrossMap behavior where UCSC and CrossMap differ.

Variant conversion is the second slice because a coordinate-only VCF rewrite
would be actively misleading. It requires a target reference, correct
reverse-complement and REF/ALT handling, normalization, header updates,
symbolic-allele policy, gVCF reference-block behavior, rejected-record reasons,
and comparison with modern variant liftover oracles.

Alignment, signal, annotation, and MAF follow only after their format
foundations or private I/O and independent compatibility gates are ready.

## Compatibility gates

Exact-head CI run `30729541904` passed the stable slice on native Ubuntu 24.04
and macOS 15 for both `x86_64` and `aarch64`. The macOS jobs downloaded the
pinned official executable and required every live differential to run; Linux
oracle evidence is the separately recorded release benchmark below.

- Pin the official UCSC macOS and Linux binaries by download URL and hash
  because the executable has no useful version flag.
- Run live differentials for mapped and rejected outputs on both strands,
  target and query gaps, absent sequences, partial deletion, split mapping,
  ties, multiple mappings, threshold boundaries, chain-size filters, and
  preserved input.
- Cover BED3, BED4-6, BED12, extra columns, track and browser lines, comments,
  empty input, CRLF, malformed widths, invalid coordinates, integer limits,
  output aliases, and interrupted writes.
- Generate malformed chains for every header and block field and verify that
  no failure is converted to a skipped record.
- Use real assembly chains only as externally supplied oracle fixtures. Keep
  redistributable synthetic fixtures in the repository.
- Variant, alignment, signal, annotation, and MAF slices each require their own
  pinned oracle and semantic goldens; BED success does not validate them.
- Exact-head CI must run the stable slice on native Linux and macOS,
  `x86_64` and `aarch64`.

## Performance gates

The current slice passes this gate. The tracked evidence is
[`benchmarks/2026-08-02-linux-x86_64.md`](https://github.com/omics-rust/rsomics-liftover/blob/main/benchmarks/2026-08-02-linux-x86_64.md).
At exact source `db57617`, the official Linux x86_64 oracle and rsomics produce
byte-identical mapped and rejected files across single-chain, BED12,
minus-heavy, dense multiple-mapping, and real `hg38ToHg19` workloads. A pinned
five-run 20,000-block/100,000-record distribution measured median wall time of
0.22 seconds for rsomics and 8.74 seconds for UCSC, with 2,688 and 12,544 KiB
maximum RSS respectively. The real-chain run measured 0.44 versus 1.66 seconds
and 15,232 versus 40,320 KiB. The record includes machine, command, binary,
fixture, output, and generator hashes plus component accounting.

The durable criteria remain:

- Measure chain parse and index construction separately from record mapping.
- Use a deterministic synthetic chain with hundreds of thousands of blocks and
  at least one million varied BED3, BED6, and BED12 records.
- Add an externally supplied real assembly chain run without committing or
  redistributing the chain file.
- Compare identical mapped and rejected bytes against the official UCSC
  binary. Report both tools' exact hashes, commands, fixture generator and
  hashes, warmups, timing distribution, CPU time, peak RSS, and output hashes.
- Include single-chain, dense-overlap, multiple-mapping, and minus-strand-heavy
  workloads so a best-case synthetic index does not define the claim.
- A stable hot path must be strictly faster or use materially fewer resources.
  The unreproducible historical 2.3-times note does not satisfy the release
  gate.

## Publication verification

Publish workflow `30729592353` uploaded version 0.1.0 from exact head
`597220ee2ed1102360bc3ab15646e2c9067a1a84`. The live crates.io API reports a
non-yanked 30,074-byte archive with SHA-256
`a02e02938feb29c9dd27e9f9ea1a2864df5a9019b4ed8e00cffbeebffbef6e76`,
Rust 1.91, the expected library, and one `rsomics-liftover` binary. An
independently downloaded registry archive has the same checksum and records
the same VCS identity.

`cargo install --locked rsomics-liftover@0.1.0` succeeded on an isolated
external-disk target. The installed binary reported version 0.1.0 and returned
the expected JSON chain-validation summary for the committed two-chain golden.
The org registry token was granted to this repository only for the publish
run, removed immediately after success, and verified absent; the selected
repository count returned from ten to nine.

## License and attribution

The retained Rust implementation is team-owned and remains MIT OR Apache-2.0.
UCSC documents `kent/src/hg/liftOver` and the LiftOver software as a
non-commercial exception to the repository's default MIT license. Its source
is not copied, linked, vendored, or translated. The executable is used only as
a separately obtained black-box oracle.

UCSC also restricts its distributed liftOver chain files to non-commercial
use. No UCSC chain is included in the crate, repository, test fixtures, or
release artifacts; users provide chains and remain responsible for their
terms.

CrossMap is GPL-3.0-or-later. Its documentation and separately installed
executable may define and test behavior, but its source is not merged. Picard
and the BCFtools/Liftover implementation are permissively licensed and remain
attributed if their tests or algorithms materially inform a future variant
slice.

## Explicit exclusions

- Publish only the verified BED and chain slice; deferred formats remain absent.
- No silent malformed-record or malformed-chain skipping.
- No automatic chain download, bundled UCSC chain, or embedded UCSC binary.
- No generic line-by-line GFF/GTF lifting advertised as annotation-safe.
- No coordinate-only VCF mode that ignores target-reference and allele
  semantics.
- No direct dependency on another Layer B product.
- No public chain or liftover foundation until two consumers demonstrate a
  policy-free contract.
- No `genePred`, sample, PSL-target, UCSC database chain table, remote URL, or
  cross-species correctness promise in the first release.
