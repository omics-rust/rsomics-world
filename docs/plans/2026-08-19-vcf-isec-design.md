# rsomics-vcf isec design

Status: product boundary, bcftools 1.24 behavior, historical assets, matching
semantics, output transaction, foundation requirement, compatibility
differences, and release gate audited. Implementation is intentionally not
started while `rsomics-vcf` main remains the exact unpublished 0.6.0 release
revision `682942cfa69768dc3a127a8544f2f07213b704ea`. The target release is
0.9.0 after the complete `concat` and `merge` slices.

## Purpose and boundary

`isec` performs set operations across two or more coordinate-sorted variant
files. It identifies compatible records at each coordinate, records which
inputs contain each matched record group, selects groups by input count or an
exact presence bitmap, and emits either a site table or unchanged source
records.

This is not cohort merge. Samples, alleles, INFO, FORMAT, and headers from
different inputs are never combined into a new record. When a source record is
selected, it is written with that source's header and values. `merge` owns
cross-sample field reconciliation; `concat` owns same-sample chunk assembly;
`view` owns single-input filtering. These policies remain separate even where
their readers share product-private mechanisms.

`isec` is one subcommand of `rsomics-vcf`, not a public crate. It reuses the
product's typed VCF/BCF reader, index and region layer, expression engine,
output encodings, JSON contract, and unified `rsomics-help` command tree. The
complete stable slice ships together; a two-file text-key intersection is not
published as an intermediate command.

## Authorities and evidence

The behavior oracle is bcftools 1.24 `isec` and HTSlib 1.24's synchronized
record matcher. Format authority remains the VCF 4.1 through 4.5, BCF2, TBI,
and CSI specifications. The official manual, tagged source, installed binary,
and live probes were inspected instead of reconstructing behavior from the
historical crate.

Pinned evidence:

- bcftools tag commit:
  `fb9f0f783e0f67d734f6fa7fe4df9d230522f196`;
- installed bcftools 1.24 executable SHA-256:
  `33100a6b961c529e915394d53b4737a0f8dd7a164eac352afe4e74e1ced51f60`;
- tagged `vcfisec.c` SHA-256:
  `c9f5028ec64390896477705d384d80582d9fb05c499661de6a9dd8ee12cce015`;
- HTSlib `bcf_sr_sort.c` SHA-256:
  `9831d8a7a35ecd803c3a60e65294e795782a6830022cf10d7698b389409b3d30`;
- HTSlib `synced_bcf_reader.h` SHA-256:
  `9b5753f786d88b59a4ffb41e59a3942df235b1bf8bd5a59be6b0c1b6a7f39f84`.

Bcftools and the inspected HTSlib sources carry the MIT license. Their names,
option provenance, and compatibility role are attributed in the command
documentation and retained test metadata. No upstream source is copied merely
to reproduce its internal structure.

## Historical asset disposition

Historical repository `rsomics-vcf-isec` is retained at revision
`86bedb28892ccbcb6137bfb3c82925fe931609f1`, version 0.1.2. It is team-owned
and licensed MIT OR Apache-2.0. Relevant hashes are:

| Asset | SHA-256 | Decision |
|---|---|---|
| `src/lib.rs` | `447194222847045315b15eef067569272a17111178df8e963c43964442a51db6` | discard implementation |
| `src/cli.rs` | `c262f487763d0bfe4173d1cc54698bb62ab20bf7245440f7f7497d0917bcbafe` | discard CLI |
| `tests/compat.rs` | `f8093e71341c7f94fc8f31ab95c0db3ebd4cef5c3aeb04b045528e9e2e60097e` | retain malformed-input and tiny oracle seeds |
| `tests/smoke.rs` | `48b13cd64a82475ce9a5f5c49f73da5ca9ef671eb85e1683765e775020f7ae4f` | retain one fixture expectation |
| `benches/bench.rs` | `7acddc03b48905ec80e4fab26ba4d0416b929b648f740de727d61f0325c91070` | discard launch benchmark |

The old library loads the complete second text VCF into a string `HashSet`
and scans the first. Its identity is the literal CHROM, POS, REF, and ALT
text, so it cannot reproduce ALT-set equality, relaxed matching, symbolic END,
duplicates, or deterministic multi-input grouping. It accepts neither BCF nor
BGZF/index semantics, and has no count selectors, bitmaps, filters,
expressions, regions, targets, source projections, output directory, or
transaction.

