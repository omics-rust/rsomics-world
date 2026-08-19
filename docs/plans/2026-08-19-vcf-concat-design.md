# `rsomics-vcf concat` design

Status: product boundary, upstream contract, historical assets, deliberate
differences, shared-foundation requirement, and release gate audited. Product
implementation is intentionally not started while `rsomics-vcf` main remains
the exact publishable 0.6.0 revision
`682942cfa69768dc3a127a8544f2f07213b704ea`.

## Purpose and boundary

`concat` combines coordinate-sorted VCF or BCF chunks that have identical
sample columns. Ordinary concatenation preserves chunk order. Overlap mode
performs a deterministic coordinate merge and may remove records repeated
across different chunks. Ligate mode reconciles phased diploid haplotypes in
chunk overlaps. Naive mode splices compatible BGZF streams without
recompressing their record blocks.

This is one operation of `rsomics-vcf`, not another crate. Every mode shares
the product's VCF/BCF headers, typed records, regions, output encodings,
transactions, JSON delivery, and installation identity. The operation remains
absent from public help and README until the complete stable contract in this
document passes its release gate. The target release is 0.7.0 after 0.6.0 is
published and independently verified.

The compatibility oracle is bcftools 1.24 `concat`, tag commit
`fb9f0f783e0f67d734f6fa7fe4df9d230522f196`, its official manual and help,
and black-box differential fixtures. The audited tag's `vcfconcat.c` has
SHA-256 `7a7c212fdcf8d9b9cb40b545ee88a5fe4598855a48e7cd8781ec039ba568bc90`.
The installed Apple-arm64 oracle has SHA-256
`33100a6b961c529e915394d53b4737a0f8dd7a164eac352afe4e74e1ced51f60`.
VCF 4.1 through 4.5, BCF2, BGZF, CSI, and TBI remain the format authorities.
The audited `vcfconcat.c` and HTSlib synced-reader sources use the MIT license
and are used as attributed behavior oracles; no upstream source text or source
structure is copied into the implementation.

The historical `rsomics-vcf-concat` revision
`15088a2e6cbaef6bfb49669e9625e50b6ace7e50` is a source-asset pool, not a
base implementation. It opens each input twice, accepts only text VCF through
plain or generic gzip, infers header identity from strings, materializes every
reader, performs no coordinate validation, creates named output directly, and
silently discards variant output when `--json` also owns standard output. It
does not implement BCF, structural BGZF validation, overlap merging, duplicate
policies, indexed regions, ligation, genotype removal, safe raw concatenation,
output conversion, compression workers, or atomic persistence.

Historical asset disposition is:

| Asset | SHA-256 | Disposition |
|---|---|---|
| `src/lib.rs` | `45572cb6a71cbe135ef4a5c4a866e0fefb73feebdd6364b4145935bdab7c18bf` | Header-union and same-sample behavior seed only; implementation discarded |
| `tests/compat.rs` and `tests/golden` | `de535fd38b7e0d66a1622c176cd85b1b4daaaf1ccdbcdc5603540addf2703116` for the test driver | Refactor useful fixtures into the product oracle; replace skip-on-missing tests |
| `src/cli.rs` | `811aeaaa7079c7e47fd4e0c04c4bbf12639836929ad41cfd954fabdaf5b544e3` | Discard standalone CLI, manual help schema, and JSON sink behavior |
| `benches/bench.rs` | `ac67bb985857d8fc7f5d5ba12c48172465615f942675b06839ed8bea1bb2e4fa` | Discard command-launch benchmark over tiny fixtures |

## Stable command contract

The command is:

```text
rsomics-vcf concat [OPTIONS] <VARIANT>...
```

One or more positional inputs are required unless `-f, --file-list FILE` is
used. The two input forms conflict. A file list contains one path per line;
blank lines and lines whose first non-space character is `#` are ignored.
Relative entries are resolved from the process working directory, matching
the useful bcftools path rule. An empty list, repeated standard input, or a
missing path fails before output is opened.

`-o, --output FILE` defaults to standard output. `-O, --output-type TYPE`
accepts the product-wide `v`, `z`, `b`, and `u` spellings. Normal modes default
to plain VCF. `--threads INT` controls bounded BGZF compression workers and is
valid only for compressed output. The command accepts global `--json` only
with named variant output; JSON never replaces or discards the variant stream.

The remaining stable options are:

- `-a, --allow-overlaps` for a sorted multi-input merge;
- `-d, --rm-dups MODE` with `snps`, `indels`, `both`, `all`, or `exact`;
- `-D, --remove-duplicates` as the exact-mode compatibility alias;
- `-G, --drop-genotypes` to remove FORMAT definitions, FORMAT values, and
  sample columns;
- `-l, --ligate`, `-c, --compact-PS`, `--ligate-force`, `--ligate-warn`, and
  `-q, --min-PQ INT` for phased chunk ligation;
- `-r, --regions REGIONS`, `-R, --regions-file FILE`, and
  `--regions-overlap pos|record|variant` for indexed overlap-mode selection;
- `-n, --naive` for validated raw BGZF concatenation.

Duplicate removal and regions require `--allow-overlaps`. `--allow-overlaps`
and `--ligate` conflict. `--drop-genotypes` and `--ligate` conflict.
`--compact-PS` and `--min-PQ` require ligation. `--ligate-force` and
`--ligate-warn` conflict. Naive mode conflicts with overlap, ligation,
genotype removal, regions, compression workers, and any explicit output type
that differs from the first input's compressed VCF or BCF encoding.

Exactly one standard-input marker is accepted in ordinary ordered mode. Its
reader is retained after header preflight instead of being reopened, fixing an
observed bcftools 1.24 failure after it consumes the stream during preflight.
Overlap, regions, ligation, and naive mode require reopenable named inputs.

The summary reports input files, records read, records written, duplicates
removed, overlap-only records dropped, samples dropped, phase orientations
changed, output encoding, and whether raw BGZF splicing was used. Counts that
do not apply to the selected mode are zero, not omitted.

## Header preflight and merge

Every input header is parsed before the first output byte. Sample names and
order must be identical. Sites-only and sampled files cannot be mixed unless
`--drop-genotypes` removes samples from all inputs. The merged header preserves
first appearance order and applies these checks:

- contigs with the same ID must have compatible declared lengths;
- INFO and FORMAT definitions with the same ID must agree on Number and Type;
- FILTER definitions and other structured records retain the first compatible
  declaration;
- later new contigs, FILTER, INFO, and FORMAT definitions are appended;
- exact unstructured metadata lines are emitted once;
- one canonical file-format line and one column header are emitted;
- tool command lines and timestamps are not inserted.

Conflicting semantic definitions fail. In particular, the command does not
copy bcftools 1.24's warning-and-first-definition behavior for a later INFO or
FORMAT type conflict, because the surviving header can misdescribe later
records. BCF dictionary translation is built from the checked merged header;
no encoded dictionary index is reused across incompatible inputs.

`--drop-genotypes` removes every FORMAT declaration and writes exactly the
eight fixed VCF columns or the equivalent sites-only BCF record. Existing INFO
values are preserved. The command does not infer cohort annotations from the
removed genotypes.

## Ordered and overlap engines

Ordinary mode streams inputs in argument order. The global output order is
checked across records and file boundaries. Positions must be nondecreasing
within a contiguous contig block, and a contig cannot reappear after another
contig block has begun. Equal positions remain legal and retain file and
record order. A regression fails with the input path and both coordinates.

This is intentionally stronger than bcftools 1.24. The audited implementation
does not update its `prev_pos` variable in the ordinary path, and black-box
probes confirmed that it exits successfully while emitting decreasing
positions both inside one input and across chunk boundaries. Rsomics never
advertises sorted output while silently producing a disordered stream.

Overlap mode uses a bounded k-way merge over one current record per input.
Each input is independently required to be coordinate sorted. The output tie
order is input order followed by record order. Region selection uses each
input's CSI or TBI index and the product's existing position, record-overlap,
or variant-overlap semantics. Unknown contigs, stale or incompatible indexes,
coordinate regression, and truncated input are fatal.

Duplicate removal suppresses only a record matched in an earlier input;
duplicates within one input remain. Pairing is one-to-one for each later input
at a coordinate: exact allele matches are assigned first, then the selected
relaxed matches, and one earlier record cannot suppress multiple records from
the same later input. The first matching input wins. Matching uses CHROM, POS,
REF, and ALT rather than ID, QUAL, FILTER, INFO, or FORMAT:

- `exact` requires the same complete allele set;
- `snps` additionally pairs different SNP allele sets at the same position;
- `indels` additionally pairs different indel allele sets at the same
  position;
