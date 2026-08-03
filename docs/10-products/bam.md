# BAM product dossier

Status: boundary and source-asset audit complete. The seven-command first
release slice is published as `rsomics-bam 0.4.0`. The eighth command,
`depth`, has passed its local, oracle, performance, package, and
four-native-target CI gates and is published as `rsomics-bam 0.5.0`. The
ninth command, `index`, has passed the same release gates and is published as
`rsomics-bam 0.6.0`. The tenth command, `sort`, is published as
`rsomics-bam 0.7.0` after the same correctness, performance, package, and
four-native-target gates. The eleventh command, `merge`, has passed its
correctness, performance, package, and four-native-target gates and is
published as `rsomics-bam 0.8.0`.

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

### Release 0.4: streaming inspection, conversion, and pileup

- `view`
- `head`
- `flags`
- `flagstat`
- `quickcheck`
- `samples`
- `mpileup`

This is the first publishable slice because it establishes the shared format
boundary without requiring an external sorter or index builder. `view`
includes SAM/BAM/CRAM input, SAM/BAM output, header control, region selection
when a usable index exists, flag and map-quality filters, count mode, and
explicit reference requirements. `mpileup` is included because its shared
engine and compatibility surface are complete. CRAM output joins the stable
surface only after a conforming writer is available.

Subsampling is not part of the 0.4 CLI or documentation. It remains a future
`view` extension because the samtools 1.24 fraction, exit-status, seed, and
platform contracts described below are contradictory.

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

Revision `fd3c65ccd682` adds fast and uncompressed BAM modes backed by BGZF
compression levels 1 and 0. Both modes produce the same decoded header and
records as samtools 1.24, explicitly finalize the BGZF stream, and retain
transactional file output. Exact-head CI run `30607097547` passes all four
native targets.

Revision `b1ee789ca942` first applied `-@` to BAM output without allocating
separate input and output pools. Revision `a2487fcd3d22` aligns the final worker
model with samtools: `-@ N` supplies exactly N additional compression workers,
while the calling thread coordinates output and BAM input does not allocate a
second decoder pool. SAM, BAM, and CRAM conversion plus indexed-region output
pass the samtools 1.24 oracle. Exact-head CI run `30610753005` passes all four
native targets and builds samtools 1.24 for the Linux `x86_64` differential.

Revision `a2487fcd3d22` also consumes published `rsomics-bamio 0.2.0` for
validated borrowed BAM records and bounded default-level BGZF output. Sequential
BAM count and BAM-to-BAM paths avoid decoding and re-encoding unchanged record
bodies. Complete record layout is validated before flag and mapping-quality
access; malformed bodies fail non-zero, and transactional named output remains
absent after the failure. Raw and decoded filter tests cover the same flag
predicates and missing-MAPQ behavior.

Revision `aa3e278206b4` records output provenance in the alignment header.
SAM and BAM output add a unique `@PG` record with program name, version,
sanitized real command line, and the previous program ID. Existing program
records remain in order, collisions use a numeric suffix, and `--no-pg` plus
the samtools-compatible `--no-PG` alias suppresses the new record. Public
library callers must construct validated program fields explicitly. Exact-head
CI run `30611945848` passes all four native targets and the samtools 1.24
differential.

Revision `904f0e3e6b9d` adds `view --save-counts FILE`. The JSON records
processed, filter-accepted, and filter-rejected counts with the samtools 1.24
field contract across SAM, BAM, and CRAM input. A named count target cannot
alias the alignment input or primary output, and an existing count file is
replaced only after alignment processing succeeds. Standard output is rejected
as a count target so alignment and JSON streams cannot be mixed. Exact-head CI
run `30612644701` passes all four native targets; the Linux `x86_64`
differential builds samtools 1.24 and exercises all 12 oracle groups.

Revision `20963baca3ab` adds `view -m/--min-qlen`. Query length is the sum of
CIGAR operations that consume the read: `M`, `I`, `S`, `=`, and `X`. The raw
BAM and decoded SAM/CRAM paths share the same threshold contract, and a zero
threshold leaves the raw hot path without CIGAR traversal. Exact-head CI run
`30613196557` passes all four native targets and 13 samtools 1.24 oracle groups.

Revision `e481cb82f209` adds repeatable `view -r/--read-group`. Selected read
groups form a union, records without an `RG` tag remain selected, and output
headers retain only matching `@RG` records. Input and output headers are
separate so header projection cannot change CRAM read-group decoding. SAM,
BAM, CRAM, count, SAM output, and raw BAM-to-BAM paths are covered; non-string
`RG` tags fail non-zero. Exact-head CI run `30613925372` passes all four native
targets and 14 samtools 1.24 oracle groups.

Revision `5f47b594d32a` adds repeatable `view --add-flags` and
`--remove-flags`. Filtering and counts use the original flags; accepted output
then applies the union of additions followed by removals, so removal wins when
the same bit appears in both sets. A non-zero transformation leaves the raw
BAM-copy path and decodes the record before writing. SAM, BAM, and CRAM input
to SAM output plus BAM output match samtools 1.24. Exact-head CI run
`30614602607` passes all four native targets and 15 oracle groups.

