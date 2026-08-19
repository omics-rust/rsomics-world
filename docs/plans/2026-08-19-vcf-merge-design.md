# `rsomics-vcf merge` design

Status: product boundary, bcftools 1.24 surface, historical assets, typed merge
semantics, deliberate differences, and release gate audited. Implementation is
not started while `rsomics-vcf` main is held at the exact unpublished 0.6.0
release revision `682942cfa69768dc3a127a8544f2f07213b704ea` and while the
preceding complete `concat` slice remains planned.

## Purpose and boundary

`merge` combines coordinate-sorted VCF or BCF inputs with disjoint sample sets
into one multi-sample stream. It groups compatible records across files,
constructs one output allele space, remaps typed INFO and FORMAT values,
reconciles site fields, and fills samples whose file has no selected record at
that site. It never combines two records from the same input file.

This is an `rsomics-vcf` subcommand, not another crate and not a general table
join. It shares the product's header model, VCF/BCF reader and writer, region
indexes, genotype representation, allele-cardinality code, atomic output,
machine summaries, and `rsomics-help` command tree. The complete stable
contract includes ordinary cohort merge, all record-pairing modes, typed merge
rules, local alleles, and gVCF blocks. A smaller command that merely joins GT
at equal positions is not published. The target sequence is 0.8.0 after the
complete 0.7.0 `concat` release.

The compatibility oracle is bcftools 1.24 `merge`, tag commit
`fb9f0f783e0f67d734f6fa7fe4df9d230522f196`, its official manual and scaling
notes, and black-box differential fixtures. The audited tag's `vcfmerge.c` has
SHA-256 `de5f5c39dc14159c42bf673e1c5544b9462ff9f3356ee6f476a79b0f351da510`.
The installed Apple-arm64 oracle has SHA-256
`33100a6b961c529e915394d53b4737a0f8dd7a164eac352afe4e74e1ced51f60`.
VCF 4.1 through 4.5, BCF2, CSI, TBI, BGZF, and the local-allele extension remain
the format authorities. Bcftools is GPL-3.0-or-later and is used as an
attributed behavior oracle; no source text or source structure is copied into
the MIT-or-Apache-2.0 implementation.

## Historical asset disposition

The historical `rsomics-vcf-merge` revision
`571af0688ac61b857b529b0db20ae886999e04fa` is a fixture pool only. Its
0.1.1 implementation accepts plain text VCF, parses records as unchecked
strings, converts a malformed position to zero, retains metadata only from the
first input, discards every INFO value and record ID, assumes the first
record's FORMAT layout, remaps only GT, passes all other A/R/G fields through
under wrong allele indexes, and creates no named output or transaction. It
does not detect duplicate samples or incompatible definitions. It has no BCF,
BGZF, index, region, gVCF, local-allele, merge-mode, filter, rule, or header
surface. Under `--json` it silently sends the only variant output to a sink.

| Asset | SHA-256 | Disposition |
|---|---|---|
| `src/lib.rs` | `d1f7fce75214bb17c0e302c5f432764c564a5d012de43cbfad4c7c6d094f8e3b` | Keep tiny same-position and allele-index fixtures; discard implementation |
| `tests/compat.rs` | `b3b5c0aa0e3c1252defc17745cf0cd825d25f99ee79cd4c6505cb7eb6130e350` | Refactor two useful cases into a pinned product oracle; remove conditional skipping |
| `tests/smoke.rs` | `69fd90f228624df21296c882ae2eb999ae2d38387b242dea912b8a6978228a7e` | Discard record-count-only smoke test |
| `src/cli.rs` | `318b7fceb1523ccdca702401cfd7a66dcb936ab42097b31d670e5ddea7a528f6` | Discard standalone help and JSON sink |
| `benches/bench.rs` | `cb420ed2ba97751b2c2e87fa24c6ec14c5b0ea1cfb23c03835187f041d96a9ee` | Discard process-launch timing over two tiny files |

The old repository stays retired. Selected fixtures move into product tests
with their provenance recorded; no operation-sized history is revived.

## Stable command contract

The command is:

```text
rsomics-vcf merge [OPTIONS] <VARIANT>...
```

