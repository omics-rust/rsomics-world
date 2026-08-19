# rsomics-vcf stats design

Status: product boundary, report families, bcftools and VCFtools authorities,
historical assets, corrected edge cases, foundation decisions, output schemas,
compatibility matrix, and release gates are defined. The target release is
0.14.0 after the complete `convert` slice.

## Product boundary

`stats` owns descriptive and comparative reports whose primary data model is a
typed VCF or BCF stream. It combines the former operation-sized statistics
crates into one workflow family and retains the established reports that users
recognize from bcftools, its statistics plugins, and VCFtools.

The family includes:

- the complete one- and two-file bcftools-style machine report;
- lossless merging of compatible native sharded reports;
- self-contained visualization of those reports;
- FILTER-category counts and Ts/Tv;
- coordinate-window variant density;
- focused Ts/Tv reports by summary, window, alternate count, quality, or tag;
- REF/ALT character-length histograms;
- detailed site and sample indel distributions.

This boundary does not absorb every calculation that happens to read VCF.
Per-individual missingness, sample summaries, LD, relatedness, concordance
workflows with sample identity policy, trio and Mendelian analysis, and
genotype matrices stay in `rsomics-plink`. Diversity, F-statistics, ROH, and
population windows stay in `rsomics-popgen`. General numerical tests stay in
`rsomics-stats`. Index cardinalities remain `rsomics-vcf index --stats`, while
arbitrary per-record projection remains `query`.

The `counts` plugin is subsumed by the comprehensive summary-number section.
VCFtools `--hist-indel-len` is subsumed by the typed indel distribution. No
metric, bin, or report becomes another crate.

## Upstream authority

The primary oracle is bcftools and HTSlib 1.24:

- `stats` for one-file summaries and two-file set/genotype comparison;
- `plot-vcfstats` for report merging and visual meaning;
- `+allele-length` for the fixed-width character-length table;
- `+indel-stats` for site, genotype, length, VAF, and consequence strata.

The audited bcftools tag is revision
`fb9f0f783e0f67d734f6fa7fe4df9d230522f196`. Relevant SHA-256 values are:

- `vcfstats.c`:
  `7d08ed06b1fa1e7e7b452a5766ac125f35ca4ab36dcc8a8be5b7bfd8e85462c8`;
- `plugins/allele-length.c`:
  `7124bdd26a3e6d90f0f5470e88333ee8da040afec81efdadc84e6870269a2719`;
- `plugins/indel-stats.c`:
  `4177adc65779017c29f91c713da074da8a1c8ae206514126b25a640418b4e2b5`;
- installed 1.24 `plot-vcfstats`:
  `cfd70cd97e69b4b68383e295a9604745e8afe41113dcb79a441c1fb726728e39`.

These sources are MIT licensed. The complete official 1.24 `stats`,
`allele-length`, and `indel-stats` regression fixtures are compatibility
assets with file-level provenance.

The focused FILTER, density, and Ts/Tv authorities are VCFtools 0.1.17 and its
public manual. The audited tag is revision
`1c53c3c73be141103069965403e655536dda9c87`; its
`src/cpp/variant_file_output.cpp` SHA-256 is
`112b2647ed4b0071ec06c7f2f14c0b670c6cbb63d837b20f39fe2456400556b6`.
VCFtools is LGPL. Its source is behavior evidence only and is not copied.
Historical team-owned implementations and independently checked black-box
fixtures supply the Rust implementation assets.

VCF 4.5 and BCF2 define typed records. Transition and transversion classes are
limited to distinct canonical A, C, G, and T single-base substitutions. VCF
coordinates remain one-based; window labels follow the selected report's
documented convention.

## Historical assets

Six retired repositories contribute to this family.

