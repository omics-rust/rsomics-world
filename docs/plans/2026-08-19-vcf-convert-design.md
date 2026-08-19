# rsomics-vcf convert design

Status: product boundary, bcftools 1.24 format families, historical assets,
legacy-format grammars, grouped output transactions, compatibility oracle,
deliberate safety differences, and release gates are defined. The target
release is 0.13.0 after the complete `consensus` slice.

## Product boundary

`convert` translates between typed VCF or BCF data and established external
variant or genotype interchange formats. It also expands gVCF reference blocks
and supplies one canonical VCF-to-BED preset. These profiles share typed
alleles, genotypes, sample order, coordinate spans, selection, reference
access, output grouping, and validation, so they remain one subcommand family
inside `rsomics-vcf`.

The retained profiles are:

- VCF/BCF to and from Oxford GEN/SAMPLE;
- VCF/BCF to and from HAP/SAMPLE;
- VCF/BCF to and from HAP/LEGEND/SAMPLE;
- whitespace-delimited or 23andMe-style genotype tables to VCF/BCF;
- gVCF reference-block expansion to ordinary site records;
- VCF/BCF record spans to BED3.

Generic VCF, BGZF VCF, raw BCF, and compressed BCF re-encoding remains in
`view`, which already owns that complete contract. Arbitrary text projection,
the historical genotype matrix, and `to-tsv` remain `query` formats. PLINK
binary, PED/MAP, PGEN, and association-data conversion belongs to
`rsomics-plink`; the commented bcftools PLINK and PBWT stubs are not public
upstream operations. No encoding, table, or profile becomes another crate.

## Upstream and format authority

The compatibility oracle is bcftools 1.24 `convert`, with bcftools 1.24
`query -f '%CHROM\t%POS0\t%END\n'` as the BED3 behavior oracle. VCF 4.5 and
BCF2 remain the variant authorities. Oxford GEN, HAP, LEGEND, and SAMPLE
grammar is checked against the bcftools manual and the current PLINK format
reference. Indexed FASTA follows FAI conventions. BED output follows the UCSC
zero-based, half-open interval definition.

The audited bcftools tag is revision
`fb9f0f783e0f67d734f6fa7fe4df9d230522f196`:

- `vcfconvert.c` SHA-256:
  `45181a2a809f450969b67c256c0962b319bad96aae4940fd3a44fea1902f1eff`;
- `convert.c` SHA-256:
  `06e08937f219d9881874d004fa874c21293b45effd8ad9fb9ab09eb9bae47794`;
- `tsv2vcf.c` SHA-256:
  `ae977ce8abd246319ea207c31a22e2a740b86599239828edeb9f01282aa2f631`.

These files carry MIT licenses. The implementation may reproduce documented
formats and black-box behavior but does not copy bcftools or HTSlib code.
Copied upstream fixtures retain file-level provenance and license attribution.

## Historical assets

### `rsomics-vcf-convert`

The retired repository is clean at revision
`0322987e3f53b2d4099bede46d6b5df3f4f5efe0`, version 0.1.1. Its public name
suggests the full upstream operation, but its implementation has two unrelated
partial paths:

- a line-preserving VCF-text pass-through with plain or gzip output;
- VCF-text export to HAP/LEGEND/SAMPLE.

The pass-through duplicates the current `view` boundary. It reads only plain
or gzip VCF, rejects BCF, does not validate typed records, mislabels ordinary
gzip as BGZF, truncates named output directly, and discards data under JSON.

The HAP/LEGEND/SAMPLE path parses a few text columns, skips multiallelic and
unparseable records through one `None` branch, converts missing or unsupported
alleles to reference, assumes diploidy, does not preserve partial missing or
haploid states, and writes three final files independently. It does not support
HAP/SAMPLE, GEN/SAMPLE, any reverse direction, sample selection, sex files,
IDs, regions, targets, expressions, BCF, gVCF expansion, TSV import, or BED.