At least two inputs are required. `--force-single` explicitly permits one
input for header normalization, region extraction, or rule testing. Positional
inputs and `-l, --file-list FILE` are mutually exclusive. File lists contain
one path per line; blank lines and lines whose first non-space character is
`#` are ignored. Relative paths use the process working directory. An empty
list, duplicate standard-input marker, or missing input fails before output.

Default mode requires every input to be BGZF VCF or BCF with a valid CSI or
TBI index. `--no-index` accepts plain, BGZF, or raw BCF streams whose contig
order agrees with their headers and with every other input. At most one
standard-input marker is allowed under `--no-index`; its reader remains open
after header preflight. Regions require indexes and therefore conflict with
`--no-index`.

`-o, --output FILE` defaults to standard output. `-O, --output-type TYPE`
uses the product-wide `v`, `z`, `b`, or `u` spellings and defaults to plain
VCF. `--threads INT` supplies bounded BGZF decompression and compression
workers. The global `--json` requires named variant output and never replaces
the data stream.

The stable merge controls are:

- `--force-samples`, which deterministically renames a duplicate sample from
  input N as `N:NAME`, adding another `N:` until it is unique;
- `--force-single`;
- `--print-header` and `--use-header FILE`;
- `-0, --missing-to-ref`;
- `-f, --apply-filters LIST`;
- `-F, --filter-logic union|pass-if-any`, accepting `+` and `x` as
  compatibility values;
- `-i, --info-rules RULES`;
- `-M, --missing-rules RULES`;
- `-m, --merge MODE`;
- `--trim-unseen variant|all`, with quoted `MODE,*` and `MODE,**` accepted as
  compatibility spellings;
- `-L, --local-alleles INT` with INT at least one;
- `-g, --gvcf -|REFERENCE`;
- `-r, --regions REGIONS`, `-R, --regions-file FILE`, and
  `--regions-overlap pos|record|variant`;
- `--no-index`.

`--print-header` stops after complete input and header validation. A supplied
header must contain exactly the final samples in final order and compatible
definitions for every field and contig that selected records can use. It may
add or replace descriptive metadata, but it cannot silently reorder samples,
change Number or Type, or drop a referenced FILTER. The output BCF dictionary
is rebuilt from the validated header and every source record is translated.
Validation finishes before the header is written.

The summary reports files, input samples, output samples, coordinate groups,
records read and written, records filtered, multiallelic groups, alleles
added, duplicate samples renamed, missing genotypes synthesized, gVCF blocks
split, localized samples, output encoding, and whether indexes or streaming
were used.

## Header and sample model

All input headers are parsed and reconciled before output. The output sample
order is input order followed by header sample order. Duplicate samples fail
unless `--force-samples` is present. Sites-only inputs contribute no samples
and remain valid.

Header reconciliation preserves first appearance while enforcing semantic
compatibility:

- matching contigs must have compatible declared lengths;
- matching INFO and FORMAT IDs must agree on Number and Type;
- matching FILTER IDs must be compatible;
- later new structured definitions are appended;
- exact unstructured metadata lines are emitted once;
- one canonical file-format line and one final sample header are written;
- existing input provenance metadata is preserved, but this command adds no
  command line or timestamp.

Conflicts fail during preflight. BCF records are translated through checked
per-input dictionaries. The command does not continue after a warning and
discover an incompatible value only after output has begun.

Existing standard AC and AN definitions are retained for recomputation; absent
definitions remain absent. Local-allele mode adds LAA and, when their source
fields occur, LAD and LPL definitions with the current specification
semantics. gVCF mode requires typed END and validates every implied INFO and
FORMAT rule against the final schema.

## Coordinate synchronization and pairing

The indexed path uses each input's CSI or TBI records. The no-index path uses
one forward reader per input. Both feed the same product-private coordinate
group abstraction and require each input to be sorted. The no-index path also
requires identical contig order across headers. Coordinate regression,
noncontiguous contigs, stale indexes, index/header disagreement, malformed
records, and incomplete compression are fatal.

At one coordinate, the engine buffers only the records present there. It forms
one output group at a time and never selects two records from the same input.
Exact allele overlap is preferred over type-only compatibility, matching the
bcftools 1.24 case in which an exact A>G record is paired before an earlier
A>C record at the same coordinate. Remaining records produce later output
groups. This pairing order receives direct three-input and within-file-
duplicate fixtures; it is not left to hash or heap iteration order.