| Repository | Revision | Version | Tracked Rust and manifest lines | Disposition |
|---|---|---:|---:|---|
| `rsomics-vcf-stats` | `66299f4d56e26b1d4c1498ffba9b489cfb7d5f85` | 0.1.0 | 445 | fixtures only; implementation discarded |
| `rsomics-vcf-filter-summary` | `f8323af72303498bcc59f16c4a9feb897b992d3f` | 0.1.3 | 666 | refactor grouping and formatting; discard parser and CLI |
| `rsomics-vcf-snp-density` | `c8f2c9b1507712bcfb967693b18fc8d936f14465` | 0.1.3 | 519 | retain format and boundary fixtures; replace accumulator |
| `rsomics-vcf-tstv-strat` | `f1697b722b7a1d99c6393a82ca924f69773e4c38` | 0.1.2 | 843 | refactor formatting and corrected goldens; replace counting |
| `rsomics-vcf-allele-length` | `d4e3b56d5132e4e6bb96faeddc1ee9992fe6ee53` | 0.1.1 | 500 | refactor histogram and writer; replace parser and ALT policy |
| `rsomics-vcf-indel-stats` | `a7774e648149a7b12dbfbbb60870d54d1cf2a373` | 0.1.0 | 1,461 | refactor typed accumulators and report rows; replace orchestration |

All clones except `rsomics-vcf-stats` are clean. That repository contains only
an untracked `Cargo.lock`; it remains untouched and is not treated as source.

### Basic stats

The old basic implementation line-parses plain VCF, silently skips short
records, discards valid no-ALT and spanning-deletion records from its total,
classifies alleles by string length, and exposes eight counters under a name
that implies the complete upstream operation. It lacks BCF, compression,
standard input, samples, typed fields, selection, comparison, histograms,
report compatibility, atomic output, and bounded workers. Its live test checks
only three summary fields and can skip whenever bcftools is absent. Its two
small VCFs remain adversarial seeds; all production code is discarded.

### FILTER summary

The old FILTER implementation reproduces the useful VCFtools combination-key
and `%g` formatting on compact fixtures, including PASS, missing FILTER,
multiple filters, equal counts, and scientific notation. Its line parser does
not validate the header or typed record and examines only the first ALT.
Grouping and table-format expectations are refactor assets. Input, CLI, help,
JSON, and process-launch benchmark code are discarded.

### Density

The old density implementation preserves VCFtools's zero-origin window labels,
unique-position counting, first-seen contig order, zero bins between observed
bins, and `%g` formatting. It stores every unique position in nested maps and
sets, so memory grows with records and its huge-coordinate fixture succeeds by
changing the upstream storage model without defining sortedness. The fixtures
and formatter remain. The parser and accumulator are replaced by a typed,
coordinate-checked streaming reducer.

### Ts/Tv strata

The old implementation records VCFtools's biallelic rules, quality thresholds,
missing QUAL spelling, and alternate-count table shape. It also exposes why
the upstream contract needs correction: VCFtools allocates only `2N` bins, so
an all-alternate diploid count of `2N` writes out of bounds and is absent from
the report. The Rust clone avoids the write but silently drops the site. Its
quality golden intentionally replaces the upstream final-row uninitialized
value with zero.

The formatting functions and ten compact goldens remain. GT parsing, diploid
bin sizing, whole-input quality collection, CLI, and comments that narrate the
audit are replaced.

### Allele length

The old histogram is byte-identical to the bcftools 1.24 oracle on its
comprehensive fixture; the confirmed output SHA-256 is
`48a7b81ed899c98ad6108d71fb74076c5ff5c2a95755293144dfae487093b132`.
Its fixed 512 buckets, overflow clamp, base classification, table writer, and
fixtures are useful refactor assets. The implementation line-parses VCF and
silently uses only the first ALT. The target iterates typed alleles and makes
first-ALT compatibility explicit.

### Detailed indel stats

The old implementation contains the strongest reusable algorithmic material:
signed length bins, per-genotype VAF, heterozygous minor-allele fraction,
frameshift/in-frame summaries, and bcftools-compatible `DEF`, `SN`, `DVAF`,
`DLEN`, `DFRAC`, and `NFRAC` rows. Its data rows remain byte-identical between
the frozen 1.23.1 output and bcftools 1.24; the confirmed row SHA-256 is
`74815c5c78bfcad1ccb9ad0eed2ad67ac2d7e193b918aae5b24725ef89a377ed`.

It still supports only plain VCF, the implicit `all` stratum, and one named
file. It lacks expressions, threshold expansion, regions, targets, BCF,
compressed input, standard input, sample selection, and grouped report
integration. It also copies the upstream accidental requirement that any GT
forces FORMAT/AD. Typed accumulation and report layout are refactored into the
product; the parser, CLI, direct output, incomplete surface, and explanatory
comment blocks are discarded.

