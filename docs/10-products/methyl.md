# Methyl product dossier

Status: 0.1.0 published. The first-release command surface and retained
compatibility and performance matrix are complete.

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

Revisions `c5503685f9b1` and `3c3fa8168e8b` move the complete-alignment scan to
published `rsomics-bamio 0.7.0`, consume the `rsomics-pileup 0.7.0` projection
fast path, fix the indexed-reference cache at chunk boundaries, avoid paired
evidence allocation for unpaired columns, share reference names, buffer
transactional output, and format the standard bedGraph hot path without
per-field formatter allocation. File contents use `sync_data` before the
atomic replacement and the containing directory is still synchronized after
the rename. All 62 local tests, strict Clippy, rustdoc, package verification,
and exact-head four-native-target CI `30761821717` pass.

### Representative single-end WGBS gate, 2026-08-03

The retained fixture is
`rsomics-fixtures/methyl/wgbs-single-4m-20260803`. It is generated by
`benchmarks/generate_fixture.rs` with four million coordinate-sorted,
100-base single-end alignments. The 48,600,111-byte reference has SHA-256
`3a1bb38e032acaf57be9911e0b0f94fcb892081ad6a88905d9adb2baf6ebf4b5`;
the 77,438,045-byte BAM has SHA-256
`fe4f1977a9eb9352faafec62f5ab44e77f93757fd5557917d83b4558bc5530d6`.
Their FAI and BAI hashes are
`4c4b8c775373b8a8f5ae41d716aedde4a1ee78761964aec106fb076c3a650e45`
and `9c904c043df9e2252bcb527a571ac46d8947882e6a3e4c53abc0fe6e01c0bb7f`.

The oracle is MethylDackel revision
`3c77bda12141e99d80234d416e668a90ec70b3f7`, version 0.6.1, built by Apple
Clang 21 against HTSlib 1.24 and libBigWig revision
`43c294ef1721a73b760803ca5e9410d581b98f17`. Its binary SHA-256 is
`7a9dd657887d561fa47424e20573e46ed02e11aea077660cee09b23f41c69890`.
The Rust release binary SHA-256 is
`dd40d8b4f271296ca26e88d682a5a3b81e3be9426daafb4a585572dd025fc0ec`
and was built by Rust 1.97.1. The machine is `Mac14,3`, Apple M2, 8 GiB,
running macOS 26.6 build 25G72. Input is read from a USB APFS rotating disk;
both outputs are written to the same USB APFS SSD.

Both commands run at one thread:

```console
rsomics-methyl extract reference.fa alignments.bam --output-prefix rust
MethylDackel extract -@ 1 -o upstream reference.fa alignments.bam
```

After one warmup per binary, ten pairs alternate which binary runs first.
Rust completes in `10.804 ± 1.671` seconds versus
`12.090 ± 1.079` seconds for MethylDackel, a 10.6% mean wall-time advantage.
Rust wins nine of ten pairs; the paired mean difference is 1.286 seconds with
a paired standard deviation of 1.275 seconds. Mean user CPU is 5.058 versus
5.357 seconds and mean system CPU is 1.011 versus 1.080 seconds. A separate
resource run records 9,273,344 versus 12,189,696 bytes maximum RSS. The
4,800,006 data rows are identical with SHA-256
`8764f4a9266ad8d579c96ee392c1fc243a6317d2af4921eefd26b70a42df8d17`;
the complete output is about 156 MB.

The paired raw timing files are retained as
`extract-paired-ssd-syncdata-rust.time` and
`extract-paired-ssd-syncdata-upstream.time`, with SHA-256
`b6f4bc4d8097de93e2563f740f9ed258d637d86fbddc2070926e053a869cbc93`
and `713d59cbeeb10b44b6900916a94e874e0bd05d6725014438c42d69ca6c63a753`.
The resource records have SHA-256
`3fa2b456600b578e508a932162f96fd6d36e8ceb02b13611e78f7a2bb498fe95`
and `ce3665b7969d31a771e484f7c89b17ee5dd03fadcfc779d4f606615f40b4ecb5`.