- `both` enables the SNP and indel relaxations without pairing the two
  classes to each other;
- `all` pairs any records at the same position.

These rules mirror HTSlib 1.24 collapse modes while making the cross-input
scope and deterministic winner explicit.

## Ligation

Ligate mode consumes phased chunks in ascending chunk order and keeps at most
two active chunks plus their overlap buffer. Inputs must be coordinate sorted,
indexed, sample-identical, and contain records from at most one contig per
file. Empty chunks are ignored. Separate files may advance from one contig to
the next; decreasing chunk starts fail. The per-file restriction prevents the
silent record loss observed when bcftools 1.24 ligates multi-contig chunks.

For every sample, informative overlap sites are complete, phased, diploid,
heterozygous genotypes present in both chunks. Direct and swapped haplotype
matches determine whether the later chunk's phase orientation is reversed.
Ties preserve the current orientation. Homozygous, haploid, polyploid,
unphased, missing, and GT-absent sites do not vote; malformed typed GT values
fail rather than becoming a warning.

The first chunk supplies duplicate overlap records before the handoff, the
later chunk supplies records after it, and compatible overlap records appear
once. `FORMAT/PQ` records the sample-wise orientation quality at the handoff.
`FORMAT/PS` starts at the first coordinate and starts a new phase set when PQ
is below `--min-PQ`, whose default is 30. `--compact-PS` emits PS only at a
phase-set start. Existing incompatible PQ or PS definitions fail.

Default ligation requires perfect overlap: every site inside the shared span
must exist in both chunks with a matchable variant key. `--ligate-warn` drops
unpaired overlap sites and reports one structured warning summary.
`--ligate-force` keeps unpaired sites and also permits nonoverlapping chunks.
The three policies are tested separately against bcftools 1.24. Phase updates
are typed and preserve all fields unrelated to GT, PQ, and PS.

## Naive BGZF mode

Naive mode accepts only BGZF-compressed VCF inputs or only BGZF-compressed BCF
inputs. It preserves that format and compression class. The first header is
written once, later headers and every per-input EOF marker are removed, record
blocks are copied without recompression, and exactly one canonical BGZF EOF
marker terminates the output.

All inputs are fully preflighted before raw copying. Preflight verifies the
gzip header and arbitrary extra subfields, the BGZF BC subfield, declared
frame size, deflate stream, CRC32, uncompressed size, one canonical EOF,
absence of trailing bytes, format magic, samples, and complete canonical
header compatibility. BCF also requires identical dictionary order. This is
stricter than bcftools 1.24, whose safe naive check accepts differing INFO
types when IDs retain the same dictionary positions; a live probe confirmed
that an Integer/Float conflict exits zero.

`--naive-force` is deliberately absent. Skipping header compatibility can
produce BCF whose dictionary indexes are interpreted under the wrong schema.
An unsafe corruption switch is not part of a fail-loud product contract.
Ordinary mode remains available when compatible headers need checked merging
or dictionary translation.

## Format, transaction, and product structure

Named output rejects aliases with every named input and uses
`rsomics-common::AtomicFile`. Commit occurs only after all readers reach valid
EOF, the writer finishes, output sync succeeds, and any requested product
quickcheck passes. Configuration, header, index, record, order, compression,
write, finish, sync, and broken-pipe errors propagate to the top-level nonzero
exit. Standard output cannot be transactional, but argument and header
preflight finishes before its first byte.

The implementation extends a small number of coherent private modules:

```text
src/
├── concat.rs
├── concat/
│   ├── header.rs
│   ├── stream.rs
│   ├── ligate.rs
│   └── naive.rs
└── commands/
    └── concat.rs
```

`concat.rs` owns typed options, summaries, and mode dispatch. `header` owns
sample and schema reconciliation. `stream` owns ordered and heap-based record
flow plus duplicate policy. `ligate` owns the two-chunk phase state. `naive`
owns VCF/BCF header-boundary handling over shared BGZF frames. The command
module owns Clap conversion, `rsomics-help` presentation, output separation,
alias checks, and transactions. No operation-sized repository or public
concat library API is created.

Source comments remain limited to stable invariants or non-obvious reasons.
The order state, duplicate classes, phase state, and frame state use named
types rather than narrative phase comments.

## Shared-foundation decision

`concat --naive` confirms a real shared BGZF framing component. The named
product consumers are:

1. `rsomics-bam cat` and `rsomics-bam reheader`, currently backed by
   `rsomics-bam::bgzf_rewrite`;
2. `rsomics-vcf reheader` and the planned `rsomics-vcf concat --naive`,
   currently backed by `rsomics-vcf::format::bgzf`.

The two implementations already duplicate frame-header parsing, BC size
checks, EOF handling, raw copying, and malformed-stream tests. The VCF copy is
less complete because it assumes the canonical six-byte extra field and does
not itself expose CRC and uncompressed-size validation. The BAM copy accepts
arbitrary extra-subfield order and performs the stronger checks.

The format-neutral frame layer will move to `rsomics-seqio::bgzf`, not a new
crate and not `rsomics-common`. `rsomics-seqio` is already a long-term Layer A
I/O foundation, already carries gzip and BGZF dependencies, and is already a
dependency of both products. Its public item is limited to reusable buffered
frame parsing, validation, decoding, raw frame access, and canonical EOF
handling. BAM headers, VCF headers, BCF dictionaries, record boundaries,
transaction policy, and command behavior remain private to their products.

Extraction occurs only after 0.6.0 publication allows the VCF consumer to move
off its frozen release head. It requires consumer-side tests in both products,
an API review against their concrete call sites, and a benchmark demonstrating
that the shared buffer-reusing path does not regress BAM cat/reheader or VCF
reheader/concat. Until those gates pass, the duplicated private code is not
deleted.

No other Layer A API is justified. `rsomics-common` already supplies the
transaction, alias, error, and JSON contracts. `rsomics-help` already supplies
the unified command presentation. Header merge, duplicate collapse, ligation,
and VCF/BCF policy have no second product consumer and stay internal.

## Deliberate compatibility differences

Normal successful cases are compared with bcftools 1.24 after removing only
its provenance and timing lines. These observed upstream behaviors are defects
or unsafe switches and are recorded as explicit differences:

- ordinary mode rejects within-file and cross-file coordinate regressions
  that bcftools 1.24 emits successfully because its position tracker is not
  advanced;
- semantic header conflicts fail instead of retaining the first declaration
  after a warning;
- one ordinary standard-input stream is retained and works rather than being
  consumed during preflight and reopened at EOF;
- file-list comments are ignored and the command-line/list conflict message
  names `--file-list` rather than incorrectly naming `-l`;
- `--naive` checks complete schema compatibility and BGZF integrity before
  raw copying;
- ligation rejects a chunk containing multiple contigs instead of silently
  dropping records from its later contig, as observed with bcftools 1.24;
- `--naive-force`, provenance stamping, numeric compression levels,
  per-command verbosity, and automatic output indexing are not exposed;
- named output is atomic rather than truncated before a later input failure.

Automatic indexing remains an explicit product-wide exclusion until variant
output and its index can be committed as one grouped transaction. Users run
`rsomics-vcf index` after successful concatenation. Unified JSON and error
presentation replace the upstream verbosity flag. Provenance belongs in
machine summaries and benchmark records, not mutable variant headers.

## Verification and release gate

Tests precede each implementation group. The local gate covers:

- CLI help, list/positional exclusivity, blank and comment list lines,
  standard input restrictions, aliases, option conflicts, JSON separation,
  outputs, workers, and output aliases;
- header unions across all four encodings, sample count and order, sites-only
  inputs, later definitions, contig lengths, INFO/FORMAT Number and Type,
  FILTER metadata, arbitrary metadata, and BCF dictionary translation;
- ordered concatenation, equal boundaries, coordinate regression,
  noncontiguous contigs, malformed records, truncated compression, and one or
  many inputs;
- overlap tie order, per-input sort validation, every duplicate mode,
  within-input duplicates, three or more simultaneous files, indexed regions,
  overlap rules, stale indexes, and unknown contigs;
- genotype removal across VCF and BCF and preservation of unrelated INFO;
- ligation with direct, swapped, tied, missing, homozygous, unphased, haploid,
  polyploid, GT-absent, perfect, imperfect, forced, and nonoverlapping chunks,
  plus exact PQ/PS and compact-PS boundaries;
- naive VCF and BCF, multi-frame headers, arbitrary BGZF extra fields, mixed
  formats, header and dictionary conflicts, CRC and size corruption, partial
  frames, missing or repeated EOF, trailing bytes, and one output EOF;