## Command tree

```text
rsomics-vcf stats report [OPTIONS] A [B]
rsomics-vcf stats merge [OPTIONS] REPORT...
rsomics-vcf stats plot [OPTIONS] REPORT...
rsomics-vcf stats filters [OPTIONS] [INPUT]
rsomics-vcf stats density [OPTIONS] [INPUT]
rsomics-vcf stats tstv summary [OPTIONS] [INPUT]
rsomics-vcf stats tstv windows [OPTIONS] [INPUT]
rsomics-vcf stats tstv alt-count [OPTIONS] [INPUT]
rsomics-vcf stats tstv quality [OPTIONS] [INPUT]
rsomics-vcf stats tstv tag [OPTIONS] [INPUT]
rsomics-vcf stats allele-length [OPTIONS] [INPUT]
rsomics-vcf stats indels [OPTIONS] [INPUT]
```

`INPUT` defaults to standard input for single-stream leaves. `report` accepts
one input or two named inputs; at most one may be standard input. `merge` and
`plot` require named reports so they can validate complete provenance before
producing output.

Every leaf is rendered through `rsomics-help`. The first help screen explains
which report answers which question; leaf help contains only the relevant
selection, bin, genotype, reference, or output options. Any nested-tree API
improvement required in `rsomics-help` must remain generic and retain the
existing BED, BAM, sequence, and VCF consumers.

## Shared input and output contract

All VCF-reading leaves accept plain VCF, BGZF VCF, raw BCF, compressed BCF,
and standard input by content. They share:

- `--include EXPR` or `--exclude EXPR`, never both;
- indexed regions and streaming targets with the product overlap policies;
- selected samples only on reports whose values use FORMAT data;
- explicit FILTER selection where applicable;
- bounded compressed-input workers;
- typed malformed-input diagnostics with record and field context.

Named report output uses `-o, --output FILE` and an atomic file. Text goes to
standard output by default. `--json` emits the same typed report through the
product envelope; it never emits a text prefix or a second document. A broken
pipe is distinguished from invalid input. Counters do not overflow silently.

Reports are produced only after input validation completes, so malformed input
does not leak a plausible partial report to standard output. Accumulators are
bounded by configured bins, samples, report sections, and distinct FILTER
groups rather than record count. Leaves that require coordinate order fail on
regression or a reopened contig block.

## Comprehensive report

`stats report A` emits the complete stable bcftools 1.24 sections:

- set definitions and summary numbers;
- all-ALT and first-ALT Ts/Tv;
- frameshift and repeat-context indel summaries when supporting inputs exist;
- singletons and non-reference allele-frequency bins;
- quality and user-selected tag bins;
- signed indel-length and mean-VAF distribution;
- the twelve directed substitution classes;
- genotype and site depth distributions;
- per-sample SNP/general counts, indel counts, and VAF;
- observed heterozygosity quantiles by first-ALT frequency.

Reference-only, identical REF/ALT, SNP, MNP, indel, mixed, symbolic, breakend,
spanning-deletion, and multiallelic categories use the current product type
model. Mixed records may contribute to several type counters exactly where the
report declares allele-level rather than record-exclusive semantics.

`--af-bins` accepts a strictly increasing finite sequence within `[0,1]`.
`--af-tag TAG` requires a declared numeric Number=A tag. Without it, frequency
uses checked INFO/AC and INFO/AN when their cardinality and range are valid,
then selected FORMAT/GT. The report records the source and the number of
records that could not supply frequency. It never combines stale INFO counts
with a changed sample subset.

`--depth MIN,MAX,STEP` requires nonnegative bounds, `MIN <= MAX`, and a positive
step. FORMAT/DP and INFO/DP are separate typed sources. Missing values do not
become zero. User Ts/Tv binning requires a declared scalar numeric field,
finite bounds, a positive bin count, and an explicit vector subscript when the
field is not scalar.

An indexed reference enables repeat-consistency context through the existing
`rsomics-seqio::IndexedFasta`. An indexed one-based inclusive exon table enables
frameshift context. Missing support suppresses only the sections whose data is
unavailable and records that decision; explicitly requesting such a section
without its input fails.