Its hand-built `-a/-b` CLI is unrelated to the product command tree. JSON mode
replaces the variant output writer with a sink and therefore reports success
after discarding the data. The compatibility test can skip when tools are
missing, compares only sorted CHROM/POS/REF/ALT strings, and exercises one
two-record fixture. The benchmark repeatedly launches the binary over that
fixture without an upstream comparison, RSS, semantic hash, or representative
input. None of these are release evidence.

The disposition is therefore:

1. direct merge: none;
2. refactor then merge: none;
3. fixture or adversarial-test seeds: the compact valid and malformed text
   inputs only;
4. discard: parser, hash-set algorithm, CLI, JSON behavior, and benchmark.

## Command contract

The public spelling is:

```text
rsomics-vcf isec [OPTIONS] INPUT...
```

At least two named inputs are required. Every input is BGZF-compressed VCF or
BCF with a valid TBI or CSI index appropriate to the encoding. Plain VCF and
raw BCF are accepted only after BGZF compression and indexing; standard input
is excluded because synchronized indexed access cannot give it the same
contract. Single-input filtering remains `rsomics-vcf view`.

`-l, --file-list FILE` reads one path per line. Blank lines and lines whose
first non-whitespace character is `#` are ignored. Relative paths resolve
against the process working directory, matching other product file-list
options. Positional inputs and `--file-list` are mutually exclusive, and a
duplicate path or alias fails before readers open. Bcftools 1.24 prepends
positional inputs to list inputs; rsomics rejects that order-sensitive mixture.

All input headers and indexes are opened and validated before any output is
created. Different sample sets, field schemas, contig declaration order, and
descriptive metadata are allowed because records remain in their source
files. Each referenced field must still be valid under its own header, every
record must be coordinate sorted, and every index must agree with its data.
Contigs synchronize by exact name. The first input determines cross-contig
output order, while contigs absent from that header are appended in first
appearance order from later headers.

The input list is stable public state. Presence bit zero and write index one
refer to the first input, and no internal scheduling may reorder it.

## Presence selectors

Exactly one of `-C, --complement` and `-n, --nfiles SELECTOR` may be given.

`--complement` selects a group present in the first input and absent from every
later input. It is equivalent to a bitmap beginning with `1` followed by one
`0` per remaining input, but retains the familiar upstream spelling.

`--nfiles` accepts one of:

- `N` or `=N`: exactly N contributing inputs;
- `+N`: at least N contributing inputs;
- `-N`: at most N contributing inputs;
- `~BITMAP`: exactly the inputs marked `1` in a zero/one bitmap.

The grammar is parsed completely. N is a nonnegative decimal integer and a
bitmap contains exactly one digit per input. Values above the number of inputs
are valid empty selections, matching the mathematical and bcftools behavior;
trailing text such as `=2x` fails instead of being silently accepted.

With three or more inputs and no selector, the default is `+1`, the union.
With exactly two inputs, no selector is the four-way Venn convenience and
requires `-p, --output-dir`. Without the directory it fails and requests an
explicit selector. This preserves the established two-input behavior without
making a partial multi-output stream.

Presence is counted after `--apply-filters`, include/exclude expressions,
regions, targets, and record pairing. Multiple records from one input never
increase the file count of one group.

## Record compatibility

`-c, --collapse MODE` chooses record compatibility at the same CHROM and POS.
The canonical modes are:

- `exact`: equal REF and the same complete ALT set;
- `some`: equal REF and at least one shared ALT;
- `snps`: `some` plus compatibility between different SNP allele sets;
- `indels`: `some` plus compatibility between different indel allele sets;
- `both`: `some` plus the SNP and indel relaxations, without pairing those
  classes to each other;
- `all`: any records at the coordinate;
- `id`: the same nonmissing ID column at the coordinate.

`none` is accepted as an upstream compatibility alias for `exact` but is not
the help-tree canonical spelling. This avoids the bcftools inconsistency where
`isec -c none` means exact allele sets while `merge -m none` means shared or
subset ALT compatibility.