Merge modes are:

- `none`: compatible REF plus overlapping or subset ALT sets may form one
  record, but disjoint ALT sets do not;
- `exact`: complete REF and ALT sets must match;
- `snps`: SNP records at a coordinate may form multiallelic SNPs;
- `indels`: indel records may form multiallelic indels;
- `both`: SNPs may merge with SNPs and indels with indels, but the two classes
  remain separate; this is the default;
- `snp-ins-del`: SNPs, insertions, and deletions each form separate groups;
- `all`: any records at a coordinate may merge;
- `id`: records at the same coordinate merge only when they share a
  nonmissing ID.

The `id` rule deliberately treats `.` as absent rather than a shared ID.
Bcftools 1.24 merges unrelated missing-ID records under `-m id`; a live probe
produced one A>C,G record from two inputs whose IDs were both `.`. Rsomics
keeps those records separate.

Compatible indel records are right-padded to one REF allele space at the same
POS before ALT union. This is representation reconciliation, not reference
normalization: inputs that need left alignment or trimming should pass through
`rsomics-vcf norm` first. Symbolic, breakend, spanning-deletion, and
unobserved-reference alleles follow typed VCF compatibility rules and are
covered explicitly.

`--trim-unseen variant` removes `<*>` or `<NON_REF>` from groups containing a
real alternate allele. `--trim-unseen all` also removes it from reference-only
sites. The option remaps every affected A/R/G field and genotype; it never
drops an allele string without rewriting dependent values.

## Site field reconciliation

Output CHROM and POS are the synchronized coordinate. REF and ALT use the
checked common allele space. Nonmissing IDs are joined once in first-selected
order with semicolons. QUAL is the maximum nonmissing numeric value and is
missing only if every selected record is missing.

`--apply-filters` selects source records before grouping. A record passes when
its FILTER contains at least one requested value; `PASS` and `.` remain
distinct explicit values. If no selected record remains for an input at the
coordinate, its samples are filled as absent.

Default FILTER logic is `union`: collect every named failing filter from the
selected records; PASS contributes no failing value. `pass-if-any` writes PASS
when any selected record is PASS, otherwise it writes the union. Definitions
must exist in the final header.

INFO reconciliation is schema driven. When defined, AC and AN are excluded
from ordinary copying and recomputed from the final typed GT values unless the
user supplies an explicit AC or AN rule. AC has one value per final ALT and AN
counts every nonmissing allele. Existing AF is not silently presented as a
cohort-wide recalculation; it follows its configured or allele-vector merge
rule.

Without explicit rules, DP and DP4 use sum when defined. Other scalar or fixed
fields take the first selected input value. Number=A, R, and G fields are
translated into the output allele space; values for distinct alleles occupy
their mapped slots and the bcftools 1.24 last-nonmissing rule resolves two
inputs that map a value to the same slot. Missing and vector-end values remain
distinct. Cardinality, type, numeric range, and overflow are checked.

`--info-rules` is a comma-separated list of `TAG:METHOD`, where METHOD is
`sum`, `avg`, `min`, `max`, or `join`. `-` disables the DP and DP4 defaults.
Numeric reducers ignore missing values and operate element-wise after allele
mapping. `join` preserves input order and changes the output definition to
variable Number when required. Duplicate rules, undefined tags, unsupported
types, impossible cardinality, nonfinite results, and integer overflow fail.

## FORMAT and genotype reconciliation

The output FORMAT layout is the ordered union of tags present in the selected
records. GT appears first when present. A sample whose record lacks one tag
receives a typed missing value. A sample whose input has no selected record
receives a missing GT and missing values for every other tag. Under
`--missing-to-ref`, only that synthesized GT becomes unphased reference;
ordinary missing GT values already present in a record are not rewritten.

The synthesized GT ploidy is the maximum GT ploidy present in the selected
records at that output site, matching bcftools 1.24. A live haploid/diploid
probe produced haploid `0` fills at haploid sites and `0/0` fills at a diploid
site. This assumption is stated prominently because absence from a VCF is not
biological proof of a reference genotype; gVCF input is preferred when
reference confidence matters.