Most tests compare record counts, source passthrough, or a live tool only when
it happens to be installed. The committed three-record HAP golden is labelled
bcftools 1.13. The audit reran its exact input with bcftools and HTSlib 1.24;
the output remains byte-identical at SHA-256
`f0d35db7656249f4dd807398a7cf669d41703208fa91432ff8ba89be1e8bb1ec`.
It is a useful narrow regression, not proof of the missing profiles. The
benchmark measures an 11-record process launch.

The source classification is:

1. direct merge: none;
2. refactor then merge: none;
3. test, fixture, or benchmark asset only: the small VCFs, phased HAP fixture,
   current-oracle-confirmed HAP golden, and output-layout expectations;
4. discard: both production paths, parsers, gzip writer, genotype fallback,
   output handling, standalone CLI and help, skip-capable harness, and launch
   benchmark.

### `rsomics-vcf-to-bed`

The retired repository is clean at revision
`50e87e5821cdc498e950f6e412d6b25fb016c944`, version 0.1.1. It line-parses
plain VCF, turns an invalid POS into zero, skips short rows, derives END only
from REF length, and writes directly to the destination. It therefore loses
typed symbolic `INFO/END`, BCF, compression, malformed-record, standard-input,
selection, transaction, and JSON behavior. On the audit `<DEL>` record,
bcftools reports BED end 35 while the historical function would report 30.

Its compact coordinate fixture and zero-based expectation remain test seeds.
The parser, writer, CLI, and compatibility harness are discarded. The target
BED profile reuses the current typed `%POS0` and `%END` projection rather than
maintaining another span calculation.

### `rsomics-vcf-utils`

The historical `genotypes` operation takes only the first colon-delimited
sample field and silently skips short records. `to-tsv` merely removes `##`
lines and the leading `#` from the column header. Both are incomplete `query`
presets, not external format conversions. Their tiny fixtures may seed query
regressions; no source is routed into `convert`.

## Command tree

```text
rsomics-vcf convert gen export [OPTIONS] [INPUT]
rsomics-vcf convert gen import [OPTIONS] --gen FILE --samples FILE
rsomics-vcf convert hap export [OPTIONS] [INPUT]
rsomics-vcf convert hap import [OPTIONS] --hap FILE --samples FILE
rsomics-vcf convert hap-legend export [OPTIONS] [INPUT]
rsomics-vcf convert hap-legend import [OPTIONS] --hap FILE --legend FILE --samples FILE
rsomics-vcf convert tsv import [OPTIONS] [INPUT]
rsomics-vcf convert gvcf expand [OPTIONS] [INPUT]
rsomics-vcf convert bed [OPTIONS] [INPUT]
```

`INPUT` defaults to standard input for one-stream profiles. An import may use
standard input for at most one named component. Export bundles require
`--output-prefix PREFIX`; component paths are derived deterministically and no
bundle member is multiplexed onto standard output. Import, gVCF, and BED
profiles use `-o, --output FILE`, with standard output as the non-JSON default.

The tree is rendered through `rsomics-help`. Help first explains the format
family and direction, then only the options valid for that leaf. The upstream
single-letter `-g`, `-G`, `-h`, and `-H` mode collision is not copied, and `-h`
remains unified help.

### Shared VCF-side options

VCF/BCF exporters accept:

- `--include EXPR` or `--exclude EXPR`, never both;
- indexed regions and streaming targets with the product overlap policies;
- sample inclusion, exclusion, and reordering;
- all four VCF/BCF input encodings and standard input;
- bounded compressed-input workers where the current format layer permits.

Import and gVCF profiles can emit plain VCF, BGZF VCF, raw BCF, or compressed
BCF through the product-wide output-type spelling. Compressed VCF/BCF supports
bounded workers. `--write-index csi|tbi` is available only for compatible
compressed output and commits the index with the variant file. Sort order and
contig blocks are validated before an index is staged.

Numeric compression-level spelling, version command-line stamps, permissive
verbosity values, extra positional inputs, output type inferred from a suffix,
and arbitrary HTSlib options remain excluded under product-wide policy.

### Export bundle naming

`--output-prefix DIR/NAME` creates:

| Profile | Compressed default | `--compression none` |
|---|---|---|
| GEN/SAMPLE | `NAME.gen.gz`, `NAME.samples` | `NAME.gen`, `NAME.samples` |
| HAP/SAMPLE | `NAME.hap.gz`, `NAME.samples` | `NAME.hap`, `NAME.samples` |
| HAP/LEGEND/SAMPLE | `NAME.hap.gz`, `NAME.legend.gz`, `NAME.samples` | `NAME.hap`, `NAME.legend`, `NAME.samples` |

Compressed matrix and legend files are gzip-compatible BGZF streams, matching
the useful upstream prefix behavior. SAMPLE remains plain text. Paths are
derived from the complete prefix rather than interpreted as labels. All
component paths must be distinct from one another and from every input after
normalization. Existing regular files may be replaced only as one group.

The upstream comma syntax, `.` output suppression, multiple bundle members on
one standard-output stream, and suffix-driven compression are excluded because
they make a standard bundle incomplete or ambiguous. A complete profile always
produces its complete standard file set.

## GEN/SAMPLE contract

### Export

`gen export` writes one Oxford GEN row per retained biallelic record and an
ordered SAMPLE file. `--layout 3n5|3n6` selects the original five leading
fields or the newer chromosome plus five-field layout. The default is `3n6`
because chromosome identity is a field rather than an overloaded ID.

`--probabilities gt|pl|gl|gp` selects FORMAT/GT hard-call triples, normalized
PL, normalized GL, or validated GP values. The field must exist in the header
and every retained record with schema-correct `Number=G` cardinality. Values
must be finite; GP values must be in `[0,1]` with a sum no greater than one
within the declared tolerance. Missing genotypes or all-missing likelihoods
emit `0 0 0`, preserving the Oxford no-call convention rather than inventing a
uniform called genotype.

GEN stores three biallelic diploid probabilities per sample. A haploid GT is
represented as the corresponding homozygous triple. Ploidy greater than two,
partial `Number=G` vectors, out-of-range allele indices, and nonfinite
likelihoods fail. PL and GL normalization uses a stable log-space calculation;
rounding is fixed and tested independently of locale.

`--ids generated|vcf` controls the second identifier. The first identity is
always the reversible `CHROM:POS_REF_ALT[_END]` spelling needed for strand and
symbolic-span checks. Missing VCF IDs remain `.` under the VCF policy and are
never confused with generated identity.

`--duplicates error|first|all` defaults to `error` for repeated positions.
`first` reproduces the useful upstream deduplication explicitly, and `all`
retains ordered records. `--unsupported error|skip` defaults to `error` for
no-ALT or non-biallelic records. `skip` reports exact category counts and never
groups malformed input with unsupported but valid records.

An optional `--sex-file FILE` maps each selected sample to `M`, `F`, or the
documented unknown value. Unknown samples, duplicate mappings, invalid codes,
or missing selected samples fail before commit. Without it, SAMPLE uses the
standard missing-sex form.

### Import

`gen import` accepts plain, gzip, or BGZF GEN by content and a plain SAMPLE
file. `--layout auto|3n5|3n6` defaults to `auto`; exact sample count makes the
two field counts unambiguous. Every nonblank GEN row must have exactly `3N+5`
or `3N+6` fields for `N` unique samples. Comments are accepted only where the
format authority permits them.

At least one of the two ID columns must parse as
`CHROM:POS_REF_ALT[_END]`. The encoded position and alleles must agree with the
dedicated fields, allowing the documented REF/ALT strand swap only when the
probability triple is reversed at the same time. `--ids generated|source`
chooses the VCF ID from reversible identity or the other source column.

Every probability is finite and in `[0,1]`; each triplet sum must not exceed
one beyond tolerance. `0 0 0` becomes missing GT and missing GP. Otherwise GP
is preserved and GT is the unique maximum-probability genotype; ties become
missing rather than silently selecting hom-ref. An optional hard-call threshold
may turn lower-confidence unique maxima into missing without changing GP.

