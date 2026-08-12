# Variant format, calling, and copy-number product dossiers

Status: boundary, upstream-operation, and historical-source audit complete.
`rsomics-vcf` 0.2.0 is published with the complete first-release `head`,
`query`, `validate`, `index`, and `view` slice plus typed `filter`.
`rsomics-call` 0.1.1 is published with its complete three-command first
release. `rsomics-cnv` does not yet exist and is not published.

## Portfolio decision

The historical VCF-prefixed source pool mixed five user workflows. The input
format does not determine the product:

| Workflow | Target | Routed assets |
|---|---|---:|
| VCF/BCF inspection, transformation, filtering, indexing, and format statistics | `rsomics-vcf` | 30 |
| Alignment pileup, genotype likelihoods, and lightweight small-variant calling | `rsomics-call` | 2 |
| BAF/LRR copy-number and chromosome-level polysomy analysis | `rsomics-cnv` | 2 |
| Transcript-aware variant consequence annotation | `rsomics-annotation` | 2 additional |
| Genotype QC, LD, concordance, association, ROH, and pedigree reports | `rsomics-plink` | 11 additional |

`rsomics-call` and `rsomics-cnv` are not names reserved around placeholders.
Each has a real upstream workflow, two complementary historical
implementations, a shared state model, and a complete first-release contract.
They replace the rejected workflow and expression candidates, bringing the
accepted portfolio to 30 products without reviving micro-crates.

```mermaid
flowchart LR
    align["SAM/BAM/CRAM"] --> call["rsomics-call"]
    call --> variants["VCF/BCF artifacts"]
    variants --> vcf["rsomics-vcf"]
    variants --> annotation["rsomics-annotation"]
    variants --> plink["rsomics-plink"]
    variants --> cnv["rsomics-cnv"]
    bamio["rsomics-bamio"] --> call
    pileup["rsomics-pileup"] --> call
    stats["rsomics-stats"] -. consumer-proven kernels .-> plink
    stats -. consumer-proven kernels .-> cnv
```

Artifact arrows describe data flow, not Cargo dependencies. No Layer B product
depends on `rsomics-vcf` or another Layer B product.

## Upstream contracts

The live review uses:

- [bcftools 1.24](https://samtools.github.io/bcftools/bcftools.html), released
  2026-07-09, for command, plugin, expression, output, and default behavior;
- [VCF 4.5 and BCF2](https://github.com/samtools/hts-specs) as the format
  authority;
- [HTSlib 1.24](https://github.com/samtools/htslib/releases/tag/1.24) for
  typed-value, BGZF, CSI/TBI, and malformed-record behavior;
- vcftools 0.1.17 only for named legacy report profiles that remain useful in
  `rsomics-plink` or VCF format statistics.

The installed audit binaries are bcftools 1.24 and HTSlib 1.24. Bcftools 1.24
has 22 named commands plus a plugin dispatcher and 41 installed plugins. A
plugin name is not automatically a public subcommand; retained plugin behavior
must fit one of the product workflows above.

Version 1.24 invalidates several historical compatibility claims:

- multiallelic `MAC` and `MAF` expression values were corrected;
- `view --types` now distinguishes `ref`, `bnd`, `other`, and `overlap`;
- `annotate`, `merge`, `norm`, and `stats` fixed observable typed-field,
  localized-allele, symbolic-END, and sample-order behavior;
- `fill-tags` changed integer conversion and added grouped embedded functions;
- `af-dist` added per-sample HWE output;
- `cnv` and `roh` corrected HMM quality offsets;
- `mpileup` added `FORMAT/QM`;
- `split-vep` added SnpEff support and stopped lowercasing severity.

Historical 1.21 or 1.23.1 goldens are evidence seeds, not the current release
oracle.

## `rsomics-vcf`

### Boundary

One product owns VCF/BCF representation, inspection, transformation,
filtering, indexing, and format-centered reports. It accepts plain VCF,
BGZF-compressed VCF, and BCF where the operation permits them. It preserves
declared format versions and does not silently downgrade typed values.

The current bcftools commands retained here are:

- `annotate`, `concat`, `consensus`, `convert`, `filter`, `head`, `index`,
  `isec`, `merge`, `norm`, `query`, `reheader`, `sort`, `stats`, and `view`.

Named plugin behavior that fits this boundary includes allele-length
inspection, fill-tags, fixref, indel statistics, setGT, split, and
variant-distance. Arbitrary dynamic plugin loading is excluded: a Rust product
does not promise the bcftools C plugin ABI.

Historical convenience operations collapse into the product:

- `extract` becomes `query`;
- sample selection becomes `view --samples`;
- filter summary, SNP density, Ts/Tv stratification, allele length, and indel
  statistics become named `stats` reports;
- `to-bed` becomes a `convert` output profile;
- variant distance and fill-tags become `annotate` modes where their option
  contracts compose cleanly;
- the 30-operation `vcf-utils` source is split among existing commands, and
  duplicate or ungrounded one-liners are discarded.

### First release slice

- `view`
- `query`
- `head`
- `validate`
- `index`

This slice proves the shared format contract before complex record rewriting:

- streaming plain VCF, BGZF VCF, and BCF readers and writers;
- VCF header schema, sample order, contig order, typed INFO/FORMAT values,
  ploidy, phase, missing values, and BCF vector-end handling;
- basic record/sample/region selection and VCF/BCF conversion in `view`;
- current bcftools query-format parsing and typed rendering;
- strict VCF 4.x validation with line, field, and record context;
- TBI and CSI creation, listing, stats, and indexed region access;
- `rsomics-help` command layout, examples, diagnostics, output selection, and
  exit mapping.

`view` must implement the complete stable operation declared in help. The
historical subset of type, FILTER, and sample predicates is insufficient.
Region selection must use a compatible index when required and must not
pretend that a full streaming scan is equivalent.

### Current implementation

`rsomics-vcf` revision
`36d400df74666fb4c8b3bc16fb3cd74d1d56be71` implements the complete
first-release slice and typed filtering. The initial slice consolidated the
historical
`rsomics-vcf-head`, `rsomics-vcf-query`, `rsomics-vcf-extract`,
`rsomics-vcf-valfmt`, `rsomics-vcf-validate`, `rsomics-vcf-index`,
`rsomics-vcf-sample`, and `rsomics-vcf-view` assets into private product
modules, fixtures, and predicates while replacing their partial whole-file
implementations. `rsomics-help` supplies the shared command layout.
`rsomics-common` 0.12.3 supplies the owned output transaction used by both
VCF and BAM products. It preserves complete invalid validation reports in the
shared JSON and exit contract and rolls back named variant output on failure.

`head`:

- accepts plain VCF, gzip or BGZF VCF, uncompressed BCF 2.2, compressed BCF
  2.2, named files, and standard input;
- preserves ordered header lines, installs the canonical PASS definition,
  removes BCF-internal `IDX` fields, and renders typed records as VCF text;
- implements header limits, record limits, and output beginning at `#CHROM`;
- matches bcftools 1.24 byte-for-byte on the declared valid VCF, compressed VCF,
  BCF, stdin, and option-combination oracle matrix;
- fails non-zero on invalid numeric arguments, malformed headers, invalid POS,
  QUAL, typed INFO or FORMAT values, record/sample cardinality, truncated
  compression, and invalid BCF structure.

`query`:

- accepts the same VCF, gzip/BGZF VCF, BCF 2.2, named-file, and stdin inputs;
- projects fixed columns, typed INFO/FORMAT values, samples, genotypes,
  translated and IUPAC genotypes, variant types, whole FORMAT/record lines,
  headers, and zero-based vector elements;
- preserves current bcftools rules for sample-loop field shadowing, explicit
  fixed fields, singleton INFO subscripts, missing and vector-end values, and
  automatic newlines;
- uses one parser and header/value model across text and BCF, with a private
  direct BCF path for numeric projection and a typed fallback for complex
  records;
- validates every referenced tag and selected sample before replacing a named
  output, and keeps projection data separate from `--json` summaries;
- explicitly excludes regions, targets, expressions and functions,
  multi-input masks, `PBINOM`, `N_PASS`, `TBCSQ`, `VKX`, and undefined-tag
  fallback from the current command contract.

`validate`:

- accepts plain or gzip/BGZF-compressed VCF 4.1 through 4.5, raw BCF 2.2,
  BGZF-compressed BCF 2.2, named files, and standard input;
- validates header structure and dictionaries, fixed columns, declared and
  reserved INFO/FORMAT types, `Number=A/R/G/LA/LR/LG/P/B`, ploidy,
  cardinality, allele indices, phase and structural-variant invariants,
  coordinate order, contiguous contig blocks, and normalized duplicates;
- preserves total error and warning counts while bounding retained
  diagnostics, identifies line and field context, and exits 1 for completed
  invalid validation rather than treating it as an I/O failure;
- supports the EBI-compatible explicit `--require-evidence` policy and emits
  the same structured report on JSON success or validation failure;
- uses one validation model after decoding VCF, BGZF, raw BCF, or BGZF BCF,
  and retains only the normalized-variant coordinate window that can still
  collide with future sorted records.

`index`:

- builds CSI by default for complete BGZF VCF or BCF and TBI for BGZF VCF,
  with custom CSI minimum shifts, real multithreaded BGZF decompression,
  explicit replacement, named output, and named-output support for stdin;
- chooses HTSlib-compatible CSI depth from the format and declared contig
  lengths, preserves deep spanning-variant queries through propagated linear
  offsets, and uses `REF`, `INFO/END`, symbolic `INFO/SVLEN`, and gVCF
  `FORMAT/LEN` spans;
- validates BCF shared and sample blocks while reusing per-record buffers,
  rejects malformed dictionaries, truncated BGZF, ordinary gzip, unsorted
  coordinates, noncontiguous contig blocks, and TBI for BCF;
- reports total or per-contig mapped counts from variant paths, direct
  `.csi`/`.tbi` paths, and explicit `##idx##` paths, retaining unknown
  empty-contig counts as `.` like bcftools;
- builds the complete index before common 0.9 atomically replaces the
  destination, so parse, compatibility, allocation, write, flush, or sync
  failures do not expose a partial index.

`view`:

- accepts plain VCF, BGZF VCF, raw BCF 2.2, and BGZF BCF 2.2, converting
  between VCF, compressed VCF, raw BCF, and compressed BCF;
- implements full, header-only, and no-header output, rejecting headerless BCF
  because BCF records require their header dictionaries;
- selects, excludes, reorders, or reads samples from a file, rejects duplicate
  inclusions, supports explicit force for missing samples, and emits sites-only
  output when every requested sample is absent under that policy;
- removes genotypes or recalculates `AC` and `AN` after sample projection,
  including the current no-ALT behavior;
- filters by FILTER state, known or novel IDs, allele count, and the complete
  bcftools 1.24 `snps`, `indels`, `mnps`, `ref`, `bnd`, `other`, and `overlap`
  type model;
- streams targets and uses CSI or TBI for record-, position-, or
  variant-overlap regions, merging region lists and deduplicating only repeated
  index hits for the same record;
- validates every record on the direct text path, uses typed conversion where
  the requested transformation requires it, and atomically replaces named
  output only after the complete stream succeeds;
- keeps expressions, allele trimming and remapping, frequency and genotype
  predicates, output indexing, and compression workers outside the public
  first-release contract.

The representative oracle covers multi-ALT, numeric and missing INFO arrays,
flags, string INFO, FORMAT scalars and arrays, phased and missing genotypes,
sample inclusion and exclusion, headers, and VCF/BGZF/BCF equivalence. When
all samples are excluded, rsomics consistently removes them from `%FORMAT`
and `%LINE` for both VCF and BCF. Bcftools 1.24 retains BCF samples on that
path while its VCF path removes them; the format-dependent upstream result is
recorded as a rejected compatibility defect.

The rsomics spelling is `-H, --headers`, retaining `-h` for unified help.
`--records` and `--samples` are mutually exclusive rather than inheriting
bcftools's last-option-wins behavior. The bcftools behaviors where an invalid
number becomes zero, `-1` becomes an effectively unbounded count, or a
malformed record prints a diagnostic but exits zero are compatibility defects,
not contracts.

The validation release oracle traverses 465 VCF files at pinned hts-specs
revision `da617203a9527537746e200abda2885bec3a822c` and 955 files at pinned
EBI vcf-validator 0.10.2 revision
`0aacc4a44430ab9cee87d9925aacb28a2fb0a9fb`. Production validation remains
specification-correct where those corpora contain stale passed fixtures:
every exception is restricted by exact path and either exact diagnostics or a
zero-error classification. Recorded defects include contradictory `ALT=.`
allele data, an invalid float token, trailing empty records, stale CHROM
grammar, an unsorted VCF 4.5 passed fixture, and the EBI `%` FORMAT
compatibility extension. No corpus-specific exception is present in production
code.

The initial `head` exact-head CI run `30619149446`, query run `30622684140`,
validation run `30627803709`, index run `30630841891`, and view run
`30632972348` pass native Linux and macOS on `x86_64` and `aarch64`. Final
release-head run `30633237582` passes the same four native target classes at
revision `bbc09be7ed38`. The Linux `x86_64` job builds bcftools 1.24 from the
official SHA-256-pinned archive, fetches both exact validation corpora, and
passes command, validation, index-region, spanning-variant, stats,
malformed-BCF, and view compatibility suites.

### Later slices

1. `norm`, `annotate`, `reheader`, and `setgt`, with complete allele-remapping,
   symbolic-variant, gVCF, expression-reuse, and header contracts.
2. `concat`, `merge`, `isec`, `sort`, and `split`, with bounded memory,
   external runs, header/sample reconciliation, localized alleles, and
   transactional multi-output behavior.
3. `consensus`, `convert`, and format-centered `stats` profiles.

An unfinished command is absent from public help. No operation ships as a
flag-compatible shell around a partial line parser.

### Release 0.2: typed filtering

Release 0.2 adds `filter`. It is a record-selection and
record-annotation operation, not another spelling of `view`: `view` selects
records by identity, type, sample, and location, whereas `filter` evaluates a
typed expression and may annotate failing records or rewrite only the failing
samples. `setgt` remains a later operation for rule-driven genotype editing
outside this failure contract.

The behavior oracle is bcftools 1.24 `filter` and its shared expression
language. The installed executable has SHA-256
`33100a6b961c529e915394d53b4737a0f8dd7a164eac352afe4e74e1ced51f60`.
The official 1.24 source archive has SHA-256
`8caddc22610ee2851666047c859bb91da0c1e32d0c2ec553db6f153ad130e46f`;
its MIT-licensed `vcffilter.c` and `filter.c` have SHA-256
`8df4eab21b2e0c9b9261faff889b5bb23f47ef9c09e840f5798bb3ba4ff344a5`
and `8768cd317e3b051d1ae6731de4cd1062967e8d0dfba8f174b13918afd63b6e83`.
The VCF 4.1-4.5 and BCF2 specifications remain the format authority.

The stable command accepts plain or BGZF VCF, raw or BGZF BCF, named input or
standard input, and writes VCF, BGZF VCF, raw BCF, or BGZF BCF. It includes:

- mutually exclusive include and exclude expressions;
- numeric and string constants, file-backed sets, arithmetic, comparisons,
  case-sensitive or explicitly case-insensitive regular expressions,
  parentheses, and the distinct `&`, `&&`, `|`, and `||` sample/site rules;
- fixed columns, typed INFO and FORMAT fields, FILTER-set comparisons,
  genotype classes, variant type, missing values, arrays, sample and element
  subscripts, genotype-selected allele subscripts, and calculated allele,
  sample, missingness, and indel-length variables;
- site and per-sample aggregate functions, string and numeric functions,
  binomial and Fisher tests, phred scaling, and `N_PASS`/`F_PASS`;
- hard filtering, named or generated soft filters, replace/add/reset modes,
  and setting failing sample genotypes to missing or reference while updating
  existing valid `AC` and `AN` fields;
- SNP-gap and indel-cluster filtering with bounded look-ahead;
- inline or file-backed masks, streaming targets, and indexed regions with
  position, record, or variant overlap policy;
- transactional named output, bounded compression workers, shared JSON
  summary, and non-zero failure on malformed schema, record, expression,
  region, index, or output data.

Perl callbacks are excluded because they are an optional bcftools build-time
extension rather than a portable format contract. Automatic output indexing
is also excluded from this increment; the product already has an explicit
transactional `index` operation, and coupled output-plus-index replacement
needs a later grouped-transaction contract. Upstream verbosity levels are not
copied into the public interface. Additional positional regions are rejected
in favor of the explicit region options. None of these exclusions appears in
help as accepted behavior.

The expression implementation is private to `rsomics-vcf`. It evaluates the
existing typed record model after one header-bound compile step and is shared
internally by `filter`, later `setgt`, and later expression-enabled `view` and
`annotate` modes. Those are commands of one product, not independent Layer A
consumers. No public VCF expression or VCF I/O crate is added.

The historical `rsomics-vcf-filter` revision
`93d91c114d2ce0fc31a6b1c7176280f558c06f3c` contributes its small malformed,
sites-only, compound-FILTER, and quality fixtures only. Its whole-file input,
VCF-text-only output, three-predicate surface, direct destination creation,
untyped expressions, and narrated source are discarded. Historical
`rsomics-vcf-expr` revision
`94722777e8e1851182cb4c5ccf0b3ae9127eca2f` is a grammar and regression seed,
not mergeable implementation: it rejects regexes and array indexes, reads
only the first INFO value, does not use header types, and implements only a
small fraction of the current expression algebra.

The implementation adds private `expression` modules for tokens, syntax,
typed values, header binding, evaluation, and functions, plus private
`filter` modules for application, genotype rewriting, masks, and gap state.
The command adapter contains only CLI conversion and common-layer output.
Existing format readers/writers, region indexes, variant classification,
transaction ownership, and `rsomics-help` styling are reused.

The ordinary release matrix covers every operator and value family, scalar
and vector missingness, mixed ploidy, multiallelic records, flags, FILTER
sets, undefined or wrongly typed tags, numeric edge values, sites-only input,
all four encodings, standard input, every annotation mode, genotype and
`AC`/`AN` updates, masks, regions, targets, gap boundaries, malformed and
truncated inputs, broken pipes, target aliasing, and rollback. The live oracle
compares normalized headers, records, selected samples, and exit decisions
with bcftools 1.24. Representative text and BCF fixtures record input and
output hashes, repeated timing distributions, peak RSS, versions, flags, and
machine provenance. The principal filter hot path shows a strict throughput
and memory advantage.

Revision `404cb98310e08a520a033fb19733963fbceb98a3` completes the command.
The root CLI uses `rsomics-help`; `view` and `filter` share private typed
region, target, sample, and output selection; and named output uses the owned
`rsomics-common::AtomicFile` transaction. The filter accepts all four declared
VCF/BCF encodings, bounded BGZF workers, hard and soft filtering, failed-sample
genotype replacement, masks, SnpGap and IndelGap, indexed regions, and
streaming targets. The complete typed evaluator remains the semantic path.
A statically restricted scalar-numeric expression path avoids constructing a
full owned record only for eligible hard-filter expressions on plain VCF; all
other expressions, annotation modes, transformations, encodings, masks, gaps,
and selections use the typed implementation.

Feature-head CI run `31553731245` passes at revision `404cb98310e0` on native
Linux and macOS `x86_64` and `aarch64`. Its Linux `x86_64` oracle job builds
bcftools 1.24 and exercises typed expressions, hard and soft filters, genotype
rewriting, all output encodings, masks, gap filters, exact and open-ended
regions, targets, transactional rollback, invalid output combinations, and
record and exit compatibility. Release-head CI run `31554294004` repeats the
four native target classes at revision `36d400df7466`.

The release benchmark used an Apple M2 Mac mini with 8 GB RAM on arm64 macOS
26.6.1 build 25G76. The generated 250,000-record, 7.5 MiB plain VCF has
SHA-256
`f3352403c8a071f09e71da3b38901bca091a73fdd6f7b0b2a49281095964512a`.
Filtering `INFO/DP >= 20 && QUAL >= 30` to `/dev/null` took 57.164–59.240 ms
with a 58.241 ms Criterion estimate for rsomics and 111.94–118.35 ms with a
114.91 ms estimate for bcftools 1.24. This principal scalar-numeric text path
is approximately 1.97 times faster. Three warm `/usr/bin/time -l` runs
observed maximum RSS of 3,702,784 bytes for rsomics and 6,782,976 bytes for
bcftools. The committed `benches/filter.rs` regenerates the input and
benchmarks both filters and their view baselines under the same conditions.

### Release 0.3: allele normalization

The next release increment is `norm`. It owns equivalent VCF
representations of one variant: reference-guided left alignment and trimming,
complex-allele atomization, multiallelic split and join, and the typed field
remapping required by each transformation. It does not absorb reference index
creation, general sorting, or arbitrary annotation merely because those can be
composed with normalization.

The behavior oracle is bcftools 1.24 `norm`. The installed executable has
SHA-256
`33100a6b961c529e915394d53b4737a0f8dd7a164eac352afe4e74e1ced51f60`.
The official 1.24 archive has SHA-256
`8caddc22610ee2851666047c859bb91da0c1e32d0c2ec553db6f153ad130e46f`;
`vcfnorm.c`, `abuf.c`, and `gff.c` have SHA-256
`97163126a8d2f04f25daf793e67731f3ed070d8fcd52ae54c28efeb7c2c69657`,
`19c85ba4831ed4e1a169206a7b44352e8bad644a2aa040164adfa48602523e1c`,
and `70a55a5b1cbe531e03e4608fd4c56bb53ca96b20c81c04ab75495f63b554d821`.
The VCF 4.5 specification PDF has SHA-256
`7a1f6990dbfca4a20d9b6f21a4cb0bcbfd876853bfc86a832af2de6153abffa5`.

The stable command accepts plain or BGZF VCF, raw or BGZF BCF, named input or
standard input, and writes all four encodings. Its declared behavior includes:

- indexed plain or BGZF FASTA reference access, sequence-name and range
  validation, left alignment, parsimonious trimming, IUPAC reference handling,
  and the exit, warn, exclude, or REF-fix mismatch policies;
- MNV and complex-allele atomization with missing or spanning-deletion overlap
  alleles, original-record annotation, and composition with normalization and
  splitting;
- multiallelic split and biallelic join for SNPs, indels, separate mixed types,
  or any type, with configurable overlapping-allele replacement and strict
  merged FILTER behavior;
- schema-driven INFO and FORMAT remapping for fixed, `A`, `R`, `G`, and
  variable cardinalities across integer, float, string, character, flag, and
  genotype fields, including mixed ploidy, phasing, missing values, vector-end
  values, AD sum preservation, and valid AC and AN updates;
- symbolic alleles, breakends, spanning deletions, gVCF alleles and blocks,
  telomere coordinates, duplicate alleles, and duplicate-record policies;
- optional expression selection through the existing private typed engine,
  indexed regions, streaming targets, position or lexicographic local output
  order, bounded displacement buffering, transactional named output, bounded
  compression workers, and JSON separation from variant output;
- GFF3-directed HGVS 3-prime right alignment for unambiguous forward-strand
  transcripts, with left alignment retained for overlapping or conflicting
  transcript orientation.

The experimental upstream `--force` behavior is excluded: a malformed
allele-indexed field fails rather than being silently discarded. The deprecated
`-D` alias is excluded in favor of the explicit duplicate policy. Automatic
output indexing remains excluded until a grouped output-and-index transaction
exists. Upstream verbosity, version stamping, and positional region shortcuts
are not copied into the public interface. These exclusions are absent from
help rather than accepted as no-ops.

The historical `rsomics-vcf-norm` revision
`c4eeb5026199141a08ddd7b710be14488887edc2` contributes its small split
fixtures and regressions for GT and `Number=R` remapping. Its whole-file load,
VCF-text-only parser and writer, partial `Number=A/R` INFO handling, direct
output creation, one-flag CLI, and source narration are discarded. The old
implementation does not perform reference normalization despite its product
name and is not a mergeable command.

Reference access is a public `rsomics-seqio` item because it has two
concrete consumers: the existing indexed and chunk-cached FASTA paths in
`rsomics-call` and the new `rsomics-vcf norm` realignment path. The public
contract is zero-based half-open range access by reference name over indexed
plain or BGZF FASTA with bounded cache ownership and contextual errors. It does
not expose VCF policy or caller-specific reference IDs. `rsomics-call` tests
cross-line indexed FASTA windows through its reference cache, and
`rsomics-vcf` tests indexed plain and BGZF references through normalization.

The default implementation at `777c38362c14` covers reference normalization,
typed multiallelic split and join, biallelic and complex atomization, AD sum
preservation, original-record tracing, explicit duplicate policies,
split-overlap policy, strict joined FILTER precedence, and REF mismatch exit,
warn, skip, and fix behavior. Expression selection controls transformation
without dropping unselected records. Streaming targets filter sequential
input, while indexed regions query TBI or CSI inputs before transformation;
region queries merge overlaps, suppress repeated spanning records, support all
three overlap rules, and compose with targets. Join supports SNPs, indels,
separated mixed types, or any type with typed `A`, `R`, `G`, scalar, GT,
mixed-ploidy, symbolic, breakend, and allele-extension handling. GFF3-directed
HGVS right alignment, explicit local sort modes, and bounded BGZF compression
workers are included. The command and its `rsomics-seqio` dependency are no
longer feature-gated.

The contiguous implementation range is `90bd113` through `777c383`. Exact-head
runs `31571842323`, `31572567989`, and `31573693688` pass native Linux and
macOS on both `x86_64` and `aarch64`; the final Linux `x86_64` job packages the
default command and passes all 24 bcftools 1.24 normalization oracles. Those
oracles cover all four variant encodings, indexed plain and BGZF FASTA, IUPAC
and missing REF repair, phased and mixed-ploidy GT remapping, AC updates,
duplicate alleles created by REF swaps, split-plus-atomize origin tracing,
every join class, strict FILTER behavior, expression selection, target and
region overlap modes, local sort modes, GFF orientation, and indexed VCF and
BCF queries. The default local gate passes 123 unit tests, 24 CLI tests, all 24
oracles, release tests, strict Clippy, formatting, and package verification.

The formal benchmark used revision `998ff3e06927`, bcftools/HTSlib 1.24,
hyperfine 1.20.0, three warmups, and ten measured runs on the Apple M2 host
described below. Record bodies were byte-identical before timing. The
500,000-record reference-guided indel fixture has SHA-256
`c525d895c7d836ec0a30e099adb36d6ad02a2d05761fbefd4c8e5c8528410a1a`;
rsomics took `0.920359 ± 0.008359` seconds versus
`1.169889 ± 0.017242` seconds for bcftools, a 1.27-times throughput win. One
resource run used 6,799,360 versus 7,012,352 bytes RSS. The 200,000-record,
eight-sample typed split fixture has SHA-256
`1456d81007b74bf54ff0d28cc76941b9519d84dec71359808bed145dca21e162`;
it produces 400,000 records and took `5.456640 ± 0.061248` seconds versus
`7.455577 ± 0.047262` seconds, a 1.37-times throughput win. Its single-run RSS
was higher at 10,518,528 versus 8,437,760 bytes, so the decision rests on the
strict throughput advantage rather than a false memory claim. The committed
`benchmarks/norm-vs-bcftools.sh` regenerates inputs, verifies bodies, and
records binaries, hashes, flags, timings, RSS, and machine provenance.

### Current structure

```text
src/
├── lib.rs
├── main.rs
├── cli.rs
├── head.rs
├── index.rs
├── norm.rs
├── query.rs
├── query_bcf.rs
├── query_format.rs
├── regions.rs
├── validate.rs
├── variant_type.rs
├── view.rs
├── expression/
│   ├── bind.rs
│   ├── evaluate.rs
│   ├── raw.rs
│   ├── syntax.rs
│   └── value.rs
├── filter/
│   ├── gaps.rs
│   └── stream.rs
├── format/
│   ├── reader.rs
│   ├── record.rs
│   ├── value.rs
│   ├── text.rs
│   └── writer.rs
├── index/
│   ├── bcf_record.rs
│   ├── build.rs
│   ├── csi.rs
│   ├── stats.rs
│   └── vcf.rs
├── norm/
│   ├── atomize.rs
│   ├── cardinality.rs
│   ├── duplicate.rs
│   ├── gff.rs
│   ├── merge.rs
│   ├── reference.rs
│   └── split.rs
├── validation/
│   ├── definitions.rs
│   ├── header.rs
│   ├── record.rs
│   └── v44.rs
├── view/
│   ├── regions.rs
│   ├── samples.rs
│   └── selection.rs
└── commands/
    ├── filter.rs
    ├── head.rs
    ├── index.rs
    ├── norm.rs
    ├── query.rs
    ├── validate.rs
    ├── variant.rs
    └── view.rs
```

The initial format layer is private. The products that consume VCF/BCF can use
the standards-focused noodles crates directly and keep their own narrow
adapters. Another rsomics IO wrapper is not justified merely to wrap noodles.

### Historical source assets

| Asset and audited revision | Disposition | Target |
|---|---|---|
| `rsomics-vcf-allele-length` `d4e3b56d5132e4e6bb96faeddc1ee9992fe6ee53` | Refactor then merge | `stats allele-length` |
| `rsomics-vcf-annotate` `c958d89eeb5ff8ec0ce343ded3ab9ddfe10e957a` | Test and algorithm seed | Later typed `annotate` |
| `rsomics-vcf-concat` `15088a2e6cbaef6bfb49669e9625e50b6ace7e50` | Refactor then merge | `concat`; replace VCF-text-only plumbing |
| `rsomics-vcf-consensus` `bb016cf71d28ffa12562e875e0c5db7a431d148c` | Refactor then merge | Later `consensus` |
| `rsomics-vcf-convert` `0322987e3f53b2d4099bede46d6b5df3f4f5efe0` | Test and conversion seed | Later complete `convert` profiles |
| `rsomics-vcf-extract` `3bca5d5a6d2dbec187a00a29620c8c04b2fabe0d` | Selection and fixture merge complete | First-slice `query` |
| `rsomics-vcf-fill-tags` `a28b803cebc218468fd53280f5166ea76198f03a` | Refactor then merge | Later `annotate fill-tags`; update 1.24 rounding and groups |
| `rsomics-vcf-filter` `93d91c114d2ce0fc31a6b1c7176280f558c06f3c` | Fixture and expression-integration seed merged; whole-file implementation discarded | Complete typed `filter` |
| `rsomics-vcf-filter-summary` `f8323af72303498bcc59f16c4a9feb897b992d3f` | Merge report fixture | `stats filters` |
| `rsomics-vcf-fixref` `d6efd2bd79067b2b7b2f738703e428ca40dc56f1` | Refactor then merge | Later `fixref`; retain reference-access performance seed |
| `rsomics-vcf-head` `0297fa20cb271124c9ccc15d51fff973f1df50b6` | Refactor then merge | First-slice `head`; add BCF |
| `rsomics-vcf-indel-stats` `a7774e648149a7b12dbfbbb60870d54d1cf2a373` | Refactor then merge | `stats indels` |
| `rsomics-vcf-index` `5eafb949d64a101c1c4e2d21e9a311ad9379ac65` | Spanning-variant regression and linear-offset seed merged; implementation replaced | Complete first-slice TBI/CSI `index` |
| `rsomics-vcf-isec` `86bedb28892ccbcb6137bfb3c82925fe931609f1` | Test and merge-loop seed | Later `isec` |
| `rsomics-vcf-merge` `571af0688ac61b857b529b0db20ae886999e04fa` | Test asset | Replace incomplete header, allele, and FORMAT reconciliation |
| `rsomics-vcf-norm` `c4eeb5026199141a08ddd7b710be14488887edc2` | Test and split seed | Later complete `norm`; add reference realignment and 1.24 semantics |
| `rsomics-vcf-query` `1bd16a4562e931010d6138e71c3a6112040edd29` | Refactor merge complete; partial parser replaced | First-slice streaming `query` |
| `rsomics-vcf-reheader` `e25a2942b13b912fefc21e739d3f10876a59ac74` | Refactor then merge | Later transactional `reheader` |
| `rsomics-vcf-sample` `3217323c7e6a22f2086367f8bdf9cc8bde6abd88` | Sample-projection fixture merge complete; implementation replaced | First-slice `view --samples` |
| `rsomics-vcf-setgt` `a01b957b2259f4a75834c8354b2467cc3ea78cf6` | Refactor then merge | Later `setgt`; share internal expression engine |
| `rsomics-vcf-snp-density` `c8f2c9b1507712bcfb967693b18fc8d936f14465` | Merge legacy report fixture | `stats density` |
| `rsomics-vcf-sort` `2ba24aa3573557117fc47900892264f358bdf96d` | Test asset only | Replace whole-file in-memory sorter |
| `rsomics-vcf-split` `4b84ce255e2ccd1292d4caa49d6011bf7e30f8bc` | Refactor after dirty-diff attribution | Later transactional `split` |
| `rsomics-vcf-stats` `66299f4d56e26b1d4c1498ffba9b489cfb7d5f85` | Test asset after dirty-diff attribution | Later `stats`; current implementation is too narrow |
| `rsomics-vcf-to-bed` `50e87e5821cdc498e950f6e412d6b25fb016c944` | Merge coordinate fixtures | Later `convert bed` |
| `rsomics-vcf-tstv-strat` `f1697b722b7a1d99c6393a82ca924f69773e4c38` | Merge legacy report fixture | `stats tstv` |
| `rsomics-vcf-utils` `61287add0d662df97a4808ae05c473791b922ec4` | Split, refactor, and discard | Fold grounded operations into `view`, `query`, `stats`, and `convert`; discard duplicates |
| `rsomics-vcf-validate` `e6fef96f3cdfde5d5740d57cb8c5185cfc5285ff` | Test seed merged; implementation replaced | Strict first-slice `validate` |
| `rsomics-vcf-variant-distance` `b9a86dd089539bd9d3147acae72f3b19bfe8015a` | Refactor then merge | Later `annotate distance`; unsorted input fails |
| `rsomics-vcf-view` `d0c187ec2c85033f721ac135be874cf0aa48eb02` | Type and FILTER fixtures and predicate seed merged; whole-file implementation replaced | Complete first-slice `view` |

The four dirty repositories contain only untracked `Cargo.lock` files during
this audit. They are not copied until ownership is confirmed. The other 26
listed repositories are clean.

### Existing implementation gaps

Every routed source has tests, compatibility files, and benchmark targets, but
most do not establish release evidence. Many compatibility tests skip without
bcftools, pin 1.21 or 1.23.1, or compare only selected fields. Historical CI is
primarily Ubuntu `x86_64`.

The source structure repeats incompatible VCF line parsers:

- `view`, `filter`, `norm`, and `sort` read and inflate the entire file before
  processing;
- most operations accept VCF text and gzip but not BCF despite CLI
  descriptions that claim VCF/BCF;
- `view` treats a zero-difference REF/ALT as a SNP and lacks the 1.24 `ref`,
  `bnd`, and `overlap` type contract;
- `head` explicitly sends BCF users back to bcftools;
- `query` has a useful streaming renderer but a partial format language and a
  separately maintained header/value model;
- `norm` primarily splits multiallelic text records and does not supply the
  reference-backed normalization implied by the command;
- `sort` stores every record and its copied fields in memory;
- `validate` checks a small line-level subset and does not validate typed
  header/record cardinality or BCF layout.

Source comments repeatedly narrate audit history and upstream code. Migration
keeps only comments that explain stable format invariants, safety boundaries,
or non-obvious compatibility decisions.

### Internalized libraries

`rsomics-vcf-expr` at
`94722777e8e1851182cb4c5ccf0b3ae9127eca2f` has two historical dependents but
only one target-product consumer, `rsomics-vcf`. Its 1,296-line parser and
evaluator become the product's internal expression module. The public
expression contract is not frozen until current 1.24 multiallelic MAC/MAF,
site/sample aggregation, missing values, vectors, regex, indexing, and type
errors pass differential tests.

`rsomics-vcf-valfmt` at
`90c50e3713342c4925a45c1fbad33354efeb9e54` likewise has one target product.
Its htslib-compatible integer and float rendering becomes an internal
typed-value module. `head` and `query` are two commands of one product, not two
product consumers.

No public `rsomics-vcfio`, expression, or value-format crate is added.

### Compatibility and performance gates

The stable slice covers:

1. VCF 4.1 through 4.5 headers and records, BCF2, BGZF, TBI, and CSI;
2. `Number=A/R/G/.`, ploidy, phase, missing/vector-end values, duplicate and
   undefined tags, multiallelic records, breakends, symbolic alleles,
   spanning deletions, gVCF blocks, `INFO/END`, and IUPAC bases;
3. malformed header dictionaries, record cardinality, allele indices,
   coordinates, sample widths, BCF lengths, compressed streams, and indexes;
4. byte or typed-value differentials against exact bcftools/HTSlib 1.24 for
   every declared behavior;
5. native Linux and macOS exact-head CI on `x86_64` and `aarch64`.

The first-slice performance fixture contains many samples, multiallelic and
symbolic records, large INFO/FORMAT vectors, compressed and BCF inputs, and
real region queries. It records input/output digests, machine, versions,
worker count, compression, trial ordering, timing distribution, and peak RSS.
At least the principal `view` or `query` hot path must strictly beat bcftools
1.24 in throughput or resource use.

Historical results are promising but not current release claims:

- fill-tags reports 2.48 times bcftools 1.23.1 throughput on a 142 MB,
  200-sample VCF using eight Apple M2 cores;
- fixref reports 3.93 times throughput on 500,000 sites on the same machine;
- setGT states approximately 2.5 times throughput without a complete
  product-level provenance record;
- variant-distance asserts a win without recording numeric results in its
  README.

They require current 1.24 output, repeated timing, RSS, and exact source
revision evidence.

The implemented `head` path has a current local gate on an Apple M2
(`Mac14,3`), macOS 26.6, Rust 1.91.0, bcftools/HTSlib 1.24, and hyperfine
1.20.0. The fixture has 1,000,000 records and 33,889,047 bytes. Seven measured
runs after two warm-ups produced:

| Input | rsomics mean | bcftools mean | Decision |
|---|---:|---:|---|
| plain VCF | 191.8 ms | 474.2 ms | rsomics 2.47 times faster |
| BGZF VCF | 194.0 ms | 449.1 ms | rsomics 2.31 times faster |
| BCF | 1,378.0 ms | 203.5 ms | typed fallback is 6.77 times slower |

The plain fixture SHA-256 is
`e828855a7ba1975624a9123d0d9b6bb37debadec79f7d2ae1a57d935a7803130`;
the BGZF fixture is
`72ad19cdda99612f294936df37b4d93daa4111b780f394dee20161b0b956b991`;
the BCF fixture is
`e8c1a675d911aef115d4121cb50f98e56c73bb165f01c7d3861788d6dfe022c1`.
One plain-VCF resource run observed approximately 3.2 MiB peak RSS for rsomics
and 6.3 MiB for bcftools. The retained hyperfine JSON is on external scratch;
the repository benchmark independently reports approximately 174.9 MiB/s for
the 100,000-record VCF path.

The final `query` revision was measured separately on the same machine and
toolchain with `%CHROM\t%POS\t%INFO/DP\n`, two warm-ups, and seven measured
runs over 1,000,000 records:

| Input | rsomics mean | bcftools mean | Peak RSS, rsomics / bcftools | Decision |
|---|---:|---:|---:|---|
| plain VCF | 241.7 ± 5.6 ms | 450.5 ± 36.4 ms | 3.4 / 6.5 MiB | rsomics 1.86 times faster |
| BGZF VCF | 262.5 ± 3.9 ms | 442.2 ± 28.2 ms | 3.5 / 6.7 MiB | rsomics 1.68 times faster |
| BCF | 921.2 ± 21.8 ms | 209.0 ± 5.1 ms | 3.5 / 6.6 MiB | rsomics 4.41 times slower |

The query plain fixture has 31,889,099 bytes and SHA-256
`9873405b77dc5d3af91f4e33b65d1f0c66f41808933396fccb715500ff43f3fa`.
The BGZF and BCF inputs use the digests recorded above. The retained final
hyperfine files are `query-plain-final.json`, `query-bgzf-final.json`, and
`query-bcf-final.json` under the external performance scratch.

The plain VCF and BGZF principal query paths pass both throughput and memory
gates. Direct typed BCF numeric projection improves the original generic
fallback but does not pass the throughput gate. The remaining 4.41-times
deficit is explicit and belongs to the shared raw BCF/index work used by
`view`; it is not hidden behind the faster text result or claimed as a win.

The validation revision uses the same 1,000,000-record plain VCF query fixture,
31,889,099 bytes with SHA-256
`9873405b77dc5d3af91f4e33b65d1f0c66f41808933396fccb715500ff43f3fa`.
On the Apple M2 host above, two warm-ups and ten measured runs compared
`rsomics-vcf validate` with the official EBI vcf-validator 0.10.2 macOS arm64
binary using `-l error -r summary`:

| Validator | Mean ± standard deviation | Range | Peak RSS |
|---|---:|---:|---:|
| rsomics-vcf | 681.1 ± 121.5 ms | 592.5–983.4 ms | 2.50 MiB |
| EBI vcf-validator | 1,670.2 ± 167.5 ms | 1,570.0–2,119.0 ms | 9.61 MiB |

Rsomics is 2.45 times faster on this fixture and uses approximately one
quarter of the observed peak memory. The EBI binary SHA-256 is
`ccee11e55354f4cff0289efa33a24a4e5a59e7177d5ea1effea96288fd0d0811`.
The retained Hyperfine JSON is
`/Volumes/KIOXIA/Developments/tmp/rsomics-vcf-validate.VtRaBO/hyperfine-bounded.json`
with SHA-256
`23d194d6fb3151aac36ef446b6f632253a67f333b264a68da2cd668500af1594`.

The final `index` revision was measured on the same Apple M2 host and toolchain
against bcftools/HTSlib 1.24 with three warm-ups and 20 measured runs. Both
sides built default CSI indexes for the same one-million-record inputs:

| Input | rsomics mean | bcftools mean | Peak RSS, rsomics / bcftools | Decision |
|---|---:|---:|---:|---|
| BGZF VCF | 170.4 ± 32.5 ms | 136.9 ± 2.8 ms | 3.36 / 6.44 MiB | bcftools 1.25 times faster; rsomics uses 48% less RSS |
| BCF | 133.0 ± 17.9 ms | 77.2 ± 15.5 ms | 3.30 / 6.64 MiB | bcftools 1.72 times faster; rsomics uses 50% less RSS |

The BGZF VCF is 1,715,168 bytes with SHA-256
`72ad19cdda99612f294936df37b4d93daa4111b780f394dee20161b0b956b991`;
the BCF is 2,319,778 bytes with SHA-256
`e8c1a675d911aef115d4121cb50f98e56c73bb165f01c7d3861788d6dfe022c1`.
The wall deficit is explicit: rsomics validates the complete BCF record
structure and flushes, syncs, renames, and parent-syncs the index transaction,
whereas the comparison does not provide the same durable replacement
contract. The release claim is lower memory and stronger failure semantics,
not higher indexing throughput. The retained Hyperfine files are
`hyperfine-final-vcf.json` and `hyperfine-final-bcf.json` under the external
index benchmark scratch, with SHA-256
`c119210634775b0d278d1854a654eeaa56948af84d608c171d3e0a747704eb54`
and
`cb5f4fbc8645fb099455dc703ca2aa40f462dbed4d35c0ac7b992de8405a356f`.

The final `view` revision was measured on the same Apple M2 host and toolchain
against bcftools 1.24 with three warm-ups and 20 measured runs. Both commands
read the same one-million-record BGZF VCF, emit matching record bodies, and
write uncompressed VCF:

| Viewer | Mean ± standard deviation | Peak RSS |
|---|---:|---:|
| rsomics-vcf | 200.7 ± 9.2 ms | 3.14 MiB |
| bcftools | 459.2 ± 12.8 ms | 6.62 MiB |

Rsomics is 2.29 times faster and uses approximately 53% less observed peak
memory. The 1,715,168-byte input has SHA-256
`72ad19cdda99612f294936df37b4d93daa4111b780f394dee20161b0b956b991`;
both output bodies have SHA-256
`e18ba5e1a82390b96b878196158b005a7258f200e0b8870a093aa71a18d00890`.
The retained final Hyperfine JSON has SHA-256
`05c367276fee4c032a3d7a2c080c72f79e4b17d658d02bde5aeeb3fab9a4898f`.

### Publication decision

`rsomics-vcf` 0.1.0 published only the complete first-release slice. Final
exact-head CI run `30633237582` passed at
`bbc09be7ed3873d9069e9bde88fa0119b134129a`; publish run `30633450270`
completed successfully. The crates.io archive has SHA-256
`abc40a340f2e1c3814dd1c33085b54865ee7e8a6fc58b4814affa2e5e74f9731`,
is not yanked, and reports Rust 1.91 with the MIT OR Apache-2.0 license.
A fresh `cargo install rsomics-vcf --version 0.1.0 --locked` reported version
0.1.0, matched the bcftools record-body fixture, and the downloaded archive
passed the complete normal test suite.

`rsomics-vcf` 0.2.0 publishes the completed typed-filtering slice at revision
`36d400df74666fb4c8b3bc16fb3cd74d1d56be71`. Exact-head CI run
`31554294004` and publish run `31554518679` completed successfully, and a
fresh registry fetch resolved and downloaded `rsomics-vcf` 0.2.0 from
crates.io. Remaining later-slice commands stay absent rather than shipping
placeholders.

`rsomics-vcf` 0.3.0 publishes the completed allele-normalization slice at
revision `8051f3bfaff9b04958fd0f6a6264582af4acd6b3`. Exact-head CI run
`31574424729` passed all four native target classes, including package and
bcftools 1.24 oracle verification on Linux `x86_64`; publish run `31574785496`
completed successfully from the same revision. The non-yanked crates.io
archive has SHA-256
`fcd6ca3f83cebd2bd7695a8fb4fac39cfe22a49c60410c140ed65682a886d70c`.
A fresh registry install downloaded and compiled 0.3.0 on external storage,
reported version 0.3.0, exposed the unified `norm` help, and split the formal
200,000-record typed fixture into 400,000 records with body SHA-256
`03b3ab4553d2289049bf9fb92566b97b43a55d151a5e70763c29cbb9a6123c56`,
matching the pre-publication oracle.

## `rsomics-call`

### Boundary

One lightweight small-variant calling product owns alignment pileup, genotype
likelihood generation, and Bayesian site calling. The public workflow is:

```text
rsomics-call pileup
rsomics-call call
rsomics-call run
```

`pileup` emits a standards-compliant VCF/BCF likelihood stream. `call`
consumes compatible likelihood records. `run` fuses both stages without a
materialized intermediate while preserving equivalent output and provenance.
The product does not own generic VCF editing or BAM inspection.

### Upstream operation map

The behavior oracle is bcftools 1.24. Its two relevant upstream commands form
one workflow but expose distinct contracts:

| Stage | User-recognizable behavior retained in `rsomics-call` |
|---|---|
| `bcftools mpileup` | one or more SAM/BAM/CRAM inputs; read-group-to-sample mapping; flag, mapping-quality, base-quality, overlap, and depth policy; BAQ; indexed regions versus streaming targets; SNP and indel genotype likelihoods; per-sample and site annotations; VCF/BCF likelihood output |
| `bcftools call` | consensus and multiallelic models; sample selection and ploidy; reference, variant-only, and gVCF-compatible output; regions and targets; likelihood, genotype, and bias annotations; VCF/BCF call output |
| composed pipeline | header and sample propagation, deterministic stage equivalence, failure propagation, and no hidden temporary file |

`pileup` is the clearer product verb; the upstream `mpileup` spelling is not a
separate public operation. `run` is an rsomics workflow operation justified by
avoiding serialization and compression between the two stages.

### First release slice

All three commands ship together. The stable `pileup` contract includes:

- one or more SAM, BAM, or CRAM inputs, plus an alignment-list file;
- reference-backed and explicitly reference-free modes;
- read-group sample discovery, explicit sample selection, and one-input-one-
  sample behavior when read groups are ignored;
- the four distinct all/any set/unset FLAG predicates, anomalous-pair policy,
  overlap adjustment, mapping/base-quality thresholds and caps, and bounded
  per-input depth with an explicit sampling seed;
- default partial BAQ, full BAQ, forced BAQ recalculation, and disabled BAQ;
- SNP and established indel likelihoods with explicit gap, pooled or per-sample
  support, fraction, and ambiguous-read allele-depth policy;
- indexed regions and streaming targets, each accepting inline and file forms;
- the bcftools 1.24 default annotation set plus `FORMAT/DP`, `FORMAT/ADF`,
  `FORMAT/ADR`, `FORMAT/QM`, `FORMAT/QS`, `FORMAT/SP`, `FORMAT/SCR`,
  `INFO/AD`, `INFO/ADF`, `INFO/ADR`, `INFO/FS`, `INFO/NMBZ`, `INFO/NM`, and
  `INFO/SCR`;
- uncompressed or BGZF VCF and uncompressed or compressed BCF.

The stable `call` contract accepts rsomics or bcftools-compatible likelihood
VCF/BCF and includes:

- explicitly selected consensus or multiallelic calling models;
- sample selection, diploid or haploid defaults, GRCh37/GRCh38 sex-aware
  presets, and checked custom ploidy files;
- mutation priors, prior allele-frequency tags, alternate-allele retention,
  masked-reference policy, SNP/indel skipping, and variant-only output;
- reference records and threshold-grouped gVCF blocks;
- indexed regions and streaming targets;
- `FORMAT/GQ`, `FORMAT/GP`, and `INFO/PV4` where supported by the selected
  model;
- the same four output encodings and transactional named output behavior.

`run` accepts the union of these stage options, streams a typed likelihood
site directly from pileup into the selected caller, and must be record-
equivalent to `pileup | call` after normalizing provenance header lines.

A single-sample SNP-only command is not a publishable slice.

### Internal boundary

The serialized likelihood record and the fused path share one typed model:

```text
alignment record
  -> validated pileup column
  -> allele candidates and per-sample likelihood evidence
  -> LikelihoodSite
  -> consensus or multiallelic caller
  -> called variant record
```

`LikelihoodSite` owns contig identity, zero-based position, reference and
alternate alleles, per-sample genotype likelihoods and depth evidence, and
site annotations. `pileup` serializes it to VCF/BCF; `call` reconstructs it
from compatible VCF/BCF; `run` passes it directly. No command parses VCF as
tab-separated text.

The target modules are:

| Module | Responsibility |
|---|---|
| `cli` | one Clap tree parsed and styled through `rsomics-help` |
| `alignment` | input opening, headers, samples/read groups, references, regions, and targets |
| `pileup` | BAQ, candidate alleles, SNP/indel likelihoods, annotations, depth sampling, and `LikelihoodSite` production |
| `calling` | ploidy, priors, consensus model, multiallelic model, genotype fields, and gVCF blocking |
| `format` | typed likelihood VCF/BCF input and transactional VCF/BCF output |
| `run` | bounded stage composition without an intermediate file |

The product may expose a library for these typed stages, but it exposes one
binary. Product policy stays out of the Layer A foundations.

### Explicit first-release exclusions

The first release does not advertise:

- the experimental bcftools 1.24 `--indels-2.0` or `--indels-cns` models and
  platform profiles that require them;
- trio constraints, target-allele constraints, or insertion of sites omitted
  by pileup;
- output auto-indexing;
- deprecated Illumina 1.3 quality recoding or deprecated caller aliases;
- bcftools 1.24 `--platforms`, which stores and frees its argument but never
  consumes it in the mpileup or indel path;
- plugin behavior, somatic calling, structural variants, copy-number calling,
  annotation, or generic VCF transformations.

These exclusions keep unstable or separate workflows out of the initial
contract without reducing it to a toy caller.

### Historical assets

| Asset and audited revision | Disposition |
|---|---|
| `rsomics-vcf-mpileup` `e4e7ed173492299c2679fb10b0d986929e89f1bd` | Error-model and output fixture seed; refactor after the shared pileup boundary |
| `rsomics-vcf-call` `a978d132ccbd79b957330da7f1fd6f7b8dbf0dc5` | Consensus-model and VCF likelihood fixture seed; replace the partial caller shell |

Both repositories are clean. The pileup asset explicitly lacks multisample,
indel, multiallelic, BAQ, ploidy, indexed region, reference-only, and most
annotation behavior. It targets bcftools 1.21. The call asset implements only
the older consensus diploid model over PL records.

### Foundations and gates

`rsomics-bamio 0.3.0` established validated SAM/BAM/CRAM streams, indexed access,
and the shared fallible raw-record encoder used by both BAM and call. Revision
`94641eff97d7` moves appended and alternative BAI/CSI/CRAI discovery plus
indexed-reference setup into the foundation after consumer-side BAM and call
tests established the same policy-free contract. Exact-head four-native-target
CI `30658611800` and publish run `30658840221` pass. The downloaded archive
checksum is
`6ac17eb096cd976f6000ff813430236df0b723eb360c926427d7928e46702a93`.
Revision `d563f0160c2a` aligns the foundation on `rsomics-common` 0.10;
version 0.4.0 passes exact-head four-native-target CI `30714717087`, publish
run `30714794220`, and downloaded-archive verification with checksum
`7dbfdde57d3f0553f962ab1836ff06ce59540637b6a501757690cdf66c85876b`
without changing the consumer contract.
Revision `82a8668717b5` caches the validated sequence and quality layout in the
raw-record value rather than rechecking the BAM variable-length sections on
every hot-path accessor. Call, BAM, and pileup consumer suites pass against
the path-patched candidate. Version 0.4.1 passes exact-head four-native-target
CI `30721193395`, publish run `30721286006`, and downloaded-archive
verification with checksum
`1ff830a8263e4a5c8784101c3d6674e4bd86ec8ced446327d94d35731db13600`.

`rsomics-pileup` revision `2680f6c328be` supplies a fallible sorted projection
kernel, checked CIGAR and long-CIGAR projection, overlap handling, retry-safe
borrowed columns, bounded column state, source-isolated overlap state, an
optional per-source active-depth limit, and standard or extended BAQ. Column
preparation supports full BAQ and the bcftools 1.24 partial trigger after
current active records have entered overlap handling. The call product owns
the 250-read and 500-base defaults instead of freezing those policies into the
foundation. Exact-head four-native-target CI `30654312487` passes.
`rsomics-call` and `rsomics-bam mpileup` provide the two implemented consumers.
Revision `a69743a8097f` aligns the public raw-record type with bamio 0.3 without
changing the projection hot path. Its samtools 1.24 oracle, ordinary and 250×
benchmarks, exact-head four-native-target CI `30659084469`, and publish run
`30659248849` pass. The resulting pileup 0.3.0 archive checksum is
`def4cc70d0cd250f8b9ebb1d1e0280c1a890cffb66504a832cb1819cff9f8581`.
Revision `4b48bfdafecd` aligns the dependency on bamio 0.4 without changing the
hot path. Version 0.4.0 passes exact-head four-native-target CI `30714930834`,
publish run `30715042037`, and downloaded-archive verification with checksum
`d33b5fa1c3ddbe86c8f53c2bb3fa870e482e90957aa5e559fceca0600ff56533`.

`rsomics-call` revision `e53ede5a0777` owns the typed allele, ploidy,
likelihood-site, called-site, and per-sample evidence models. It validates and
coordinate-merges plain or BGZF SAM, raw or BGZF BAM, and CRAM inputs, checks
their reference dictionaries, builds samples from source and read-group
metadata, and streams records through `rsomics-pileup` into multisample SNP
likelihood sites. Its MAQ error model and deterministic deep-evidence sampling
match HTSlib 1.24; reference-only, two-sample, and per-input-depth records match
bcftools 1.24. Its multiallelic caller matches bcftools 1.24 for reference,
biallelic, triallelic, alternate-only, haploid, diploid, mixed/absent-ploidy,
and independently grouped samples while keeping callable and emitted alleles
distinct. Its consensus caller ports the bcftools 1.24 allele-frequency
posterior and matches reference, heterozygous, alternate, triallelic, and
haploid calls, including optional genotype quality. The fused path passes typed
likelihood sites directly into either caller and is record-equivalent to its
materialized counterpart. A strict schema decodes likelihood records and
encodes called records, while content-detected streaming supports plain VCF,
BGZF VCF, raw BCF, and BGZF BCF with record context on input or call failures
and explicit output finalization. The format oracle includes a real bcftools
1.24 multiallelic likelihood record and matching call. Revision
`b009e95faf1f` adds the established bcftools 1.24 indel model with
sample-specific reference consensus, glocal realignment, STR penalties, and
typed annotations; exact-head four-native-target CI `30657419390` passes.
Revision `5eaf1c5fa88b` adds indexed single-region likelihoods, k-way merges
records across inputs, and clips emitted SNP and indel sites to the requested
interval. Its typed output matches a live bcftools 1.24 multi-input region
oracle, all 62 debug and release tests pass, and exact-head four-native-target
CI `30659762334` passes. Revision `4758ced5a863` accepts normalized
multi-region selections: it sorts by alignment-header order, merges
overlapping or adjacent intervals, removes duplicates, resets pileup state
between disjoint intervals, and rejects an empty selection. Region clipping
now precedes BAQ and likelihood calculation so excluded columns do not advance
the deterministic sampling stream. The live typed oracle matches bcftools 1.24
for two disjoint normalized regions over two indexed BAMs; all 65 debug and
release tests, strict Clippy, rustdoc, and package verification pass locally.
Exact-head CI `30660633513` passes all four native targets. Revision
`edd1d7ac5b5c` adds normalized streaming inclusion targets without requiring
an alignment index and intersects them with indexed regions when both are
present. Unknown references and intervals beyond the reference yield no
selected sites, overlapping or adjacent targets merge, an empty target set
yields no output, and alignment-header order is preserved. A no-index SAM
oracle and a combined region-plus-target oracle both match bcftools 1.24 typed
likelihood records. All 66 debug and release tests, three live oracle groups,
strict Clippy, rustdoc, and package verification pass locally; exact-head CI
`30661640916` passes all four native targets.

Revision `e53ede5a0777` adds target files with bcftools-compatible suffix
semantics: `.bed`, `.bed.gz`, and `.bed.bgz` use zero-based half-open
coordinates; ordinary tabular files use one-based inclusive positions or
intervals; `.vcf` and `.vcf.gz` select POS. Compression is detected from
content and accepts plain, gzip, or BGZF input. Invalid coordinates, malformed
lines, unreadable input, and truncated compression fail with path and line
context. The live no-index SAM oracle matches bcftools 1.24 for BED, tabular,
and VCF target files. All 69 debug and release tests, three live oracle groups,
strict Clippy, rustdoc, and package verification pass locally; exact-head CI
`30662152861` passes all four native targets.

Revision `65ca410b4159` completes the stable pileup annotation surface without
introducing another foundation. Per-sample `DP`, `ADF`, `ADR`, `QM`, `QS`,
`SP`, and `SCR`, and site-level `DP`, `AD`, `ADF`, `ADR`, `FS`, `NMBZ`, `NM`,
`SCR`, `VDB`, `RPBZ`, `MQBZ`, `BQBZ`, `MQSBZ`, `SCBZ`, `SGB`, `MQ0F`, and
`I16` use typed evidence models and bcftools/HTSlib 1.24 numerical semantics.
SNP and indel collection preserve their distinct upstream support and
histogram rules, and allele trimming keeps all allele-indexed annotations in
lockstep. Four live BCF oracle groups cover rich SNP evidence, indels,
regions, and target files. All 72 debug and release tests, strict Clippy,
rustdoc, and package verification pass locally; exact-head CI `30664419838`
passes all four native targets.

Revision `754616c72967` carries those annotations through both callers and
removes the internal `I16` field from called records. A dedicated typed caller
annotation model derives `DP4`, model-specific `MQ`, and `PV4` with the
bcftools/HTSlib 1.24 Fisher, one-sided t-test, and incomplete-beta semantics.
Allele trimming preserves the complete INFO and FORMAT annotation contract.
All 74 debug and release tests, five live bcftools 1.24 oracle groups, strict
Clippy, rustdoc, and package verification pass locally; exact-head CI
`30704136238` passes all four native targets.

Revision `a59681a4417e` adds product-local typed ploidy resolution for constant,
GRCh37, GRCh38, and checked custom definitions. It binds samples to declared
sexes or fixed absent, haploid, and diploid states, computes the cohort maximum
used by caller priors, validates sample counts against likelihood sites, and
uses borrowed two-level interval lookup plus a reusable result buffer on the
per-site path. Ambiguous same-sex overlaps and malformed coordinates fail with
file and line context. The GRCh37 preset and an equivalent custom file match
live bcftools 1.24 calls across PAR boundaries, X, Y, MT, and `chr` aliases.
All 79 debug and release tests, six live oracle groups, strict Clippy, rustdoc,
and package verification pass locally; exact-head CI `30704669406` passes all
four native targets.

Revision `a8b6edcd1d22` adds typed gVCF reference blocking for the multiallelic
caller. It groups consecutive reference calls by the minimum per-sample depth
bucket, retains per-sample minimum depth, applies the bcftools 1.24 diploid PL
pair-selection rule, and flushes on bucket, contig, gap, non-reference, and
duplicate-coordinate SNP/indel boundaries. Collapsed records retain the first
site's alleles and genotypes, emit `END` only for multi-site spans, and limit
their fields to `MIN_DP`, `GT`, optional `PL`, and `DP`; below-threshold
reference calls remain ordinary records with `MIN_DP`. Invalid ordering,
sample dimensions, thresholds, and present non-diploid PL vectors fail
explicitly. The four-site live oracle is record-identical to bcftools 1.24
across two depth buckets. All 86 debug and release tests, seven live oracle
groups, strict Clippy, rustdoc, and package verification pass locally;
exact-head CI `30705253547` passes all four native targets.

Revision `6315fc8db7d2` projects likelihood samples during typed record decode.
Explicit inclusion preserves requested order, exclusion preserves input-header
order, and the projected schema drives the called-record header. Unknown and
duplicate names, empty results, and selection after record reading fail
explicitly. Both reordered inclusion and exclusion match bcftools 1.24 sample
headers, genotypes, and depths. All 88 debug and release tests, eight live
oracle groups, strict Clippy, rustdoc, and package verification pass locally;
exact-head CI `30705856695` passes all four native targets.

Revision `3ef9def7ad90` adds the bounded likelihood-calling workflow used by
both the serialized `call` path and the future fused `run` path. It validates
the projected sample count and gVCF depth schema before opening the output,
resolves per-site ploidy into a reused buffer, selects the consensus or
multiallelic output schema, streams records through the caller and optional
gVCF blocker, and preserves record context for call failures. Reordered sample
projection, diploid ploidy, multiallelic calling, and two depth buckets are
record-identical to one live bcftools 1.24 `call` workflow. All 91 debug and
release tests, nine live oracle groups, strict Clippy, rustdoc, and package
verification pass locally; exact-head CI `30706383364` passes all four native
targets.

Revision `3d0c4c107980` adds typed call-sample file binding without expanding
a foundation. Inclusion preserves file order and binds each selected sample
to an explicit sex, fixed zero-copy, haploid, or diploid state, or the ploidy
definition's default sex. Exclusion preserves input-header order and applies
the definition default to the retained samples. Comments and blank lines are
ignored; malformed rows, duplicate names, missing input samples, undeclared
sexes, empty selections, and I/O failures are errors rather than warnings.
An explicit-sex file with reordered haploid and diploid samples matches the
bcftools 1.24 header, genotypes, and depths. All 95 debug and release tests,
ten live oracle groups, strict Clippy, rustdoc, and package verification pass
locally; exact-head CI `30707009273` passes all four native targets.

Revision `dcb4f5ee7fec` extracts the product-private likelihood schema and
sample projection shared by sequential and indexed readers without changing
the public record contract. All 95 debug and release tests and ten live
bcftools 1.24 oracle groups pass; exact-head CI `30707694316` passes all four
native targets.

Revision `c23140b7938d` adds true indexed likelihood-region calling rather than
a filtered sequential scan. BGZF VCF or BCF inputs are opened with their TBI
or CSI index; inline regions are validated before output, ordered by input
header, merged when overlapping or adjacent, and deduplicated when a spanning
record intersects multiple disjoint queries. Sample projection and ploidy
binding use the same typed schema as sequential calling. A spanning-record
oracle matches `bcftools call -r` 1.24. All 98 debug and release tests and 11
live oracle groups pass; exact-head CI `30708181137` passes all four native
targets.

Revision `f864788cf1cc` adds `-R`-equivalent region files and consolidates the
previous target-only file parser into one product-private region-file module.
Tabular region files require consistently two-column one-based positions or
three-column one-based inclusive intervals; BED uses zero-based half-open
coordinates; VCF region files use POS. Plain, gzip, and BGZF content is
accepted with path and line context on failure. Region-file chromosome order
follows first appearance in the file, coordinates are ordered within each
chromosome, and repeated index hits are removed. Live bcftools 1.24 oracles
cover tabular, BED, and VCF files, including cross-chromosome order, overlap,
and the VCF POS-only rule. All 100 debug and release tests and 12 live oracle
groups pass; exact-head CI `30708699415` passes all four native targets.

Revision `ba7415dadbec` adds a typed call-output policy for masked-reference,
variant-only, and SNP/indel skip behavior. A live bcftools 1.24 oracle verifies
the default masked-reference omission, `-M`, `-v`, and both `-V` categories,
including the non-obvious rule that `-V snps` removes a SNP likelihood record
even when its final call is reference. All 101 debug and release tests and 13
live oracle groups pass; exact-head CI `30709355039` passes all four native
targets.

Revision `2a5df4b2ebce` composes independently grouped samples through the
multiallelic workflow rather than leaving grouping as a low-level caller-only
function. Group dimensions and contiguous membership are validated before
output, and consensus-model misuse fails explicitly. A live
`bcftools call -G` 1.24 oracle with `FORMAT/QS` matches reordered genotype and
depth output. All 102 debug and release tests and 14 live oracle groups pass;
exact-head CI `30709683149` passes all four native targets.

Revision `293488fcd66d` adds multiallelic alternate retention without leaving
the record dimensions inconsistent. Unused ALT alleles and their PL, AD, QS,
and AC entries remain aligned after allele selection. Live bcftools 1.24
oracles cover ordinary unused alternates as well as the distinct symbolic
allele behavior: `<*>` is still removed, while `<NON_REF>` is retained. All
103 debug and release tests and 15 live oracle groups pass; exact-head CI
`30710165666` passes all four native targets.

Revision `04a715938579` applies configurable integer panel allele counts to the
multiallelic frequency estimate used for allele selection, including
independently grouped samples. The reader requires checked INFO Number=1/A
integer definitions, rejects inconsistent counts, and carries the selected
prior coordinate system through standard and custom tag names. A live
bcftools 1.24 oracle matches selected alleles, genotypes, likelihood fields,
and QUAL. When the caller removes an ALT, rsomics also projects the panel
counts and total to the retained allele set; bcftools 1.24 instead leaves a
custom Number=A vector at its old length. All 105 debug and release tests and
16 live oracle groups pass; exact-head CI `30710931710` passes all four native
targets.

Revision `d3a0609d9356` adds explicit reference-free likelihood runs for both
sequential and indexed-region alignment input. The mode emits `N` as REF,
retains the observed SNP allele and likelihood dimensions matched by
`bcftools mpileup --no-reference`, and rejects BAQ or indel configuration at
the builder boundary. This also fixes the previously unreachable `N`
reference path in SNP allele selection. All 106 debug and release tests and 17
live oracle groups pass; exact-head CI `30711321842` passes all four native
targets.

Revision `8d5b5c2d8610` composes the complete likelihood and call workflow
without serializing the intermediate records. It builds the same checked
likelihood schema and reuses the same ploidy resolver, sample groups, caller,
site-output policy, gVCF blocker, and called-record writer as the materialized
path. A combined grouped-sample, diploid, variant-only, and gVCF test is
byte-identical to writing and reading the intermediate likelihood VCF. All 107
debug and release tests and 17 live oracle groups pass; exact-head CI
`30711752880` passes all four native targets.

Revision `9eaeec978b4d` completes the established indel candidate and ambiguous
read policy. Candidate support can be evaluated across the cohort or per
sample. Ambiguous low-quality reference matches can be dropped, distributed
over observed forward and reverse allele depths, or assigned to reference
depth without changing genotype likelihoods. The latter path also matches the
bcftools VCF integer ceiling when compensated depth has no quality-error mass.
All 109 debug and release tests and 19 live oracle groups pass; exact-head CI
`30712522200` passes all four native targets. The pinned bcftools 1.24 source
at `fb9f0f783e0f` confirms that `--platforms` has no behavioral consumer, so
the first release does not expose a no-op flag.

Revision `c942e00884b7` adds indexed alignment region-file likelihood runs with
and without a reference. Reference sequences follow their first appearance in
the file, intervals within each sequence are sorted and merged, and repeated
or overlapping selections emit each site once. The complete typed output for a
cross-sequence file matches bcftools 1.24 `mpileup -R`. All 110 debug and
release tests and 20 live oracle groups pass; exact-head CI `30712873521`
passes all four native targets.

Revision `dd1bbe5a7cf7` exposes the verified typed workflows through one
`rsomics-help` command tree with `pileup`, `call`, and fused `run` commands.
It binds positional and list-file alignment inputs, sample projection,
reference-backed and reference-free modes, region and target selection,
complete pileup and calling policy, custom ploidy and sample groups, gVCF,
prior-frequency tags, all four VCF/BCF encodings, and
`rsomics-common::write_output` for standard output or transactional named
output. Fused output is byte-identical to the materialized two-stage path.
All 117 debug and release library tests, seven command integration tests, and
21 live bcftools 1.24 oracle groups pass. Strict Clippy, rustdoc, locked
packaging, and exact-head four-native-target CI `30714255852` also pass.

Revision `e8394bf82f5a` aligns the complete product on `rsomics-common` 0.10,
`rsomics-bamio` 0.4, and `rsomics-pileup` 0.4. `cargo tree -d` is empty; the
same 117 library tests, seven command integration tests, and 21 live oracle
groups pass in debug and release mode. Exact-head four-native-target CI
`30715205443` passes.

The current region-file reader materializes and normalizes the requested
regions before querying. A source review corrected the earlier claim that
bcftools 1.24 `mpileup -R` streams a tabix-indexed region file: that path also
loads its region selections before alignment processing. Region-file
streaming is therefore neither an upstream compatibility requirement nor an
independent release blocker.

Two bcftools 1.24 sample-file behaviors remain contradictory. Its man
page says a missing second column assumes sex `F`, while `vcfcall.c` assigns
fixed ploidy 2; the installed binary also emitted a diploid genotype for a
documented numeric ploidy-1 row. The first command contract therefore accepts
only the unambiguous one-column sample-selection form. Its typed binder rejects
unknown and duplicate samples instead of reproducing the upstream
warning-and-continue path. Numeric sample-file ploidy remains excluded.

Call-stage targets expose another upstream contradiction. The bcftools 1.24
manual presents `-t` and `-T` as streaming region or interval selectors, but
the installed `call -t chr1:2-3` reports that it cannot open the argument,
exits successfully, and emits every input site. With `-T`, `vcfcall.c` reads
only CHROM and the first one-based position; a `chr1 2 3` row selects position
2 rather than the documented interval. An interval-correct reader therefore
cannot match that binary. No call-target API is committed until the desired
documented-versus-binary behavior is chosen.

Target exclusion remains deliberately absent. The bcftools 1.24 manual defines
`^targets` as the logical complement, but `mpileup.c` filters out records with
no target overlap before it inverts the site-level predicate. The installed
1.24 binary consequently emits no records for complement selections that
should retain reads wholly outside the excluded interval. This contradicts
the documented contract, so neither that defect nor a corrected behavior is
being frozen into the public interface without an explicit compatibility
decision. Target exclusion and sample-default compatibility remain unresolved.

Unseen-allele handling is likewise not a frozen option. The bcftools 1.24 help
describes `--keep-unseen-allele` as retaining `<*>` or `<NON_REF>`, but the
installed binary still removes `<*>` with `-A --keep-unseen-allele` and retains
`<NON_REF>` with `-A` even when the flag is absent. The verified `-A` behavior
is implemented; a separate unseen-allele switch would currently be either a
no-op matching the binary or a correction matching the help text.

The first-release contract was re-audited against the public Rust API before
and after implementing the command tree. The resulting ledger is:

| Contract area | Current evidence | Deferred or remaining release work |
|---|---|---|
| alignment inputs and samples | SAM/BAM/CRAM, alignment-list files, read-group discovery, explicit sample projection, and one-input-one-sample mode are implemented and exposed | none for the first-release contract |
| pileup record policy | all four FLAG predicates, mapping-quality filtering, anomalous-pair policy, overlap adjustment, per-source depth, quality bounds, deterministic sampling, pooled/per-sample indel support, and ambiguous-read depth policy are typed, exposed, and verified as one command configuration | none for the first-release contract |
| reference and likelihood generation | reference-backed SNP, BAQ, and indel paths plus explicit reference-free SNP likelihoods are implemented, exposed, and verified; reference-free configuration rejects BAQ and indels | none for the first-release contract |
| pileup selections | indexed inline/file regions and streaming inline/file targets are implemented and verified | target complement remains blocked on the documented-versus-binary decision below |
| likelihood output | the four VCF/BCF encodings, complete annotation schema, standard output, and transactional named output are implemented and exposed | none for the first-release contract |
| call models and ploidy | consensus, multiallelic, mutation prior, sample projection, fixed/preset/custom ploidy, sample groups, and gVCF blocking are implemented and exposed | numeric sample-file ploidy is excluded because the manual, source, and installed binary disagree |
| call allele and site policy | callers trim selected alleles and preserve typed annotations; masked-reference, variant-only, SNP/indel skip, grouped-sample, keep-alternates, and prior-frequency workflows are implemented and checked against bcftools 1.24 | unseen-allele handling remains blocked on the help-versus-binary decision below |
| call selections | indexed inline and file regions are implemented and verified | streaming targets remain blocked on the documented-versus-binary decision below |
| fused workflow | the complete selected call policy, ploidy, sample groups, gVCF, and output schema run without serialization and are byte-equivalent to the materialized pipeline | none for the first-release contract |
| product UX | one `rsomics-help` 0.4 command tree exposes `pileup`, `call`, and `run`; `rsomics-common` provides the adopted diagnostic, exit-code, JSON, standard-output, and atomic-output layers | none for the first-release contract |

Revision `85579cb94f9a` narrows the hot likelihood path with inline small-vector
state, generated checked model constructors, and one typed annotation
observation. It consumes `rsomics-bamio` 0.4.1. All 117 release library tests,
seven ordinary CLI tests, and 21 live bcftools 1.24 oracle groups pass; exact-
head four-native-target CI `30721413157` passes. The final API and production
hot-path review found no new shared public component: allele selection,
calling policy, likelihood schema, annotations, ploidy, and output remain
product-local.

The representative Linux `x86_64` gate uses a deterministic 5 Mb, 30x wgsim
fixture, pins both tools to one CPU, performs one warm-up, and alternates five
timed rounds. Every round emits the same 5,024 sorted
`CHROM/POS/REF/ALT/GT` calls. `rsomics-call run` has a 25.64 s median and
22,400 KiB peak RSS; bcftools/HTSlib 1.24 `mpileup | call` has a 26.05 s
median and 47,488 KiB peak RSS. This is a 1.6% median wall-time advantage and
52.8% lower observed peak RSS on the declared fixture. Raw rounds, input and
binary checksums, commands, machine provenance, limitations, and the fail-on-
mismatch comparison script are tracked in the product's `PERFORMANCE.md` and
`benchmarks/call-vs-bcftools.sh` at revision `74bd99fee96d`.

Revision `8f29a887dc96` moves the CIGAR-derived per-record state retained across
pileup columns onto `rsomics-pileup` 0.9's generic record-state contract. The
product still owns its variant-calling policy and cached `CigarMetrics`; the
foundation only retains the consumer-supplied state. It also aligns on
`rsomics-bamio` 0.8 and `rsomics-common` 0.12. All 118 library tests, seven
ordinary command tests, and 21 live bcftools 1.24 oracle groups pass. Exact-
head four-native-target CI `31473004262` passes.

The migration regression gate at documentation revision `018dad978474` reuses
the same deterministic Linux `x86_64` 5 Mb, 30x fixture and compares the
published 0.1.0 head with revision `8f29a887dc96`. Both revisions emit the
same 5,024 normalized calls and use 22,400 KiB peak RSS. Across five measured
runs, the candidate has a 34.08 s median versus 50.64 s for the baseline; the
baseline measurements were noisy under shared-server load, so this is evidence
of no regression rather than a replacement throughput claim. Raw hyperfine,
RSS, environment, command, and checksum evidence remains on external storage.
Exact-head four-native-target CI `31473510015` passes.

Calling likelihoods, allele selection, ploidy policy, priors, annotations, and
VCF output remain in the product. `rsomics-stats` receives a numerical kernel
only if another product demonstrates the same contract.

Compatibility uses pinned bcftools 1.24 `mpileup`, `call`, and their composed
pipeline, plus adversarial BAM/CRAM and reference fixtures. The audited
bcftools 1.24 source archive has SHA-256
`8caddc22610ee2851666047c859bb91da0c1e32d0c2ec553db6f153ad130e46f`.
Performance compares the fused `run` workflow with the equivalent composed
upstream pipeline. The result above establishes both a strict throughput
advantage and a material memory advantage without changing the normalized
calls.

### Publication decision

`rsomics-call` 0.1.0 publishes the complete `pileup`, `call`, and `run` slice
and leaves the contradictory target-complement, call-target, numeric sample-
ploidy, and unseen-allele options absent. Release head
`b34cc226242ba2211d4b8135f50b8b5adc231482` passes exact-head four-native-
target CI `30722248488`; publish run `30722470067` completed successfully. The
first dispatch `30722408098` stopped before upload because the org secret did
not yet grant this repository access; adding `rsomics-call` to that selected-
repository scope resolved it without reading the token. The crates.io archive
is not yanked and has checksum
`83e2750f2b73b477da315d76f619db11c74e369528851a5108a69f4dd52bbde5`,
identical to the local exact-head package.

`rsomics-call` 0.1.1 publishes the record-state migration at release head
`7d20d6b119dc`. Exact-head four-native-target CI `31474757244` and publish run
`31475365157` pass. The crates.io version is not yanked, records the same Git
head, and its 135,087-byte archive has SHA-256
`e22ccb74c60e4e5b9cd5de6b97285700e6686596556c7612e7bf1e59fb69cb7b`.

## `rsomics-cnv`

### Boundary

One copy-number product owns BAF/LRR HMM segmentation and chromosome-level BAF
mixture analysis:

```text
rsomics-cnv call
rsomics-cnv polysomy
```

Both operations select samples from typed VCF/BCF input, validate BAF/LRR
schema and finite ranges, model copy-number state, and write related reports.
They share sample selection, chromosome partitioning, signal extraction,
quality models, output transactions, and report provenance.

This is a scientific workflow, not a VCF format command. It does not depend on
the Layer B VCF product.

### Historical assets and first release

| Asset and audited revision | Disposition |
|---|---|
| `rsomics-vcf-cnv` `09af90225defbea05173c199aaae5fd5c7639469` | Refactor then merge as `call`; correct the bcftools 1.24 HMM quality offset and validate all report files |
| `rsomics-vcf-polysomy` `cf511935afdc2299f717f84614f5db0e6c2945fb` | Refactor then merge as `polysomy`; retain mixture solver, decisions, fixtures, and performance seed |

Both repositories are clean. The first release completes both operations:

- plain VCF, BGZF VCF, and BCF input;
- single and explicit multi-sample selection;
- checked BAF, LRR, allele-frequency, chromosome, and missing-value policy;
- deterministic HMM and mixture fitting under explicit seeds;
- transactional output directories with schema-versioned machine-readable
  results in addition to compatibility reports;
- failure on insufficient data, invalid model parameters, non-finite state,
  inconsistent sample fields, and incomplete output finalization.

The historical polysomy result reports 435.8 milliseconds versus 2,564
milliseconds for bcftools 1.23.1 on a 300,000-record Apple M2 fixture, a 5.88
times throughput advantage. It remains a seed until output identity or
decision equivalence, repeated 1.24 trials, input digest, source revisions, and
peak RSS are re-established. The HMM caller has no accepted performance
record.

HMM and Levenberg-Marquardt code stays product-private initially.
`rsomics-stats` promotion requires a second product consumer with identical
parameter, convergence, missingness, and error contracts.

Do not publish `rsomics-cnv` until both operations pass current bcftools 1.24
decisions and reports, representative performance and memory gates, and all
four native exact-head CI classes.

## Cross-routed source record

The complete 47-asset historical VCF-prefixed pool is accounted for. The 30
format assets and four new-product assets are listed above. The remaining
assets are:

| Target | Assets |
|---|---|
| `rsomics-annotation` | `rsomics-vcf-csq` `0cbbba412ee08b48ff83ff172e5aadb4b85555d4`; `rsomics-vcf-split-vep` `e844a185391221549674415827afb8f35ff2674a` |
| `rsomics-plink` | `rsomics-vcf-af-dist` `5d3587b3f87fad72f15ab6131076218201bb3b65`; `rsomics-vcf-contrast` `c029a24cf4792dd4640247c031973f1eaafc2874`; `rsomics-vcf-freq-table` `0acb37c728e0e075b132ac6a31255bd6a4fd1680`; `rsomics-vcf-geno-r2` local non-Git source; `rsomics-vcf-gtcheck` `1c324f170cf613fc2808f1c8d8b46d7f4a3b7b7e`; `rsomics-vcf-indv-stats` `01f3c4c453e6636f59f202a94e5e24e0e6d6eb09`; `rsomics-vcf-missing-stats` `dfe256b41fa8708baac839d8ac6bd64ec32fdeb4`; `rsomics-vcf-roh` `61314b504884dfacdab17066c26fb3b7c5f1bfa0`; `rsomics-vcf-site-depth` `be0404d7ae5eb83b2d77b0ccdbccef4fb50ab99d`; `rsomics-vcf-smpl-stats` `613065f0c67c4788459ef12392ee202612e23f33`; `rsomics-vcf-trio-stats` `88a1dd5304482203734beae25588084dafe881f3` |

Their detailed dispositions are recorded in the
[annotation](interval-annotation-index.md#rsomics-annotation) and
[genotype-analysis](genotype-popgen.md#rsomics-plink) dossiers. None is
revived under its historical crate name.