The result closes the representative single-end WGBS extraction gate only.
RRBS-like sparse coverage, high-depth targeted data, all contexts, CRAM, and
separate `mbias`, `merge-context`, and `per-read` measurements remain required
before the first product publication.

### Overlapping paired-end WGBS gate, 2026-08-03

Revision `8a746cf8a2ac` replaces per-column heap allocation with inline evidence
storage. Columns of at most 32 retained records pair mates by a bounded linear
scan; higher-depth columns retain the hash-indexed path. A unit test forces
both branches through the same agreement and quality-adjustment contract. All
63 local tests, strict Clippy, rustdoc, package verification, and exact-head
four-native-target CI `30762469742` pass.

The retained `wgbs-paired-overlap-2m-20260803` fixture contains two million
fragments and four million coordinate-sorted 100-base alignments, with mates
offset by ten bases. Its 50,625,121-byte reference and 93,979,516-byte BAM have
SHA-256
`2229b2631c87c8cd84ee8e38c010351e11f206bdf7002dd630a61102156840cc`
and `6efe332101303a59350a6d66202ae1eda51a38a7084f10565041134ab28930c7`.
The FAI and BAI hashes are
`e4257313524c80aa008f31184bbfe2e7704f7ac98b7f5a9a6e260c789c0e9fa0`
and `b03e217ab50876367478960f69985a0c6fe07215eb3d8673ce019668ac6cb88a`.

The same machine, oracle, storage placement, one-thread commands, warmup, and
alternating ten-pair method are used. Rust completes in
`9.657 ± 0.778` seconds versus `10.717 ± 0.892` seconds for MethylDackel, a
9.9% mean wall-time advantage. Rust wins seven of ten pairs; the paired mean
difference is 1.060 seconds with a paired standard deviation of 1.363 seconds.
Mean user CPU is 5.254 versus 5.955 seconds. A separate resource run records
9,420,800 versus 13,254,656 bytes maximum RSS. The 4,999,919 data rows are
identical with SHA-256
`01b7a4306cab5cde4da8accd5bb3652df25f2bae228aca901cef8244aaa590d4`.
The final Rust binary has SHA-256
`3c78bcabf0cb110bb1b4725ed48239d2816a317d895d574308374435c9714e2f`.

The Rust and upstream paired timing records have SHA-256
`9aaaf7d4cf684ca3d5bf20c8404761b74c0e514ab200676198062e27cf7e9005`
and `eaaff95b07be4722b8b80ff85b44603fc57866931a2f298303ed06f741aed3f1`.
The resource records have SHA-256
`f307e4052fe4c8cb49e9073c8cd20ab48d426dce8c3779640b3d377de9006925`
and `138d1970a91ccd3d2bc55e5ce18b67edc274b53854b513d67412c038dcb1352e`.

This closes the overlapping paired-end row of the matrix. Sparse RRBS-like,
high-depth targeted, all-context, CRAM, and separate subcommand measurements
remain open.

### High-depth targeted all-context gate, 2026-08-03

Revision `e99aba7f89a2` adds the retained
`targeted-mixed-4m-20260803` fixture: four million coordinate-sorted reads
concentrated into mixed CpG, CHG, and CHH targets. The reference, FAI, BAM, and
BAI SHA-256 values are
`22d8b864c0760eedb86d707ec9339f8fb64d5cd69e5e69fb6400e036ceed332a`,
`af6079307282fc5574f6c59cfe0041439195df7f6439eede0eca5c350ed62b96`,
`58b3a45f4757eac7d840112e382baaf0b181fccc56985ead59cf89854bdfec39`,
and `b0e12c59b48aa2c895763f2dfda956bc57dba510c9e9e4e55e7517a1b340c79b`.
Rust and MethylDackel produce the same 130,219 CpG, 130,218 CHG, and 65,109
CHH lines including headers. The header-independent data hashes are
`fd6cc730ea73b0e34a2b41de1f84052d88f368256b5751ab1be8f6d3dcc3e587`,
`c3f2b81ff18b3b7d6fa98ca505701edce36ae67042db6a7394702a58ec7d7b33`,
and `6e04629b1b7b6cb31a15514eb6bc5ea961d38d05a1160bbdd773ba260cab98fa`.