The generated VCF 4.5 header declares GT, GP, samples, and observed contigs.
Contig length is omitted unless an optional indexed reference supplies the
real value; `2147483647` is never invented as a placeholder length. Input row
order is preserved and coordinate regression is reported when indexing is
requested.

## HAP/SAMPLE contract

### Export

`hap export` writes reversible identity, selected ID, POS, REF, ALT, then one
ordered haplotype value per allele copy for every selected sample. It supports
biallelic concrete and symbolic alleles with `INFO/END` included in generated
identity when required. Non-biallelic and no-ALT policy uses the same explicit
`--unsupported error|skip` contract as GEN.

Diploid phased and unphased genotypes, complete missing values, and haploid
genotypes retain the bcftools/SHAPEIT encodings: allele indices, `? ?` for a
missing diploid call, `-` for the absent second haplotype, and `*` on both
alleles for an unphased call. The external grammar cannot represent one known
and one missing diploid allele. `--partial-missing error|complete-missing`
therefore defaults to `error`; the explicit lossy mode writes `? ?` and counts
the conversion. `--haploid preserve` is the default; `--haploid diploid`
duplicates a known haploid allele. Allele indices greater than one, invalid
marker pairs, and ploidy greater than two fail rather than becoming reference.

Generated versus VCF IDs and optional sex mappings follow the GEN contract.
Repeated positions are valid because each row retains reversible allele
identity. Input order is preserved.

### Import

`hap import` reads one HAP row and the exact number of haplotype values implied
by the SAMPLE file. Reversible identity, POS, REF, ALT, optional symbolic END,
selected source ID, sample order, missing markers, ploidy markers, and phasing
are validated completely. A partial-missing pair is rejected because it is not
a valid IMPUTE2 known-haplotype call. Extra or missing columns, duplicate
samples, impossible allele values, and inconsistent identity fail.

The output VCF declares typed GT and END when needed. Generated versus source
ID policy is explicit. The import never turns `?` into reference or turns a
haploid call into diploid unless requested.

## HAP/LEGEND/SAMPLE contract

This family has the same genotype and sample semantics as HAP/SAMPLE, but
variant identity is stored in LEGEND while HAP contains only per-sample allele
values. Export writes the standard `id position a0 a1` header and one legend
row per HAP row. Import advances HAP and LEGEND in lockstep and requires both
to reach EOF together.

The SAMPLE schema is the four-column `sample population group sex` form. An
optional sex map changes only sex; population and group default to the sample
identifier unless future evidence justifies separately typed metadata. Every
string field is parsed as a field, not a shell or path token.

`--ids generated|vcf` applies to export. Import uses reversible legend IDs to
recover CHROM and symbolic END and rejects ambiguous VCF-ID-only legend input.
This is stricter than accepting a row that cannot reconstruct a variant.

## TSV import contract

`tsv import` accepts tab- or ASCII-whitespace-delimited rows. `--columns`
binds each source field to `CHROM`, `POS`, `ID`, `AA`, `REF`, `ALT`, or `-`.
CHROM and POS are required. Either one AA field per declared sample or the
sites-only REF and ALT pair is required. The number of bound and ignored
fields must equal every data-row field count.

AA accepts forward-reference A, C, G, T, or N alleles, two characters for
diploid calls, one for haploid calls, and `--` or `.` for missing. It does not
encode indels. REF/ALT mode supports concrete indels and typed symbolic alleles
without samples. Sample names come from `--samples` or `--samples-file`, are
unique, and match the number of consecutive AA fields.

An indexed plain or BGZF FASTA is required through the existing
`rsomics-seqio::IndexedFasta`. Every contig and one-based position must exist,
reference lookup must succeed, REF must match when supplied, and an AA call is
mapped against the forward reference base. Invalid rows fail with row, field,
and token context. They are never silently counted as skipped.

The output VCF 4.5 header uses true reference contig lengths and typed GT. Row
order is preserved; an optional output index adds sorted-coordinate validation.
The profile supports all four VCF/BCF outputs and standard input for the table
when the reference is named.

## gVCF expansion contract

