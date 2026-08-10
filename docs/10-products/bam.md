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
published as `rsomics-bam 0.8.0`. The twelfth command, `collate`, is published
as `rsomics-bam 0.9.0` after the same gates. The thirteenth command, `fixmate`,
is published as `rsomics-bam 0.10.0` after compatibility, representative
performance, package, and four-native-target gates.
The fourteenth command, `markdup`, is published as `rsomics-bam 0.11.0` after
correctness, compatibility, representative performance, package, and
four-native-target gates. The fifteenth and sixteenth commands, `cat` and
`reheader`, are published as `rsomics-bam 0.12.0` after the same release
gates. The seventeenth and eighteenth commands, `fasta` and `fastq`, are
implemented as one shared extraction engine at revision `d6cbf1070706` and
have passed their local correctness, live-oracle, package, and representative
performance gates. They are published as `rsomics-bam 0.13.0` from revision
`4829bbb3be06` after exact-head CI `31376119277` passed on native Linux and
macOS for both x86_64 and aarch64, including the FASTA/FASTQ samtools 1.24
oracle on Linux x86_64. Publication workflow `31376581939` completed
successfully. The nineteenth command, `import`, is published as
`rsomics-bam 0.14.0` from revision `d54924462ad6` after exact-head CI
`31383580026` passed the four native targets and the complete samtools 1.24
oracle. Publication workflow `31384051315` completed successfully.
The twentieth command, `addreplacerg`, is published as `rsomics-bam 0.15.0`
from revision `fe2beb388a75` after exact-head CI `31387911685` passed the same
four native targets and complete oracle. Publication workflow `31388331846`
completed successfully. The twenty-first through twenty-third commands,
`bedcov`, `coverage`, and `idxstats`, are published together as
`rsomics-bam 0.16.0` from revision `be3cafe21867` after exact-head CI
`31398246573` passed the four native targets and complete samtools 1.24 oracle.
Publication workflow `31398905778` completed successfully.
The twenty-fourth command, `calmd`, is published as `rsomics-bam 0.17.0` from
revision `0debc103993f` after exact-head CI `31407557237` passed the same four
native targets and complete oracle. Publication workflow `31408461408`
completed successfully. The twenty-fifth command, `depad`, is published as
`rsomics-bam 0.18.0` from revision `5304f278bfaa` after exact-head CI
`31414206433` passed the same four native targets and complete oracle.
Publication workflow `31415017446` completed successfully. The paired
`ampliconclip` and `ampliconstats` workflow is published as `rsomics-bam
0.19.0` from revision `6d82ba05b172` after exact-head CI `31424768417` and
publication workflow `31425446427` completed successfully. The twenty-eighth
command, `cram-size`, is published as `rsomics-bam 0.20.0` from revision
`5ecdcc33ccbe` after exact-head CI `31431000225` and publication workflow
`31431664650` completed successfully.

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

Revision `24095b8650c2` implements standard `collate`. It accepts SAM, BAM,
and CRAM input, emits BAM, places every QNAME in one contiguous group, and sets
`SO:unsorted` plus `GO:query`. Ordering between QNAME groups is deliberately
unspecified. The implementation may use a deterministic hash order, but that
order is not a public contract. Memory is a total record budget, temporary
runs have bounded merge fan-in, named output is transactional, and temporary
files are removed after success or failure. Its compatibility oracle compares
the complete record multiset, QNAME-group contiguity, first/second ordering
within groups, header semantics, filtering absence, and failure behavior rather
than requiring samtools' incidental group order.

The ordinary suite covers group and segment ordering, header behavior, stdout
finalization, aliases, transactional failures, and a forced case with more
than 32 temporary runs and at least two merge passes. The pinned samtools 1.24
matrix covers SAM, BAM, CRAM, and forced external collation. All prior samtools
oracles were rerun. Feature CI `30786031225`, performance-record CI
`30786898804`, and release CI `30787583253` each pass native Linux and macOS on
`x86_64` and `aarch64`.

The representative default gate used a 4,000,000-record WGBS BAM and 12
alternating pairs. `rsomics-bam collate` averaged 8.3400 seconds versus
samtools 1.24 at 13.5742 seconds, a 1.63 times wall-time advantage, and won all
12 pairs. With four additional workers for each tool, eight pairs averaged
9.2463 versus 11.6563 seconds, a 1.26 times advantage, and the product won all
eight pairs. The product used more CPU and peak RSS in both configurations, so
neither is claimed as an advantage. Every output had an identical complete
header, an equal full-record multiset fingerprint, and contiguous QNAME groups.

Fast `collate -f`, compression-level selection, SAM or CRAM output, and the
samtools prefix-as-output convention are not part of this first collate slice.
Fast mode has a different primary-paired-record filter and bounded early-pair
buffer; it remains absent until both that filtering contract and spill behavior
have direct oracle coverage. The historical `rsomics-bam-collate` whole-file
`HashMap` is retained only as a fixture seed. Its first-seen group order and
unbounded record retention are discarded.

This operation is the input-preparation edge of the real
`collate -> fixmate -m -> coordinate sort -> markdup` workflow. The following
stable slice implements `fixmate` against valid name-grouped input before
`markdup` is exposed. It includes default mate-coordinate, flag, TLEN, MC, and
MQ repair plus `-m`, `-r`, and `-p`. Supplementary records receive mate
coordinates, orientation, MC, and MQ from the opposite primary as in samtools
1.24. Record order is preserved. A coordinate-sorted declaration is rejected;
parse, truncation, output, and finalization failures remain fatal.

The command accepts SAM, BAM, CRAM, or standard input, emits BAM to a named
transaction or standard output, accepts a CRAM reference, uses bounded
QNAME-group memory, and shares the product's thread and program-header
contracts. The compatibility oracle is samtools 1.24 `fixmate -z off` over
primary, orphan, secondary, supplementary, mapped, unmapped, cross-reference,
missing-quality, existing-tag, and multi-primary templates. Sanitizer
selection, `-c`, `-M`, uncompressed output, and alternate output formats remain
absent until their independent contracts are implemented. The product accepts
valid records rather than silently presenting a partial sanitizer as the
samtools default.

The historical implementation supplies the raw-record editing shape, fixture
seeds, and performance seed. Its standalone CLI, comment-heavy source,
BAM-only boundary, incomplete supplementary and multi-primary handling,
missing sort-order and finalization checks, and non-transactional named output
are discarded. Shared record I/O and output plumbing remain private to the BAM
product; this workflow creates no new Layer A requirement.

Revision `a8a684ba57c6` implements this slice with the public product API limited
to typed `Options`, `Summary`, and `write`. It retains at most one QNAME group,
uses narrow mate snapshots rather than cloned records, edits validated raw BAM
records in place, and decodes `CG:B,I` only for long CIGARs. CRAM input passes
through the product's reference-aware MD/NM completion so the BAM result
matches HTSlib behavior. The ordinary suite covers long CIGARs, grouped
templates, stdin/stdout, transactional failure, and option semantics. The
samtools 1.24 matrix covers SAM, BAM, and CRAM across the default, `-m`, `-r`,
`-p`, and combined modes. Feature CI `31352649611` passes native Linux and
macOS on `x86_64` and `aarch64`.

The representative gate used 4,000,000 records in 2,000,000 consecutive
paired templates. Over 12 alternating default pairs, `rsomics-bam fixmate -m`
averaged 2.2708 seconds versus samtools 1.24 at 7.0083 seconds, won all pairs,
and used 7,002,795 versus 7,289,515 bytes mean peak RSS. The 3.09 times
wall-time advantage uses the product's automatic four additional compression
workers; its mean CPU time was 27.71% higher. With four additional workers for
both tools, means were 2.3300 and 2.5050 seconds with 7 of 12 product wins and
a -1.40 paired t-statistic, so no stable equal-worker throughput advantage is
claimed. Mean peak RSS was 7,028,736 versus 13,100,373 bytes, a 46.35%
reduction. Every warm-up and measured pair had an identical complete header
and order-sensitive full-record semantic hash after normalizing only auxiliary
tag order. Performance-record revision `68204535ccea` passes exact-head CI
`31353739943` on all four native targets.