After one warmup per binary, ten alternating pairs complete in
`7.560 ± 0.357` seconds for Rust and `8.697 ± 1.052` seconds for
MethylDackel. Rust wins nine pairs, a 13.1% mean wall-time advantage; the
paired mean difference is 1.137 seconds with standard deviation 1.111 seconds
and paired t statistic 3.235. Mean user CPU is 5.819 versus 6.384 seconds and
mean system CPU is 0.425 versus 0.461 seconds. The Rust and upstream timing
records have SHA-256
`7d1b2c733906bcc585869ce49c308d2d3dfb501856efd0cb4bff9b6face7beea`
and `01397387bf2c5ae035ac8ad0bc1b588d46625c3128bd1b108aa37949ee9eced7`;
the resource records have SHA-256
`88c89ebc239288a9a8787f50a6b8d44327dd2f189626b50146cf2ab5bc1f24c1`
and `8bca058e32480032b446ef778cf4db2f7ef98f9f84511123a4342f6ad53376b4`.
Exact-head four-native-target CI `30762903663` passes.

### CRAM streaming gate, 2026-08-03

The representative CRAM is a reference-compressed encoding of the four-million
read single-end fixture. Its 383,267-byte CRAM and 3,083-byte CRAI have
SHA-256
`5a50c33c39fd6646f92de132a83ac8c7b4e34496e85ca01fe9f01f525091d055`
and `9c7e2d9001022317d5885d851a0ee78340965b33d0996404c8ef02ca9c850b44`.
The original full-container decode took 26.87 seconds and 149,389,312 bytes
maximum RSS, versus 12.86 seconds and 32,522,240 bytes for MethylDackel. A
slice-local variant still took 21.73 seconds and 162.7 MB and was rejected.

`rsomics-bamio 0.8.0` instead decodes sequential CRAM through HTSlib into one
reused record while retaining the existing validated `RawRecord` contract and
Noodles indexed queries. Field-level CRAM and 65,536-operation long-CIGAR
tests cover the conversion. Version 0.8.1 makes this backend the default
`cram-htslib` feature; BAM-only consumers can disable it. Accordingly,
`rsomics-pileup 0.8.0` disables the feature while `rsomics-methyl` enables it
through its direct `bamio` dependency. This keeps the shared pileup library
free of product-local CRAM linkage without duplicating alignment parsing.
Exact-head four-native-target CI passes for `rsomics-bamio 0.8.1` in
`30765211183`, `rsomics-pileup 0.8.0` in `30765656887`, and methyl revision
`0002a5a07798` in `30766173656`. Both foundation releases were independently
downloaded from crates.io before the methyl lockfile was accepted.

After one warmup per binary, ten alternating pairs complete in
`16.069 ± 2.820` seconds for Rust and `16.453 ± 2.342` seconds for
MethylDackel. Rust wins six pairs, but the paired 0.384-second difference has
standard deviation 3.730 seconds and paired t statistic 0.326, so no wall-time
advantage is claimed. Mean user CPU is `6.024 ± 0.268` versus
`6.745 ± 0.332` seconds, and mean maximum RSS is
`22,480,486 ± 1,535,741` versus `28,167,373 ± 1,032,692` bytes. The Rust and
upstream timing records have SHA-256
`a9678976fca00fc2b18aee56f2030a8db7708964ac22980da6212ee657c71fe0`
and `1b72b4bb4ebd0ccc29b6de1bc07556c26748ffa5e5a57c6b59beb2f60f5df633`.