`gvcf expand` recognizes reference blocks whose ALT is absent, `<*>`, `<X>`,
or `<NON_REF>` and whose typed INFO/END is present. A selected block becomes
one record for each one-based position from POS through END. END is removed,
REF is replaced with the exact indexed reference base, other alleles and typed
INFO/FORMAT values are preserved, and nonblock variant records pass through
unchanged.

Include or exclude expressions choose which blocks are expanded; they do not
drop records. This unusual upstream behavior is stated in help. Users run
`view` first when record filtering is intended. Indexed regions and streaming
targets restrict the input stream before expansion under their existing
overlap rules.

The indexed reference is accessed one bounded window at a time. Block END must
be at or after POS, within the declared contig, and before the next record.
`--block-overlap error|trim` defaults to `error`; `trim` reproduces upstream
clipping to the next record with an exact warning and counter. REF mismatch,
unknown contigs, malformed END, coordinate regression, and incomplete typed
fields fail.

All four VCF/BCF inputs and outputs, standard input, bounded compression
workers, transactional output, optional grouped index, and the product JSON
summary are supported. The operation does not normalize alleles or merge
blocks.

## BED3 contract

`convert bed` emits exactly three tab-delimited fields with no header:

```text
CHROM    POS0    END
```

POS0 is zero-based and END is the exclusive typed record end. The end includes
valid INFO/END semantics for symbolic and gVCF records rather than merely
adding REF byte length. Every VCF/BCF input encoding, standard input,
expressions, regions, and targets are supported. Sample selection is absent
because BED3 has no sample columns.

The implementation is a named preset over the current query fields and record
span model. Arbitrary BED columns, names, scores, strand, allele projection,
or genotype matrices remain explicit `query` formats. BED3 output may go to
standard output or one transactional named file; JSON requires a named data
output.

## Transactions and summaries

Every bundle member is staged through `rsomics-common::AtomicFile` and committed
with the existing `AtomicFile::commit_all` contract. Parse, selection,
compression, write, flush, sync, or commit failure restores every prior
component. The complete bundle is reopened and checked for row counts, matching
sample counts, matching variant counts, compression EOF, and lockstep files
before commit.

Single VCF/BCF output plus an index uses the same grouped transaction. Single
BED output uses the ordinary atomic file. Output aliases are rejected before
opening any destination. Input and output aliases, duplicate derived
components, directories, and unsupported special files fail.

JSON summaries report profile, direction, inputs, committed outputs, encoding,
sample count, input rows, output rows, skipped unsupported rows under explicit
policy, missing genotypes, probability source, reference lookups, block
expansion, and index metadata. JSON never suppresses or replaces a required
bundle member.

No manifest is added to standard GEN, HAP, LEGEND, or SAMPLE bundles. The
structured summary is command output, not an unexpected ecosystem file.

## Foundation decision

No new Layer A item is required.

- The current private VCF format, expression, region, target, sample, query,
  record-span, writer, and index layers own variant policy.
- `rsomics-seqio::IndexedFasta` already serves `rsomics-call` and
  `rsomics-vcf norm`; TSV import and gVCF expansion are additional consumers,
  not reasons for a second reference abstraction.
- `rsomics-common::AtomicFile::commit_all` already has multiple product call
  sites and owns the required generic grouped file transaction.
- `rsomics-help` renders the nested product tree and common diagnostics; any
  ergonomic extension must remain generic and preserve current BED, BAM, VCF,
  and sequence consumers.

GEN probability policy, HAP markers, legend synchronization, TSV columns,
gVCF blocks, BED presets, and bundle suffixes remain private product modules.
There is no second product consumer for a public legacy-genotype-format crate.

## Product structure

```text
src/
├── convert.rs
├── convert/
│   ├── bed.rs
│   ├── bundle.rs
│   ├── gen.rs
│   ├── gvcf.rs
│   ├── hap.rs
│   ├── hap_legend.rs
│   ├── identity.rs
│   ├── probability.rs
│   ├── sample.rs
│   └── tsv.rs
└── commands/
    ├── convert.rs
    └── convert/
        ├── bed.rs
        ├── gen.rs
        ├── gvcf.rs
        ├── hap.rs
        ├── hap_legend.rs
        └── tsv.rs
```