Revision `f9c8bf0789ee` adds repeatable `view -N/--qname-file [^]FILE`.
Read-name files are parsed as bytewise whitespace-delimited sets and repeated
files form a union. A leading `^` selects the complement, while mixing include
and exclude files fails before alignment processing. Missing QNAME values match
the SAM `*` spelling, file open and read errors propagate, and unchanged BAM
output retains the borrowed-record path. SAM, BAM, and CRAM count and record
output plus BAM output match
[samtools 1.24](https://github.com/samtools/samtools/blob/1.24/sam_view.c#L284-L358).
Exact-head CI run `30615295251` passes all four native targets and 16 oracle
groups.

Revision `7b46246d0a5a` adds `view -l/--library`. The filter resolves library
membership through the input header's `@RG ID` to `LB` mapping, following the
[samtools 1.24 lookup](https://github.com/samtools/samtools/blob/1.24/bam.c#L35-L52).
Records without `RG`, with an unknown group, or whose group has no matching
`LB` are rejected. Decoded and borrowed BAM paths share one typed `RG` accessor,
and non-string tags fail non-zero. Library filtering does not project the
output read-group header. SAM, BAM, and CRAM count and record output plus BAM
output match the oracle. Exact-head CI run `30615973249` passes all four native
targets and 17 oracle groups.

Revision `3b4896ac19bd` adds repeatable `view -x/--remove-tag` and
`--keep-tag`, including the samtools `-x ^TAG` keep shorthand. Values are
unions of comma-separated two-byte tags. Filtering and counts inspect the
original record; selected output preserves field order while removing the
chosen tags or the complement. Keep and remove modes are mutually exclusive
and fail non-zero when mixed, rather than copying
[samtools 1.24's silent keep precedence](https://github.com/samtools/samtools/blob/1.24/sam_view.c#L232-L272).
Tag-changing BAM output deliberately leaves the borrowed raw-copy path and
uses typed records. SAM, BAM, and CRAM output plus BAM output match the oracle
for each valid mode. Exact-head CI run `30616604980` passes all four native
targets and 18 oracle groups.

Revision `d3be2001212a` adds the first `mpileup` slice as a real subcommand,
not a wrapper around the deleted micro-crate. Coordinate-sorted SAM, BAM, and
CRAM records feed `rsomics-pileup 0.3.0`; the raw BAM path validates and copies
record bodies while SAM and CRAM use `rsomics-bamio 0.3.0` to produce the same
checked representation. The implementation includes samtools-compatible
quality and flag filters, per-input depth, anomalous-pair policy, overlapping
mate adjustment, covered/used/all-reference position modes, indexed rolling
reference access, standard and redo BAQ, and exact pileup text for insertions,
deletions, skips, and read boundaries. Named output is transactional and
machine summaries use the shared rsomics JSON envelope. The local release gate
passes formatting, strict Clippy, debug and release tests, rustdoc, clean
packaging, and all 19 samtools 1.24 oracle groups, including the new SAM, BAM,
CRAM, BAQ, overlap, and indel matrix. Exact-head four-native-target CI
`30654810659` passes; its Linux x86_64 job rebuilds and runs the pinned
samtools 1.24 oracle. This closes the second concrete consumer gate for the
pileup foundation, but does not make the incomplete BAM product publishable.
Revision `ae1c1561f941` then replaces the product-private BAI/CSI/CRAI
discovery and indexed-reference setup with the shared bamio 0.3 contract,
removing 101 duplicate lines. All 19 samtools 1.24 oracle groups pass locally
and exact-head four-native-target CI `30659400959` passes.

Revision `0d1a38d3f172` aligns the product on `rsomics-common` 0.10,
`rsomics-bamio` 0.4, and `rsomics-pileup` 0.4 without duplicate dependency
versions. The 27 library tests, 40 ordinary compatibility cases, four mpileup
cases, and 19 live samtools 1.24 oracle groups pass in debug and release mode;
exact-head four-native-target CI `30715405895` passes.

Revision `78925d1d019b` adopts published `rsomics-bamio 0.4.1`; all 19 live
samtools 1.24 oracle groups pass locally and exact-head four-native-target CI
`30722822003` passes. Revision `2e3781c5eaa5` removes the remaining
input-reachable raw-record and pileup conversion panics, replacing them with
explicit invalid-input errors. The 27 library tests, 40 ordinary compatibility
tests, four pileup tests, all 19 live oracle groups, strict Clippy, rustdoc, and
package verification pass; exact-head CI `30723046915` passes all four native
targets.

Revision `c2441aef1efe` records the release performance gate. On the 3,000,000
record, 188,400,612-byte BAM fixture, five alternating Linux x86_64 rounds at
`-@ 4` produced byte-identical decoded headers and records. Median wall time
was 2.39 seconds for `rsomics-bam view` and 4.14 seconds for samtools 1.24;
median CPU time was 7.12 versus 9.02 seconds, and median peak RSS was 4,480
versus 10,752 KiB. Exact-head CI `30723489891` passes all four native targets,
including the 19-group samtools differential on Linux x86_64. This is a BAM
hot-path gate only; the release makes no throughput claim for SAM or CRAM.

### Release 0.5: streaming depth

Revision `ebf7f9606db8` adds `depth` without reviving the deleted
`rsomics-bam-depth` micro-crate. It accepts coordinate-sorted SAM, BAM, and
CRAM; merges multiple inputs into separate depth columns; supports input lists,
BED restriction, indexed regions, used- and all-reference zero-depth output,
base and mapping quality thresholds, read-length and flag filters, deletion
counting, overlapping-mate suppression, headers, transactional named output,
and machine-readable summaries. Reference dictionaries must agree across
inputs, and malformed or unsorted streams fail non-zero.

The single-BAM path borrows validated raw records from `rsomics-bamio`; the
multi-input path keeps one record of lookahead per source. Depth values use a
bounded ring whose capacity follows the maximum live alignment span rather
than reference length or file size. Long-CIGAR `CG:B,I` records use the shared
validated fallback. The historical implementation retained value only as an
algorithm and fixture seed: it buffered whole-reference events in hash maps,
accepted one BAM only, and lacked quality, region, multi-input, and overlap
contracts.

`depth` deliberately remains product-internal rather than extending
`rsomics-pileup`. Samtools depth and mpileup have materially different default
deletion, overlap, filtering, multi-input, and output semantics. No second
product currently needs the depth-specific accumulator contract, so promoting
it would violate the two-consumer foundation rule.

SAM, BAM, CRAM, multi-input, indexed-region, BED, zero-position, quality,
flag, deletion, overlap, and header outputs match samtools 1.24 in the live
oracle matrix. A 5,000,000-base, approximately 30x BAM also produced the exact
78,888,186-byte samtools output with SHA-256
`f9bbc936dab1d5e7ef17c834a505187edefcfb854935be7021614fd51aaf2a69`.
Formatting, strict Clippy, debug and release tests, rustdoc, and package
verification pass. Exact feature-head CI `30772124000` passes native Linux and
macOS on both `x86_64` and `aarch64`; its Linux `x86_64` job builds samtools
1.24 and runs all 19 prior oracle groups plus the depth differential.

The release scope does not expose samtools' deprecated no-op `-d` and `-m`
options, generic input-format option injection, diagnostic verbosity controls,
or a custom index-file argument. These are absent rather than accepted and
ignored. A later release adds any of them only with a concrete user contract
and tests.

### Release 0.6: alignment indexing

Revision `4639c3676283` replaces the retired BAI-only
`rsomics-bam-index` shell with the product `index` subcommand. It builds BAI
for coordinate-sorted BAM or BGZF SAM, CSI with configurable minimum shift for
the same inputs, and CRAI for CRAM. It supports a custom output, the legacy
second positional output, multiple input files, and explicit worker counts.
When `-@` is omitted, the product selects up to four additional workers; `-@ 0`
requests one-thread indexing. Machine output records the actual worker count,
format, index kind, minimum shift, and CSI depth.

Input and output paths are resolved before work begins. Existing hard links,
symlinks, alternate spellings, duplicate multi-input destinations, standard
input, standard output, and attempts to overwrite any alignment input fail
before indexing. BAM and BGZF SAM require a complete BGZF EOF marker, CRAM
requires its EOF container, and a named destination is replaced only after
HTSlib has built the full index and noodles has parsed it back successfully.
Malformed, truncated, unsorted, and out-of-range inputs therefore fail
non-zero without replacing an existing index.

The implementation uses the established HTSlib indexing core through a
product-private backend. The initial custom noodles indexer was discarded
after its representative BAM gate was 1.49 times slower than samtools and
produced a larger BAI. There is no public indexing foundation: format policy,
CLI selection, path ownership, and the performance decision belong to this
product, and no second product requires the construction API. The shared
`rsomics-bamio 0.8.4` change is narrower: indexed alignment readers now load
the samtools-default appended BAI for BGZF SAM, with a real region-query
consumer test.

BAI, CSI, CRAI, BGZF SAM, custom minimum shift, explicit workers, query output,
and `idxstats` results match samtools 1.24 in the live oracle. Ordinary tests
also cover transactional failure, hard-link aliases, multiple inputs, legacy
output syntax, JSON summaries, and explicit `-@ 0`. Formatting, strict Clippy,
debug and release tests, rustdoc, clean packaging, and publish dry-run pass
locally.

The exact 0.6.0 release binary at revision `dce21e7341cf` was measured over 20
alternating pairs on a 4,000,000-record, 77,438,045-byte BAM. Default
`rsomics-bam index` selected four additional workers and averaged 0.4280
seconds; default samtools 1.24 averaged 0.7675 seconds. The product path was
1.79 times as fast, used 13.34% less mean peak RSS, and won 19 of 20 pairs,
while spending 47.38% more mean CPU time. Every generated BAI was byte
identical with SHA-256
`9c904c043df9e2252bcb527a571ac46d8947882e6a3e4c53abc0fe6e01c0bb7f`.
This is a default BAM/BAI latency gate, not an equal-thread, CSI, BGZF SAM, or
CRAM performance claim.

The stable scope intentionally rejects CSI minimum shifts outside `1..=30`,
does not expose generic input-format option injection or diagnostic verbosity,
and does not index uncompressed SAM. CRAM always produces CRAI, including when
BAI/CSI selection flags are present, matching the format-level samtools
behavior.

Standalone `view -n` remains unresolved. Samtools 1.24 emits the two tagged
records from the current CRAM fixture with `view -n` but reports zero for
`view -c -n`. In the
[`sam_view.c` option switch](https://github.com/samtools/samtools/blob/1.24/sam_view.c#L1112-L1126),
`-r` requests the CRAM `SAM_RGAUX` field while `-n` does not; combining `-r`
and `-n` restores the expected count. The option stays unimplemented until
the compatibility decision explicitly chooses the consistent record-filter
contract or the 1.24 count bug.

Read-group file exclusion is also unresolved. Samtools 1.24 correctly applies
`view -R ^FILE` as a record-level exclusion, but its unconditional
[`sam_hdr_remove_lines` call](https://github.com/samtools/samtools/blob/1.24/sam_view.c#L1355-L1359)
retains only the excluded `@RG` IDs. The resulting output can contain records
tagged with `rg2` while declaring only `rg1` in the header. `-R` stays
unimplemented until the compatibility decision chooses this contradictory
header or a consistent complement projection.

Unselected output has a related header boundary. With `view -r rg1 -U
rejected.sam`, samtools 1.24 writes the selected header projection to both
files, so the rejected file can contain `RG:Z:rg2` while declaring only
`@RG ID:rg1`. It also silently ignores `-U` in count mode. `-U` stays
unimplemented until the compatibility decision chooses between those
behaviors and the recommended dual-header contract: the selected output uses
the projected header, the rejected output uses the complete input header, and
`-c -U` fails non-zero.

The samtools 1.24 subsampling audit found two unresolved compatibility
boundaries. Its documentation defines a retained fraction from zero through
one, but `--subsample 0` retains every record and `NaN` is accepted; invalid
negative, greater-than-one, and infinite fractions print an error while the
process still exits zero. `rsomics-bam` must retain its non-zero failure
contract, and the zero-fraction behavior requires an explicit compatibility
decision. Samtools also scrambles non-zero seeds through platform libc
`rand()`: seed 1 becomes 16807 on macOS and 1804289383 on glibc Linux. The
implementation must not accidentally claim cross-platform-identical selection
while matching this platform-dependent step.

Subsampling, direct `-n`, read-group file exclusion, unselected output, CRAM
output, and parallel CRAM decoding are not part of release 0.4. None appears in
the CLI or public documentation. They become release gates only when a later
version chooses and exposes their contracts; no placeholder behavior is
accepted. The stable 0.4 commands instead fail explicitly when a requested
format or thread mode is unavailable.

The locked noodles-cram 0.93 synchronous reader exposes sequential
`read_container`, `records`, and query iteration but no worker-count or
multithreaded reader API. The current noodles-cram 0.95
[`Reader`](https://docs.rs/noodles-cram/0.95.0/noodles_cram/io/reader/struct.Reader.html)
has the same synchronous boundary; its optional async runtime does not itself
provide ordered parallel container decoding. The product will not claim CRAM
thread support by merely accepting `-@`. A custom container pipeline would
first need explicit ordered emission, reference-cache ownership, bounded
decoded-slice memory, error cancellation, and native-platform performance
evidence. Until that contract exists, CRAM input with a non-zero thread request
continues to fail before processing.

### Slice 2: file lifecycle

- `sort`, with bounded memory, external runs, merge fan-in, temporary-path
  ownership, and coordinate/name/template-coordinate modes;
- `merge`, `collate`, `cat`, `reheader`, `split`, `fixmate`, and `markdup`.

This slice requires explicit header reconciliation, reference dictionary
validation, `@RG` and `@PG` translation, stable tie behavior, transactional
outputs, and cleanup after failure. The historical in-memory sorter and
first-header merge are not acceptable implementations.

Release 0.7 implements `sort` without claiming the rest of Slice 2. SAM, BAM,
and CRAM inputs share coordinate, natural query-name, bytewise query-name, and
template-coordinate ordering with BAM output. The implementation enforces a
total record-memory budget, external compressed runs, a 32-way maximum merge
fan-in, multi-pass merging, stable ties across runs, validated end markers,
transactional named output, and temporary-file cleanup. A forced test creates
more than 32 runs and proves tie stability through at least two merge passes.
The live samtools 1.24 differential covers all four orders, all three input
formats, and forced external runs. Tag sorting, minimizer sorting, CRAM output,
write-index coupling, and compression-level controls remain unexposed rather
than appearing as placeholders.

The product reused its existing `rsomics-bamio` raw records, input boundary,
transactional writer, and mandatory `rsomics-help` command tree. Program-header
provenance moved to one product-internal type shared by `view` and `sort`; the
old public `rsomics_bam::view::Program` path remains available. No new Layer A
item was justified.

Revision `83b73a0c7274` implements the ordered, header-aware `merge`. It is the
second concrete consumer of the product's coordinate, natural-name,
bytewise-name, and template-coordinate keys; the shared ordering model now
lives in a private product module without changing the public foundation set.
The command accepts up to 32 named SAM, BAM, and CRAM inputs, writes BAM,
validates each input's declared and observed order, reconciles reference
dictionaries, deterministically resolves read-group and program collisions,
translates record reference, mate, `RG`, and `PG` fields, and commits named
output transactionally. Per-input readers emit approximately 1 MiB raw-record
batches through capacity-one channels, keeping read-ahead bounded by input
count and the largest record rather than total file size. Conflicting
reference definitions, order-destroying dictionary layouts, unknown record
header IDs, truncation, and finalization errors fail non-zero.

The ordinary suite covers reconciliation and translation, combine modes,
transactional failure, declared and observed order, stdout finalization,
dictionary conflicts, path aliases, and the input cap. The live samtools 1.24
matrix covers all four ordering modes plus mixed SAM, BAM, and CRAM inputs.
Because the ordering code moved out of `sort`, its complete samtools oracle was
also rerun. Formatting, strict Clippy, debug and release tests, package
verification, and exact-head CI run `30784029825` pass; that CI runs native
Linux and macOS on both `x86_64` and `aarch64` and builds the pinned oracle on
Linux `x86_64`.

The 8,000,000-record natural-name BAM gate used feature revision
`83b73a0c7274`, the 4,000,000-record fixture twice, and 12 alternating default
pairs. `rsomics-bam merge` averaged 3.0158 seconds versus samtools 1.24 at
9.3550 seconds, a 3.10 times wall-time advantage. Mean peak RSS was 20,959,232
versus 13,873,152 bytes and mean CPU time was 1.34 times samtools, so neither
is claimed as an advantage. With four additional workers for both tools,
eight pairs averaged 3.3613 versus 3.3288 seconds and did not establish a
throughput advantage. Every warm-up and timed output matched complete headers
and order-sensitive full-record checksums.

This release does not expose tag sorting, region/BED restriction, custom
indexes, write-index coupling, arbitrary compression levels, CRAM output, or
thousands-of-files fan-in. These remain absent until their complete contracts
and performance evidence exist. The historical merge repository supplied only
a fixture and heap-shape seed: its first-header policy, BAM-only reader,
fallible-coordinate suppression, and non-transactional output were discarded.

The next stable operation is standard `collate`. It accepts SAM, BAM, and CRAM
input, emits BAM, places every QNAME in one contiguous group, and sets
`SO:unsorted` plus `GO:query`. Ordering between QNAME groups is deliberately
unspecified. The implementation may use a deterministic hash order, but that
order is not a public contract. Memory is a total record budget, temporary
runs have bounded merge fan-in, named output is transactional, and temporary
files are removed after success or failure. Its compatibility oracle compares
the complete record multiset, QNAME-group contiguity, first/second ordering
within groups, header semantics, filtering absence, and failure behavior rather
than requiring samtools' incidental group order.

Fast `collate -f`, compression-level selection, SAM or CRAM output, and the
samtools prefix-as-output convention are not part of this first collate slice.
Fast mode has a different primary-paired-record filter and bounded early-pair
buffer; it remains absent until both that filtering contract and spill behavior
have direct oracle coverage. The historical `rsomics-bam-collate` whole-file
`HashMap` is retained only as a fixture seed. Its first-seen group order and
unbounded record retention are discarded.

This operation is the input-preparation edge of the real
`collate -> fixmate -m -> coordinate sort -> markdup` workflow. The following
slice implements `fixmate` against name-grouped input before `markdup` is
exposed. Shared ordering, temporary-run, and output plumbing remains private to
the BAM product; this workflow creates no new Layer A requirement.

### Slice 3: projection, pileup, and statistics

- `consensus`, `calmd`, `depad`, `phase`, `reference`, and
  `targetcut`;
- `bedcov`, `coverage`, `idxstats`, `stats`, `ampliconstats`, and
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
├── alignment_order.rs
├── lib.rs
├── main.rs
├── cli.rs
├── header_merge.rs
├── input.rs
├── md.rs
├── merge.rs
├── output.rs
├── sort.rs
├── commands/
│   ├── depth.rs
│   ├── flags.rs
│   ├── flagstat.rs
│   ├── head.rs
│   ├── index.rs
│   ├── merge.rs
│   ├── mpileup.rs
│   ├── quickcheck.rs
│   ├── samples.rs
│   ├── sort.rs
│   └── view.rs
└── filter.rs
```

Format detection, alignment headers, decoded-record policy, and indexed access
remain private product modules. `rsomics-bamio` contains only the policy-free
validated raw-record and bounded BGZF primitives already exercised by this
product, with `rsomics-methyl` and `rsomics-peak` recorded as the next concrete
reader consumers. Product modules own command policy, filter composition,
transactional path ownership, user-facing output, and samtools compatibility
choices. Later slices add command modules only when their implementation is
real.

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
| `rsomics-bam-depth` `cdc0a4ff70119edc193cd6bdfadaba6b6e190b61` | Test and algorithm seed; replacement merged | `depth`; discard whole-file event maps and keep the accumulator product-internal |
| `rsomics-bam-divide` `71504b275797ec30df2399ef2fbe03d1c9b1e6b5` | Refactor then merge | `split --parts`; preserve disjoint-cover and seeded-partition fixtures |
| `rsomics-bam-fasta` `ba661eddd57b45f725751f02a288546442acd3e7` | Refactor then merge | `fasta` |
| `rsomics-bam-fixmate` `645e4e3c31f3e689e854c2de63e726b877d770ea` | Refactor then merge | `fixmate`; include supplementary mate behavior from 1.24 |
| `rsomics-bam-flags` `921a428ba5e11f47fca875e1b9ae1335b3b5cb8f` | Refactor then merge after dirty-diff attribution | `flags` |
| `rsomics-bam-flagstat` `ce1cc819d59fe37a56c762ba005ba0d9c91d3ba3` | Refactor then merge | First-slice `flagstat` |
| `rsomics-bam-head` `76ffd4d379191a968f1095a1854d0ce4c8fe49db` | Refactor then merge | First-slice `head` |
| `rsomics-bam-idxstats` `f96b6aed4452243a982c9d7ca495e6fa23d8b497` | Refactor then merge | `idxstats`; require index-kind coverage |
| `rsomics-bam-import` `ba7f8fc7630676e1cdbe95a21c0ae35677f5b958` | Refactor then merge | `import`; share `rsomics-seqio` only through a concrete contract |
| `rsomics-bam-index` `167e86bd0f5ee0cf13bf18e9ded89cb1f99a46a5` | Test asset; replacement merged at `4639c3676283` | `index`; discard the BAI-only wrapper |
| `rsomics-bam-markdup` `e865796930fb72d8a185e3a0b18024d217ca6128` | Refactor then merge | `markdup`; retain scoring and duplicate fixtures |
| `rsomics-bam-merge` `7334fce53ec3666f63893b450710daa4efd43641` | Test asset; replacement merged at `83b73a0c7274` | Discard first-header policy and swallowed decode failures |
| `rsomics-bam-mpileup` `5e51a7825384fd65aca38345a12ad7c89ad31143` | Refactor then merge after pileup API | Add BAQ and reference-aware default behavior |
| `rsomics-bam-phase` `9f475c325e8e8c30873a12df5979c44023e78c1d` | Test and algorithm asset | Replace tolerance-only compatibility decisions |
| `rsomics-bam-quickcheck` `5982123dbed16ab0f625495d550630c43d55f3ba` | Refactor then merge | First-slice `quickcheck`; cover all three formats |
| `rsomics-bam-region` `902f6f333a9d0ea623006f76d4e360e4fe5f5f0f` | Merge useful predicates | First-slice `view --region` |
| `rsomics-bam-reheader` `bdf6f6ec0ed0b16307e781b0ef335dc71699cae2` | Refactor then merge | `reheader`; transactional BAM and CRAM paths |
| `rsomics-bam-reset` `121947733112098c2b66d6151c23331cb4307e1f` | Refactor then merge | `reset`; current flag and auxiliary-tag behavior |
| `rsomics-bam-samples` `40b39137a2f03333a7b9af0505b43ccffc311bc9` | Refactor then merge | First-slice multi-input `samples` |
| `rsomics-bam-sort` `99144c7ba8d9abe78add7301cb300e74b5c11fe0` | Test asset; replacement merged at `8433aea711d5` | Discard the whole-file `Vec` sorter |
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

The historical foundation at
`dc4b19df5bc6664b39088b938136afecf48e21a9`, version 0.1.10, was a
1,514-line BAM-oriented reader/writer with noodles and libdeflate paths. Its
public raw-record constructors accepted arbitrary bytes, several accessors
relied on indexing or `unwrap`, and a declared BAM block length was treated as
proof that all variable fields were internally consistent. The parallel writer
also discarded its sink on finalization, joined workers with `unwrap`, and did
not surface a final sink flush.

`rsomics-bamio 0.2.0` at `51257940677b` replaces those boundaries. The release:

- validates fixed fields, NUL-terminated read names, CIGAR, sequence, quality,
  and auxiliary-data layout with checked arithmetic before field access;
- makes owned and borrowed raw-record construction fallible and supplies a
  valid default unmapped record;
- writes borrowed raw records without product-specific policy;
- exposes a bounded ring-based BGZF writer whose `finish` returns the sink,
  flushes it, and maps worker failure or panic into structured I/O failure;
- reduces implementation commentary to public contracts and stable
  concurrency invariants.

Native Linux and macOS CI on `x86_64` and `aarch64` passed at exact-head run
`30610310217`. The controlled publish run `30610459857` produced the crates.io
package with SHA-256
`c763f5d7d93597718946912f7637347b799a1c41a60d57e615c04bd10eebffd3`.
The GitHub release is
[`rsomics-bamio-v0.2.0`](https://github.com/omics-rust/rsomics-bamio/releases/tag/rsomics-bamio-v0.2.0).
The published package is consumed from the registry by `rsomics-bam`.
Revision `3bcbe0ed9bb2` adds the policy-free `RawRecordEncoder` used by BAM for
generic SAM/CRAM records and by call for its alignment stream. Version 0.2.1
passed exact-head four-native-target CI `30653883521`, publish run
`30654036896`, and downloaded-archive verification with checksum
`2075d1a7c10a353437148743143b0a9326258bc0a82336ca7ae890cb38e49e00`.
Both consumers pass malformed-record, encoding, filtering, and round-trip
tests against the registry release.

Revision `94641eff97d7` adds the consumer-proven indexed reader without moving
product filtering or region policy into the foundation. It accepts BGZF SAM,
BAM, and CRAM, appended or common alternative BAI/CSI/CRAI names, and an
optional indexed reference. BAM and call both exercise the contract. Version
0.3.0 passed exact-head four-native-target CI `30658611800`, publish run
`30658840221`, and downloaded-archive verification with checksum
`6ac17eb096cd976f6000ff813430236df0b723eb360c926427d7928e46702a93`.

Revision `d563f0160c2a` aligns the foundation on `rsomics-common` 0.10 while
retaining the same consumer-proven API. Version 0.4.0 passed exact-head
four-native-target CI `30714717087`, publish run `30714794220`, and
downloaded-archive verification with checksum
`7dbfdde57d3f0553f962ab1836ff06ce59540637b6a501757690cdf66c85876b`.
BAM and call pass their complete consumer suites on this release.

The remaining multi-product foundation contract is:

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
and `rsomics-call`. BAM and call are the implemented 0.3.0 consumers.
`rsomics-methyl` and `rsomics-peak` have concrete dossier plans for validated
alignment readers; their product-specific methylation, fragment, filter, and
CLI policy remains outside the foundation. No additional public reader,
indexing, header, or transactional-path item is added until a second product
implements and tests the same policy-free contract.

### `rsomics-pileup`

The provenance baseline is revision
`5bd34dde15c5bc94e44d27a1ede2e9f9bf3e5fc2`, version 0.1.0. Revision
`2b2cb7071381bb4c86e8a7068f76c96b7035e1dd` replaces its infallible `feed`
boundary with a fallible validated stream and passes exact-head four-native
CI run `30635856887`.

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

Revision `2680f6c328be` checks header reference IDs and lengths, coordinates,
CIGAR kinds and spans, BAM `CG:B,I` long CIGAR, zero-reference-span behavior,
every mapped record's sorted watermark, and overlap adjustment. It adds
HTSlib-compatible standard and extended BAQ plus full and bcftools-compatible
partial column preparation. Exact-head four-native-target CI `30654312487`
passes; the Linux x86_64 job also runs the pinned samtools 1.24 column oracle.
The ordinary and 250× engine benchmark records bounded RSS and sustained
streaming with and without the partial trigger scan. Call and BAM now supply
the two concrete consumers. Version 0.2.0 was published by run `30654567905`;
the downloaded archive checksum is
`0a2d901c6854470dbebae190ef30d3535333768c4c18c6cc47c03eeb33872684`.
Revision `a69743a8097f` updates the shared raw-record dependency to bamio 0.3
without changing the hot path. Local samtools 1.24 compatibility and ordinary
and 250× benchmarks pass. Exact-head four-native-target CI `30659084469` and
publish run `30659248849` pass; the pileup 0.3.0 archive checksum is
`def4cc70d0cd250f8b9ebb1d1e0280c1a890cffb66504a832cb1819cff9f8581`.

Revision `4b48bfdafecd` aligns the shared raw-record dependency on bamio 0.4
without changing the projection hot path. Version 0.4.0 passed the samtools
1.24 oracle, debug and release suites, exact-head four-native-target CI
`30714930834`, publish run `30715042037`, and downloaded-archive verification
with checksum
`d33b5fa1c3ddbe86c8f53c2bb3fa870e482e90957aa5e559fceca0600ff56533`.

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

A provisional BAM-output comparison used a 170,283,848-byte BAM with 3,000,000
records and SHA-256
`48091653a1d4165be293df9bb7e5f1427bc6846e93e0a0b80dec38d47f1da1be`.
On the same Apple M2 host, after three warm-ups, ten trials measured:

| Implementation | Mean | Standard deviation | Range |
|---|---:|---:|---:|
| `rsomics-bam b1ee789 view -b -@ 0` | 6.398 s | 0.217 s | 6.129–6.841 s |
| `rsomics-bam b1ee789 view -b -@ 2` | 4.245 s | 0.254 s | 3.912–4.547 s |
| `rsomics-bam b1ee789 view -b -@ 4` | 2.192 s | 0.096 s | 1.947–2.301 s |
| `samtools 1.24 view -b -@ 4` | 1.509 s | 0.117 s | 1.343–1.706 s |

Four-thread rsomics output is about 2.92 times faster than its single-thread
path, so the bounded worker control has measured value. Samtools remains about
1.45 times faster at four threads, so the release-performance gate still
fails. Both decoded outputs had SHA-256
`91f653165a241b0a07b22e62be7850c795011836d0553212d03d96a02597abe2`.
The JSON result is retained at
`/Volumes/KIOXIA/Developments/tmp/rsomics-bam-view-output-thread-comparison.json`;
the run still lacks peak RSS and alternating trial order.

The validated raw-record integration at `a2487fcd3d22` supersedes that output
timing. It used the same 170,283,848-byte, 3,000,000-record BAM and the same
Apple M2 host. Every generated default, fast, and uncompressed BAM passed
samtools `quickcheck`, contained 3,000,000 records, and had decoded header and
record SHA-256
`91f653165a241b0a07b22e62be7850c795011836d0553212d03d96a02597abe2`.
The record-only digest was
`f0aa61994623f4701bf0b26f26a611d06fd87061180b6b004d1cf0481412e51d`.

After three warm-ups, 20 four-additional-thread trials measured:

| Implementation | Mean | Median | Standard deviation | Range |
|---|---:|---:|---:|---:|
| `rsomics-bam a2487fc view -b -@ 4` | 1.640 s | 1.489 s | 0.336 s | 1.413–2.803 s |
| `samtools 1.24 view -b -@ 4` | 1.830 s | 1.779 s | 0.175 s | 1.626–2.234 s |

The rsomics mean is 1.12 times faster and its median is 1.19 times faster,
despite one slow rsomics trial. Twenty count trials measured 0.901 ± 0.025
seconds for rsomics and 0.963 ± 0.025 seconds for samtools, a 1.07-times mean
advantage. Twelve single-thread output trials measured 6.236 ± 0.582 seconds
for rsomics and 6.498 ± 0.992 seconds for samtools; this 1.04-times mean
difference is noisy and is not treated as a strong claim.

The retained hyperfine JSON files and SHA-256 digests are:

| Measurement | Path | SHA-256 |
|---|---|---|
| four-thread output | `/Volumes/KIOXIA/Developments/tmp/rsomics-bam-view-raw-parallel-final.json` | `13bbf1601700fe0e71d869a34bfb18ca89c71f7556404701302b9391527f8cab` |
| count | `/Volumes/KIOXIA/Developments/tmp/rsomics-bam-view-raw-count-final.json` | `06bd1b04df612185aba47421e5c55083f516bf0ec47223fb697d50079bda9a24` |
| single-thread output | `/Volumes/KIOXIA/Developments/tmp/rsomics-bam-view-raw-single-final.json` | `78163ee5a6eb662d0cfc51b92f3b91efe0aa497de047ab6d6805b8e68f1079cc` |

The formal Linux x86_64 gate at `c2441aef1efe` supersedes these provisional
runs. It records peak RSS, alternates five measured rounds after warm-up, and
verifies the decoded header and all 3,000,000 records after every round. SAM
and CRAM remain correctness contracts in release 0.4; no cross-format
throughput claim is made.

The release 0.5 depth gate used revision `ebf7f9606db8`, samtools/HTSlib 1.24,
and a 36,459,282-byte coordinate-sorted BAM containing 1,000,000 records over
5,000,000 reference bases at approximately 30x coverage. The fixture SHA-256
is `33b6780ec3758a8ccde746935366dec441e89aaafb5b0253a19cfa1af350282c`.
On an 8 GiB Apple M2 Mac mini, one warm-up preceded 20 AB/BA paired rounds.
Both tools formatted the complete depth stream to `/dev/null`; a separate
complete-file pass compared all 78,888,186 output bytes exactly.

| Tool | Mean wall | Mean user | Mean system | Mean peak RSS |
|---|---:|---:|---:|---:|
| `rsomics-bam depth` | 0.4400 s | 0.3735 s | 0.0200 s | 3,978,854 bytes |
| `samtools depth` | 0.4900 s | 0.4020 s | 0.0315 s | 6,572,442 bytes |

The paired wall-time difference was -0.0500 +/- 0.0801 seconds, with a paired
t-statistic of -2.790 and rsomics winning 16 of 20 pairs. This fixture shows a
10.20% wall-time, 7.09% user-time, 36.51% system-time, and 39.46% peak-RSS
reduction. The reproducible benchmark script is tracked in the product. The
timing ledger and generated summary SHA-256 values are
`ca2cad4d5d55aa0e492f1916c1d7c3ccf646308c63323b6615b99fa5afaec193`
and `dccf26d1c6b115e7a86811ef0a7114ba998ade2a94a76f8c99936a5cca25ea45`.
This is a default BAM depth-path claim only.

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

`rsomics-bam 0.4.0` is published from `e025f22de09d`. The stable seven-command
slice, explicit exclusions, samtools 1.24 compatibility, BAM hot-path
advantage, package, public metadata, and all four native CI targets are
complete. Exact-head CI `30723735961` passes; the Linux x86_64 job includes
strict Clippy, package verification, and all 19 live oracle groups.

A publication workflow was added before this dossier gate was rechecked and
briefly published source revision `0d1a38d3f172` as 0.4.0 in run
`30715708839`. The exact archive is retained with SHA-256
`528b6103da6ab1c7bc3aa43a84e53301d201987ff4a7c83202dd4ef0115c440c`.
The crate was immediately deleted under the registry's new-release deletion
rule; the live registry API returns 404, publication-secret access was removed,
and revision `0d81a978f9d2` removes the premature workflow. Its exact-head
four-native-target CI `30715893286` passes. This event does not satisfy or
weaken any product release gate.

The gated retry in run `30723829713` packaged and verified all 48 files and
reached the crates.io upload endpoint. The registry rejected only the recently
deleted name, reporting that reuse becomes available after
`2026-08-02T19:56:02Z`. No package was created. The BAM repository was removed
again from the selected publication-secret repositories after the failure.

After the cooldown, publish run `30766493471` completed from the same selected
revision. A separate registry lookup downloaded `rsomics-bam 0.4.0` and
confirmed its package metadata. The downloaded crate archive has SHA-256
`016736c669e52155b999da533f78325c06e09c003946539ff6feb1767885762e`.

`rsomics-bam 0.5.0` is published from release head `6d50829ec0d0`. Exact-head
CI `30773087784` passes native Linux and macOS on `x86_64` and `aarch64`; the
Linux `x86_64` job includes strict Clippy, package verification, samtools 1.24,
the 19 prior oracle groups, and the depth differential. Publish run
`30773247644` succeeds from the same revision. An independent static-registry
download matches the locally verified package byte for byte with SHA-256
`65da80cd273df134369bc21278843d5fc744d90c814dd43a06284cc1d6b729f8`.
Its registry metadata declares Rust 1.91 and the embedded VCS revision is the
exact release head.

`rsomics-bam 0.6.0` is published from release head `b38546813217`. Exact-head
CI `30778102423` passes native Linux and macOS on `x86_64` and `aarch64`; the
Linux `x86_64` job includes strict Clippy, package verification, samtools 1.24,
the prior compatibility groups, and the alignment-index differential. Publish
run `30778426695` succeeds from the same revision. An independent
static-registry download is byte-identical to the clean local package with
SHA-256
`678d0e59f4b4d5e4fb9e5540a2ad2c2edd1c78efd535caee53184e07d5d387d9`.
The archive embeds the exact release head and registry metadata declares Rust
1.91.

`rsomics-bam 0.7.0` is published from release head `5e225f473b57`. Exact-head
CI `30781282074` passes native Linux and macOS on `x86_64` and `aarch64`; the
Linux `x86_64` job includes strict Clippy, package verification, samtools 1.24,
all prior compatibility groups, and the new sort differentials. Publish run
`30781583903` succeeds from the same revision. The independently downloaded
registry archive is byte-identical to the clean local package with SHA-256
`cf5c17f5cf8b28a82bb07be6cabd4efba33ea1d456d070b1a05128f74e186f7c`.
Its embedded VCS revision is the exact release head, registry metadata declares
Rust 1.91, and `cargo info` resolves 0.7.0 from crates.io.

`rsomics-bam 0.8.0` is published from release head `01ba89e30b91`. Exact-head
CI `30784684718` passes native Linux and macOS on `x86_64` and `aarch64`; the
Linux `x86_64` job includes strict Clippy, package verification, samtools 1.24,
all prior compatibility groups, and the new merge differentials. Publish run
`30784995810` succeeds from the same revision. The independently downloaded
registry archive is byte-identical to the clean local package with SHA-256
`22b59bdbc7fae1e6502719b1050f0413598ff872cb9bc924e104f2146d32f0e3`.
Its embedded VCS revision is the exact release head, the release is not yanked,
and registry metadata declares Rust 1.91. A fresh locked registry install
reports version 0.8.0, exposes the complete unified merge help, and writes a
seven-record BAM that passes samtools quickcheck.

The default coordinate-sort gate at implementation revision `8433aea711d5`
used 20 alternating pairs on a 4,000,000-record, 77,438,055-byte
query-name-sorted WGBS BAM. `rsomics-bam sort` averaged 6.7300 seconds and
848,793,600 bytes peak RSS; default samtools 1.24 averaged 11.5080 seconds and
960,046,694
bytes. The product path won 20 of 20 pairs, was 1.71 times as fast by mean wall
time, used 11.59% less mean peak RSS, and used 31.81% more CPU time through its
default automatic parallelism. Every warm-up and measured pair had identical
headers and order-sensitive full-record checksums. The ledger, environment,
summary, fixture, and outputs are retained under
`/Volumes/Zane's HDD/rsomics-fixtures/bam/sort-4m-20260803`; this gate does not
claim an equal-worker advantage or extend to other sort orders or input
formats.
