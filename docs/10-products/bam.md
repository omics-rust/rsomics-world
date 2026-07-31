# BAM product dossier

Status: boundary and source-asset audit complete. The target repository exists,
the first release slice is in progress, and no release has been published.

## Boundary

`rsomics-bam` is one installable product for inspecting, converting, editing,
indexing, and analysing SAM, BAM, and CRAM alignment files. User-recognizable
operations are subcommands. File format, header, record, and stream contracts
are shared modules inside the product or come from `rsomics-bamio`; they are
not repeated per operation.

The compatibility baseline is:

- [samtools 1.24](https://www.htslib.org/doc/1.24/samtools.html), released
  2026-07-09, as the command and default-behavior oracle;
- the canonical
  [SAM/BAM, CRAM, tag, BAI, and CSI specifications](https://github.com/samtools/hts-specs)
  for format invariants;
- [HTSlib 1.24](https://github.com/samtools/htslib/releases/tag/1.24) for
  current format, index, and reference behavior.

The installed audit binaries are samtools 1.24 and HTSlib 1.24. HTSlib 1.24
removed its experimental CRAM 4 implementation. The product therefore targets
the stable CRAM 3 family and must not advertise CRAM 4.

This boundary does not imply byte-for-byte cloning of every incidental
diagnostic line. Stable data output, filtering decisions, ordering, headers,
exit behavior, and relevant warnings are compatibility contracts. Help and
presentation use the common rsomics CLI layer.

## Current operation map

The current samtools surface contains 40 operations. Three reference-index
operations belong to `rsomics-index`; the remaining 37 fit this product.

| Upstream group | `rsomics-bam` operations | Decision |
|---|---|---|
| Indexing | `index` | BAI, CSI, and CRAI lifecycle stays with the alignment product because it depends on alignment headers, coordinate sorting, and alignment-format policy |
| Editing | `calmd`, `fixmate`, `reheader`, `targetcut`, `addreplacerg`, `markdup`, `ampliconclip` | Product subcommands |
| File operations | `collate`, `cat`, `consensus`, `merge`, `mpileup`, `sort`, `split`, `quickcheck`, `fastq`, `fasta`, `import`, `reference`, `reset` | Product subcommands |
| Statistics | `bedcov`, `coverage`, `depth`, `flagstat`, `idxstats`, `cram-size`, `phase`, `stats`, `ampliconstats`, `checksum` | Product subcommands |
| Viewing | `flags`, `head`, `tview`, `view`, `depad`, `samples` | Product subcommands |
| Reference indexing | `dict`, `faidx`, `fqidx` | `rsomics-index` operations |

The three rerouted operations share a reference-sequence input and produce
reference lookup metadata. `rsomics-bam-dict` is therefore routed to
`rsomics-index` despite the historical crate name.

Historical convenience boundaries collapse as follows:

- `region`, `subsample`, and `sam-to-bam` become `view` options or format
  selections;
- RSeQC `divide_bam`, `split_bam`, and `split_paired_bam` become `split`
  partition, annotation, and mate modes;
- `to-fastq` becomes `fastq`;
- `to-bed` remains a conversion subcommand because its input model and
  filtering policy are alignment-specific;
- BAM coverage summaries remain here, while bigWig signal generation belongs
  to `rsomics-signal` and RNA-seq alignment QC belongs to
  `rsomics-rnaseq-qc`.

`reference`, `cram-size`, and `tview` have no historical implementation asset.
They remain legitimate later operations, but no placeholder command or help
entry is created before each is complete.

## Release slices

### Slice 1: streaming inspection and conversion

- `view`
- `head`
- `flags`
- `flagstat`
- `quickcheck`
- `samples`

This is the first publishable slice because it establishes the shared format
boundary without requiring an external sorter, index builder, or pileup
engine. `view` includes SAM/BAM/CRAM input, SAM/BAM output, header control,
region selection when a usable index exists, flag and map-quality filters,
count mode, deterministic subsampling, and explicit reference requirements.
CRAM output joins the stable surface only after a conforming writer is
available.

Samtools 1.24 changed the default `view --subsample` seed from zero to a value
derived from the input header. The rsomics default and its reproducibility
tests must match the 1.24 behavior; a user-supplied zero seed retains the old
behavior.

The slice is not complete if it only accepts BAM. It must prove:

- SAM, BAM, and CRAM format detection and explicit format overrides;
- header and record validation before public raw-record access;
- indexed and streaming region behavior;
- deterministic output, thread-budget handling, and compression controls;
- non-zero exit on malformed records, missing references, invalid filters,
  truncated streams, and output failures.

At revision `acb8b3a5a150`, `flags`, `flagstat`, `head`, `quickcheck`, and
`samples` are implemented. They use `rsomics-help` and `rsomics-common`, accept
the declared alignment formats where applicable, and pass seven samtools 1.24
oracle groups. `head` covers file and standard-input SAM, BAM, and CRAM,
including reference-backed CRAM MD/NM reconstruction for mismatches, insertions,
deletions, skips, ambiguous bases, and `=` sequence symbols. Exact-head CI run
`30604810443` passes native Linux and macOS on `x86_64` and `aarch64`.

At revision `b735539ffc75`, `view` streams SAM, BAM, and CRAM input; emits SAM
body, header, header-only, or count output; and applies required, excluded,
any-of, and all-of flag filters plus minimum mapping quality. Revision
`12b6991e47b6` adds transactional BAM output with explicit BGZF finalization
and passes SAM-to-BAM, BAM-to-BAM, and CRAM-to-BAM samtools 1.24 oracle
comparisons. Exact-head CI run `30605884207` passes native Linux and macOS on
`x86_64` and `aarch64`.

CRAM output remains intentionally unavailable. noodles-cram 0.93 converts the
CRAM read-group data series into the record read-group field but also retains
the `RG` auxiliary tag, producing duplicate `RG` fields after a round trip.
The same conversion logic is present at upstream revision
[`87efef3f77cb`](https://github.com/zaeleus/noodles/blob/87efef3f77cb28b9a7327a00f06bc6c258f9f326/noodles-cram/src/io/writer/record/convert.rs).
The product will not expose known-corrupting CRAM output while that contract is
unresolved.

Revision `b4322a5ee03d` adds BAI, CSI, and CRAI-backed region queries for BAM
and CRAM, alternative index-name discovery, ordered multi-region behavior, and
the unmapped `*` selector. Region-query records from BAM and CRAM inputs match
samtools 1.24 across overlapping regions, appended and replacement index
names, BAI, CSI, and CRAI. Exact-head CI run `30606532049` passes all four
native targets.

The crate stays unpublished until samtools 1.24-compatible subsampling,
remaining output and header semantics, compression and worker controls, and
the performance gates are complete.

### Slice 2: file lifecycle

- `sort`, with bounded memory, external runs, merge fan-in, temporary-path
  ownership, and coordinate/name/template-coordinate modes;
- `index`, with BAI, CSI, and CRAI selection, custom output, minimum shift,
  and multiple inputs where the oracle permits;
- `merge`, `collate`, `cat`, `reheader`, `split`, `fixmate`, and `markdup`.

This slice requires explicit header reconciliation, reference dictionary
validation, `@RG` and `@PG` translation, stable tie behavior, transactional
outputs, and cleanup after failure. The historical in-memory sorter and
first-header merge are not acceptable implementations.

### Slice 3: projection, pileup, and statistics

- `mpileup`, `consensus`, `calmd`, `depad`, `phase`, `reference`, and
  `targetcut`;
- `bedcov`, `coverage`, `depth`, `idxstats`, `stats`, `ampliconstats`, and
  `cram-size`;
- `fasta`, `fastq`, `import`, `to-bed`, `reset`, `addreplacerg`,
  `ampliconclip`, and `checksum`.

Pileup-dependent work proceeds with the `rsomics-pileup` contract described
below. `checksum` ships only if it meets the same performance or material
benefit gate as every other established-tool replacement. Its historical
implementation is slower than the recorded samtools comparison and receives
no exemption.

### Slice 4: interactive viewing

`tview` is a complete terminal interface, not a formatting helper. It stays
out of public help until navigation, reference display, color modes, terminal
failure behavior, and native-platform tests are complete.

## Target structure

The initial repository should use a narrow structure rather than copy 38
historical binaries:

```text
src/
├── lib.rs
├── main.rs
├── cli.rs
├── input.rs
├── md.rs
├── output.rs
├── commands/
│   ├── flags.rs
│   ├── flagstat.rs
│   ├── head.rs
│   ├── quickcheck.rs
│   ├── samples.rs
│   └── view.rs
└── filter.rs
```

Format detection, alignment headers, record decoding, and the current
libdeflate-backed BAM path remain private product modules while BAM is the only
implemented consumer. They move to `rsomics-bamio` only after a second product
exercises the same policy-free contract. Product modules own command policy,
filter composition, user-facing output, and samtools compatibility choices.
Later slices add command modules only when their implementation is real.

The binary must use `rsomics-help`. Product code supplies typed arguments,
contracts, examples, and command-specific validation; `rsomics-help` supplies
the shared layout, terminology, version/help behavior, stream conventions,
error presentation, and exit mapping. Foundation changes are driven through
the first slice rather than designed independently.

## Historical source assets

The 41 routed repositories are implementation and evidence inputs, not target
crate boundaries. Exact revisions below are the audited source snapshots.

The retired top-level packages were also recovered and checksum-verified:

- `rsomics-bam 0.1.0`, source revision
  `80f6186da312ccca7a5d2c6930628a7d77bb55e0`, archive SHA-256
  `0de37d0acc3dfdd2b2824b72ef285972ceabb37e1445b3d0a7cacb371f4cca89`;
- `rsomics-bam 0.2.0`, source revision
  `bff7af027e7bbe7a6d77c240a574dd8b859de556`, archive SHA-256
  `bf9a41381eeda74ca12ee1ed0d244d7e4e815ecc35518f53202d6f709b794239`.

Both packages implement only `view -c` over rust-htslib plus synthetic count
tests and a benchmark seed. The 0.2 compatibility suite skips when samtools or
the network fixture is unavailable. Preserve the fixtures and count benchmark;
discard the incomplete command shell, inherited common flags, and claims of
SAM/CRAM support.

| Asset and revision | Disposition | Target |
|---|---|---|
| `rsomics-bam-addreplacerg` `26354a3724f7f2e32bdb4d686b3ac13b59eeb6b4` | Refactor then merge | `addreplacerg`; retain tag and header fixtures |
| `rsomics-bam-ampliconclip` `94784e5b4132d39adcd0b784bb7d6ad7c0e69258` | Refactor then merge | `ampliconclip`; replace local format plumbing |
| `rsomics-bam-ampliconstats` `d748a727eb870583059bc801f89c3d115f4dcbc5` | Refactor then merge | `ampliconstats`; retain oracle fixtures and performance seed |
| `rsomics-bam-bedcov` `93204eea9155d118154ed237c84961b34ad7e29d` | Refactor then merge | `bedcov`; share validated pileup and interval input |
| `rsomics-bam-calmd` `6d3a4d0657c5c4e534269767b98534cc0a5d383e` | Refactor then merge | `calmd`; preserve MD/NM fixtures |
| `rsomics-bam-cat` `e0a21da2cf6c8f0f7eb1af87878a5dd03c02e211` | Refactor then merge | `cat`; retain block-copy ideas after header checks |
| `rsomics-bam-checksum` `95fc3dc4dfd477fae92306208ee61058b60ec638` | Test and benchmark asset until gate passes | `checksum`; do not retain the performance exemption |
| `rsomics-bam-collate` `f6f9b8ed029d6e1a30f4ecbc8bfe0ca2d25ad9ef` | Refactor then merge | `collate`; replace unbounded buffering |
| `rsomics-bam-consensus` `f202e114caa95ef38cd80dc40df8ee6a3f8ceae7` | Test asset and algorithm seed | `consensus`; historical simple mode is not the current default contract |
| `rsomics-bam-coverage` `e115cd0bceb0735e584d75125e7a6940e896d4fe` | Refactor then merge | `coverage`; summary output only |
| `rsomics-bam-depad` `de243fd7ccb7e0c313742b4e529fe95bad3833d4` | Refactor then merge | `depad`; retain padded-reference fixtures |
| `rsomics-bam-depth` `cdc0a4ff70119edc193cd6bdfadaba6b6e190b61` | Refactor then merge | `depth`; share pileup kernel |
| `rsomics-bam-divide` `71504b275797ec30df2399ef2fbe03d1c9b1e6b5` | Refactor then merge | `split --parts`; preserve disjoint-cover and seeded-partition fixtures |
| `rsomics-bam-fasta` `ba661eddd57b45f725751f02a288546442acd3e7` | Refactor then merge | `fasta` |
| `rsomics-bam-fixmate` `645e4e3c31f3e689e854c2de63e726b877d770ea` | Refactor then merge | `fixmate`; include supplementary mate behavior from 1.24 |
| `rsomics-bam-flags` `921a428ba5e11f47fca875e1b9ae1335b3b5cb8f` | Refactor then merge after dirty-diff attribution | `flags` |
| `rsomics-bam-flagstat` `ce1cc819d59fe37a56c762ba005ba0d9c91d3ba3` | Refactor then merge | First-slice `flagstat` |
| `rsomics-bam-head` `76ffd4d379191a968f1095a1854d0ce4c8fe49db` | Refactor then merge | First-slice `head` |
| `rsomics-bam-idxstats` `f96b6aed4452243a982c9d7ca495e6fa23d8b497` | Refactor then merge | `idxstats`; require index-kind coverage |
| `rsomics-bam-import` `ba7f8fc7630676e1cdbe95a21c0ae35677f5b958` | Refactor then merge | `import`; share `rsomics-seqio` only through a concrete contract |
| `rsomics-bam-index` `167e86bd0f5ee0cf13bf18e9ded89cb1f99a46a5` | Test asset after dirty-diff attribution | Replace the BAI-only implementation |
| `rsomics-bam-markdup` `e865796930fb72d8a185e3a0b18024d217ca6128` | Refactor then merge | `markdup`; retain scoring and duplicate fixtures |
| `rsomics-bam-merge` `7334fce53ec3666f63893b450710daa4efd43641` | Test asset and merge-loop seed | Replace first-header policy and swallowed decode failures |
| `rsomics-bam-mpileup` `5e51a7825384fd65aca38345a12ad7c89ad31143` | Refactor then merge after pileup API | Add BAQ and reference-aware default behavior |
| `rsomics-bam-phase` `9f475c325e8e8c30873a12df5979c44023e78c1d` | Test and algorithm asset | Replace tolerance-only compatibility decisions |
| `rsomics-bam-quickcheck` `5982123dbed16ab0f625495d550630c43d55f3ba` | Refactor then merge | First-slice `quickcheck`; cover all three formats |
| `rsomics-bam-region` `902f6f333a9d0ea623006f76d4e360e4fe5f5f0f` | Merge useful predicates | First-slice `view --region` |
| `rsomics-bam-reheader` `bdf6f6ec0ed0b16307e781b0ef335dc71699cae2` | Refactor then merge | `reheader`; transactional BAM and CRAM paths |
| `rsomics-bam-reset` `121947733112098c2b66d6151c23331cb4307e1f` | Refactor then merge | `reset`; current flag and auxiliary-tag behavior |
| `rsomics-bam-samples` `40b39137a2f03333a7b9af0505b43ccffc311bc9` | Refactor then merge | First-slice multi-input `samples` |
| `rsomics-bam-sort` `99144c7ba8d9abe78add7301cb300e74b5c11fe0` | Test asset only | Discard the whole-file `Vec` sorter |
| `rsomics-bam-split` `0393f01120602b785c30538954389d5742e9d7e7` | Refactor then merge | `split`; add tag and transactional multi-output policy |
| `rsomics-bam-split-gene` `e401744815fc1630f5c44d3f7cdf298d39f5b909` | Test and routing asset | `split --genes`; replace permissive BED12 row skipping |
| `rsomics-bam-split-pe` `8962f619d341cd18ea06d1cdf315efbfb4e2fa85` | Refactor then merge | `split --mates`; retain pairing-flag and mate-field fixtures |
| `rsomics-bam-stats` `25c3689b1267431fc0428bdfc873d81cf23c8d7c` | Refactor then merge | `stats`; re-audit 1.24 output and customized-index behavior |
| `rsomics-bam-subsample` `93052bf1e726f95022d6a6b8a549b9646c1e358a` | Merge algorithm after semantic update | First-slice `view --subsample` |
| `rsomics-bam-targetcut` `9d7fa02f6557cca7b52dfaf8ca73f837ee55e400` | Refactor then merge | Later `targetcut`; preserve fosmid-specific scope |
| `rsomics-bam-to-bed` `6d500bbcaa04ef307dc093170738bdbe4682d326` | Refactor then merge | Later `to-bed` |
| `rsomics-bam-to-fastq` `9675f305021dceb00ed03e9b847fa7d7a1a89d6c` | Refactor then merge | Later `fastq` |
| `rsomics-bam-view` `dde533dbcbe4f30243a004815da4c179ca52f12d` | Test and filter seed | Replace the BAM-only command shell |
| `rsomics-sam-to-bam` `f125e730d0edf498bc299a3ae37e7ec6fe1b8260` | Test asset | First-slice `view` format conversion |

The `flags` worktree has a modified `Cargo.lock`; `index` has an untracked
`Cargo.lock`. Neither diff is attributed to the target implementation until
ownership is resolved. All other listed source snapshots were clean during the
audit.

## Source quality findings

Every routed asset contains tests, a compatibility file, and a benchmark
target, but those booleans do not establish release evidence. Most
compatibility suites skip when samtools or bedtools is absent, historical CI
only covered Ubuntu, and several suites compare against samtools 1.21 or
1.23.1 rather than 1.24.

The recurring implementation problems are structural:

- several commands claim SAM/BAM/CRAM scope while accepting BAM only;
- `view` exposes only a small BAM filter subset and does not provide the
  format-conversion contract implied by its name;
- `sort` decodes the complete file into a `Vec`, so its memory use scales with
  the input rather than the configured budget;
- `merge` copies the first header without reference, read-group, program, or
  tag translation and converts some decode failures into absent sort keys;
- `index` only creates a default BAI beside one BAM input;
- `consensus` implements only a simple mode, while the current upstream
  contract includes Bayesian consensus, FASTQ output, regions, reference
  fill, allele modes, and base modifications;
- `mpileup` lacks BAQ and cannot provide the current reference-aware default;
- `phase` accepts loose outcome ranges where exact decisions are observable;
- public format records are not consistently validated before indexed
  accessors and can panic on malformed input.

The historical code also contains extensive phase and audit narration in
source comments. Selected algorithms are moved into named modules and narrow
functions; only stable invariants, safety requirements, or non-obvious
compatibility reasons retain comments.

## Foundation work

### `rsomics-bamio`

The audited foundation is revision
`dc4b19df5bc6664b39088b938136afecf48e21a9`, version 0.1.10. It is a
1,514-line BAM-oriented reader/writer with noodles and libdeflate paths. It has
tests but no external compatibility suite or benchmark recorded by the
inventory.

Its current public raw-record constructors accept arbitrary bytes, and several
accessors rely on indexing or `unwrap`. Reading a declared BAM block length
does not prove that the record's variable fields are internally consistent.
The eventual multi-product foundation contract is:

- auto-detected SAM, BAM, and CRAM readers with explicit input-format metadata;
- typed headers, decoded records, references, and structured errors;
- a validated raw BAM fast path whose unchecked constructor is not public;
- BAI, CSI, and CRAI indexed access with explicit reference requirements;
- caller-supplied worker budgets and deterministic output-format and
  compression settings;
- transactional writers that surface close and finalization failures.

Product-specific filtering, CLI policy, and samtools defaults remain in
`rsomics-bam`.

Named consumers are `rsomics-bam`, `rsomics-count`, `rsomics-methyl`,
`rsomics-minimap2`, `rsomics-peak`, `rsomics-rnaseq-qc`, `rsomics-signal`,
and `rsomics-call`. An API item becomes public
only after two product repositories exercise the same policy-free contract.
The first BAM slice alone does not justify publishing a redesigned API.
Revision `acb8b3a5a150` therefore removes `rust-htslib` from `rsomics-bam` and
implements the first consumer contract privately with noodles 0.110,
noodles-util 0.79, and libdeflate-backed noodles-bgzf 0.47. Revisions
`b735539ffc75` and `12b6991e47b6` extend that private contract through
streaming filters and transactional SAM/BAM output. They do not depend on or
release a speculative `rsomics-bamio` revision. Promotion waits for a second
implemented consumer such as `rsomics-count` or `rsomics-methyl`, with
consumer-side tests.

### `rsomics-pileup`

The audited foundation is revision
`5bd34dde15c5bc94e44d27a1ede2e9f9bf3e5fc2`, version 0.1.0. It currently
accepts a raw record through an infallible `feed` method and does not enforce
coordinate order at the boundary.

The shared contract needed by `rsomics-bam`, `rsomics-call`, and
`rsomics-methyl` is:

- a validated, coordinate-sorted record stream with fallible ingestion;
- checked reference IDs, coordinates, CIGAR projection, sequence, quality,
  and auxiliary fields;
- overlap-quality handling and BAQ where the operation requires it;
- policy-free pileup columns and explicit end-of-reference/end-of-stream
  behavior;
- bounded state with performance evidence on deep and ordinary coverage.

Unsorted input, coordinate overflow, malformed CIGAR, and inconsistent
sequence or quality lengths must fail rather than silently alter a pileup.
Peak-calling signal accumulation is product-private unless it later proves the
same contract.

### Other foundations

`rsomics-help` is mandatory for every product command. `rsomics-common` may
provide already-demonstrated path, thread-budget, or transactional-output
primitives, but BAM-specific records and policy do not move there.
`rsomics-intervals` is used only where the existing checked half-open geometry
contract fits region or BED operations; it does not absorb alignment indexing.

No new public foundation is needed.

## Compatibility gates

Each stable operation receives:

1. fixtures covering valid SAM, BAM, and CRAM where the operation accepts all
   three, including auxiliary tags, long CIGAR, empty references, unmapped
   records, supplementary records, and CRAM reference modes;
2. malformed and truncated inputs that prove a structured non-zero failure;
3. a pinned samtools 1.24 differential over data output, headers, ordering,
   filters, exit status, and relevant diagnostics;
4. cross-format round trips and index-kind coverage where applicable;
5. tests for deterministic output under explicit seeds and thread budgets.

Security-sensitive format boundaries include QNAME length, reference-ID bounds,
record-layout consistency, integer multiplication and addition, index entries,
and close/finalization errors. These checks are part of correctness, not
optional defensive duplication.

The release repository must pass formatting, strict Clippy, debug and release
tests, package verification, and exact-head CI on native Linux and macOS for
both `x86_64` and `aarch64`.

## Performance gates

The first slice uses representative indexed and streaming inputs for SAM, BAM,
and CRAM. It records input digest and shape, format and compression options,
worker count, machine, tool revisions, warm-up, alternating trial order,
timing distribution, output digest, and peak RSS.

At least the principal streaming `view` path must strictly beat samtools 1.24
in throughput or resource use. Raw-BAM fast-path measurements must still
validate identical accepted records and output before timing. Small synthetic
files and startup-only wins are not release evidence.

The historical `ampliconstats` README reports 0.143 seconds versus 0.607
seconds for samtools 1.23.1 on an Apple M2, 131 MB BAM, and one million read
pairs. This is a promising seed, not a current claim: it lacks the complete
repeated-trial, output, RSS, and 1.24 provenance required here.

The historical `checksum` result reports approximately 0.92 to 0.97 times
samtools throughput. That is a failed replacement-performance gate unless a
fresh implementation demonstrates a material correctness, resource, or
workflow benefit.

A provisional backend comparison on 2026-07-31 used a 13,712,741-byte BAM with
200,000 records and SHA-256
`bed20ddc9b79ebc952fe7ef555b683a8016a0d2a56c5f27185c226da9845b98b`.
The machine was an Apple M2 with 8 GiB RAM, macOS arm64, Rust 1.91.0, samtools
and HTSlib 1.24, and hyperfine 1.20.0. After five warm-ups, 30 single-thread
trials measured:

| Implementation | Mean | Standard deviation | Range |
|---|---:|---:|---:|
| `rsomics-bam acb8b3a flagstat` | 159.9 ms | 14.4 ms | 146.0–182.7 ms |
| `rsomics-bam 900b9c8 flagstat` | 260.8 ms | 22.6 ms | 230.7–289.2 ms |
| `samtools 1.24 flagstat` | 135.7 ms | 9.4 ms | 116.1–146.9 ms |

Both current tools produced output SHA-256
`ebe0882d0575383215efe688bb770202102ab9895f89f779d9ed8c518c8f152a`.
The private noodles backend is about 1.63 times faster than the replaced
rust-htslib product implementation, but samtools remains about 1.18 times
faster. With four additional decoder threads, rsomics averaged 119.6 ms and
samtools 65.6 ms. This supports the backend migration but fails the product
release-performance gate. It is not the final release benchmark because it
does not yet include `view`, peak RSS, alternating trial order, or
representative SAM and CRAM inputs.

## Explicit exclusions

- `dict`, `faidx`, and `fqidx` are `rsomics-index` operations.
- deepTools coverage, comparison, and fingerprint workflows belong to
  `rsomics-signal`.
- RSeQC, regtools, and Picard RNA-seq QC workflows belong to
  `rsomics-rnaseq-qc`.
- Variant calling belongs to `rsomics-call`; VCF/BCF format policy belongs to
  `rsomics-vcf`.
- Experimental CRAM 4 is outside the supported format contract.
- Remote object-store protocols are not implied by accepting local
  SAM/BAM/CRAM. They require their own error, credential, retry, and
  performance contract before exposure.

## Publication decision

Do not publish `rsomics-bam` yet. The product repository exists and its
streaming inspection commands, filters, and SAM/BAM output now have
four-native-platform exact-head CI plus samtools 1.24 oracle evidence. The
first slice remains incomplete: compatible subsampling, complete header and
output controls, and a strict performance or resource-use advantage are not
yet demonstrated. CRAM output is blocked by the verified duplicate-read-group
behavior above, and the shared `rsomics-bamio` contract still lacks a second
implemented consumer. Historical micro-crate versions do not reduce these
gates.