### Two-file report

`stats report A B` streams sorted inputs through the same record matcher used
by `isec`. It emits A-only, B-only, and shared set sections plus:

- genotype concordance by non-reference allele frequency for SNPs and indels;
- non-reference and genotype-class discordance;
- dosage correlation with paired missingness;
- per-sample concordance and complete genotype transition tables;
- optional per-site discordance rows.

`--collapse none|snps|indels|both|some|all` uses the already defined set-match
semantics. Exact allele identity remains the default. Duplicate keys within an
input fail because comparison multiplicity is otherwise ambiguous. Selected
sample names must exist in both headers; their requested order is retained.
GT ploidy, missingness, allele indices, and FORMAT cardinality are checked
before a comparison cell is updated.

Dosage correlation uses a stable paired online accumulator and reports the
number of contributing genotypes. Fewer than two finite pairs or zero variance
produces a declared missing value, not zero or NaN disguised as a measurement.

### Text compatibility

The default text schema preserves the established section identifiers and
column meanings so `plot-vcfstats` and existing parsers can consume the data
rows. Volatile command-line and absolute-path prose is replaced with stable
schema, tool, source-label, and option metadata. `--label` controls source
names without changing input paths. Numeric formatting is locale-independent.

Native text reports additionally carry comment-prefixed `RSM` metadata and
sufficient-statistic rows. Legacy readers ignore those rows, while
`stats merge` uses them to recompute quantiles, means, ratios, and
correlations exactly. JSON represents the same state as named fields rather
than an opaque payload.

Compatibility tests compare section rows and typed values rather than forcing
the old executable name or absolute path into the header. JSON uses named
fields and explicit nulls for unavailable values.

## Report merge and visualization

`stats merge` combines compatible native report shards without reading VCF
again.
Schema version, source labels, sample order, selected sections, AF/depth/tag
bins, type policy, reference identity, and comparison policy must match.
Counts and sufficient statistics merge exactly; ratios, means, correlations,
and quantiles are recomputed rather than averaged.

Rsomics reports record their selected coordinate partitions. Overlapping
partitions, duplicate shard identities, incomplete contig partitions, or mixed
one-file and two-file schemas fail. A legacy bcftools report can be visualized
directly but cannot be passed to `stats merge`: it lacks both partition
provenance and sufficient state for exact recomputation of every section. The
user must regenerate a native report; there is no silent or opt-in lossy merge.

`stats plot` accepts one compatible report or a mergeable set and writes one
self-contained HTML file with embedded SVG charts, accessible tables, report
metadata, and no network resources. It covers summary counts, Ts/Tv, AF,
quality, indel length/context, depth, per-sample counts, VAF, heterozygosity,
and pairwise concordance when present. Missing sections are omitted with a
reason rather than rendered as zero.

PDF/LaTeX generation, editable generated Python, rasterization flags, and
browser telemetry are excluded. The HTML artifact removes the upstream
Matplotlib and LaTeX runtime dependency while preserving plot meaning.
`stats merge` supplies the upstream merge-only workflow separately.

## Focused FILTER report

`stats filters` groups by the exact FILTER set after typed normalization.
PASS and the missing value remain distinct. Each row contains record count,
canonical biallelic SNP transitions, transversions, and ratio. Records with
multiple FILTER values belong to one combination group by default;
`--group tags` additionally counts each named tag independently and labels the
schema so the two meanings cannot be confused.

Rows sort by decreasing record count and then deterministic key order for the
VCFtools-compatible default. An empty group set emits only the header. A zero
denominator is represented as `null` in JSON and the selected compatibility
spelling in text.

## Density report

`stats density --window BASES` requires a positive integer window. VCFtools
zero-origin labels are retained: a record at POS equal to the window size has
`BIN_START=window`. `--type variants|snps` defaults to `variants`, accurately
describing the upstream behavior that any non-reference record contributes.
`--unit positions|records` defaults to unique positions for VCFtools
compatibility; record mode counts duplicate records explicitly.