The next editing increment is the non-optical core of samtools 1.24
[`markdup`](https://www.htslib.org/doc/1.24/samtools-markdup.html). It accepts
coordinate-sorted SAM, BAM, CRAM, or standard input produced by `fixmate -m`,
emits transactional BAM to a named path or BAM to standard output, and retains
the product's reference, compression-worker, program-record, and structured
summary conventions. Default template mode and explicit sequence mode use
unclipped five-prime coordinates, orientation, left/right template position,
and the mate CIGAR. Paired reads outrank single reads at the same signature;
otherwise the highest sum of base qualities at least 15 wins, a non-QC-fail
read outranks a QC-fail read, and equal paired scores select the
lexicographically smaller QNAME.

The stable CLI slice includes removal of duplicate records, clearing prior
duplicate flags and `do`/`dt` tags before a fresh decision, inclusion of
QC-failed primary reads, template or sequence decision mode, and a configurable
maximum read length for the bounded coordinate window. Existing duplicate
flags remain authoritative unless clearing is requested, so removal also
removes already flagged records. Secondary, supplementary, and unmapped
records otherwise pass through unchanged. The implementation rejects a
query-name-sorted declaration, any observed decrease in mapped coordinate
order, missing or wrongly typed `MC` and `ms` tags when a paired comparison
requires them, malformed CIGAR or auxiliary data, truncated input, a required
but unavailable CRAM reference, same-target input/output, output or
finalization failure, and
named BAM or CRAM input without its required EOF marker.

The operation is one bounded streaming engine inside `rsomics-bam`. Validated
raw records enter through the existing product input boundary; a key module
computes checked ordinary and `CG:B,I` long-CIGAR geometry; a marker owns the
live coordinate window and winner maps; the existing output boundary writes
records and commits named output only after successful input validation and
writer finalization. The public library surface is limited to typed mode,
options, summary, and write interfaces. This creates no new Layer A API:
`rsomics-bamio` already owns policy-free record validation and raw editing,
while duplicate-selection policy stays in the BAM product.

The samtools 1.24 source tests for query-name order, observed bad order,
missing `MC`, missing `ms`, default marking, and removal are direct oracle
fixtures. Product tests add template and sequence modes, paired-versus-single,
quality and QNAME tie breaks, QC-fail inclusion, clearing, pre-existing
duplicates, soft and hard clipping, `CG:B,I`, SAM/BAM/CRAM, stdin/stdout,
transactional failure, and EOF failure. Data-output comparison covers header,
record order, every field, auxiliary tags, and duplicate flags; diagnostics
need not clone incidental samtools wording. The representative performance
gate will use a non-trivial coordinate-sorted output of the published
`collate -> fixmate -m -> sort` product path, alternate runs against samtools
1.24 at default and equal worker budgets, record wall time, CPU, peak RSS,
machine and fixture provenance, and require semantic identity for every run.

Supplementary/secondary/unmapped propagation (`-S`) is not in this increment:
it requires an independently tested disk spool and second pass because a
non-primary alignment can precede the duplicate primary. Optical duplicate
coordinates and chains, barcode or UMI partitioning and movement, read-group
partitioning, original-name and duplicate-count tags, samtools-specific stats
files, alternate output formats, write-index, and uncompressed output also
remain absent. They will be added only as complete compatible extensions, not
as accepted but ineffective flags. The historical `rsomics-bam-markdup`
revision `e865796930fb` supplies scoring and signature seeds, three small
compatibility fixtures, and a performance baseline. Its standalone CLI,
BAM-only raw stream, template-only key, unchecked auxiliary reads, ordinary-
CIGAR-only geometry, comment-heavy source, non-transactional output, and
partial statistics are discarded.

Release 0.11 implements this stable slice at product revision
`c581a8955f6b`. The bounded engine supports default template and sequence
signatures, marking or removal, clearing prior duplicate state, QC-fail
inclusion, long CIGARs, SAM/BAM/CRAM and standard input, transactional BAM
output, explicit compression-worker control, program provenance, and the
shared JSON envelope. It preserves exact samtools 1.24 CRAM field and
auxiliary-tag order by completing only missing `MD` and `NM` tags. The local
release gate passes 35 library tests, 25 markdup lifecycle tests, ten direct
markdup compatibility tests, the complete product integration suite, all 27
release-oracle tests, strict Clippy, rustdoc, and clean crate-package
verification. Exact-head CI `31359628927` passes on native Linux and macOS for
both x86_64 and aarch64. A lifecycle regression test also exposed and fixed a
shared single-thread BAM finalization path that had written two EOF blocks; the
corrected writer and the pre-existing sort fixture now use one standard EOF.

The representative four-million-record BAM gate uses feature revision
`5c7dc5603dab` and compares the complete `samtools view -h --no-PG` stream on
every warm-up and timed run. With default worker policies, the product averaged
2.3475 seconds versus samtools 1.24 at 7.6617 seconds, won all 12 pairs, and
used 6,980,949 versus 7,587,157 bytes mean peak RSS. With four additional
workers for both tools, means were 2.5317 and 2.7792 seconds with ten product
wins, a -2.26 paired t-statistic, 8.63% lower mean CPU time, and 48.82% lower
mean peak RSS. The exact decoded stream fingerprint is
`6279ec79c152d1b2f6092b31021a32f8a62935615a0e2f3668c42e9a17011c99`.
These claims cover default duplicate marking on this BAM fixture; other input
formats, removal, clearing, sequence mode, and different duplicate
distributions remain correctness contracts rather than performance claims.
Publication workflow `31360005824` released 0.11.0 from the exact tested head.
The unyanked 133,328-byte registry archive has SHA-256
`9134d34ef7752f8f3be0f38d1680c8a879a61c164211d5e557ea49a67edcd285`.
A fresh registry install reports 0.11.0, exposes the shared command tree, and
matches samtools byte-for-byte on the complete 16-record markdup smoke stream;
docs.rs serves the corresponding public library documentation.

The next file-operation increment consolidates samtools 1.24
[`cat`](https://www.htslib.org/doc/1.24/samtools-cat.html) and
[`reheader`](https://www.htslib.org/doc/1.24/samtools-reheader.html). Both are
header rewrites followed by compressed-record passthrough, so they share one
private BGZF rewrite engine inside `rsomics-bam`; they are not separate
products and do not create a Layer A API. The first stable slice is BAM-only.
CRAM concatenation has container-version, read-group ordering, region,
fraction, and boundary-recoding semantics, while CRAM reheadering has distinct
container and in-place layouts. Those paths remain absent until complete CRAM
output support can satisfy their own compatibility and performance gates.

`cat [--list FILE]... [--header FILE] [-o FILE] [--no-pg] BAM...` accepts one
or more named BAM inputs, repeated files of filenames, an optional alignment
file supplying the output header, a named output or standard output, and
program-record suppression. List entries precede
positional inputs as in samtools. It preserves record order and compressed
record blocks, takes the first input header by default, and appends read-group
lines whose IDs are absent from that base. Before output begins it reads every
header, requires every input to have a canonical BGZF EOF marker, rejects mixed
formats and same-target aliases, and requires identical reference names,
order, and lengths across inputs and the optional header source. Conflicting
read-group IDs retain the base definition. Named output is transactional and
is committed only after the complete result passes product quickcheck.

`reheader [-o FILE] [--no-pg] HEADER BAM` accepts a replacement header from
SAM, BAM, or CRAM, a named BAM input, a named BAM output or standard output,
and program-record suppression.
It validates both headers, requires the replacement reference count to match
the input while allowing names and lengths to be corrected without remapping
record reference IDs, preserves compressed record blocks and record order,
requires the input EOF marker, rejects same-target aliases, and commits named
output transactionally after quickcheck. External shell transformation
(`-c`) and in-place editing (`-i`) are not exposed in this slice: the former
needs a separate process, quoting, and failure contract, and the latter is a
CRAM-only destructive operation.

The shared engine parses variable-length BGZF extra fields, locates and checks
the `BC` subfield and declared frame size, inflates and validates only frames
needed to consume the BAM header, reframes the boundary tail, structurally
checks and copies all complete record frames, removes each source EOF marker,
and writes exactly one output EOF marker. Malformed headers, inconsistent
dictionaries, invalid frame structure, premature or missing EOF, bytes after
EOF, input or output I/O errors, and output finalization errors remain fatal.
The public product library exposes narrow typed options, summaries, and write
interfaces for the two operations; BGZF framing remains private product
plumbing because no second Layer B consumer exists.

The samtools 1.24 oracle covers ordinary concatenation, read-group merging,
file lists, external headers, reheadering with renamed references, program
records, and complete decoded header and record order. Product tests add
malformed and truncated frames, dictionary and reference-count disagreement,
same-target aliases, standard output, transactional failure, write failure,
large headers spanning several frames, unusual valid BGZF extra layouts, and
exactly one output EOF marker. Representative benchmarks use non-trivial BAM
shards and the same full-record input for reheader, alternate command order at
least 12 times, record wall time, CPU, peak RSS, machine and fixture
provenance, and require semantic identity after every run.

Historical `rsomics-bam-cat` revision `e0a21da2cf6` and
`rsomics-bam-reheader` revision `bdf6f6ec0ed0` contribute boundary-frame
fixtures and raw-copy algorithm seeds. Their duplicated BGZF modules,
standalone CLIs, non-transactional writes, BAM-header slice assumptions,
permissive EOF handling, benchmark exemptions, skip-on-missing-oracle tests,
and comment-heavy source are discarded. Samtools and HTSlib remain
MIT/Expat-licensed compatibility sources and receive command-level
attribution.

The stable slice is implemented at product revision `764bbe91bf1f`. The shared
private rewrite engine validates complete BGZF frame structure in bounded
batches, handles frames split across reader-buffer boundaries, preserves raw
record blocks, and writes one canonical EOF marker. Both commands complete
all input and alias checks before writing, and named outputs use same-directory
temporary files followed by product quickcheck and atomic persistence. The
public library exposes only typed `cat` and `reheader` contracts; no new Layer A
item was needed because the framing policy has no second product consumer.

The local gate passes 40 library tests, nine file-operation lifecycle tests,
four direct samtools 1.24 compatibility tests, the complete product integration
suite in debug and release profiles, strict Clippy, and rustdoc. The oracle
matrix covers ordinary and list-based concatenation, read-group merging,
external SAM/BAM/CRAM header sources, reference renaming, program records,
decoded header and record order, and failure atomicity. Exact-head CI
`31367441880` passes on native Linux and macOS for both x86_64 and aarch64;
Linux x86_64 also builds samtools 1.24 and runs the explicit compatibility
suite.

The representative `cat` gate concatenates four 1,000,000-record BAM shards
totalling 90,038,862 bytes. Across 12 alternating pairs, the selected bounded
2 MiB path averaged 1.0667 seconds versus samtools at 0.9225 seconds and used
5,503,659 versus 7,019,179 bytes mean peak RSS. It therefore makes a 21.59%
memory-use claim, not a throughput claim. A separately measured 4 MiB buffer
was rejected after it used more memory and remained slower. The `reheader`
gate uses a 4,000,000-record, 92,673,552-byte BAM and averaged 0.6258 seconds
versus 0.7225 seconds for samtools while using 5,496,832 versus 6,980,949 bytes
mean peak RSS: 13.38% lower wall time and 21.26% lower memory on this fixture.
Every timed result passed complete decoded-stream identity checks; detailed
machine, input, command, timing, and checksum provenance is retained in the
product performance record.

Release revision `dfbc321d9dbe` passed exact-head four-native-target CI
`31368507277`. Publication workflow `31368957921` released 0.12.0 from that
head. The unyanked 144,963-byte registry archive has SHA-256
`df10fadae75e377e4a3c40244ad5bfd19d47010a874cc8063b81641fc8d1182b`
and embeds the exact release revision. A fresh registry install reports
0.12.0 and exposes `cat` and `reheader` through the shared help tree. Its
two-shard cat smoke and four-million-record reheader smoke match samtools 1.24
complete decoded streams at
`0c9f5514885e469f4720858c25bef106a529091844c33c37ccc449ba45feb675`
and `8bc5ca00000bfa575068de363b70bf3224cbebb3919519b58e2e01f410a19a15`;
docs.rs serves the corresponding public library documentation.

The sequence-recovery increment consolidates samtools 1.24
[`fasta` and `fastq`](https://www.htslib.org/doc/1.24/samtools-fasta.html).
They are two views of one alignment-to-sequence engine, not separate products
or duplicated modules. The first stable slice accepts named SAM, BAM, or CRAM
input and standard input, supports the existing product reference and
compression-worker controls, applies samtools record filters in the documented
`-d/-D`, `-f`, `-F`, `--rf`, `-G` precedence, and writes one complete FASTA or
FASTQ stream to standard output or a transactional `-o/--output` path. The
product-wide `-o` contract means the entire default stream; it deliberately
does not inherit samtools fastq's command-local use of `-o` as an alias for
only read categories 1 and 2.

Filtered records are grouped by adjacent QNAME and classified as read 1, read
2, or other from the READ1 and READ2 bits. At most one record per category is
written for each group. The first record with qualities wins, or the first
record when every candidate lacks qualities. Output within a group is read 1,
read 2, then other, matching the upstream default stream. Reverse-strand
records restore original read orientation by reverse-complementing the full
stored sequence and reversing qualities. Soft clips remain because their
bases are stored; hard-clipped bases cannot be recovered. Secondary and
supplementary records are excluded by default. Default names receive `/1` or
`/2`; `-n` suppresses suffixes. FASTQ uses Phred 1 when qualities are absent,
with `-v` selecting another valid default, and `-O` prefers a valid `OQ` tag.

Named `.gz`, `.bgz`, and `.bgzf` outputs use BGZF as samtools 1.24 does. That
capability belongs in `rsomics-seqio`, not in BAM-specific code: an explicit
plain-or-BGZF encoder will wrap a caller-owned stream without choosing paths,
opening files, or imposing transaction policy. Its named product consumers
are `rsomics-bam fasta/fastq` and `rsomics-seq convert/grep`. Both products
must add compressed-output round trips, finalization failures, and
transactional replacement tests before the public API is released. Existing
strict `rsomics-seqio::Writer` record validation remains the FASTA/FASTQ
contract; alignment grouping, filters, orientation, qualities, and path suffix
policy remain inside the BAM product.

The initial slice does not expose split read-1/read-2/other/singleton outputs,
tag copying, UMI and CASAVA name decoration, barcode-derived index reads,
soft-clip removal, arbitrary output-tag selection, or alternate compression
formats. Each affects selection or coordinates several output streams and
will be added only with a complete failure and transactional policy. `-N` is
also unnecessary while there are no split destinations because mate suffixes
are already the single-stream default. These omissions remain absent from
help rather than accepted as ineffective flags.

The samtools 1.24 oracle covers adjacent QNAME grouping, category order,
quality-preferred duplicates, missing qualities, reverse orientation, OQ,
mate suffixes, default and explicit filters, and SAM/BAM/CRAM/stdin input.
Product tests add malformed OQ and quality lengths, ambiguous READ1/READ2
flags, empty sequences, declared coordinate order, truncated input,
input/output aliases, BGZF suffixes, standard-output and named-output failures,
and the shared JSON envelope. The representative performance gate uses the
existing 4,000,000-record query-name-sorted `fixmate` fixture, alternates at
least ten default pairs against samtools 1.24, verifies the complete FASTA or
FASTQ stream for every run, and records wall time, CPU, peak RSS, machine,
command, binary, input, and output fingerprints.

Revision `d6cbf1070706` implements that shared engine and revision
`84c9c5e3c854` records its reproducible performance gate. Five ordinary CLI
tests cover empty sequences, quality and filter policy, BGZF round trips,
transactional failure, and JSON output. A separately invoked live suite is
byte-identical to samtools 1.24 for SAM, BAM, CRAM, and standard-input paths
under the default, `-n`, `-O`, `-v`, and `-F 0` policies. The historical
384-byte fixture still matches its retained FASTQ golden byte for byte. The
complete product suite, strict Clippy, rustdoc, benchmark entry, and packaged
source all pass locally with registry releases `rsomics-common 0.12.2` and
`rsomics-seqio 0.5.1`.

The reproducible macOS arm64 gate uses a 92,673,124-byte, 4,000,000-record
query-name-sorted BAM with SHA-256
`b949852de15f08a5e13d8c6d908b6d5801ef9f254eca58699aa353883cf88326`.
Across ten alternating pairs, `rsomics-bam fasta` takes 1.294 +/- 0.032 s and
5,559,091 bytes peak RSS, versus 1.705 +/- 0.091 s and 6,709,248 bytes for
samtools 1.24. `rsomics-bam fastq` takes 1.773 +/- 0.071 s and 5,559,091 bytes,
versus 2.608 +/- 0.071 s and 6,696,141 bytes. The Rust implementation wins all
ten pairs for both views: 24.11% lower FASTA wall time, 32.02% lower FASTQ wall
time, and about 17% lower peak RSS. Complete output hashes match at
`7e13841514dd9137e08b0d9994afa5b4baafd0583bf7228740949bfcd6de80e3`
for FASTA and
`a961649b1a0ee9beec27439fae61441e1d48694aa1486c7c74c1c64238ce4988`
for FASTQ. The tracked benchmark script, environment, raw timings, summary,
input, and exact binary fingerprints make the result auditable.

This increment supplied both concrete consumers required for
`rsomics-seqio::OutputEncoder`: BAM `fasta`/`fastq` and sequence
`convert`/`grep`. The public encoder was therefore released only after each
product exercised compressed output and finalization behavior. The empty SAM
SEQ oracle also exposed a shared FASTA edge case; `rsomics-seqio 0.5.1` now
represents it as one canonical empty sequence line while continuing to reject
blank lines inside non-empty records. Transactional permission testing in the
sequence consumer similarly found and fixed exact existing-mode preservation
in `rsomics-common 0.12.2`. Neither foundation API was added speculatively.

The unyanked 151,519-byte registry archive has SHA-256
`97cc23593d5b92a7f3c49c19ab9b9c014e466ecc37e12e392b07ccaac27cf056`
and embeds exact release revision `4829bbb3be06`. A fresh registry install
reports 0.13.0, exposes both commands through the shared help tree, and writes
FASTA plus BGZF FASTQ streams that match samtools 1.24 at
`a69efdbf4ebf740457c7df6e52112d1a56b63c388ad493c2a0f9ffbc0f8e61f8`
and `ca6ae968349466db34aa481149c0fc005689a3595cc3a3f8627139316754d733`.

Historical `rsomics-bam-fasta` revision `ba661eddd57b` and
`rsomics-bam-to-fastq` revision `9675f305021d` share one 384-byte BAM fixture
with SHA-256
`aeee5e08c912b3e82b611a330fd7f44f4ef88aa75a53db52f21acb0d169d5e1f`.
The fixture, reverse-complement mapping, and FASTQ golden file are retained as
test seeds. Their standalone CLIs, per-record extraction, direct file
truncation, skip-on-missing-oracle tests, tiny process-launch benchmarks,
duplicate complement functions, per-record allocations, comment-heavy source,
and failure to implement QNAME category selection and missing-quality behavior
are discarded. Samtools and HTSlib remain MIT/Expat-licensed compatibility
sources and receive command-level attribution.

### Release 0.14: FASTQ import

`import` converts FASTQ streams into
unmapped SAM or BAM records inside the existing product; it is not a new
sequence or BAM micro-crate. The compatibility contract is
[`samtools import` 1.24](https://www.htslib.org/doc/1.24/samtools-import.html)
and the SAM/BAM specifications.

The stable surface is:

- one positional FASTQ, two positional mate FASTQs, `-0`, `-s`, and the
  `-1`/`-2` pair;
- transparent plain, gzip, and BGZF FASTQ input through `rsomics-seqio`;
- automatic `/1` and `/2` read-name interpretation for a single input,
  including when it was supplied with `-0` or `-s`;
- unmapped single and paired FLAGs, original read orientation, exact sequence
  and quality transfer, and rejection of unequal paired-file record counts;
- `-r`/`--rg-line`, `-R`/`--rg`, `--order`, `--no-PG`, `-o`, `-O`, `-u`, and
  `-@` with transactional named output.

SAM remains the standard-output default. Named output is inferred from `.sam`
and `.bam`, with `-O sam|bam` as the explicit override. CRAM output is excluded
until this product has a conforming CRAM writer. Index FASTQs, CASAVA parsing,
UMI extraction, SRA second-field names, arbitrary FASTQ-comment auxiliary tags,
custom barcode tags, and HTSlib format-option strings are also excluded from
this increment. They remain extensions of `import`, not separate products.

Historical `rsomics-bam-import` revision `ba7f8fc76306` is refactor-then-merge
input. Retain its three FASTQ fixtures, direct BAM payload encoder, nucleotide
packing tests, read-group seeds, and paired-count failure cases. Discard its
standalone CLI and help schema, forced single/interleaved mode model, output
path implying BAM regardless of extension, first-mate-name reuse for both
records, unconditional ASCII-quality subtraction, direct destination
truncation, skip-on-missing-oracle tests, process-launch microbenchmark, and
comment-heavy source.

The target modules are `src/import.rs` and `src/commands/import.rs`. BAM payload
validation and output use the existing `rsomics-bamio` and product-private
writer contracts. FASTQ parsing uses the already-public `rsomics-seqio` reader.
No new Layer A item is justified by this slice.

Compatibility tests must require samtools 1.24 and compare decoded headers,
records, FLAGs, names, sequence, quality, read-group tags, order tags, SAM/BAM
selection, compressed input, standard I/O, malformed input, and differing
paired-file lengths. The representative gate uses a non-trivial single-end and
paired FASTQ corpus, records complete output hashes, wall-time distributions,
peak RSS, tool fingerprints, flags, and machine provenance, and requires a
strict throughput or resource-use win over samtools for the BAM hot path.
Samtools `bam_import.c` and its manual are MIT-licensed behavior references and
receive command-level attribution.

Feature revision `1df18368dd7c` implements this contract in the planned two
modules. Thirteen product tests cover the input modes, compression and standard
input, flags, tags, output inference, transactions, aliases, lowercase IUPAC
normalization, invalid bases, and configuration conflicts. Three live samtools
1.24 groups cover the declared mode and tag matrix, BAM, gzip, standard input,
and invalid-base decisions. The complete product suite passes in debug and
release profiles together with strict Clippy, rustdoc, and clean package
verification. No new Layer A item was added.

The final 12-pair macOS arm64 performance gate used 500,000 reads in each mate
file and four additional compression workers. Single-input mean wall time was
0.3150 seconds for rsomics and 0.5200 seconds for samtools; paired mean wall
time was 0.6133 versus 0.8825 seconds. Rsomics won all 12 pairs in both modes
and reduced mean peak RSS by 42.18% and 40.05%. Stable headers and complete
record streams matched. The measured rsomics binary had SHA-256
`b3e81cc1945cba86999d37839e44c57c16f100f04dfeb1caece7d03ddb1bfe25`.

Release revision `d54924462ad6` passed exact-head four-native-target CI
`31383580026`; its Linux x86_64 job rebuilt samtools 1.24 and ran the complete
oracle. Publication workflow `31384051315` produced the unyanked 162,589-byte
registry archive with SHA-256
`d092eb6d53b301d1e9be0d9e17671502f66d216d1e5b3eb63e4f311da442dcef`
and exact VCS metadata. A fresh registry install reports 0.14.0 and passes
single-end SAM, gzip-standard-input, paired-BAM, and invalid-base smokes.

### Release 0.15: read-group editing

`addreplacerg` edits the read-group dictionary and record `RG` tags as one
alignment-header workflow. The compatibility oracle is
[`samtools addreplacerg` 1.24](https://www.htslib.org/doc/1.24/samtools-addreplacerg.html)
and its MIT-licensed `bam_addrprg.c` implementation.

The stable surface is:

- SAM, BAM, and reference-backed CRAM input, including standard input;
- SAM or BAM output, with SAM on standard output, extension inference for a
  named target, and `-O sam|bam` as an explicit override;
- repeatable `-r` fields for a new `@RG` record, `-R` for an existing ID, or
  the first existing header read group when neither is supplied;
- `-m overwrite_all|orphan_only`, `-w`, `-u`, `-@`, `--reference`,
  `--no-PG`, and transactional named output.

`-r` and `-R` are mutually exclusive. A new read group must contain exactly
one non-empty `ID` field. Replacing the same header ID requires `-w`. In
`overwrite_all` mode, a new read group becomes the only `@RG` header record
and every alignment receives its ID. In `orphan_only` mode, existing header
read groups and record tags remain while only records without an `RG` tag are
stamped. Selecting an existing ID, explicitly or by default, preserves the
header dictionary in either mode. Any existing record field named `RG` counts
as present for `orphan_only`; overwrite mode replaces it with a string tag.

CRAM output, HTSlib format-option strings, automatic output indexing, and
arbitrary verbosity settings are excluded from this release. They are future
extensions of the same command, not separate products. Named output cannot
alias its input, and a failed read, write, or validation cannot replace an
existing destination.

Historical `rsomics-bam-addreplacerg` revision `26354a3724f7f` is
refactor-then-merge input. Retain its small BAM fixture, raw auxiliary-field
editing seed, overwrite/orphan cases, and benchmark corpus generator. Discard
its standalone CLI and help schema, BAM-only boundary, mandatory `-r`/`-R`,
incorrect same-ID replacement and header-retention behavior, direct output
truncation, skip-on-missing-oracle tests, process-launch microbenchmark, and
comment-heavy text-header manipulation.

The target modules are `src/addreplacerg.rs` and
`src/commands/addreplacerg.rs`. The command uses the existing typed header,
input, raw-record, output, program-provenance, and transaction contracts.
There is no second product consumer or missing shared primitive, so this slice
does not add a Layer A API.

Compatibility tests must require samtools 1.24 and cover all source and mode
combinations, repeated fields, escaped tabs and backslashes, same-ID conflict
and replacement, implicit first-ID selection, malformed header fields, SAM,
BAM, CRAM, standard input, both output formats, uncompressed BAM, record-tag
type replacement, program provenance, and output failure. The performance gate
uses a representative BAM with mixed present and absent tags, records complete
decoded output hashes, wall-time distributions, peak RSS, tool fingerprints,
flags, and machine provenance, and requires a strict throughput or resource-use
advantage on the BAM-to-BAM hot path.

Feature revision `033a7fa6c274` implements this contract with typed header
editing and a validated raw BAM-to-BAM auxiliary-field path. Eight product
tests cover modes, source selection, SAM/BAM and standard input, output
transactions, JSON, aliases, conflicts, and record failures. Two live samtools
1.24 groups cover the source and mode matrix plus BAM and CRAM input. The full
debug and release suites, strict Clippy, rustdoc, clean packaging, and every
live product oracle pass. No Layer A API was added.

The representative macOS arm64 gate used 4,000,260 records split equally
between present and absent `RG` tags, four additional workers, and 12
alternating pairs. Overwrite mode averaged 1.8125 seconds for rsomics and
2.4600 seconds for samtools; orphan-only mode averaged 1.7692 versus 2.4825
seconds. Rsomics won all pairs, reduced mean wall time by 26.32% and 28.73%,
and reduced mean peak RSS by 44.46% and 45.21%. Complete record streams and
normalized headers matched.

Release revision `fe2beb388a75` passed exact-head four-native-target CI
`31387911685`, including the complete samtools 1.24 oracle on Linux x86_64.
Publication workflow `31388331846` produced the unyanked 171,070-byte registry
archive with SHA-256
`79dec6d6cf7deff0a27443539974bec188fba213c7d0e9485059a94ddef61527`
and exact VCS metadata. A fresh registry install reports 0.15.0 and matches
samtools for overwrite, orphan-only, and implicit-first-read-group smokes. It
also rejects a conflicting header ID and emits the expected shared JSON
summary.

### Release 0.16: coverage and index summaries

`bedcov`, `coverage`, and `idxstats` form one coverage-summary increment. They
share alignment filtering, reference dictionaries, indexed access, pileup
semantics, and structured reporting, but remain separate subcommands because
their output units are BED regions, reference summaries, and index counts.
They do not become separate crates.

`bedcov` accepts a BED file and one or more named SAM, BAM, or CRAM inputs with
usable indices. Its stable surface includes `-Q`, `-g`, `-G`, `-j`, `-d`,
`--max-depth`, `-c`, `-H`, `-X`, `-@`, `--reference`, and transactional named
output in addition to standard output. It preserves BED row order and original
columns, emits one coverage column per input, and optionally emits per-input
depth-threshold and overlapping-read counts. Header, comment, `track`,
`browser`, malformed-row, unknown-reference, deletion, and reference-skip
behavior follow
[`samtools bedcov` 1.24](https://www.htslib.org/doc/1.24/samtools-bedcov.html).

`coverage` accepts one or more SAM, BAM, or CRAM inputs, a file list, or
standard input for one unindexed full scan and emits the nine-column
per-reference table from
[`samtools coverage` 1.24](https://www.htslib.org/doc/1.24/samtools-coverage.html).
The stable table slice includes `-l`, `-q`, `-Q`, `--rf`, `--ff`, `-d`,
`--min-depth`, `-r`, `-b`, `-H`, `-o`, `-@`, and `--reference`. Region mode
requires usable indices; named output is transactional. Histogram rendering
(`-m`, `-D`, `-A`, and `-w`) is excluded from 0.16 rather than implemented as
an inconsistent one-off terminal UI. It remains a later mode of `coverage`,
not a new product or crate.

`idxstats` emits reference name, reference length, mapped segments, and
unmapped segments followed by the unplaced-unmapped row. It reads BAI, CSI, or
CRAI metadata when available and otherwise scans a coordinate-sorted SAM, BAM,
or CRAM stream, matching
[`samtools idxstats` 1.24](https://www.htslib.org/doc/1.24/samtools-idxstats.html).
`-X` selects an explicit index; `-o`, `-@`, and `--reference` follow product
conventions. The shared JSON mode returns typed reference rows and the
unplaced count rather than embedding ad hoc JSON in the text stream.

The product-private target modules are `src/bedcov.rs`,
`src/bedcov/{bed,sweep}.rs`, `src/coverage.rs`, `src/coverage_engine.rs`,
`src/idxstats.rs`, and matching files under `src/commands/`. Existing
`rsomics-pileup` columns, `rsomics-bamio` records and indexed readers, and BAM
input and transaction contracts are reused. All three command trees render
through `rsomics-help` and use the shared JSON envelope. Any scan merging or
interval routing needed only by this product stays private. This slice has no
second target-product consumer for a new foundation item, so it does not
expand a Layer A API.

Historical `rsomics-bam-bedcov` revision `93204eea9155` contributes its dense
and sparse fixtures, adaptive single-pass idea, raw CIGAR-span seed, and
representative 50,000-region performance corpus. Its BAM-only boundary,
partial option surface, ignored thread argument, direct output creation,
skip-on-missing oracle, magic crossover policy, process benchmark, and
comment-heavy standalone CLI are discarded. Historical
`rsomics-bam-coverage` revision `e115cd0bceb0` contributes only fixtures and
basic interval-union tests; its incomplete seven-column output, whole-file
event vectors, missing quality semantics, and standalone shell are discarded.
Historical `rsomics-bam-idxstats` revision `f96b6aed4452` contributes its
index-metadata fixture and expected rows; its BAI-only lookup, missing scan
fallback, skipped oracle, duplicate JSON mode, and standalone shell are
discarded.

Live samtools 1.24 tests may not skip. The matrix covers SAM, BAM, and CRAM;
BAI, CSI, CRAI, explicit indices, and unindexed `idxstats`; multi-input and
file-list behavior; every declared filter and depth option; missing qualities;
BED comments, headers, unsorted and overlapping regions, deletions, and skips;
reference-dictionary disagreement; malformed or truncated input; standard and
named output; JSON; aliases; and transaction preservation on failure.

The performance gate remeasures samtools 1.24 rather than inheriting the old
1.21 results. `bedcov` uses both a sparse indexed workload and the retained
3,000,000-record, 50,000-region dense workload, with complete output equality.
`coverage` measures the complete nine-field calculation on a representative
alignment rather than the historical incomplete implementation. Each hot path
records timing distributions, peak RSS, exact flags, machine and tool
fingerprints, input and output hashes, and must show a strict throughput or
resource advantage. `idxstats` records indexed and scan-fallback costs; its
material product value is the unified install, explicit index handling, and
typed shared JSON, not a process-launch microbenchmark.

Feature revision `e21b823bea14` implements the three commands with one private
alignment stream and one private coverage engine. No Layer A item was added.
The complete local debug and release suites, strict Clippy, packaging, and live
SAM/BAM/CRAM oracle passed. On the four-million-record gate, `coverage` was
1.10 times as fast as samtools and used 35.9% less mean peak RSS. Dense
`bedcov` was 10.40 times as fast, while its sparse indexed path was 1.46 times
as fast and used 25.0% less mean peak RSS. Batched indexed `idxstats` was 1.44
times as fast and used 24.9% less mean peak RSS. All compared outputs were
byte-identical.

Release revision `be3cafe21867` passed exact-head four-native-target CI
`31398246573`, including the complete samtools 1.24 oracle on Linux x86_64.
Publication workflow `31398905778` produced the unyanked 190,502-byte registry
archive with SHA-256
`47f7bf82915054ac2a1fc1b66dbed35c77b47940fdb3f6c680ce789478be3345`
and exact VCS metadata. A fresh registry install reports 0.16.0, exposes all
three commands through the shared help tree, and passes text and shared-JSON
smokes for the three new workflows.

### Release 0.17: alignment tag recalculation

The twenty-fourth command, `calmd`, recalculates
the standard `MD` and `NM` auxiliary tags from each mapped record, its CIGAR,
and an indexed reference FASTA. The compatibility contract is
[`samtools calmd` 1.24](https://www.htslib.org/doc/1.24/samtools-calmd.html),
the SAM tag specification, and the 1.24 `bam_md.c` implementation at revision
`dc71c7274044d1050ccb64901731373ec7e915b6`.

The stable surface accepts SAM, BAM, or reference-backed CRAM from a named path
or standard input and emits SAM or BAM to standard output or a transactional
named output. It includes `-e`, `-b`, `-u`, `-O`, `-o`, `-@`, and `--no-pg`.
SAM is the standard-output default, matching samtools; a named `.bam` output
selects BAM unless `-O` overrides it. JSON requires a named alignment output
and reports processed, recalculated, corrected-tag, and missing-sequence counts
through the shared envelope.

The product-private `md` module becomes the single decoded and raw-record
implementation used by both existing CRAM completion paths and `calmd`.
The target files are `src/calmd.rs`, `src/commands/calmd.rs`, and the existing
`src/md.rs`; the only public surface is the BAM product library and subcommand.
Query `=` bases follow samtools' nucleotide-code match rule. Invalid CIGAR,
sequence, reference bounds, auxiliary data, or reference dictionaries fail the
command instead of producing a partial MD string. A mapped record without a
stored query sequence is preserved and counted because secondary and
supplementary records legitimately use this representation. A missing required
reference sequence is an error. BAM-to-BAM may use validated raw records to
preserve auxiliary-field order and integer representation; all format policy,
warnings, transactions, and program records remain outside that hot path.

Historical `rsomics-bam-calmd` revision `6d3a4d0657c5` contributes its MD/NM
walk, raw auxiliary-tag fixtures, reference fixture, and performance corpus as
refactor-then-merge assets. Its standalone command, duplicate reference and
output plumbing, BAM-only behavior, direct JSON-on-stderr, fixed temporary
paths, optional live oracle, silent out-of-bounds truncation, and dense source
commentary are discarded. No code or API moves to Layer A: only this product
currently consumes the recalculation policy. Samtools and HTSlib are MIT
licensed and retain source and behavior attribution; the historical Rust asset
is team-owned and remains under this product's MIT OR Apache-2.0 license.

Live samtools 1.24 tests may not skip. The matrix covers SAM, BAM, CRAM, and
standard input; SAM, compressed BAM, and uncompressed BAM output; default and
`-e` sequence behavior; matches, mismatches, insertions, deletions, skips,
ambiguous bases, existing correct and incorrect tags, missing query sequence,
unsorted reference revisits, program records, thread counts, JSON, aliases,
malformed and truncated inputs, missing references, output aliasing, and
transaction preservation. Complete headers and record fields are compared;
BAM tests additionally preserve auxiliary tag ordering and numeric subtype
where the source value is already correct.

Feature revision `5e8e28129b3d` implements the command, reuses the validated
mutable raw-BAM path, and adds no Layer A item. Revision `934137e01b37`
separates literal CRAM completion semantics from `calmd`'s nucleotide-code
match rule for query `=` bases. The samtools 1.24 oracle covers that distinction
for SAM, BAM, and CRAM.

The representative gate used a coordinate-sorted one-million-record BAM over
a 5,000,000-base reference at approximately 30x, with four additional workers
for both tools. After one warm-up, 20 alternating pairs gave mean wall times of
0.589 seconds for `rsomics-bam calmd` and 0.922 seconds for samtools 1.24.
Rsomics won all 20 pairs and was 1.57 times as fast, used 2.6% less mean CPU
time, and used 5.2% more mean peak RSS. Both complete decoded outputs had
SHA-256 `d1e0cfd0c1f1c1c88482e7140efc505ef323b0027ef1fac89be4c0b49d978eb9`.
The claim is limited to default compressed BAM on this fixture.

Release revision `0debc103993f` passed exact-head four-native-target CI
`31407557237`, including the complete samtools 1.24 oracle on Linux x86_64.
Publication workflow `31408461408` produced the unyanked 198,528-byte registry
archive with SHA-256
`6fd2ef2ad1c0072b3912d606b4bf52a2ee7d841a74a8af96383f53843eb6efc2`
and exact VCS metadata. A fresh registry install reports 0.17.0, exposes
`calmd` through the shared help tree, and passes a named-BAM and shared-JSON
smoke over all one million records with the decoded-output hash above.

BAQ realignment and mapping-quality capping (`-r`, `-A`, `-E`, and `-C`) are
excluded from 0.17 rather than exposed incompletely. Their later foundation
boundary is decided only with concrete `rsomics-bam` and `rsomics-call`
consumer tests. CRAM output and undocumented legacy `calmd` switches are also
excluded from this increment and stay absent from public help.

### Release 0.18: padded-reference projection

The twenty-fifth command, `depad`, converts padded-reference alignments and
coordinates to their ordinary unpadded representation. The compatibility
contract is [`samtools depad` 1.24](https://www.htslib.org/doc/1.24/samtools-depad.html),
the SAM/BAM specifications, and samtools `padding.c` at revision
`dc71c7274044d1050ccb64901731373ec7e915b6`. The upstream source is MIT
licensed; the historical Rust implementation is team-owned and remains under
this product's MIT OR Apache-2.0 license.

The stable input surface accepts SAM, BAM, no-reference CRAM, and standard
input. It writes SAM or BAM to standard output or a transactional named file.
`-T` supplies an uncompressed padded FASTA whose `*` or `-` characters are gap
columns; the index is built in memory so the command does not create a sidecar
beside user data. The release includes `-s`, legacy no-op `-S`, `-u`, `-1`,
`-O`, `-T`, `-o`, `-@`, and `--no-pg`. CRAM output, automatic output indexing,
and general HTSlib format-option strings remain absent because they are not
part of the product's validated output boundary.

Without `-T`, each reference requires an embedded reference record before its
mapped reads, and output `@SQ LN` remains padded with an explicit warning.
With `-T`, every input reference must exist at exactly its padded header length;
the output header length is the count of non-gap bases. An embedded reference
and FASTA for the same sequence must agree column by column. Embedded reference
records are retained with an all-`M` CIGAR, matching samtools. Query columns are
projected to `M`, `I`, `D`, or `P`; redundant internal pads are removed, and
mapped POS, same-reference PNEXT, cross-reference PNEXT, and BAM BIN are
recomputed. Cross-reference mate projection requires `-T`. Unmapped records
pass through unchanged. Input `N` is treated as `D` with a warning, while
input `I` or `P`, invalid reference characters, missing references, dictionary
mismatches, and coordinate or CIGAR bounds fail non-zero.

Historical `rsomics-bam-depad` revision `de243fd7ccb7` contributes the
padded-reference SAM/BAM fixture, position-map cases, CIGAR projection seed,
and benchmark corpus as refactor-then-merge assets. Its standalone CLI,
duplicate BAM layout and output plumbing, whole-output SAM conversion,
fixed temporary path, optional oracle, direct stderr JSON, and narrative
comments are discarded. Its implementation also changes unmapped coordinates,
does not remap cross-reference mates, never performs its documented `-T`
header correction, and accepts invalid FASTA bytes as ambiguous bases; none of
those behaviors are retained.

The target is one public product module, `src/depad.rs`, plus
`src/commands/depad.rs`. Padded-reference lookup, projection state, warnings,
format policy, transactions, and program records stay product-internal.
Validated `rsomics-bamio` records may be used for the BAM hot path, but no
Layer A API is added: no second product consumes depadding policy.

Live samtools 1.24 tests may not skip. The matrix covers embedded and FASTA
references; SAM, BAM, CRAM, and standard input; SAM, compressed BAM, fast BAM,
and uncompressed BAM output; corrected and uncorrected headers; multiple
references; same- and cross-reference mates; unmapped records; clips, leading
pads, insertions, deletions, skips, and embedded-reference validation; program
records, thread counts, JSON, aliases, malformed and truncated inputs,
reference mismatches, output aliasing, and transaction preservation. Complete
headers and record fields are compared after excluding only the expected
program identity. The representative performance gate uses non-trivial padded
multi-reference BAM and requires identical decoded output plus a strict
throughput or resource-use advantage over samtools 1.24.

Feature revision `e1b8f89eed74` implements the command as one product module,
a focused CIGAR projector, and a validated raw-BAM record adapter. It keeps
reference and output policy inside the product and adds no Layer A API. Exact
feature-head CI `31413068218` passes native Linux and macOS on x86_64 and
aarch64; Linux x86_64 builds samtools 1.24 and passes the complete compatibility
matrix, including 70,000-operation long CIGAR output.

The representative gate used a 1,000,000-record, 4,034,944-byte BAM against a
5,000,000-column padded reference, with four additional workers for both tools.
After one warm-up, 20 alternating pairs gave mean wall times of 0.3680 seconds
for `rsomics-bam depad` and 0.6115 seconds for samtools 1.24. Rsomics won all
20 pairs and was 1.66 times as fast, while using 9.1% more mean CPU time and
62.0% more mean peak RSS. Both complete decoded outputs had SHA-256
`b56d7863308db97b0b081782d1bc39a8805c8c1086b00c6ff72dee68e46de904`.
The claim is limited to compressed BAM with a supplied padded FASTA on this
fixture.

Release revision `5304f278bfaa` passed exact-head four-native-target CI
`31414206433`, including package verification and the complete samtools 1.24
oracle on Linux x86_64. Publication workflow `31415017446` produced the
unyanked 210,403-byte registry archive with SHA-256
`e2a5f63c3cd11cdd8c8666883029879272467ccf1a8fa0efcd20a65675bee4f9`
and exact VCS metadata. A fresh locked registry install reports 0.18.0, exposes
`depad` through the shared help tree, and processes all one million records
through named BAM and shared JSON output without creating a FASTA sidecar. The
output passes samtools quickcheck and reproduces the decoded-output hash above.

### Release 0.19: amplicon clipping and reporting

The next increment treats `ampliconclip` and `ampliconstats` as one user
workflow rather than two historical packages. `ampliconclip` removes primer
sequence described by a BED file; `ampliconstats` consumes alignment files
that have already been clipped and the same primer BED to produce the text
sections accepted by `plot-ampliconstats`. The compatibility contracts are
[`samtools ampliconclip` 1.24](https://www.htslib.org/doc/1.24/samtools-ampliconclip.html),
[`samtools ampliconstats` 1.24](https://www.htslib.org/doc/1.24/samtools-ampliconstats.html),
the SAM/BAM specifications, and samtools `bam_ampliconclip.c` and
`amplicon_stats.c` at tag 1.24. Samtools and HTSlib are MIT licensed. The
historical Rust implementations are team-owned and remain under this product's
MIT OR Apache-2.0 license.

The shared product-private model parses standard BED rows once, preserves
reference and primer order, and exposes separate indexed views for clipping
and amplicon construction. Clipping may use the first three BED columns or,
with `--strand`, the strand in column six. Statistics requires one or more
forward primers followed by one or more reverse primers for each amplicon and
supports alternative primers. Empty intervals, negative or reversed
coordinates, malformed strand rows, invalid amplicon ordering, duplicate
output targets, and references absent from the alignment dictionary fail
non-zero. This primer policy is not a general interval contract and remains
inside `rsomics-bam`; `rsomics-intervals` and every other Layer A crate remain
unchanged.

`ampliconclip` accepts coordinate-ordered BAM and writes BAM to standard output
or a transactional named file. Its stable surface includes `-b`, `-o`, `-f`,
`-u`, `--soft-clip`, `--hard-clip`, `--both-ends`, `--strand`, `--clipped`,
`--fail`, `--filter-len`, `--fail-len`, `--unmap-len`, `--no-excluded`,
`--rejects-file`, `--primer-counts`, `--original`, `--keep-tag`, `--tolerance`,
`--no-pg`, and `-@`. It soft-clips the five-prime end by default. With
`--both-ends`, `--strand` still restricts each match to the corresponding BED
strand in the samtools 1.24 executable, despite the contrary sentence in its
manual page. Mapped-position changes downgrade coordinate sort order, clipped
reads lose stale `NM` and `MD` tags unless requested otherwise, and the `OA`
tag records the original alignment when enabled. Rejected reads, the stats
block, and per-primer bedGraph counts are complete outputs rather than side
effects that may be silently skipped.

`ampliconstats` accepts the same six-column primer BED and one or more BAM
inputs. Its stable surface includes `-f`, `-F`, `-a`, `-l`, `-d`, `-m`, `-o`,
`-s`, `-t`, `-b`, `-c`, `-D`, `-S`, and `-@`. Numeric, symbolic, hexadecimal,
and octal SAM flag expressions reuse the product's existing flag parser. Plain
output preserves the samtools 1.24 `SS`, `AMPLICON`, file-specific, and
combined section schema, ordering, sample naming, multi-reference columns,
alternative-primer rules, template-coordinate bins, and run-length depth
encoding. `--json` requires a named text output and reports the typed run
summary through the shared `rsomics-help` and `rsomics-common` envelope without
mixing JSON into the compatibility stream.

Historical `rsomics-bam-ampliconclip` revision `94784e5b4132` contributes its
raw-record CIGAR and sequence trimming seed, primer-match cases, golden SAM
records, and samtools differential matrix. Its standalone CLI, duplicate
header and BAM plumbing, incomplete option set, unchecked raw-record indexing,
generated dependency selection, tiny broken benchmark, fixed temporary paths,
and narrative comments are discarded. Its ordinary tests pass against local
samtools 1.24, but `cargo test --all-targets` fails because the benchmark feeds
SAM to its BAM-only reader; this is not retained as evidence.

Historical `rsomics-bam-ampliconstats` revision `d748a727eb87` contributes its
statistics and text-rendering seed, committed BAM/BED fixture, and
metadata-normalized default-output oracle. Its standalone CLI, duplicate BED
and BAM plumbing, direct output mutation, hard-coded samtools 1.23.1 version,
unchecked allocation products, unbounded unmatched-pair map, sparse option
coverage, and narrative comments are discarded. Its locked all-target test
passes against local samtools 1.24, but exercises only one default fixture and
does not validate the full stable surface.

The target files are `src/amplicon.rs`, `src/ampliconclip.rs`,
`src/ampliconstats.rs`, and their two command adapters. Public library modules
expose narrow typed options and results; parsing, indexing, transactions,
format policy, and rendering stay private. Existing `rsomics-common`,
`rsomics-help`, and `rsomics-bamio` contracts are reused without a speculative
foundation API. SAM or CRAM input, SAM or CRAM output, HTSlib format-option
strings, reference download, and plot generation are excluded from 0.19 and
remain absent from public help.

Live samtools 1.24 tests may not skip. The clipping matrix covers every stable
option alone and in meaningful combinations; five-prime and both-end
soft/hard clipping; forward, reverse, paired, unmapped, QC-fail, and unmatched
records; CIGAR operations and auxiliary-tag types; header sort order and
program records; stats, rejects, primer counts, JSON, threads, malformed and
truncated records, aliasing, and transaction preservation. Complete alignment
records and output ordering are compared after excluding only program identity.
The statistics matrix covers one and multiple files and references, alternative
primers, sample names, every filter and binning option, legacy single-reference
output, all documented output sections, missing mates, long templates,
malformed BED and BAM, JSON, aliasing, and transaction preservation. Text is
byte-compared after normalizing only tool version and command-line identity.

Each command has its own release-performance gate on a representative
coordinate-ordered BAM with many amplicons and at least one million read pairs.
The record count, input and output digests, BED digest, machine, tool revisions,
workers, warm-up, alternating order, timing distribution, CPU, and peak RSS
are recorded. Clipping requires identical decoded alignments, counters,
rejects, and primer counts; statistics requires identical normalized text.
Both commands must show a strict throughput or resource-use advantage over
samtools 1.24 before either is published in 0.19. The historical claim of
0.143 seconds versus 0.607 seconds for `ampliconstats` remains only a fixture
and implementation seed because it lacks repeated-trial, RSS, output-digest,
and samtools 1.24 evidence.

Feature revision `c3f5c57a08d3` implements both commands, the shared private
primer model, transactional auxiliary outputs, the unified help and JSON
contracts, committed golden fixtures, and live samtools 1.24 matrices. Its
exact-head CI run `31421209078` passes native Linux and macOS on x86_64 and
aarch64; Linux x86_64 also builds samtools 1.24, verifies it, packages the
crate, and runs the live compatibility tests. Performance revision
`99d9999f6450` replaces owned record copies on the read-only statistics path,
bounds mate state by expected coordinate, retains same-coordinate mates,
removes typical clipping-path allocations, and selects level-1 BGZF for the
clipping product's default output.

The release-performance fixture is a 6,530,043-byte coordinate-ordered BAM
containing 2,000,000 records, 1,000,000 pairs, and 100 amplicons. Its SHA-256 is
`43c7138c06090b5bf9ea67298de0c0a25301e4e6b8da1e8352608735310824ba`;
the 5,952-byte primer BED SHA-256 is
`60d3f323e7bc8097c1896fe16083c6f2272330ec5922025d41c0d8b6b39fbcc3`.
Measurements ran on an eight-core Apple M2 Mac mini with 8 GB RAM, macOS
26.6/Darwin 25.6.0, Rust 1.91.0, and samtools/HTSlib 1.24. Both tools used no
additional workers. Standard BAM output was redirected to `/dev/null` so the
gate measures record processing and BGZF compression rather than external-disk
placement. Each command received one warm-up followed by five rounds in fixed
rsomics/samtools alternating pairs under `/usr/bin/time -lp`.

For `ampliconclip`, rsomics wall times were 0.70, 0.71, 0.70, 0.80, and 0.71
seconds; samtools times were 0.70 seconds in all five trials. Median user CPU
was 0.67 versus 0.66 seconds. Median peak RSS was 5,472,256 versus 7,553,024
bytes, a 27.6% reduction, so the gate is a resource-use win at essentially
equal throughput. Named-output checks produced 6,660,190 and 6,616,556-byte
BAMs respectively; decoded records shared SHA-256
`44f25c69e6106f843f698760f921b644896e00f01f439ccad6f55ee95a86d6a5`.
The rsomics level-1 stream is 0.66% larger on this fixture.

For `ampliconstats`, rsomics wall times were 0.36, 0.37, 0.36, 0.37, and 0.36
seconds; samtools times were 0.46, 0.47, 0.46, 0.47, and 0.47 seconds. Median
wall time was 23.4% lower and median user CPU was 0.34 versus 0.44 seconds.
Median peak RSS was 20,627,456 versus 19,300,352 bytes. Normalized complete
text from both tools shared SHA-256
`960c4eaeb02a0e6a50c791519f6f2d8946e1e4e96972612e1fa674450436a336`;
normalization removes only tool-version and command-line identity lines.

Release revision `6d82ba05b1722ffaef14bc1abdd2061ec0ebb29c` passed exact-head CI run
`31424768417` on native Linux and macOS x86_64 and aarch64, including the
Linux samtools 1.24 oracle and package gate. Publish run `31425446427` then
released `rsomics-bam` 0.19.0 through `cargo publish --locked`. The live,
unyanked registry archive is 241,278 bytes with SHA-256
`73aa144cfa9a2332f123046d8f2a440424832b6f3e2181af48803617e476f115`;
its Cargo VCS record points to the same release revision. A fresh registry
install with Rust 1.91.0 on the external build volume reported version 0.19.0
and exposed both command surfaces. Its `ampliconclip` smoke processed all
2,000,000 fixture records and passed samtools `quickcheck`; the BAM, normalized
clip counters, and normalized `ampliconstats` report were byte-identical to
outputs from the exact-head release build.

### Slice 3: remaining projection, pileup, and statistics

- `consensus`, `phase`, `reference`, and
  `targetcut`;
- `stats` and `cram-size`;
- `to-bed`, `reset`, and `checksum`.

Pileup-dependent work proceeds with the `rsomics-pileup` contract described
below. `checksum` ships only if it meets the same performance or material
benefit gate as every other established-tool replacement. Its historical
implementation is slower than the recorded samtools comparison and receives
no exemption.

### Release 0.20: CRAM storage diagnostics

`cram-size` is a format-diagnostic operation inside `rsomics-bam`, not a new
crate or foundation. Its compatibility contracts are the CRAM 2.1, 3.0, and
3.1 specifications, the samtools 1.24 manual, `cram_size.c` at source revision
`dc71c7274044d1050ccb64901731373ec7e915b6`, and the upstream regression
fixture. The samtools source and fixture are MIT licensed.

The stable surface is the CRAM input, standard input, `-o`/`--output`,
`-v`/`--verbose`, and `-e`/`--encodings`. Despite the manual synopsis spelling
the input as `in.bam`, the live executable rejects BAM: this command accepts
CRAM only. Plain output preserves the content-ID ordering, raw and compressed
sizes, ratios, compact or verbose compression-method names, data-series and
tag associations, embedded-reference annotation, encoding maps per container,
and final container, slice, sequence, base, file-size, and format-overhead
totals. Repeated `-v` or `-e` has the same boolean effect as samtools.

The parser validates the file definition, supported version, container and
block lengths, ITF8 and LTF8 values, block types, slice structure, and CRAM 3
checksums before indexed access. Compression-method classification includes
gzip and bzip2 levels, rANS 4x8 order, rANS Nx16 order/PACK/RLE/stripe/CAT
variants, arithmetic variants, FQZComp, and name-tokenizer entropy choice.
Compression-header encodings supply the content-ID-to-data-series map and the
per-container `--encodings` rendering. Named output is transactional and must
differ from the input. A parse, checksum, output, or finalization failure exits
non-zero and leaves any existing destination unchanged rather than committing
samtools-style partial diagnostics.

The product-level `--json` envelope requires a named compatibility output and
returns the same typed block and file totals on standard output. No Layer A API
is added: CRAM physical-layout policy has no second product consumer and stays
private to `rsomics-bam`. Release tests compare all three text modes byte for
byte with samtools 1.24, cover stdin and transactional output, and exercise
valid CRAM 2.1, 3.0, and 3.1 plus wrong-format, truncated, oversized, malformed,
and checksum-corrupt inputs. The performance fixture must contain multiple
containers, slices, codecs, content IDs, and tag encodings; the release gate
records complete-output identity, input digest and shape, tool revisions,
workers, warm-up, alternating trials, wall and CPU distributions, and peak
RSS. Publication still requires a strict throughput or resource-use advantage.

Feature revision `74c7a7fa8f06` implements the private streaming parser,
typed report, all stable CRAM compression-method and encoding renderers,
transactional output, unified help and JSON surfaces, committed golden
fixtures, and the error-path matrix. Official and generated fixtures cover
CRAM 2.1, 3.0, and 3.1. Default and `--encodings` output for each version, plus
all three text modes on the official samtools regression fixture, are byte-for-
byte identical to samtools 1.24. A separate 14,354,392-byte release fixture
contains 100 containers, 100 slices, 1,000,000 sequences, 150,000,000 bases,
multiple codecs, content IDs, and tag encodings. Its SHA-256 is
`8b37d7ef3e2ac30236bb5b5c4bba27335b1ec2b71356e376db25e7864195d5c0`;
complete default and encoding reports from both tools share SHA-256
`e430528d73de3086be9032b811243138247d5ecca45f8f2f113b8e42a7570903`
and `78477e90b7b2a684d3da9579d570c2344deeddc16a53a7adcc29984edd9fb5f5`.

The exact performance gate at revision `1540cbfae358` ran on an eight-core
Apple M2 Mac mini with 8 GB RAM, macOS 26.6.1, Rust 1.91.0, and
samtools/HTSlib 1.24. After warm-up, twenty alternating paired rounds each
processed the release fixture ten times and retained complete output identity.
Rsomics mean and median wall time were both 0.0130 seconds versus 0.00905 and
0.00900 seconds for samtools, so no throughput advantage is claimed. Mean peak
RSS was 5,512,397 versus 7,611,187 bytes, a 27.58% reduction, satisfying the
strict resource-use gate. Implementation exact-head CI `31429722062` passed
native Linux and macOS on x86_64 and aarch64; Linux x86_64 also passed
formatting, strict Clippy, debug and release tests, package verification, and
the complete samtools 1.24 compatibility oracle.

Release revision `5ecdcc33ccbe48cb9db12efb70ff5e434dbdc66f` passed the same
four-native-target gates in exact-head CI `31431000225`. Publish run
`31431664650` released 0.20.0 through `cargo publish --locked`. The live,
unyanked registry archive is 314,444 bytes with SHA-256
`2501e8303efb214f02d5654655cecd2a8caae1480443de216f5927a47544bda2`;
its Cargo VCS record points to the release revision. A fresh registry install
with Rust 1.91.0 on the external build volume reported version 0.20.0. Its
default, verbose, and encoding reports matched every committed release golden
byte for byte across CRAM 2.1, 3.0, and 3.1. On the one-million-record fixture,
the installed binary reproduced the pre-release default and encoding report
digests exactly.

### Release 0.21: full alignment statistics

`stats` is one report engine inside `rsomics-bam`, not a statistics foundation
or a revival of the deleted micro-crate. Its compatibility contracts are the
[`samtools stats` 1.24 manual](https://www.htslib.org/doc/1.24/samtools-stats.html),
`stats.c`, `stats_isize.c`, `plot-bamstats`, and the upstream `test/stat`
regression corpus at source revision
`dc71c7274044d1050ccb64901731373ec7e915b6`. Samtools and its fixtures are MIT
licensed. The 68-file upstream stats corpus has aggregate SHA-256
`22545813e19377ce1abfc073e5bae969e56458ee616ed1dda4d707497b245386`.

The stable input surface is SAM, BAM, CRAM, or standard input; optional indexed
regions; a customized index; a reference FASTA; target regions; read-group or
sample selection; required and excluded flags; duplicate and read-length
filters; coverage bins and target thresholds; insert-size limits and bulk;
BWA-style quality trimming; overlap removal; sparse insert rows; tagged split
reports and prefixes; reference statistics and chunk size; and additional
decompression threads. The upstream `-s`/`--sam` compatibility flag remains an
accepted no-op because input format is detected. Release 0.21 does not expose
the global `--input-fmt-option` or `--verbosity` controls because the product
reader does not offer their full contract; they fail as unknown options rather
than being silently ignored. `--reference` is an alias of `--ref-seq`.

Compatibility text preserves the section order, labels, columns, rounding,
and conditional presence consumed by `plot-bamstats`: `CHK`, `SN`, `FFQ`,
`LFQ`, `MPC`, `GCF`, `GCL`, `GCC`, `GCT`, `FBC`, `FTC`, `LBC`, `LTC`, barcode
content and quality sections for `BC/QT`, `CR/CY`, `OX/BZ`, and `RX/QX`, `IS`,
`RL`, `FRL`, `LRL`, `MAPQ`, `ID`, `IC`, `COV`, `GCD`, and `RFS`. The 39
ordinary summary rows include filtered, primary, supplementary, pairing,
duplicate, length, mapped-CIGAR, mismatch, quality, insert-orientation, and
proper-pair statistics; target mode adds its two coverage rows. Read-cycle
orientation, clipping, indel-cycle position, NM handling, missing sequence,
barcode validation, coordinate-order detection, insert orientation, coverage
bin boundaries, reference percentiles, and checksum accumulation follow the
1.24 oracle rather than the reduced historical model.

`-o`/`--output` is the product-level named-output extension. Named main and
split outputs are staged and committed only after input parsing, reporting,
flush, and every auxiliary write succeed. Output paths must not alias the
alignment, index, reference, or target inputs; duplicate split destinations
and unsafe tag-derived names fail before replacement. Product-level `--json`
requires a named compatibility report and returns the same typed summary,
histograms, distributions, barcode sections, target statistics, and reference
statistics through the common envelope. The first three provenance lines
identify rsomics and its invocation; compatibility tests compare the complete
stable body byte for byte with samtools after normalizing only those identity
lines.

The historical `rsomics-bam-stats` revision
`25c3689b1267431fc0428bdfc873d81cf23c8d7c` is classified as refactor then
merge for its small primary-record counter seed, plus test and benchmark asset
for its two BAM fixtures. Its 0.1.2 archive SHA-256 is
`8b030f692f9866827bc94d96ef3c0d26e4cb1fa15b31a5dce2fccce843761a8b`.
It emits only 14 custom SN rows, does not retain samtools field names, derives
only a minority of the full report, and has no coverage, cycle, insert, indel,
barcode, reference, region, split, or full format contract. Its recorded
12.96-fold claim lacks a representative fixture, complete-output identity,
RSS, and reproducible repeated-trial evidence and is not a release result.

The implementation stays private under `src/stats/`: typed accumulators,
CIGAR and cycle accounting, barcode validation, coverage and overlap state,
region and reference handling, insert-size classification, checksums, and text
rendering are separate modules behind one narrow report API. Existing product
input, flags, transactional output, `rsomics-help`, and common JSON envelope
are reused. No Layer A API is added because no second product consumes this
samtools-specific report policy.

The release matrix imports the upstream default, reference, large-coordinate,
supplementary, secondary, split, target, indexed-region, overlap, barcode,
large-deletion, and reference-statistics cases. Product tests add standard
input, SAM/BAM/CRAM equivalence, customized-index failures, malformed headers,
records and tags, truncated streams, reference and target mismatches, output
aliasing, write failures, split rollback, JSON separation, and allocation
limits for adversarial lengths and indices. The performance gate uses a
coordinate-sorted alignment with at least one million records, mixed flags,
variable read lengths, CIGAR indels, tags, mapping qualities, and non-trivial
coverage. It records fixture and complete-output digests, versions, machine,
workers, warm-up, alternating trials, wall and CPU distributions, and peak
RSS. Publication requires a strict throughput or resource-use advantage over
samtools 1.24.

`plot-bamstats` remains the separate upstream visualization consumer and is
not embedded or reimplemented in this release. CRAM 4, remote-reference cache
management, and speculative shared statistics APIs are also excluded.

### Slice 4: interactive viewing

`tview` is a complete terminal interface, not a formatting helper. It stays
out of public help until navigation, reference display, color modes, terminal
failure behavior, and native-platform tests are complete.

## Target structure

The repository uses operation modules and narrow private plumbing rather than
copying historical binaries:

```text
src/
├── addreplacerg.rs
├── alignment_stream.rs
├── bedcov.rs
├── bedcov/
│   ├── bed.rs
│   └── sweep.rs
├── cli.rs
├── coverage.rs
├── coverage_engine.rs
├── coverage_hts.rs
├── cram_size.rs
├── cram_size/
│   ├── encoding.rs
│   ├── parser.rs
│   ├── render.rs
│   └── varint.rs
├── idxstats.rs
├── lib.rs
├── main.rs
├── cat.rs
├── reheader.rs
├── bgzf_rewrite.rs
├── header_source.rs
├── collate.rs
├── fixmate.rs
├── fastx.rs
├── import.rs
├── markdup.rs
├── markdup/key.rs
├── merge.rs
├── sort.rs
├── stats.rs
├── stats/
│   ├── barcode.rs
│   ├── checksum.rs
│   ├── coverage.rs
│   ├── insert_size.rs
│   ├── record.rs
│   ├── reference.rs
│   ├── regions.rs
│   └── render.rs
├── depth.rs
├── flagstat.rs
├── head.rs
├── index.rs
├── mpileup.rs
├── quickcheck.rs
├── samples.rs
├── view.rs
├── alignment_order.rs
├── filter.rs
├── header_merge.rs
├── hts_metadata.rs
├── hts_quickcheck.rs
├── input.rs
├── md.rs
├── output.rs
├── program.rs
└── commands/
    ├── addreplacerg.rs
    ├── bedcov.rs
    ├── cat.rs
    ├── collate.rs
    ├── coverage.rs
    ├── cram_size.rs
    ├── depth.rs
    ├── fastx.rs
    ├── import.rs
    ├── fixmate.rs
    ├── flags.rs
    ├── flagstat.rs
    ├── head.rs
    ├── index.rs
    ├── idxstats.rs
    ├── markdup.rs
    ├── merge.rs
    ├── mpileup.rs
    ├── quickcheck.rs
    ├── reheader.rs
    ├── samples.rs
    ├── sort.rs
    ├── stats.rs
    └── view.rs
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
| `rsomics-bam-addreplacerg` `26354a3724f7f2e32bdb4d686b3ac13b59eeb6b4` | Test, fixture, and raw-editing seed; replacement merged at `033a7fa6c274` | Discard the standalone shell, text-header policy, and process benchmark |
| `rsomics-bam-ampliconclip` `94784e5b4132d39adcd0b784bb7d6ad7c0e69258` | Refactor then merge | `ampliconclip`; replace local format plumbing |
| `rsomics-bam-ampliconstats` `d748a727eb870583059bc801f89c3d115f4dcbc5` | Refactor then merge | `ampliconstats`; retain oracle fixtures and performance seed |
| `rsomics-bam-bedcov` `93204eea9155d118154ed237c84961b34ad7e29d` | Refactor then merge | `bedcov`; share validated pileup and interval input |
| `rsomics-bam-calmd` `6d3a4d0657c5c4e534269767b98534cc0a5d383e` | Refactor then merge | `calmd`; preserve MD/NM fixtures |
| `rsomics-bam-cat` `e0a21da2cf6c8f0f7eb1af87878a5dd03c02e211` | Refactor then merge | `cat`; retain block-copy ideas after header checks |
| `rsomics-bam-checksum` `95fc3dc4dfd477fae92306208ee61058b60ec638` | Test and benchmark asset until gate passes | `checksum`; do not retain the performance exemption |
| `rsomics-bam-collate` `f6f9b8ed029d6e1a30f4ecbc8bfe0ca2d25ad9ef` | Test asset; replacement merged at `24095b8650c2` | Discard whole-file buffering and first-seen group order |
| `rsomics-bam-consensus` `f202e114caa95ef38cd80dc40df8ee6a3f8ceae7` | Test asset and algorithm seed | `consensus`; historical simple mode is not the current default contract |
| `rsomics-bam-coverage` `e115cd0bceb0735e584d75125e7a6940e896d4fe` | Refactor then merge | `coverage`; summary output only |
| `rsomics-bam-depad` `de243fd7ccb7e0c313742b4e529fe95bad3833d4` | Fixture, algorithm, and benchmark seed; replacement merged at `e1b8f89eed74` | Discard standalone plumbing and known semantic defects |
| `rsomics-bam-depth` `cdc0a4ff70119edc193cd6bdfadaba6b6e190b61` | Test and algorithm seed; replacement merged | `depth`; discard whole-file event maps and keep the accumulator product-internal |
| `rsomics-bam-divide` `71504b275797ec30df2399ef2fbe03d1c9b1e6b5` | Refactor then merge | `split --parts`; preserve disjoint-cover and seeded-partition fixtures |
| `rsomics-bam-fasta` `ba661eddd57b45f725751f02a288546442acd3e7` | Fixture, mapping, and golden seed; replacement merged at `d6cbf1070706` | Discard standalone CLI and per-record extraction |
| `rsomics-bam-fixmate` `645e4e3c31f3e689e854c2de63e726b877d770ea` | Test, fixture, and performance asset; replacement merged at `a8a684ba57c6` | Discard the standalone shell and retain the supplementary and multi-primary oracle cases |
| `rsomics-bam-flags` `921a428ba5e11f47fca875e1b9ae1335b3b5cb8f` | Refactor then merge after dirty-diff attribution | `flags` |
| `rsomics-bam-flagstat` `ce1cc819d59fe37a56c762ba005ba0d9c91d3ba3` | Refactor then merge | First-slice `flagstat` |
| `rsomics-bam-head` `76ffd4d379191a968f1095a1854d0ce4c8fe49db` | Refactor then merge | First-slice `head` |
| `rsomics-bam-idxstats` `f96b6aed4452243a982c9d7ca495e6fa23d8b497` | Refactor then merge | `idxstats`; require index-kind coverage |
| `rsomics-bam-import` `ba7f8fc7630676e1cdbe95a21c0ae35677f5b958` | Fixture and encoder seed; replacement merged at `1df18368dd7c` | Discard standalone CLI, mode model, output policy, skipped oracles, and comment-heavy source |
| `rsomics-bam-index` `167e86bd0f5ee0cf13bf18e9ded89cb1f99a46a5` | Test asset; replacement merged at `4639c3676283` | `index`; discard the BAI-only wrapper |
| `rsomics-bam-markdup` `e865796930fb72d8a185e3a0b18024d217ca6128` | Algorithm, fixture, and performance seed; replacement specified above | Discard the standalone shell and retain scoring, signatures, and duplicate fixtures |
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
| `rsomics-bam-to-fastq` `9675f305021dceb00ed03e9b847fa7d7a1a89d6c` | Fixture and golden seed; replacement merged at `d6cbf1070706` | Discard duplicate complement code, allocations, and direct truncation |
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

`rsomics-bam 0.9.0` is published from release head `2abd073ddfcd`. Exact-head
CI `30787583253` passes native Linux and macOS on `x86_64` and `aarch64`; the
Linux `x86_64` job includes strict Clippy, package verification, samtools 1.24,
all prior compatibility groups, and the new collate differentials. Publish run
`30788060651` succeeds from the same revision. The independently downloaded
registry archive is byte-identical to the clean local package with SHA-256
`f69dce6dbe6748b7817378badbdd0eaa2f54f7f293eee0052234aa9bafc9822d`.
Its embedded VCS revision is the exact release head, the release is not yanked,
and registry metadata declares Rust 1.91. A fresh locked registry install
reports version 0.9.0, exposes the unified collate help, and groups all nine
records of the release smoke fixture into seven contiguous QNAME groups in a
BAM that passes samtools quickcheck.

`rsomics-bam 0.10.0` is published from release head `dde097afee25`.
Exact-head CI `31354120701` passes native Linux and macOS on `x86_64` and
`aarch64`; the Linux `x86_64` job includes strict Clippy, package verification,
samtools 1.24, all prior compatibility groups, and the complete fixmate
matrix. Publish run `31354567207` succeeds from the same revision. The
crates.io sparse index and independent static download agree on archive
SHA-256
`9359c28db43994eb963da645ae5f933ffd8f6a6b7467f24b23936913416ce39c`;
the release is not yanked and declares Rust 1.91. A locally generated archive
used a different gzip container, but all 86 extracted files are byte-identical
to the registry archive, including the normalized manifest and VCS record.
The registry archive embeds the exact release head. A fresh locked registry
install reports version 0.10.0, exposes the unified fixmate help, and writes a
21-record mate-repaired BAM that passes samtools quickcheck.

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