The final registry-resolved methyl build uses `rsomics-bamio 0.8.1` and
`rsomics-pileup 0.8.0`; its binary SHA-256 is
`29670089b322e4b3664b4131035eb640e529cac149abe9c7fe187dc63d8ecb03`.
A separate integration run completes in 7.27 seconds with 19,529,728 bytes
maximum RSS. Its resource record has SHA-256
`a08daefd8a1fe252780df0493e93c01837dff83675d4e4ac8fea1705ff8fae45`.
This single run is a dependency-integration check, not an additional
performance estimate. Its 4,800,006 header-independent data lines retain
SHA-256
`8764f4a9266ad8d579c96ee392c1fc243a6317d2af4921eefd26b70a42df8d17`,
identical to BAM and MethylDackel.

### Sparse RRBS-like extraction gate, 2026-08-03

Revision `6494704743c2` adds an `rrbs` generator mode and compiles the generator
under the Linux x86_64 CI job. It creates 104-base covered islands separated by
396-base gaps, with 40-fold placement inside each island. Exact-head
four-native-target CI `30766674318` passes.

The retained `rrbs-sparse-4m-20260803` fixture contains four million
coordinate-sorted reads. `samtools coverage` reports 3,958,762 nonduplicate
reads, 10,400,000 covered bases across a 50,000,100-base reference, 20.8%
coverage, and mean depth 7.91751. The generator source, reference, FAI, BAM,
and BAI SHA-256 values are
`57ffdd20771bff39e1d04244d0a9002a8a8b9cc1202f8ca49d5234341b232273`,
`54826750c63602abd57dc3c022132f97f2c189a2b3ac8f1e4764410c5d915e9f`,
`fd69fbf02c375f75a41fdb490b34a3b14940705c57ec406888c784ca05a7e965`,
`3b6e3cc0199ae0dd39159af5e7b7fbda2f301dcfbbc082bfc9c31734d292bb6b`,
and `2cdbb73dd3391b21471863e45ba07c023efc8758ed57e616ff05576a051ab001`.

After one warmup per binary, ten alternating pairs produce exactly 1,200,000
data rows with SHA-256
`047e02b2acf6176db58025e7920a09f3d19f7a96de2499ec66e4f2886615b7e9`
for both implementations. Rust completes in `4.349 ± 0.460` seconds versus
`4.188 ± 0.367` seconds for MethylDackel and wins four pairs. The paired
upstream-minus-Rust difference is `-0.161 ± 0.619` seconds with t statistic
`-0.823`, so no throughput advantage is claimed. Mean user CPU is 3.502 versus
3.478 seconds and mean maximum RSS is 9,828,762 versus 9,774,694 bytes. The
Rust and upstream timing records have SHA-256
`9aa3843d4a286f5a9b05f3dcc7e0469b3a40c48ee411674fa1242668c75106c0`
and `42aa0ae797ddfb222f5390a85fa54713c06a9035b24190c59b48ecb93712e9ab`.
This closes the sparse-coverage correctness and measurement row while leaving
the product's performance claim on the representative, paired, and targeted
extraction workloads where a strict advantage was measured.

### M-bias gate, 2026-08-03

Revision `637e84586132` replaces the per-call ordered map with dense
strand-by-read-by-position counters, avoids reference-name comparisons on the
validated alignment hot path, skips reference classification for bases that
cannot encode a methylation call, and removes unused per-record chromosome
allocation. Product policy remains private to `rsomics-methyl`; no foundation
API was added. All 63 local tests in debug and release, strict Clippy, rustdoc,
package verification, and exact-head four-native-target CI `30768331537` pass.

The benchmark uses the retained `wgbs-single-4m-20260803` fixture and the same
one-thread MethylDackel source revision. The current oracle rebuild has binary
SHA-256
`70e2296eb412bb4cf9c0ce2b74a0ab290211f2b50c29bced091db66317e8c152`;
the final Rust binary has SHA-256
`3d00ecc550801c2606e1a03657e32a0e41fb29ae892879f188b0d77286eb328c`.
After one warmup per binary, ten pairs alternate which binary runs first and
compare the TSV after every pair. All twenty 51-line files are byte-identical
with SHA-256
`2fa2af6289da84ae3f94109ddbf860d4d8a4001029598bf210d1aa3bf4ccb0a1`.