The input must be sorted by header contig order and position. The reducer holds
one contig/window state and emits zero-count bins only between its first and
last observed bins. Coordinate multiplication and addition are checked. A
sparse huge coordinate consumes constant accumulator memory even though a
request to print every intervening zero bin can still create a large output;
`--zero-bins omit` suppresses those rows explicitly.

## Ts/Tv reports

Every focused Ts/Tv leaf accepts only canonical biallelic SNPs. Multiallelic,
identical, ambiguous-base, symbolic, spanning-deletion, and non-SNP records are
counted as excluded categories in the summary.

- `summary` emits the six unordered substitution models plus Ts and Tv totals.
- `windows --window BASES` emits per-contig coordinate bins.
- `alt-count` counts typed non-reference GT alleles and emits every observed
  count from zero through the maximum; the valid all-alternate `2N` count is
  never dropped. Mixed ploidy and partial missingness use actual called alleles.
- `quality` emits each distinct finite QUAL threshold with strictly lower and
  strictly greater counts. Missing QUAL is a separate labelled threshold only
  when `--missing-qual separate` is selected; the default excludes it.
- `tag --tag FIELD --min X --max Y --bins N` bins a declared numeric field with
  explicit vector selection.

Text compatibility formats can reproduce VCFtools `nan` and `inf`; JSON uses
null plus numerator and denominator. The implementation never reads beyond a
bin vector and never copies uninitialized data.

## Allele-length report

`stats allele-length` emits the established 512 character-length rows and one
totals row. Lengths at or above 512 clamp into bucket 511. Base-only means
ASCII A, C, G, or T in either case; every other character is counted in the
non-base column.

The default counts every REF/ALT pair at a multiallelic site, incrementing REF
once per pair. `--first-alt` reproduces the plugin's first-ALT behavior and
therefore its exact historical table. Valid no-ALT records are skipped with a
reported counter rather than dereferenced or treated as malformed. Empty ALT
strings and invalid typed alleles fail.

## Detailed indel report

`stats indels` produces the stable `DEF`, `SN`, `DVAF`, `DLEN`, `DFRAC`, and
`NFRAC` sections for site and selected-sample indels. `--max-length` must be
positive. `--vaf-bins` must be at least two. Out-of-range lengths enter the two
overflow bins and VAF is validated within `[0,1]`.

The default `all` stratum can be replaced or supplemented by repeatable
`--stratum LABEL=EXPR`. Labels are unique field values, expressions use the
product evaluator, and all strata update in one pass. This replaces ambiguous
curly-brace textual expansion while retaining its one-pass threshold use case.

FORMAT/GT determines which typed alleles contribute genotype counts.
FORMAT/AD supplies optional VAF and heterozygous minor-allele fraction. If AD
is absent, site and genotype counts remain valid while AD-derived values are
reported unavailable; `--require-ad` makes absence an error. Present AD must
have Number=R cardinality, nonnegative values, and sufficient total support.
Out-of-range GT alleles and partial vectors fail.

`--consequence-tag` accepts a declared INFO string field. Frameshift and
in-frame classification recognizes complete Sequence Ontology terms from the
field schema, not arbitrary substrings. Unknown or conflicting consequences
enter not-applicable with counters.

Pedigree-restricted de novo indel analysis and `--alt2ref-DNM` are excluded
from this leaf and remain in the planned `rsomics-plink` trio workflow. This
keeps sample-family identity and Mendelian policy in one product.

## Deliberate safety differences

Live probes recorded the following behavior:

| Probe | Upstream result | rsomics contract |
|---|---|---|
| malformed VCF passed to `bcftools stats` | exits 255 after emitting a plausible report preamble to stdout | emit no report before the complete input validates |
| valid no-ALT record passed to `+allele-length` | process exits 139 with no diagnostic | skip as inapplicable and report the count |
| multiallelic `+allele-length` record | silently measures only the first ALT | count every ALT by default; first-ALT mode is explicit |
| one diploid `1/1` site passed to VCFtools `--TsTv-by-count` | exits 0 but omits count 2 because only bins 0 and 1 exist | retain the observed all-alternate count |
| final VCFtools `--TsTv-by-qual` row | prints an uninitialized nonzero transversion count | print deterministic zero/null values |
| `+indel-stats --nvaf 10` | exits 255 because the integer bin count is incorrectly validated against `[0,1]` | accept checked counts of at least two |
| historical GT without AD in detailed indel stats | aborts the complete report | keep non-AD sections; fail only when AD is present but invalid or explicitly required |
| sparse huge density coordinate | upstream dense vector grows with coordinate magnitude | bounded streaming state; zero-bin output policy is explicit |

