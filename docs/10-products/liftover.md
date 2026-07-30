# Liftover product dossier

Status: source and upstream-operation audit complete. The existing target
repository is an implementation asset, not a release candidate.

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
- User-supplied chain provenance, direction, and hash can be emitted in the
  execution report. The crate does not download or redistribute assembly chain
  files.

## Target structure

```text
src/
├── cli.rs
├── chain/
│   ├── parser.rs
│   ├── index.rs
│   ├── validate.rs
│   └── inspect.rs
├── mapping/
│   ├── candidate.rs
│   ├── interval.rs
│   └── rejection.rs
├── formats/
│   ├── bed.rs
│   ├── position.rs
│   ├── vcf.rs
│   ├── alignment.rs
│   ├── signal.rs
│   ├── annotation.rs
│   └── maf.rs
└── transaction.rs
```

Format modules are added only with a complete release slice. Empty modules and
advertised placeholder subcommands are forbidden.

The chain parser, index, and mapping engine remain product-private initially.
`rsomics-common` owns errors, exit mapping, input aliases, execution reports,
and output transactions. `rsomics-help` owns the complete CLI presentation.
`rsomics-intervals` may supply its checked half-open coordinate type after an
API review, but liftover-specific chain ordering and candidate policy do not
belong there.

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

## Existing implementation gaps

The current 302-line implementation covers only a narrow BED-like path and
must not be published in its present form.

- Header fields are indexed without length checks and numeric fields use
  production `unwrap()`.
- Invalid block tokens disappear through `filter_map`, while non-chain and
  malformed BED lines are silently ignored.
- Header bounds, target strand, endpoints, terminal block totals, identifiers,
  ordering, overlaps, negative spans, zero spans, and overflow are unchecked.
- Only `minMatch` is exposed. BED12 blocks and thick bounds, multiple mappings,
  minimum blocks, preservation, chain-size filters, and exact tie behavior are
  absent.
- The current frozen test does not invoke the upstream binary in CI.
- Mapped and rejected files are truncated before parsing or mapping succeeds.
- The CLI bypasses `rsomics-help`, uses the inherited `Tool` shell, and contains
  extensive historical and narrative comments that do not match the project
  style.

## Retained evidence

The committed small golden was regenerated on 2026-07-31 with the local
official macOS arm64 `liftOver` binary. Both mapped and unmapped outputs were
byte-identical. The oracle binary SHA-256 is
`3df0770dbd09a76cc308bfdd025478964e26e3e3f1a150c177786488850a6996`.

The historical performance note reports 0.49 seconds for rsomics and 1.12
seconds for UCSC on 300,000 intervals and a synthetic 2,000-block chain.
Neither the large fixture nor its generator is retained, and the note does not
record output hashes or timing distributions. The result is therefore a useful
recipe, not a release claim.

## First release slice

The first release contains:

- `chain validate`;
- `chain inspect`;
- `bed` for strict BED3-6 and BED12;
- `--min-match`, `--min-blocks`, `--multiple`, `--preserve-input`, chain-size
  filters, and BED12 thick-bound handling where the pinned oracle supports
  them;
- plain and gzip chain input;
- mapped and rejected transactional outputs;
- unified help, completion, error, and execution-report behavior.

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

- No publication of the current narrow implementation.
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