Rust wall time is `6.443 ± 1.084` seconds versus `6.614 ± 0.796` seconds for
MethylDackel. Each wins five pairs; the paired 0.171-second difference has
standard deviation 1.322 seconds and t statistic 0.409, so no wall-time
advantage is claimed. Mean user CPU is 4.030 versus 4.478 seconds: Rust wins
all ten pairs, reducing user CPU by 10.0%, with paired difference
`0.448 ± 0.225` seconds and t statistic 6.308. Mean system CPU is 0.384 versus
0.382 seconds, and mean maximum RSS is 8,709,734 versus 8,565,555 bytes; no
system-CPU or memory advantage is claimed.

The Rust and upstream raw timing records have SHA-256
`54100dc871313b11337e5b9b0b5ad1228e9f7f4fe435ace3e7548a7694cd48f3`
and `19444eb146f47e7e2af50aae5bdbbe72b33675e9c41cb1528b291bcb0fcd3029`.
Their parsed per-pair summaries have SHA-256
`cd7c3242fa70f8ed7f82951b457d8d019b42f89dd07c8bfe29fbdb88b4e4271f`
and `a92f7133a6e1a19b13789d768b1142b2899b70353d78c016b346e1a45d3d4e55`.
The raw logs, summaries, and matching TSVs are retained with the WGBS fixture.

The high-depth targeted, all-context, CRAM, sparse RRBS-like, and M-bias rows
are closed. The `merge-context`, `per-read`, and long-contig rows below close
the remaining retained performance matrix.

### Merge-context gate, 2026-08-03

The retained `context-fast_CpG.bedGraph` input has SHA-256
`ea9be89eb342c6ab2958e6b7c4046a1396896cf1a4eddd3913e4e2f6c9292a2e`.
Revision `073421f66ce8` uses the registry release of `rsomics-common 0.12.1`;
its Rust binary has SHA-256
`2ca23e6c6e91cf1b72369fd0990f6a8c98747136a5142872a0044fa00041960d`.
The oracle remains the one-thread MethylDackel rebuild with SHA-256
`70e2296eb412bb4cf9c0ce2b74a0ab290211f2b50c29bced091db66317e8c152`.
Both commands write through standard output redirection so the comparison does
not charge only the Rust named-output path for an explicit durability sync.

After one warmup per binary, ten alternating pairs produce the same 2,400,005
lines with SHA-256
`b43dc06f04fd162e61a9cc7d4e68da9d93c6b956e0b5b52bbe95f2265d0128e0`.
Rust wall time is `3.327 ± 0.933` seconds versus `3.166 ± 0.595` seconds and
mean user CPU is 1.629 versus 1.621 seconds, so no throughput or user-CPU
advantage is claimed. Mean system CPU is 0.246 versus 0.344 seconds; Rust wins
nine pairs, reducing system CPU by 28.5%, with paired difference
`0.098 ± 0.085` seconds and t statistic 3.637. Mean maximum RSS is 8,698,266
versus 6,240,666 bytes, so no memory advantage is claimed.

The raw Rust and upstream timing files have SHA-256
`402428891da7413d40f0e2f4f2cae854ece46d006773159eb20ca0c0a6a9cbac`
and `1dd3313d6ac10f2ade3437ad4ed64dcec1430b10c247a42224f4621c1d6c309e`.
Their parsed summaries have SHA-256
`113fac1f139e89773c13495171c4f7386f42a3be9130ca26b31c7dab5e1658e8`
and `c306553002ce64180fd8eea2b4ce409b4689feeb7c9ca1e2759b32bb4d334914`;
the aggregate statistics have SHA-256
`69a658ecd41b8af1da30ccd270d760bfd01160270b9f1d9563f69e06196d9b01`.