These differences correct crashes, undefined behavior, silent loss, and
unnecessary coupling without changing the named compatibility formats where
their semantics are sound.

## Foundation decision

No new public foundation item is required for this slice.

- Variant categories, GT/AD decoding, expression values, sample selection,
  regions, targets, and comparison identity remain private VCF policy.
- `rsomics-seqio::IndexedFasta` already owns indexed reference lookup and gains
  another concrete consumer through indel context.
- `rsomics-common::AtomicFile` owns named report transactions; the plot HTML is
  one atomic file rather than a new directory-transaction abstraction.
- `rsomics-help` owns nested rendering and shared diagnostics.
- FILTER groups, report sections, histogram bins, Ts/Tv classes, genotype
  matrices, and indel strata have no second non-VCF consumer.

Stable online covariance or correlation may eventually belong in
`rsomics-stats`, with `rsomics-vcf` comparison and `rsomics-plink` LD as named
consumers. It stays private until both consumer-side contracts agree on
missingness, weights, zero variance, numerical error, and returned sufficient
statistics. The existing `rsomics-stats::hwe_exact` is not used for the
bcftools HWE section, which reports observed heterozygosity quantiles rather
than an exact-test p-value.

## Product structure

```text
src/
├── stats.rs
├── stats/
│   ├── allele_length.rs
│   ├── compare.rs
│   ├── density.rs
│   ├── filters.rs
│   ├── indels.rs
│   ├── merge.rs
│   ├── plot.rs
│   ├── report.rs
│   ├── schema.rs
│   └── tstv.rs
└── commands/
    ├── stats.rs
    └── stats/
        ├── allele_length.rs
        ├── density.rs
        ├── filters.rs
        ├── indels.rs
        ├── merge.rs
        ├── plot.rs
        ├── report.rs
        └── tstv.rs
```

`schema.rs` owns versioned report rows and JSON types. `report.rs` owns the
one-file accumulators. `compare.rs` owns synchronized two-file state.
`merge.rs` owns sufficient-statistic validation and combination. The focused
modules own their format-specific reducers. `plot.rs` consumes only the report
schema and does not parse VCF.

Command adapters bind Clap, existing selection, output transactions,
`rsomics-help`, and summaries. They contain no allele classification,
genotype math, report merging, or direct destination creation.

## Compatibility matrix

The pinned matrix covers:

- every bcftools 1.24 one-file report section with and without samples,
  reference, exons, custom AF bins, custom AF tag, depth bins, ID split, and
  user Ts/Tv tag;
- two-file A-only, B-only, shared, every collapse mode, sample subsets and
  order, SNP/indel concordance, missing GT, dosage correlation, verbose site
  discordance, and duplicate failures;
- VCF 4.1 through 4.5, all four VCF/BCF encodings, standard input, indexed
  regions, streaming targets, symbolic alleles, breakends, spanning deletions,
  gVCF blocks, mixed types, and multiallelic records;
- FILTER PASS, missing, one tag, multiple tags, header-order differences,
  canonical grouping, ties, zero denominators, and tag-wise grouping;
- density boundaries, duplicate positions, duplicate records, gaps, multiple
  contigs, coordinate regression, reopened contigs, huge coordinates, zero-bin
  policy, variant/SNP policy, and checked arithmetic;
- Ts/Tv models, every focused mode, multiallelic exclusions, ambiguous bases,
  missing QUAL, full `2N`, mixed ploidy, partial missing, invalid GT, finite tag
  bounds, and ratio formatting;
- allele lengths 0 through overflow, all ALT alleles, first-ALT mode, no-ALT,
  symbolic/non-base alleles, empty input, BCF, and malformed alleles;
- indel insertions, deletions, complex alleles, length overflow, multiallelic
  GT, AD cardinality, absent AD, VAF bounds, SO consequences, multiple strata,
  selections, and all encodings;