ALT order does not affect `exact`; duplicate ALT values remain invalid VCF.
Allele comparison is ASCII case-insensitive for oracle compatibility, while
output preserves source spelling. Symbolic alleles include typed END in their
identity. Breakends, spanning deletion, reference-only, `<NON_REF>`, and mixed
multiallelic records are classified and compared through typed variant
semantics, not string length.

The `snps` and `indels` modes do not disable exact or shared-ALT matches for
other record classes. They add a type relaxation. Thus an exact indel still
pairs under `snps`, and SNP records sharing one ALT still pair under `indels`.
This non-obvious HTSlib behavior receives explicit tests and help text.

`id` deliberately follows the product-wide identity policy shared with
`merge`: the complete ID column must be equal and not `.`; alleles need not be
equal because ID is the selected identity. Bcftools 1.24's manual describes ID
identity, but its implementation prefixes ID to the allele key and therefore
still requires allele compatibility. Rsomics uses the meaningful documented
operation and records this as an intentional difference. Semicolon-separated
ID columns are compared as complete values; they are not parsed as unordered
sets.

## Deterministic coordinate grouping

One forward indexed cursor per input supplies every record at the next
coordinate. A coordinate group buffers only records at that coordinate. Input
order and within-input record order are stable.

The matcher emits a sequence of groups containing at most one record from each
input. It prefers, in order:

1. exact allele-set matches;
2. shared-ALT matches;
3. the selected type or ID relaxation;
4. unmatched records in source order.

Among equal choices it prefers the group containing more distinct inputs,
then the lexicographically earliest vector of input and within-input ordinals.
This is a deterministic maximum-coverage matching, not hash iteration and not
an all-to-all join. Once a record is selected for one group it cannot appear in
another. Two records from A and one compatible record from B consequently
produce one shared group and one A-only group.

This ordering matches the useful HTSlib 1.24 preference for exact and
shared-ALT pairs while defining ties that upstream leaves coupled to internal
group construction. Tests cover repeated exact duplicates, crossed best
matches, multiallelic subset matches, three or more inputs, and input-order
reversal.

The product-private synchronized reader may later serve `merge`, but the group
policy remains parameterized. Merge constructs a new allele space from a
group; isec never changes records. Concat's overlap duplicate handling is
first-wins chunk policy and does not automatically share this matcher.

## Filters and expressions

`-f, --apply-filters LIST` accepts comma-separated FILTER values. A record
passes when its FILTER column contains at least one listed value. `PASS` and
`.` are distinct explicit values. The same filter list applies to every
input before matching.

Repeated `-i, --include` and `-e, --exclude` options retain command-line order.
Either one expression is given and bound independently against every input
header, or exactly one expression position is supplied per input. A literal
`-` means no expression for that input. Mixing include and exclude across
input positions is valid. Any other count, or a single literal `-`, fails.

The existing typed `rsomics-vcf` expression engine supplies numeric, string,
flag, INFO, FORMAT, genotype, sample-vector, set-file, and missing-value
semantics. Each expression is prebound to its source header before output.
Undefined tags, invalid vector cardinality, evaluation errors, and non-finite
operations fail rather than silently removing a record.

Filtering changes presence, not source records. A record that fails its
source filter is absent from matching and from every source projection. No AC,
AN, INFO, FORMAT, FILTER, or genotype value is recomputed.

The installed binary and source use `-l` for file lists and `-f` for FILTER
selection. A current generated manual table incorrectly labels file list as
`-f`; rsomics follows the binary contract and reserves `-f` for filters.

## Regions and targets

The stable slice includes:

- `-r, --regions REGIONS` and `-R, --regions-file FILE`;
- `--regions-overlap pos|record|variant`, default `record`;
- `-t, --targets TARGETS` and `-T, --targets-file FILE`;
- `--targets-overlap pos|record|variant`, default `pos`;
- target complement through a leading `^`.

Regions use each input's index and may alter contig traversal order according
to a region file. Targets stream within that traversal. Regions and targets
compose: a record must satisfy both. Inline and file spellings within each
category are mutually exclusive. Coordinates, BED detection, overlapping
region deduplication, true-variant overlap, unknown contigs, and stale indexes
use the existing product region contract.