GT allele indexes and phase are translated into the final allele space.
Number=A and R FORMAT vectors are remapped directly. Number=G vectors use the
sample's actual ploidy and VCF genotype ordering; haploid, diploid, and
polyploid cardinalities are supported without a fixed allele limit. Number=.
and fixed fields preserve their typed vectors. An allele, ploidy, or vector
cardinality disagreement fails with file, coordinate, sample, and tag context.

`--missing-rules` fills allele-vector slots created because an input did not
contain one output allele. A rule is `TAG:.`, `TAG:NUMBER`, or `TAG:max`.
The default is missing. gVCF mode defaults PL to the maximum observed value
and AD to zero. If the source explicitly contains `<*>` or `<NON_REF>`, its
corresponding value takes precedence over a synthetic fill. Rules apply to
new allele slots, not to an absent record or an explicitly missing source
value.

## Local alleles

Number=G likelihood vectors grow combinatorially as samples and alternate
alleles are merged. `--local-alleles N` caps each sample at N relevant ALT
alleles when a site has more than N output alternates. The full site ALT list
and GT remain unchanged. LAA stores increasing 1-based indexes into output
ALT. PL becomes LPL and AD becomes LAD for the sample's REF plus local alleles.
Other allele-dependent fields retain their ordinary mapped representation.

When an input record already has at most N alternate alleles, all its alleles
are retained in output order. Otherwise the selection is derived from typed
PL likelihood mass, keeps REF, chooses the N most relevant alternates, and
then orders LAA by output allele index. Ties are deterministic. Missing PL,
non-Number=G PL, wrong ploidy cardinality, overflow, and an impossible local
selection fail instead of warning once and disabling localization for the
site. LAD and LPL cardinalities are validated for every sample.

The local representation is compared with bcftools 1.24 and the official
scaling guidance across sparse high-allele workloads. It is not a generic
compression switch; it is a typed representation change visible in FORMAT.

## gVCF block merge

`--gvcf -` merges reference-confidence blocks and uses `N` when a block split
needs a reference base not supplied by a variant. `--gvcf REFERENCE` uses an
indexed FASTA through `rsomics-seqio::IndexedFasta`. Missing contigs, short
references, invalid bases, and FAI disagreements fail.

A gVCF block is recognized from a valid typed END interval and a reference
confidence allele such as `<*>` or `<NON_REF>`. END is one-based inclusive and
must not precede POS. Per input, blocks cannot overlap another active record
or regress. The engine holds at most one active block per input and splits
output at the next variant start, active block end, region boundary, or input
block transition. Output blocks have exact POS and END coverage with no gap or
overlap introduced by the merge.

Variant records interrupt and then resume reference-confidence coverage.
Every output segment merges active samples and fills inactive ones according
to gVCF missing rules. Region selection is applied after block expansion so
the first and last output blocks are trimmed to the selected interval rather
than leaking their original END.

gVCF mode adds these default INFO rules when the tags exist: QS sum,
MinDP/MIN_DP min, I16 sum, IDV max, and IMF max, in addition to DP and DP4
sum. It adds PL max and AD zero missing rules. User rules override these
defaults explicitly. Allele mapping, GT, AC/AN, FILTER, QUAL, local alleles,
and unseen-allele trimming retain the same typed contracts as ordinary mode.

## Format, transactions, and product structure

Named output rejects aliases with every named input, list, custom header, and
reference where applicable. It uses `rsomics-common::AtomicFile`. Commit
occurs only after valid EOF from every input, writer finish, flush, sync, and
product quickcheck. Header, index, order, pairing, allele, rule, cardinality,
gVCF, reference, compression, write, finish, sync, and broken-pipe errors
propagate to the top-level nonzero exit. Standard output cannot be atomic, but
configuration and header preflight finish before its first byte.

The implementation extends a small set of coherent private modules:

```text
src/
├── merge.rs
├── merge/
│   ├── header.rs
│   ├── pairing.rs
│   ├── fields.rs
│   ├── local.rs
│   └── gvcf.rs
└── commands/
    └── merge.rs
```

`merge.rs` owns typed options, coordinate flow, summaries, and mode dispatch.
`header` owns sample and schema reconciliation. `pairing` owns record grouping
and output allele maps. `fields` owns site, INFO, FORMAT, GT, and missing-rule
reconciliation. `local` owns LAA/LAD/LPL. `gvcf` owns active block state and
splitting. The command module owns Clap conversion, `rsomics-help`, input-list
resolution, output separation, aliases, and transactions.

