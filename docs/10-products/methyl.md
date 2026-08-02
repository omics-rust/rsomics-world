# Methyl product dossier

Status: source and upstream-operation audit complete. The first-release command
surface is implemented and passes four-native-target CI. Publication remains
gated on retained representative performance and memory evidence.

## Boundary

`rsomics-methyl` is one bisulfite-sequencing methylation extraction and bias-QC
product. It converts coordinate-sorted alignments and an indexed reference into
per-site, per-context, per-read, and positional-bias methylation evidence.

The primary behavior source is
[MethylDackel](https://github.com/dpryan79/MethylDackel). The published 0.6.1
tag and current upstream revision
`3c77bda12141e99d80234d416e668a90ec70b3f7` are separate compatibility
profiles. The public SAM, BAM, CRAM, FASTA, BED, bedGraph, and BigWig contracts
define the surrounding formats.

The current revision contains two correctness fixes made after 0.6.1: right-end
trimming now indexes `l_qseq - 1 - i`, and the second mate's initial overlap
scan uses its own projected positions. The corrected behavior is the primary
oracle. The released tag remains a regression oracle for unaffected behavior;
the Rust product does not reproduce either bug for byte compatibility.

This boundary does not include differential methylation, DMR calling,
segmentation, imputation, array normalization, or epigenomic visualization.
Those are separate statistical and signal workflows.

## Upstream operation map

| Target subcommand | MethylDackel operation | Decision |
|---|---|---|
| `extract` | `extract` | CpG, CHG, and CHH calls with filters, context merging, region selection, and alternative output forms |
| `mbias` | `mbias` | read-position methylation bias metrics, inclusion suggestions, TSV, and SVG |
| `merge-context` | `mergeContext` | merge sorted per-cytosine CpG or CHG metrics from a file or stdin |
| `per-read` | `perRead` | informative CpG count and methylation fraction per alignment record |

`--mergeContext` inside extraction and the standalone `mergeContext` operation
share one context-merging engine. The operation is not split into a fifth
public crate.

### Extract contract

The complete operation map includes:

- indexed BAM and CRAM, coordinate regions, BED inclusion, and BED strand
  filtering;
- MAPQ, base quality, SAM ignore/require flags, duplicates, singletons,
  discordant pairs, NH multi-mapping, and thread controls;
- CpG, CHG, CHH, and merged CpG/CHG contexts;
- OT, OB, CTOT, and CTOB inclusion and fixed-end trimming bounds;
- conversion-efficiency and opposite-strand variant filters;
- paired-overlap quality adjustment without double counting;
- percentage/count output, fraction, total counts, logit, methylKit, and
  exhaustive cytosine report;
- minimum depth and region-aware transactional output;
- optional BigWig or BBM mappability filtering.

Mappability and BBM support remain a later slice until their I/O and retained
format value are justified. No public `rsomics-bbi` crate is recreated for a
single consumer.

## Calling model

- Reference bases and coordinates use checked types. BAM/CRAM header sequence
  names and lengths must agree with the indexed reference for every processed
  contig.
- Input coordinate order is validated. A contig cannot reappear after it has
  been finalized, and positions cannot move backward within a contig.
- Bisulfite strand is an enum with OT, OB, CTOT, CTOB, and undetermined states.
  Aligner tags, read number, orientation, and single-end fallback are tested
  independently.
- CpG, CHG, and CHH classification is explicit at contig boundaries and for
  ambiguous reference bases.
- CIGAR projection, base and mapping quality, pairing, overlap adjustment,
  region selection, trimming, conversion efficiency, and variant filtering
  produce one typed call decision.
- Overlapping mates contribute at most one call per molecule and position.
  Agreement and disagreement quality transformations are overflow-safe and
  compatibility-tested.
- Counts use a width that cannot overflow on realistic deep coverage. The
  calling path is sparse or chunked; memory is not proportional to an entire
  chromosome's base count.
- Missing reference contigs, truncated records, invalid CIGAR, malformed aux
  tags needed by the selected protocol, and header mismatches fail loudly.
- All requested context and auxiliary outputs stage successfully before any
  final path is replaced.

## Target structure

```text
src/
├── alignment.rs
├── bed.rs
├── calling.rs
├── cli.rs
├── context.rs
├── conversion.rs
├── extract.rs
├── extract_output.rs
├── mbias.rs
├── mbias_output.rs
├── merge_context.rs
├── output.rs
├── per_read.rs
├── reference.rs
├── selection.rs
├── strand.rs
└── trimming.rs
```

`rsomics-common` owns errors, exit mapping, execution reports, aliases, and
single-output transactions. Coordinated context outputs remain product-local
until a second product demonstrates the same transaction contract.
`rsomics-help` owns the complete command tree.

`rsomics-methyl` is a concrete consumer of validated BAM/CRAM readers and
records from `rsomics-bamio`. It is also a concrete driver for
`rsomics-pileup`: sortedness validation, checked CIGAR projection,
low-allocation column views, and generic overlapping-mate evidence are shared
with BAM and variant-calling products. Bisulfite strand, cytosine context,
conversion, methylation calls, bias policy, and output formats stay inside this
product.

Indexed FASTA access may use the aligned external noodles implementation.
`rsomics-seqio` is not expanded from streaming FASTA/FASTQ into speculative
random-access reference policy.

## Historical asset disposition

The one routed source candidate is the clean `rsomics-methyldackel` repository
at `9d32057f7ec5eb8bb241d53c115280f9d6acbdea`.

| Asset | Disposition |
|---|---|
| bisulfite strand classification | refactor then merge as an enum with aligner-tag goldens |
| CpG calling and overlap-quality algorithms | direct algorithm merge after checked-coordinate and pileup adaptation |
| SAM flag filters | refactor then merge; replace raw bit constants with validated record APIs |
| CIGAR-to-reference projection | tests and behavior asset; use the shared checked pileup contract |
| active mate window | refactor then merge only if shared pileup overlap handling does not supersede it |
| full-reference loader and per-contig counts vector | discard; replace with indexed reference access and sparse or chunked accumulation |
| output writer | tests and formatting asset; replace direct file creation with transactional context outputs |
| synthetic BAM, reference, index, and golden bedGraph | behavior asset only; replace with a project-owned synthetic fixture pinned to the corrected oracle |
| Criterion subprocess benchmark | recipe only; the tiny fixture is not representative |
| duplicated `HelpSpec`, inherited `Tool`, and narrative comments | discard during current common/help migration |

MethylDackel is MIT licensed, so its implementation may be read and adapted
with attribution. The historical repository already records the directly
consulted upstream modules.

## Existing implementation gaps

The historical implementation covers a narrow `extract` default and must not
be published as the target product.

- It allocates one `PosCounts` value per base of the current contig. Despite
  comments describing bounded memory, a large human chromosome can require
  multiple gigabytes before active reads are counted.
- It assumes coordinate sorting but never validates contig or position order.
- A reference contig missing from FASTA silently drops all of its reads.
- Reference and BAM header lengths are clamped to the smaller value instead of
  reporting a mismatch.
- Header sequence identifiers are used for unchecked vector indexing.
- The entire FASTA is loaded despite requiring an index in help text; the FAI
  is not used.
- Only CpG standard bedGraph output is implemented. CHG, CHH, merged context,
  bias, per-read, alternative outputs, trimming, conversion, variant,
  mappability, region, NH, and CRAM behavior are absent.
- Threads control BGZF workers only; the calling path itself is not parallel,
  and help does not accurately expose the execution contract.
- Output is created before alignment validation and is not transactional.
- The live compatibility oracle may skip, and the frozen fixture exercises
  only one tiny reference and MAPQ difference.
- The source uses unchecked integer casts, suppresses Clippy cast lints, and
  contains extensive code narration.
- CI covers only Linux `x86_64`.

## Implementation evidence

Repository revision `65944e50b82f` implements indexed-reference header
validation, CpG/CHG/CHH classification, OT/OB/CTOT/CTOB strand resolution,
record filters, checked sparse pileup calling, product-specific overlapping
mate quality adjustment, and typed per-site metrics over published
`rsomics-bamio` and `rsomics-pileup` 0.5. The project-owned fixture contains
mixed 33% and 66% CpG calls plus MAPQ, NH, and duplicate exclusions; all ten
data rows match corrected MethylDackel revision `3c77bda12141e`.
Exact-head four-native-target CI `30750659399` passes at this revision.

Repository revisions `1afbdcf` through `ab1f322` implement strict six-column
`merge-context`, indexed FASTA access, coordinate and chromosome-order
validation, transactional output, the unified help layer, and a byte-identical
live MethylDackel differential. The extraction engine remains library-only
until its complete user-visible output contract is ready; no placeholder
subcommand is advertised.

Revisions `3bc7f214bfe2`, `8debb4162bad`, and `270ece8529fd` expose transactional
extraction, add standard, fraction, counts, logit, and methylKit representations,
and merge complementary CpG and CHG calls before minimum-depth filtering. Their
exact-head four-native-target CI runs `30750968404`, `30751189579`, and
`30751453229` pass.

Revision `8dac753e1d1c` implements `per-read` over published `rsomics-bamio` and
`rsomics-pileup` 0.6. With `--ignore-nh`, its seven output rows are byte-identical
to current MethylDackel on the project fixture; both files have SHA-256
`714b5df32399395676e932aec3d0db26e613f6b3e1d0a2eb724920b130c26082`.
The default produces six rows by enforcing MethylDackel's documented NH rule.
The current upstream source advertises that rule but neither registers the
`--ignoreNH` option nor applies the filter. It also advances both query and
reference coordinates twice after a low-quality matched base and limits the
reference fetch to a 10 kb tail. The Rust path tests and corrects all three
cases, consumes the shared long-CIGAR contract, caches FAI lengths, and keeps
single-output replacement transactional. Exact-head four-native-target CI
`30752879575` passes. The complete first release remains gated on the remaining
extract filters, cytosine report, and `mbias` surface below.

Revision `aef52dd3bbe3` adds one product-local indexed region model consumed by
both extraction and per-read reporting. It accepts standard 1-based inclusive
regions, clips the requested end to the reference, rejects unknown or wholly
outside references, limits extraction columns to the requested interval, and
retains MethylDackel's per-read rule that an alignment start must lie inside the
interval. Extract data rows for `chrSynthetic:5-10` match the live corrected
oracle with SHA-256
`b670e65b1449f84dcff9d923b239b6d84e9b1f129f1a80c02064094198b5fe80`;
the corresponding per-read outside-start differential is empty on both tools.
Documentation revision `67524780d001` is the exact head validated by
four-native-target CI `30753726141`.

Revision `20fd0ef1bc57` implements the `mbias` workflow without creating a
separate product or foundation. Per-read and M-bias now consume one checked
product-local alignment call walker; extraction and M-bias also share one CLI
filter surface, while extraction and M-bias outputs share one transactional
multi-file implementation. Default and region-limited TSV rows are
byte-identical to the live corrected oracle with SHA-256
`00149a22c832613159e7e0f80a70077e47ca57697ae4bd5942a3333abef6918b`
and `3088333049b19395c67782db3db512c7c62379a53e099d9191fb1b6820e25462`.
The Agresti-Coull intervals and inclusion-bound suggestions retain explicit
MethylDackel provenance; nonzero edge detection and CTOT/CTOB read-two calls
have independent tests. Deterministic valid SVGs and the TSV commit together,
and failure injection preserves all previous outputs. Exact-head
four-native-target CI `30754414728` passes. The first release is still gated on
the remaining extraction filters, cytosine report, and retained performance
fixtures.

Revision `adb4c5ec9f83` adds one product-local trimming model shared by
extraction and M-bias. It accepts OT, OB, CTOT, and CTOB bounds for both mates;
inclusion bounds are 1-based and inclusive with zero as an unbounded sentinel,
while fixed-end bounds are counts removed from each end. The canonical CLI uses
lowercase `--ot` and `--trim-ot` forms and accepts MethylDackel-compatible
`--OT` and `--nOT` aliases. Bounds are applied before overlapping-mate evidence
selection so a retained mate is not lost when its partner is trimmed.

Fixed-end extraction and M-bias results for `--nOT 5,1,1,1` are byte-identical
to current MethylDackel after excluding the path-dependent bedGraph header,
with SHA-256 `2de7009bf2472af1f1acabe8f5ed07d839735d708c0a52d3c75d5aeb41acfe10`
and `e11fef5da452d1813408919c6c928880ac45ac24bea5ae8f1f048a093b8ecdfe`.
The current upstream inclusion implementation contradicts its own help: for
`--OT 5,30,1,30` it removes position 5, whereas the documented contract says
positions 5 through 30 are included. Its parser also does not clear `errno`
before parsing each field, so a legal zero sentinel can inherit unrelated
process state. The Rust implementation deliberately follows the documented
contract, retains position 5, parses zero deterministically, and freezes both
corrections in tests. Exact-head four-native-target CI `30754904672` passes.

Revision `06770da6bf08` adds exhaustive Bismark-style cytosine reports as a
format of `extract`, with the upstream-compatible `--cytosine-report` and
`--cytosine_report` surface. It emits 1-based coordinates, reference strand,
methylated and unmethylated counts, CG/CHG/CHH class, and oriented
trinucleotide context. Zero-coverage cytosines are emitted regardless of the
minimum-depth setting. The implementation walks reference gaps around the
sorted sparse pileup and writes each metric immediately; it does not restore
the retired whole-contig count array.

The full project extraction fixture, its region-limited subset, and its
all-alignments-filtered zero-coverage report are byte-identical to current
MethylDackel with SHA-256
`8c8b72dd82543c331b92319cc39b317d38c11a09daa8222c6c6a7f3284d0ed9a`,
`01d725411caacffe535af77fbb5d99e1dfb68a7dc6a2c97c8c206caf323461e1`,
and `26eb2f1ea20a2acda721226bc405d19919b1c68d14018c46d9d3f8955eec9c87`.
A separate empty-alignment fixture covers all three contexts, both reference
strands, contig boundaries, and an ambiguous base with byte-identical SHA-256
`22b2f8b409642391f8a65538999fb271f5119f3cb5495bd66f2cc7e56fb065bb`.
Region restriction, multi-reference streaming, incompatible context merging,
and failed-output preservation have independent tests. Exact-head
four-native-target CI `30755585468` passes.

Revision `1368a8ff1bb7` adds plain and gzip BED selection to `extract`, `mbias`,
and `per-read`. It uses the published `rsomics-intervals 0.3.0` value for
validated zero-based half-open geometry while retaining BED parsing, merged
query spans, and bisulfite-strand policy inside the product. Overlapping and
touching input intervals become disjoint per-reference top and bottom vectors;
point and alignment-span queries are logarithmic in the number of merged
intervals. Unknown references, negative, empty, inverted, wholly outside, and
invalid strand-aware records fail with a file and line number. Ends beyond a
known reference are clipped, and an empty BED safely matches nothing.

Unstranded, top-only, and bottom-only extract data rows match current
MethylDackel byte for byte with SHA-256
`b670e65b1449f84dcff9d923b239b6d84e9b1f129f1a80c02064094198b5fe80`,
`f5b0d57e2fcb812734aace45cb1527effc2311de8fa7c9ddb583cc5094aa4fa7`,
and `1e8dcc17b2e90661d4aaf32cdc477a118bf137fbd4d36c7cf195de27a8642975`.
The corresponding M-bias TSV hashes are
`3088333049b19395c67782db3db512c7c62379a53e099d9191fb1b6820e25462`,
`a867bc5059ac8b376d9caa57282c7d1efd86af2e449e9e7ded5f86ffccb33cf2`,
and `a32de680a0f50f3511e8626d463db4f6d8c7ec6a91f52a288ad034dee84791a9`.

Two upstream BED defects are deliberately corrected. Its exhaustive report
emits zero-coverage cytosines outside the requested BED; rsomics restricts
every reported cytosine. Its per-read path tests only whether a large
processing chunk overlaps any interval and never applies the requested BED
strand; rsomics requires each alignment's complete reference span and
bisulfite strand to match. The corrected top-only and bottom-only per-read
goldens have SHA-256
`427fe94dd4f07409855e2bdc5ea205f103bb38552852e72052491f3dc74926f6`
and `3d747943a39dfaf19bdaf7fad09c4235d659b53d966567e0ea41846b34469ab8`.
Exact-head four-native-target CI `30756307754` passes.

Revision `744c4eef1303` adds one product-local non-CpG conversion filter shared
by extraction and M-bias. It evaluates each complete alignment before trimming
and overlapping-mate adjustment, applies the configured base-quality floor,
and defines efficiency as unmethylated divided by informative methylated plus
unmethylated CHG/CHH calls. A read with no informative non-CpG calls has
efficiency one, equality with the requested threshold passes, and finite
thresholds outside zero through one are rejected.

At thresholds 0.5 and 0.75, extract data rows match current MethylDackel with
SHA-256 `aea36b2807f9ab1fa49a2d1f867c7728ec1bff2efde886aac9d6aa8c74b5476d`
and `b1db45d1b4341c348c96046653632aa93bfed2dcd0bbe636c729809c42bac6e3`;
the corresponding M-bias TSV hashes are
`a02ec1567f57fba7b4b63cecf28fc3931c1dbbec18d55d35db90f0da262458d4`
and `94c74bf7a4f2e8d29291dee5d28c0ba44dea10ac355383babe9df175c28422e1`.
An independent insertion-separated CIGAR test corrects current upstream's
failure to advance its reference cursor between multiple match operations;
low-quality non-CpG calls are excluded from both numerator and denominator.
Exact-head four-native-target CI `30756788744` passes.

Revision `b1cd315ec4b3` completes the opposite-strand variant filter without a
new foundation. After trimming, overlapping-mate adjustment, base-quality
filtering, and strand-aware BED selection, it counts usable opposite-strand
reference and non-reference evidence. `N` contributes to neither numerator
nor denominator, a fraction equal to the configured maximum is retained, and
minimum usable depth is enforced before the fraction test. CpG, CHG, and CHH
share the same product-local evidence model; complementary CpG and CHG spans
are excluded symmetrically when either cytosine fails.

At minimum opposite depth four and maximum fractions 0.35 and 0.6, CpG data
rows match current MethylDackel with SHA-256
`ea2804d1135abddacf6ff7af3977a1f67865fae4d48cc048e0bed53c0e4b1c0e`
and `6eee2fa4dc0d42761d05c406d4c40bb450a309c384c56887f3867e2441cf6a9b`;
the corresponding CHG hashes are
`5465d7734fdad91fe4e1405f51da3d3d4e6c879649f570813364cd83ee5821ce`
and `fd8fba48fb89bcce1716f9844add7fa834971a04e936660975901cddabb70870`.
An independent all-context differential has CpG, CHG, and CHH hashes
`624ac320bcdb5f97961eb1a8bbb399924ec9b6e281544fa7a2171d1f3125d62c`,
`a871c91a1db5f48f0de869ee2f0a67b93cff4e2aece9100615ca8c882501c81e`,
and `08c8319bf1fa5bb87817bbe3e39b077d3881ac89c2b0f78981b1733700e4f953`.

Four upstream contradictions are deliberately corrected and frozen in the
fixture. Current MethylDackel counts `N` in usable depth, excludes equality
despite describing a maximum allowed fraction, can emit one half of a merged
CpG or CHG when the earlier half is variant, and recreates an excluded site as
a zero-coverage row in an exhaustive report. The corrected exhaustive output
has SHA-256
`07b556a3ab5fec6cc1e5bea994581dd0bd8644504efcd353ceefa5b01c4400b5`.
Exact-head four-native-target CI `30757468631` passes. Variant classification,
context-span policy, and merged-output suppression remain inside
`rsomics-methyl`; the existing `bamio`, `pileup`, and `intervals` contracts need
no expansion. The implemented first-release surface is now gated on retained
representative performance fixtures rather than another functional option.

A separate historical reverification reports byte-identical data lines on a
92 MB, four-million-read BAM producing about 2.5 million CpG rows.

The useful historical performance baseline on that larger input is about 1.59
times at one thread. A later 4.43-times record uses an 839 KB, 50,000-read
fixture with 31 versus 139 millisecond means and is too small to define a
release claim. Neither large fixture is retained in the current fixture store,
so both results are migration evidence that must be reproduced.

## First release slice

The first release is the coherent extraction and QC workflow:

- `extract` for BAM and CRAM with CpG, CHG, and CHH contexts;
- standard, merged-context, fraction, counts, logit, methylKit, and cytosine
  report outputs;
- MAPQ, base-quality, flags, duplicate, singleton, discordant, NH,
  conversion-efficiency, variant, trimming, region, and BED filters;
- `mbias` TSV, suggested bounds, and SVG;
- `merge-context` from a path or stdin;
- `per-read` without the upstream's documented 10 kb span limitation;
- indexed reference access, strict order and header validation, sparse or
  chunked memory, transactions, unified help, and execution reports.

Mappability BigWig and BBM input is deferred. It remains absent from help until
the private I/O implementation or a justified shared format contract passes
compatibility, memory, and throughput gates.

## Compatibility gates

- Build MethylDackel revision
  `3c77bda12141e99d80234d416e668a90ec70b3f7` with a pinned HTSlib as the
  corrected primary oracle. Retain the 0.6.1 binary as a released-profile
  regression oracle and explicitly classify differences caused by the two
  post-release correctness fixes.
- Run live differentials for all four subcommands on Linux and macOS. Frozen,
  provenance-recorded goldens run on all four native target classes.
- Cover CpG, CHG, CHH, merged contexts, contig edges, ambiguous reference
  bases, zero and minimum depth, percentage rounding, fraction, counts, logit,
  methylKit, and exhaustive cytosine reports.
- Cover OT, OB, CTOT, CTOB, undetermined strands, Bismark tags, missing or
  malformed tags, single and paired reads, both mates, overlapping agreement
  and disagreement, equal and unequal qualities, and clipping.
- Cover every reference-consuming CIGAR operation, invalid encodings,
  supplementary and secondary records, duplicates, QC failure, singletons,
  discordant pairs, NH values, conversion thresholds, and likely variants.
- Cover regions and BED selections at exact boundaries, strand-aware BED,
  contig aliases, missing contigs, header-length mismatch, out-of-order input,
  truncated BAM/CRAM, output aliases, and interrupted multi-output writes.
- Compare `mbias` numeric metrics and suggested bounds before SVG bytes.
  Rendering receives deterministic structural or image regression tests.
- Compare `per-read` fragments below and above 10 kb and document the deliberate
  improvement over the upstream limitation.

## Performance gates

- Use a retained or reproducibly generated WGBS-like input with at least four
  million reads and enough output to dominate startup and filesystem noise.
- Add an RRBS-like sparse-coverage input, a high-depth targeted input, long
  contigs, overlapping pairs, all contexts, and CRAM.
- Compare direct binaries at equal thread counts, with identical regions,
  filters, contexts, overlap behavior, and output bytes.
- Record index and reference-cache setup separately from calling. Report wall
  distribution, CPU time, peak RSS, output size and hashes, and thread scaling.
- Assert that peak live accumulation memory follows active depth or chunk size,
  not reference contig length.
- Benchmark `extract`, `mbias`, `merge-context`, and `per-read` separately.
- Record exact revisions, binary hashes, commands, generator and fixture
  hashes, warmups, and machine details.
- The stable extraction hot path must be strictly faster or use materially
  fewer resources than MethylDackel.

## License and attribution

The retained Rust code and synthetic fixtures are team-owned and remain MIT OR
Apache-2.0. MethylDackel is MIT licensed. Adapted algorithms retain its
copyright notice and attribution at the appropriate source or product level.
Record both upstream revision
`3c77bda12141e99d80234d416e668a90ec70b3f7` and release-tag revision
`b6db120e96ec8cf9ab44e1b1074d2aa7af876932`, the pinned HTSlib, relevant
format specifications, and any directly consulted upstream modules with the
compatibility results.

## Explicit exclusions

- No publication under the retired `rsomics-methyldackel` name.
- No differential methylation, DMR, segmentation, array, or imputation
  workflow.
- No silent missing contig, reference mismatch, malformed record, or unsorted
  input handling.
- No whole-chromosome per-base allocation.
- No parsed but unenforced context, strand, filter, or thread option.
- No speculative public methylation, reference, or BigWig foundation.
- No direct dependency on another Layer B product.
- No mappability or BBM claim until that complete later slice passes.