The same location predicate is evaluated for every input, but a spanning
record may exist in one input and not another. Presence bitmaps reflect the
records that survive each source's location query and compatibility grouping.

## Site-table output

Without source projection, output is a tab-delimited site table with no header:

```text
CHROM  POS  REF  ALT  BITMAP
```

The fields are separated by tabs. POS is one-based. BITMAP has one digit per
input in input order. CHROM, POS, REF, and ALT come from the earliest input
record in the selected group; ALT preserves that record's complete source
order. The bitmap, rather than those display alleles, is the authoritative set
result under relaxed compatibility.

`-o, --output FILE` writes the table to a named atomic file; otherwise it uses
standard output. Variant `--output-type`, compression threads, and indexing
options are invalid in table mode. The table stays plain text so a `.bcf`
suffix cannot silently create text with a misleading name.

The machine summary reports input count, coordinate groups visited, selected
groups, records filtered per input, matching mode, selector, and output kind.
Global `--json` requires named table output and returns that summary on
standard output. It never replaces or suppresses the table.

## Source-record output

`-w, --write LIST` selects one-based input indices whose original records are
written for selected groups. The list is given once, contains no duplicate,
and is parsed completely. An index outside the input range fails.

Exactly one selected source may be written as one VCF/BCF stream without an
output directory. Its original header and selected records are encoded as
`v`, `z`, `u`, or `b`; plain VCF is the default. A selected group contributes
the source record only when that source's bitmap bit is one. The command does
not synthesize a record for an absent source.

`-p, --output-dir DIR` writes source projections, `sites.txt`, `README.txt`,
and `manifest.json` as one directory transaction. For ordinary selectors, one
zero-padded output is created per requested source, or per source when
`--write` is absent. Empty selected-source files retain valid source headers.

The two-input Venn convenience uses the bcftools-compatible names:

- `0000`: records private to input one;
- `0001`: records private to input two;
- `0002`: records from input one shared by both;
- `0003`: records from input two shared by both.

`--write 1` in Venn mode selects both projections derived from input one,
`0000` and `0002`; `--write 2` selects `0001` and `0003`. `sites.txt` contains
every Venn group. `README.txt` explains each file without embedding an
unstable timestamp or command-line provenance in VCF headers. `manifest.json`
provides schema version, source path and ordinal, result role, encoding,
record count, size, and content SHA-256.

`--output-dir` and `--output` are mutually exclusive. JSON is valid with a
named single record stream or directory and reports only after commit. Numeric
compression levels, arbitrary verbosity, command-line header stamping, and
output-format inference from a suffix remain excluded under product-wide
policy.

Automatic output indexing is excluded from 0.9.0 until a named variant and
its TBI or CSI can commit through one multi-artifact transaction. Directory
mode does not reproduce bcftools's implicit prefix indexing. Users run
`rsomics-vcf index` after a successful operation. `--threads` is valid only
for compressed record output and controls bounded compression workers.

## Multi-output transaction and shared foundation

A named single table or variant stream uses
`rsomics-common::AtomicFile`. Directory mode stages a new sibling directory on
the same filesystem, writes and syncs every contained artifact, validates the
manifest against the staged files, syncs the staging directory, renames it to
the absent destination, and syncs the parent. Failure removes only the private
staging directory. An existing destination is rejected and never replaced or
partially reused.

This reveals one justified extension to the existing `rsomics-common`
foundation: a narrow `AtomicDirectory` transaction. It has two named product
consumers with concrete plans:

1. `rsomics-vcf isec` and later `split` commit multi-file variant result sets;
2. `rsomics-cnv call` commits its VCF, segments, model summary, signal report,
   plots, and manifest as one report bundle.

The public API owns only same-filesystem staging, target-exists policy, sync,
commit, rollback-on-drop, and the staging path. It does not know VCF, CNV,
manifests, indexes, or report policy. Consumer-side fault-injection tests must
demonstrate no visible partial destination after parse, write, sync, or
validation failure. API review must keep replacement and deletion outside the
default contract.

The extension is implemented only alongside its first two concrete consumers;
no speculative `rsomics-output` crate is created. If the CNV call site changes
before implementation, the directory transaction remains private to
`rsomics-vcf` until another product consumer exists.