The first named-output diagnostic exposed an independent shared-layer defect:
millions of small writes reached an unbuffered file and took 61.19 seconds,
including 46.63 seconds of system CPU. `rsomics-common 0.12.1` keeps the same
transactional contract but adds the 1 MiB buffer and an explicit checked
flush. A registry-resolved integration run produces the same output in 4.04
seconds with 0.28 seconds of system CPU; it is a single correction check, not
an additional comparative estimate. Its resource record has SHA-256
`65e734c04100e39df50fbca0b2d49cba90a930440fffe1b954ffaa5b2cb01e68`.

### Per-read and long-contig gates, 2026-08-03

Revision `19c000cbe553` adds a CpG-only caller path, retains checked long-CIGAR
fallback while avoiding an allocation for the common single-operation CIGAR,
uses a borrowed streaming metric, and writes six-decimal percentages with an
integer formatter verified exhaustively through 1,024 informative bases. The
public callback exposes borrowed record fields rather than forcing an owned
copy for every emitted row.

All 64 local tests pass in debug and release together with strict Clippy,
rustdoc, and package verification. Exact-head four-native-target CI
`30769900077` passes.

The benchmark uses the 48,000,100-base single-contig reference and four-million
record BAM from `wgbs-single-4m-20260803`. After one warmup per binary, ten
alternating pairs produce the same 3,995,987 lines with SHA-256
`5ea398de4d5b1dd7c1d6db88f42ed6ef738bdd6e367f7097d6a3cd9c6f390dd7`.
The Rust and oracle binaries have SHA-256
`315465621ff3be6afc96418bdb499b6bfc773327b8e9f8325e9d0fa2965373f5`
and `70e2296eb412bb4cf9c0ce2b74a0ab290211f2b50c29bced091db66317e8c152`.

Rust wall time is `5.652 ± 0.832` seconds versus `4.467 ± 0.837` seconds and
mean user CPU is 3.739 versus 2.897 seconds. This is an explicit CPU and
throughput cost, not a speed claim. Mean maximum RSS is 9,810,739 versus
13,851,034 bytes; Rust uses 29.2% less memory and wins all ten pairs, with a
paired difference of `4,040,294 ± 911,117` bytes and t statistic 14.023.

The raw Rust and upstream timing records have SHA-256
`be4a33cc06e6d91977ef5940d692d6fe34d34bcffa695529164600e7fde50539`
and `1b1319a5ff2e2d4ffc9e9f5bf15f7abac33ad453f3ff34e590bdf18634a86139`.
Their parsed summaries have SHA-256
`1abed5775a3d59f9f4ce4f973d1f09fbeb44bd3ca6e0bc8893106d6f78a23693`
and `b6c9c28aea122273024286e6caeabdf54fdcadaae2beab7038ca9d4436fbd0b0`;
the aggregate statistics have SHA-256
`f4419199f0a6589f25a4937d9bc02ee024dee904bc9f5159404ee064f5018d60`.

The same run traverses the full reference through the 1 MiB cache without
reference-sized accumulation. The sparse RRBS fixture independently spans a
50,000,100-base contig with mean Rust RSS 9,828,762 bytes, while the
7,813,100-base targeted
fixture recorded 10,289,152 bytes in its retained resource run. Together with
the above per-read distribution and the over-20-kilobase CIGAR correctness
test, this closes the long-contig bounded-memory row. The upstream's documented
incorrect handling of alignments spanning more than 10 kb remains a deliberate
compatibility improvement rather than a byte-equivalence target.

### Release evidence, 2026-08-03

The clean release revision is `916944d39f2a`. Exact-head four-native-target CI
`30770176701` passes, including format, strict Clippy, rustdoc, package
verification, and debug and release tests. Publish workflow `30770336093`
released `rsomics-methyl 0.1.0`. An independent download from the crates.io
static archive has SHA-256
`1db1c6b7af380364bdcf93a4c71e7252faa03f5c5be635515cbfd3f6e5e924b4`;
all 64 tests from that downloaded package pass in debug and release.

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
- Retain the completed high-depth targeted, overlapping-pair, all-context, and
  CRAM rows; add an RRBS-like sparse-coverage input and long-contig stress row.
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