The planned concat overlap engine and merge share only a private coordinate-
group reader and checked header primitives where their call sites prove the
same invariant. Concat's identical-sample and first-record policy never enters
merge's disjoint-sample, allele, or field policy. Names and types replace
narrative phase comments; source comments are reserved for stable invariants
and non-obvious reasons.

## Foundation decision

No new Layer A item is justified by merge. `rsomics-common` supplies errors,
transactions, alias rejection, and JSON delivery. `rsomics-help` supplies the
unified interface. `rsomics-seqio` supplies indexed FASTA and the BGZF frame
layer already justified by BAM and concat consumers. Existing product-private
region, cardinality, genotype, and writer components are reused.

Record pairing, INFO/FORMAT merge rules, local alleles, gVCF blocks, and
sample-prefix policy are variant-product behavior. Annotation reducers have a
different source/target contract, and the calling product does not yet have a
second concrete local-allele API call site. These stay inside `rsomics-vcf`.
Promotion is reconsidered only after another product consumer exists with
consumer-side tests.

## Deliberate compatibility differences

Normal successful behavior is compared with bcftools 1.24 after removing only
newly added provenance lines. These interface accidents or unsafe behaviors
are explicit differences:

- `id` mode does not treat two missing `.` IDs as the same ID;
- positional inputs and `--file-list` conflict instead of silently composing
  two input sources in positional-then-list order;
- blank and comment file-list lines are ignored; bcftools 1.24 treats a
  comment as a literal path;
- header and custom-header conflicts fail during preflight rather than after
  output initialization or the first incompatible value;
- a missing or malformed PL required for local alleles fails instead of
  emitting one warning and silently abandoning localization at that site;
- unindexed inputs are actively checked for sort and contig-order consistency
  rather than accepted behind a warning;
- named output is atomic;
- the redundant `--force-no-index` alias, provenance stamping, numeric
  compression levels, per-command verbosity, and automatic output indexing
  are absent.

`--no-version` is unnecessary because rsomics never inserts a command line or
timestamp. Automatic indexing remains excluded until output and index share a
grouped transaction; users run `rsomics-vcf index` after a successful merge.
Machine-readable execution provenance belongs in the JSON summary and
benchmark record rather than mutable variant headers.

## Verification and release gate

Tests precede each implementation group. The local gate covers:

- CLI help, two-input default, force-single, list parsing, comments, standard
  input, indexes, regions, output encodings, workers, JSON, aliases, and every
  conflict;
- header unions, sites-only files, duplicate sample recursion, custom header,
  contig lengths, Number and Type conflicts, dictionary order, print-header,
  and all four encodings;
- sorted indexed and streaming synchronization, three or more inputs,
  within-file duplicate records, coordinate ties, stale indexes, regressions,
  contig-order disagreement, truncation, and missing EOF;
- every merge mode, exact versus subset alleles, SNP/indel separation,
  insertion/deletion separation, ID and missing ID, symbolic alleles,
  breakends, spanning deletion, REF padding, and unseen-allele trimming;
- ID, QUAL, both FILTER logics, source FILTER selection, INFO defaults, every
  INFO reducer, AC/AN recomputation, A/R/G translation, missing/vector-end,
  string vectors, type conflicts, range and overflow;
- FORMAT union, GT phase and arbitrary ploidy remapping, A/R/G fields,
  fixed and variable fields, site ploidy fill, missing-to-ref, every missing
  rule, absent records, and malformed sample cardinality;
- local-allele selection, ties, LAA order, LAD, LPL, GT stability, missing PL,
  haploid/diploid/polyploid likelihoods, high ALT count, and BCF size limits;
- gVCF block recognition, block intersections, variants inside blocks,
  mismatched boundaries, reference and N fill, region trimming, rule
  overrides, invalid END, overlap, gap, symbolic allele variants, and local
  alleles;
- rollback for every configuration, header, index, record, order, rule,
  reference, compression, writer, finish, sync, and quickcheck failure.