No other Layer A addition is justified. Synchronized VCF coordinate grouping,
variant compatibility, source projection, bitmap selection, and Venn naming
remain internal to `rsomics-vcf`. `rsomics-help` supplies presentation rather
than command policy, and `rsomics-seqio` supplies format-neutral BGZF mechanics
only where already justified by BAM and VCF consumers.

## Product structure

The intended product-local structure is:

```text
src/
├── isec.rs
├── isec/
│   ├── input.rs
│   ├── selector.rs
│   ├── matching.rs
│   ├── filters.rs
│   ├── output.rs
│   └── manifest.rs
└── coordinate_group.rs
```

`coordinate_group.rs` is introduced only when both `merge` and `isec` have
real call sites. It owns ordered N-reader advancement and bounded coordinate
buffers. Each operation retains its own selection and output state. Narrow
typed interfaces prevent `isec` from depending on merge's allele-remapping or
sample-fill policy.

No module narrates migration phases in comments. Public docs describe user
contracts. Source comments are limited to stable non-obvious invariants such
as the tie order in maximum matching and the final directory commit boundary.

## Deliberate compatibility differences

Rsomics differs from bcftools 1.24 only where the product contract is safer or
more coherent:

- at least two inputs are required; one-input identity filtering belongs to
  `view`;
- positional and file-list inputs cannot be mixed;
- blank and comment file-list lines are supported;
- input aliases and duplicate paths fail before opening readers;
- selector and write-list syntax is parsed completely rather than accepting
  trailing junk;
- `exact` is canonical and `none` is its compatibility alias;
- ID mode matches an equal nonmissing ID without requiring equal alleles, and
  never treats `.` as identity;
- every input header, index, expression, and record order is validated before
  named output becomes visible;
- output modes reject ignored options such as BCF encoding for a site table;
- output-directory creation is transactional and rejects an existing target;
- file headers receive no command-line or version stamping;
- JSON never discards biological output;
- automatic indexing, numeric compression levels, and verbosity are absent
  from the first stable slice.

The generated bcftools 1.24 HTML manual currently assigns `-f` to both
`--apply-filters` and `--file-list`. Tagged source and the installed binary use
`-f` and `-l` respectively. Rsomics follows the executable spelling.

## Failure contract

The command exits nonzero for:

- fewer than two inputs, mixed input sources, aliases, duplicates, stdin, or
  unreadable paths;
- uncompressed or unindexed VCF, missing, stale, malformed, or incompatible
  indexes, truncated BGZF, or malformed BCF;
- invalid headers, undeclared record fields, unknown contigs, unsorted records,
  or noncontiguous contig blocks;
- invalid selector, bitmap, collapse mode, write index, expression count,
  expression binding, FILTER value, region, target, or output combination;
- malformed allele, symbolic END, breakend, INFO, FORMAT, genotype, or vector
  cardinality encountered by typed parsing;
- output/input aliasing, an existing output directory, create, write, flush,
  sync, manifest, close, rename, or parent-sync failure.

All argument and header errors are preflight failures. Standard-output mode
cannot roll back bytes after a late record error, so help documents that named
output is required for transactional persistence. Named files and directories
remain invisible until complete validation and commit.

## Test matrix

Unit tests cover:

- strict count, threshold, complement, bitmap, write-list, and file-list
  parsing;
- exact, some, SNP, indel, both, all, and ID compatibility;
- ALT-order equality, multiallelic subsets, case, symbolic END, breakends,
  spanning deletion, `<NON_REF>`, reference-only, and invalid alleles;
- deterministic maximum matching, exact preference, shared-ALT preference,
  crossed choices, duplicate multiplicity, ties, input reversal, and 2, 3, 8,
  and more than 64 inputs;
- per-input include/exclude binding, shared expressions, skipped positions,
  FILTER selection, missing tags, vectors, samples, and evaluation failure;
- site-row field choice, bitmap order, source projections, empty source files,
  Venn roles, manifest schema, and summary counts.

Integration tests cover all four input and record-output encodings, TBI and
CSI, mixed input encodings, contig-order reconciliation, source-specific
headers and samples, indexed regions, region files, target streams, target
complement, overlap modes, and combined regions and targets.