`convert.rs` owns profile dispatch and typed summaries. `identity.rs` owns the
reversible variant identifier. `sample.rs` owns the two external SAMPLE
schemas and sex mappings. `probability.rs` owns checked likelihood conversion.
`bundle.rs` owns derived paths, staged writers, validation, and grouped commit.
Each format module owns only its grammar and typed conversion.

Command adapters convert Clap values, bind existing selection, use
`rsomics-help`, and render summaries. They contain no line parser, probability
math, variant span calculation, or direct destination creation.

## Compatibility contract

The differential matrix covers:

- GEN 3N+5 and 3N+6, generated and VCF IDs, GT/PL/GL/GP sources, sample sex,
  duplicates, missing values, haploid calls, and both directions;
- HAP/SAMPLE and HAP/LEGEND/SAMPLE concrete and symbolic alleles, END identity,
  VCF IDs, phased and unphased diploid calls, haploid calls, partial and complete
  missingness, sex, and both directions;
- exact SAMPLE headers, sample order, duplicate names, malformed rows, early
  EOF, extra EOF, plain and compressed components;
- TSV comments, ignored columns, field permutations, one or more samples,
  diploid, haploid and missing AA, sites-only REF/ALT, indels, unknown contigs,
  reference mismatch, bounds, and malformed tokens;
- gVCF absent ALT, `<*>`, `<X>`, `<NON_REF>`, END, expressions, selections,
  reference bases, adjacent and overlapping records, all encodings, and index;
- BED3 SNV, MNV, insertion, deletion, symbolic END, gVCF block, every input
  encoding, standard input, and selection;
- every unsupported policy, output alias, input alias, existing destination,
  broken pipe, truncated compression, write failure, and grouped rollback.

The pinned bcftools 1.24 regression corpus includes its GEN, HAP, LEGEND,
SAMPLE, TSV, 23andMe, and gVCF fixtures. Round trips compare the strongest
representable semantics rather than demanding fields that the destination
format cannot carry. Export bytes are compared where formatting is part of the
profile contract. Imports compare typed headers, records, genotypes,
probabilities, sample order, and exit decisions.

## Deliberate fail-loud differences

Live bcftools 1.24 probes recorded behaviors that are not copied:

| Probe | bcftools 1.24 | rsomics contract |
|---|---|---|
| extra positional input | silently ignores the second path and exits 0 | reject extra positional input |
| malformed VCF during three-file export with existing destinations | exits nonzero after truncating HAP and LEGEND and replacing SAMPLE | restore the complete previous bundle |
| HAP, LEGEND, and SAMPLE all name the same output | exits 0 and leaves only one clobbered component | reject every component alias before staging |
| multiallelic export | warns, skips the record, and exits 0 | fail by default; skip only under explicit unsupported policy |
| duplicate GEN position | silently omits the later record by default | fail by default; `first` or `all` is explicit |
| missing GT exported to GEN | writes `0.33 0.33 0.33`, which looks like a called uniform posterior | write the Oxford `0 0 0` no-call triple |
| partial-missing diploid GT exported to HAP | collapses the known allele into `? ?` | fail by default; lossy completion is explicit |
| TSV row with nonnumeric POS | increments skipped and exits 0 | fail with row and POS context |
| TSV POS 150 against a 100-base reference | exits 0 and emits an empty REF allele | fail before committing output |
| GEN values `1.2 -0.2 0` | accepts them as GP and calls hom-ref | reject probability values outside `[0,1]` |
| GEN values `0 0 0` | emits hom-ref GT | emit missing GT and missing GP |
| malformed GEN after one valid row with an existing output | exits nonzero after truncating the destination | preserve the previous output |
| overlapping gVCF block and next record | warns and truncates the block | fail by default; trim only under explicit block policy |