- named-output rollback for every configuration, header, index, record,
  ordering, framing, writer, finish, sync, and quickcheck failure;
- unchanged `rsomics-vcf reheader` and `rsomics-bam cat/reheader` behavior
  after the shared frame extraction.

The pinned bcftools 1.24 oracle compares ordinary, overlap, all duplicate
modes, genotype removal, indexed regions, perfect and imperfect ligation, PS
compaction, safe naive VCF and BCF, file lists, all input encodings, and all
normal output encodings. Expected-divergence tests independently demonstrate
the ordering, header-conflict, standard-input, unsafe-naive, and transactional
differences. Oracle absence is a hard failure in the Linux x86_64 gate.

Performance gates use at least three representative workloads:

- many ordered chromosome chunks, measuring the raw VCF and typed BCF paths;
- multiple overlapping indexed chunks with duplicates and region selection;
- large compatible BGZF VCF and BCF chunks in naive mode.

Each gate alternates command order after warmups, records repeated wall time,
CPU, peak RSS, versions, exact revisions, machine, input and output hashes,
record counts, and semantic equivalence. Ligation receives a separate
many-sample phased workload. Publication requires a strict throughput or
resource-use advantage on at least one relevant hot path; equal performance
without another measured material benefit is insufficient.

Release requires formatting, strict Clippy, debug and release tests, the full
oracle, the shared-foundation consumer gates if extraction occurs, performance
evidence, package verification, a fresh public-API and hot-path review, and
exact-head native CI on Linux and macOS for x86_64 and aarch64. The registry
archive is then downloaded independently, matched to the release head and
package tree, installed with fresh external Cargo state, and smoke-tested on
ordinary, overlap, ligate, and naive VCF/BCF cases.

## Audit evidence

The retained external audit directory is
`/Volumes/KIOXIA/Developments/tmp/rsomics-vcf-concat-audit-20260819`. Its live
bcftools 1.24 probes confirm sample-order failure, header union, sites-only
output, overlap merging, cross-file duplicate removal, indexed regions,
phase orientation and PS compaction, blank file-list handling, comment-line
failure, safe raw VCF/BCF concatenation, uncompressed-input rejection, output-type
restrictions, and the deliberate divergences above. These are dossier probes,
not release fixtures; stable minimal fixtures will be committed in the product
with their generation and oracle commands.

The compact probe ledger is:

| Probe | bcftools 1.24 result | Contract consequence |
|---|---|---|
| ordered `a.vcf c.vcf` | exit 0; header union and four records | compatible normal oracle |
| overlapping ordered `a.vcf b.vcf` | exit 0 with `chr1:30` followed by `chr1:20` | rsomics rejects the regression |
| one input containing positions 20 then 10 | exit 0 with decreasing output | rsomics validates every input |
| `-a` over indexed `a.vcf.gz b.vcf.gz` | exit 0; coordinate merge with input-order ties | k-way merge oracle |
| `-a -d exact` | exit 0; later exact allele matches removed | cross-input first-wins oracle |
| `-G` | exit 0; FORMAT declarations and sample columns removed | sites-only oracle |
| swapped sample columns | exit 255 | exact sample-order gate |
| Integer/Float INFO conflict | exit 0 after warning; first definition retained | rsomics fails the conflict |
| regions without `-a` | exit 255 | option dependency retained |
| `-l` and `-l -c` | exit 0; later phase swapped with PQ/PS, compact form omits repeated PS | ligation oracle |
| file list with a blank line | exit 0 | blank lines ignored |
| file list with a comment | exit 255 after treating the comment as a path | rsomics adds comment support |
| `-n` over compatible BGZF VCF and BCF | exit 0; decoded records valid | safe raw oracle |
| `-n` over matching IDs but conflicting INFO types | exit 0 | rsomics strengthens semantic checks |
| `-n` over uncompressed inputs or explicit mismatched output | exit 255 | raw-mode format gate retained |
| ordinary input `-` followed by a named file | exit 255 after preflight consumes stdin | rsomics retains the reader and succeeds |

Primary references are the
[bcftools manual](https://samtools.github.io/bcftools/bcftools#concat), the
[bcftools 1.24 concat source](https://github.com/samtools/bcftools/blob/1.24/vcfconcat.c),
the [VCF and BCF specifications](https://samtools.github.io/hts-specs/), and
the [BGZF specification](https://samtools.github.io/hts-specs/SAMv1.pdf).