Failure tests cover plain VCF, stdin, missing and stale indexes, corrupted
BGZF and BCF, sort regression, malformed records, duplicate inputs, invalid
headers, bad expression counts, invalid selectors, incompatible output flags,
output aliases, existing directories, and injected failures at every
transaction boundary. JSON tests prove that table, variant, and directory
outputs exist and validate before the envelope reports success.

Compatibility fixtures compare exact decoded outputs or site-table bytes
against bcftools 1.24 for every retained behavior. Deliberate differences have
separate tests that assert the rsomics result and record the observed upstream
result. No compatibility test silently skips in release CI; the pinned oracle
must be built or the job fails.

## Performance gate

The hot path is measured against bcftools 1.24 on representative indexed data,
not process-launch fixtures. The benchmark matrix contains:

- two 5-million-record inputs with sparse exact overlap;
- eight inputs with dense shared coordinates and multiallelic records;
- duplicate-heavy coordinates exercising maximum matching;
- exact, some, both, and all modes;
- site-table output, one selected BCF stream, and transactional multi-output;
- indexed whole-file, region, and region-plus-target workloads.

Every case records machine, versions, exact revisions, input/index hashes,
flags, warmups, alternating repeated trials, wall time, CPU time, peak RSS,
bytes read and written, output hashes, group counts, and semantic equivalence.
Inputs must exceed cache-only toy scale, and order is alternated to reduce page
cache bias.

Publication requires a strict throughput or resource-use advantage on the
relevant synchronized-reader hot path. Equal performance is insufficient
without another measured material benefit. Directory transaction overhead is
reported separately from matching so safety cost cannot be hidden or mistaken
for algorithm performance.

## Release gate

Implementation begins only after 0.6.0 is published and the preceding complete
0.7 and 0.8 slices have established their shared private readers. The command
stays absent from public help and README until all stable operations in this
document pass.

Release requires formatting, strict Clippy, debug and release tests, complete
bcftools 1.24 compatibility, transaction fault injection, the two-consumer
`AtomicDirectory` API gate if promotion occurs, formal performance evidence,
package verification, and a fresh public-API and hot-path review. Exact-head CI
must pass native Linux and macOS on x86_64 and aarch64.

After publication, the registry archive is downloaded independently, matched
to the release head and package tree, installed with fresh external Cargo
state, and smoke-tested for site-table, one-source VCF/BCF, Venn directory,
filters, regions, and deliberate ID behavior.

## Audit evidence

The retained external audit directory is:

```text
/Volumes/KIOXIA/Developments/tmp/rsomics-vcf-isec-audit-20260819
```

It contains source fixtures, compressed indexes, the reproducible `probe.sh`,
all outputs and expected failures, `results.sha256`, and
`oracle-summary.md`. Fixture hashes and the result-manifest hash are recorded
there. Representative bcftools 1.24 observations are:

| Probe | Observed result | Contract use |
|---|---|---|
| two inputs, no selector, `-p` | four VCFs plus README and sites | retained Venn layout |
| three inputs, no selector | union site table | retained default |
| `-n=2`, `+2`, `-1`, `~101` | count or exact bitmap after filters | selector oracle |
| `-C -w1` | first-input private records | complement oracle |
| two A records, one B, `-c all` | one shared A and one private A | one-to-one grouping |
| A `ALT=C`, B `ALT=C,G`, `-c some` | shared | subset oracle |
| `-c id`, same ID, different ALT | not shared | documented deliberate difference |
| `-c id`, same ID and ALT | shared | live binary behavior |
| region `chr1:31` over `30 AT>A` | record and variant overlap include it | region oracle |
| target `chr1:31` with default overlap | record excluded | target POS oracle |
| different contig declaration order | synchronized by name | header-order oracle |
| common or per-input expressions | bitmap changes after filtering | filter oracle |
| comment in file list | comment treated as a path, exit 255 | rsomics comment support |
| positional plus file list | positional input prepended | rsomics rejects mixing |
| plain VCF among inputs | exit 255, index/compression error | indexed-input gate |
| `-n=4` over three files | exit 0, empty table | retained empty selection |
| `-n=2x` | exit 0 as two | rsomics strict parse |
| prefix compressed output | every VCF auto-indexed | first release excludes auto-index |
| one input with target | identity-filtered VCF | excluded in favor of `view` |