- merge schema mismatch, overlapping shards, duplicate shards, incompatible
  bins, lossless recomputation, rejection of legacy bcftools merge inputs,
  malformed reports, and transactional output;
- plot section presence, escaping, accessibility labels, embedded data,
  deterministic SVG, empty sections, large sample sets, and malformed reports.

Historical goldens are rerun against exact installed versions before import.
The audit confirmed byte-identical representative FILTER, density, Ts/Tv
count, allele-length, and indel rows. The corrected Ts/Tv quality golden is
retained with an explicit undefined-behavior provenance note rather than being
called byte-compatible.

## Tests

Unit tests cover typed category classification, Ts/Tv, frequency fallback,
bin boundaries, ratios, stable covariance, genotype cells, FILTER keys,
density transitions, allele clamps, AD-derived VAF, consequence terms, schema
round trips, merge algebra, and HTML/SVG escaping.

Golden tests cover every text section and focused format. Differential tests
run the pinned bcftools and VCFtools oracles and normalize only volatile
provenance. Property tests split a stream into disjoint shards, merge their
reports in randomized trees, and require the same sufficient statistics and
rendered values as the unsplit stream.

Malformed suites cover bad headers, field cardinality, nonfinite values,
allele indices, unsorted streams, duplicates, truncated compression, report
schema corruption, incompatible merges, overflowing coordinates and counts,
write failure, and broken pipes. A failure does not leave a new named output or
a plausible complete stdout report.

## Performance gates

Formal comparison pins source and binary hashes, machine, filesystem, fixture
hashes, flags, worker count, warmups, alternating runs, timing distribution,
peak RSS, bytes read, report semantic hash, and plot artifact hash.

Representative workloads include:

- 10 million mixed records and 2,000 samples for the complete one-file report;
- two sorted 5 million-record cohorts with 1,000 shared samples for concordance;
- 100 disjoint chromosome shards merged into the complete report;
- 20 million sparse records for FILTER, density, and Ts/Tv profiles;
- 10 million multiallelic records for all-ALT allele length;
- 5 million indel-rich records with GT, AD, consequences, and ten strata;
- a complete large report rendered to self-contained HTML.

The complete report and at least one focused hot path must demonstrate strict
throughput or peak-memory advantage over the equivalent pinned upstream
operation. Report merging must be associative and faster than reparsing VCF.
Plot comparison records total runtime and external runtime dependencies; the
self-contained result is a material installation benefit but does not excuse a
regression in the VCF-reading hot paths.

No historical process-launch benchmark is a release claim. Dense-coordinate,
whole-quality-vector, and per-record set allocations are specifically measured
and rejected if memory grows beyond the declared state model.

## Release gate

Release 0.14.0 is complete only when:

- every command-tree leaf and every declared section is implemented without
  placeholders or advertised omissions;
- historical code and fixtures are imported only according to the recorded
  dispositions and provenance;
- all encodings, selections, sample policies, categories, bins, expressions,
  comparisons, report merges, and visual sections pass the matrix;
- corrected crash, undefined-memory, silent-loss, AD, and sparse-coordinate
  cases fail loud or produce the documented safe result;
- named reports and HTML pass transaction and fault-injection tests;
- formatting, strict Clippy, unit, integration, property, differential,
  malformed-input, transaction, plot, and benchmark smoke suites pass;
- the formal performance gate records the required strict useful advantages;
- package contents, metadata, README, nested unified help, licenses, and
  attribution receive a clean exact-head review;
- native Linux and macOS CI pass on `x86_64` and `aarch64` at that exact head;
- publication occurs only after all earlier declared release slices and this
  complete statistics family are present.

Audit evidence is retained outside the repository at:

```text
/Volumes/KIOXIA/Developments/tmp/rsomics-vcf-stats-audit-20260819
```

Primary references are the
[bcftools 1.24 stats manual](https://samtools.github.io/bcftools/bcftools.html#stats),
the [bcftools plugin list](https://samtools.github.io/bcftools/howtos/plugins.html),
the [VCFtools manual](https://vcftools.github.io/man_latest.html),
and the [VCF and BCF specifications](https://github.com/samtools/hts-specs).