Bcftools's warnings and skip counters remain oracle evidence for explicit
compatibility policies, but malformed input and silent biological-data loss do
not become defaults.

## Tests

Unit tests cover:

- reversible identity parsing, underscores, symbolic END, allele swaps,
  missing IDs, overflow, and ambiguity;
- exact GEN field counts, finite probability conversion, log-space PL/GL,
  sums, no-call, ties, thresholds, layout, duplicates, and rounding;
- both SAMPLE schemas, sex maps, duplicate samples, blank lines, comments, and
  cardinality;
- HAP markers, phasing, ploidy, complete and partial missingness policies,
  allele range, and lockstep LEGEND;
- TSV column binding, delimiter rules, repeated AA fields, reference lookup,
  allele construction, and exact field consumption;
- gVCF recognition, END bounds, expression-controlled expansion, block overlap,
  reference replacement, typed field preservation, and counters;
- BED spans for concrete and symbolic records;
- derived bundle paths, aliases, compression EOF, reopen validation, staged
  indexes, group restoration, and JSON restrictions.

Golden and differential tests use every upstream format fixture plus focused
VCF 4.5 and BCF2 cases. Malformed suites cover invalid headers, typed values,
probabilities, identifiers, sample counts, component lengths, reference
coordinates, compressed EOF, unsupported alleles, sort order, and output
failures.

Fault injection covers every component create, write, compress, flush, sync,
reopen, validate, backup, persist, restore, index, and parent sync. A failed
bundle never exposes a mixture of old and new components.

## Performance gates

Formal comparison uses pinned bcftools and HTSlib 1.24 with source and binary
hashes, machine, filesystem, input hashes, flags, warmups, alternating runs,
timing distributions, peak RSS, bytes read and written, compression ratio, and
semantic output hashes recorded.

Representative workloads include:

- 5 million biallelic records and 100 samples exported to GEN from GT and GP;
- the same cohort exported to HAP/SAMPLE and HAP/LEGEND/SAMPLE;
- 5 million GEN and HAP rows imported into compressed BCF;
- chromosome-scale gVCF blocks expanded to at least 50 million site records;
- a multi-sample TSV import with indexed reference access;
- 10 million mixed concrete and symbolic records projected to BED3.

The release requires a strict throughput or resource-use advantage on a
representative hot path. Candidate advantages are checked typed streaming,
bounded allocations, direct BED projection, and cached reference windows; none
is claimed before measurement. Bundle validation and transactions cannot hide
unbounded memory or scratch. Every timed output passes the semantic oracle
before its measurement is accepted.

## Release gate

Release 0.13.0 is complete only when:

- every declared format, direction, selection, genotype, probability,
  reference, bundle, index, and policy behavior is implemented without
  placeholders;
- all historical implementation is reclassified and only approved fixtures
  or goldens are imported with provenance;
- all four VCF/BCF encodings and every declared plain or compressed external
  component pass ordinary and malformed compatibility matrices;
- grouped bundles and output-plus-index transactions pass fault injection;
- formatting, strict Clippy, unit, integration, differential, malformed-input,
  transaction, and benchmark smoke suites pass;
- the formal performance gate records a strict useful advantage;
- package contents, repository metadata, README, nested unified help, licenses,
  and attribution are reviewed from a clean exact head;
- native Linux and macOS CI pass on `x86_64` and `aarch64` at that exact head;
- the crate is published only after all earlier declared release slices and
  this complete convert slice are present.

The live audit and generated evidence are retained outside the repository at:

```text
/Volumes/KIOXIA/Developments/tmp/rsomics-vcf-convert-audit-20260819
```

Primary references are the
[bcftools 1.24 manual](https://samtools.github.io/bcftools/bcftools.html#convert),
the [bcftools 1.24 convert source](https://github.com/samtools/bcftools/blob/1.24/vcfconvert.c),
the [PLINK format reference](https://www.cog-genomics.org/plink/2.0/formats),
the [UCSC BED format](https://genome.ucsc.edu/FAQ/FAQformat.html#format1),
and the [VCF and BCF specifications](https://github.com/samtools/hts-specs).