The pinned oracle compares bcftools 1.24 for default, all pairing modes,
unseen-allele trimming, duplicate samples, one input, file lists, indexed and
no-index inputs, standard input, regions, custom and printed headers, source
filters, both FILTER logics, INFO rules, missing rules, missing-to-ref,
multiallelic A/R/G remapping, local alleles, and gVCF with and without a
reference. VCF comparison removes only tool-added provenance. BCF comparison
uses typed headers and records. Expected-divergence tests independently prove
missing-ID mode, comment lists, mixed input sources, local-allele failure,
unindexed sort validation, header preflight, and transaction behavior. Oracle
absence is a hard Linux x86_64 CI failure.

Performance gates include:

- a representative many-sample biallelic cohort merge;
- a multi-input, multiallelic A/R/G and FORMAT-heavy merge;
- a high-ALT likelihood workload with and without local alleles;
- a gVCF cohort with overlapping reference blocks and variants;
- indexed whole-genome and bounded-region paths.

Each workload records exact revisions, versions, machine, commands, input and
output hashes, record and sample counts, allele distributions, warmups,
alternating repeated wall time, CPU, and peak RSS. Every timed output first
passes semantic equality. Publication requires a strict throughput or
resource-use advantage on at least one representative merge hot path; equal
performance without another measured material benefit is insufficient.

Release requires formatting, strict Clippy, debug and release tests, the full
oracle, representative performance evidence, package verification, a fresh
public-API and hot-path review, and exact-head native CI on Linux and macOS for
x86_64 and aarch64. The registry archive is then independently downloaded,
matched to its VCS head and local package tree, installed with fresh external
Cargo state, and smoke-tested on ordinary, multiallelic, local-allele, and
gVCF VCF/BCF cases.

## Audit evidence

The retained external audit directory is
`/Volumes/KIOXIA/Developments/tmp/rsomics-vcf-merge-audit-20260819`. The
compact live bcftools 1.24 probe ledger is:

| Probe | Observed result | Contract consequence |
|---|---|---|
| default A>C plus A>G | one A>C,G record; GT, AD, PL, AF, and R values remapped | primary multiallelic oracle |
| reversed input order | samples, ALT, IDs, and first/last INFO precedence reverse deterministically | preserve input-order semantics |
| `-m none` multi-ALT plus subset ALT | one compatible record | distinguish subset from exact |
| `-m exact` on the same inputs | two records | exact complete allele set |
| SNP plus insertion | two records by default, one under `all` | type-mode oracle |
| insertion plus deletion | one padded indel record under `both`, two under `snp-ins-del` | REF padding and class oracle |
| two missing IDs under `-m id` | one merged record | deliberate rsomics divergence |
| two records in one file plus an exact record in another | exact cross-file pair emitted before remaining record | pairing-priority oracle |
| default INFO | DP summed; scalar first; A/R vectors allele mapped; AC/AN derived when present | field-policy oracle |
| explicit INFO rules | max, join, and sum applied; join makes Number variable | reducer oracle |
| FORMAT union | tags union in selected input order with per-sample missing values | FORMAT oracle |
| `-M PL:max,AD:0` | new G slots use sample max PL and new R slots use zero | missing-rule oracle |
| `-L 1` | site keeps ALT; samples receive LAA, LAD, and LPL | local-allele oracle |
| haploid/diploid `-0` | absent GT uses active site's maximum ploidy | documented risk and oracle |
| duplicate sample | default exit 255; force mode emits `2:A` | sample policy |
| one input | default exit 255; force-single succeeds | arity policy |
| comments in file list | comment treated as a path and fails | rsomics adds comment support |
| positional plus list | positional inputs precede listed inputs | rsomics makes sources exclusive |
| `--no-index` plain VCF and stdin | succeeds with an order warning | checked streaming path |
| regions with no-index | exit 255 | option dependency |
| gVCF overlapping blocks | blocks split at positions 1, 5, 6, 10, and 11 with mapped PL/AD | gVCF oracle |
| INFO Integer/Float conflict | warning followed by record-time exit 255 | rsomics fails in preflight |

Primary references are the
[bcftools merge manual](https://samtools.github.io/bcftools/bcftools#merge),
[bcftools scaling guidance](https://samtools.github.io/bcftools/howtos/scaling.html),
the [bcftools 1.24 merge source](https://github.com/samtools/bcftools/blob/1.24/vcfmerge.c),
and the [VCF, BCF, CSI, and TBI specifications](https://samtools.github.io/hts-specs/).
