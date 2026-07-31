# Variant format, calling, and copy-number product dossiers

Status: boundary, upstream-operation, and historical-source audit complete.
`rsomics-vcf` exists with complete `head`, current-contract `query`, and
strict `validate` operations. `rsomics-call` and `rsomics-cnv` do not yet
exist. None of the three products is published.

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
`330736317e7de74ac3ad147eb3b861c59ed4764f` implements complete `head`, the
declared single-input `query` contract, and strict `validate` without
advertising the other first-slice commands. It consolidates the historical
`rsomics-vcf-head`, `rsomics-vcf-query`, `rsomics-vcf-extract`,
`rsomics-vcf-valfmt`, and `rsomics-vcf-validate` assets into private product
modules. `rsomics-help` supplies the shared command layout. `rsomics-common`
0.8 supplies transactional named output, and common 0.9 preserves complete
invalid validation reports in the shared JSON and exit contract.

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

The initial `head` exact-head CI run `30619149446` and query run `30622684140`
pass native Linux and macOS on `x86_64` and `aarch64`. Exact-head CI run
`30627803709` passes the same four native target classes at validation revision
`330736317e7d`. Its Linux `x86_64` job builds bcftools 1.24 from the official
SHA-256-pinned archive, fetches both exact validation corpora, and passes all
command and validation oracle suites.

### Later slices

1. `filter`, `norm`, `annotate`, `reheader`, and `setgt`, with complete
   expression, allele-remapping, symbolic-variant, gVCF, and header contracts.
2. `concat`, `merge`, `isec`, `sort`, and `split`, with bounded memory,
   external runs, header/sample reconciliation, localized alleles, and
   transactional multi-output behavior.
3. `consensus`, `convert`, and format-centered `stats` profiles.

An unfinished command is absent from public help. No operation ships as a
flag-compatible shell around a partial line parser.

### Target structure

```text
src/
├── lib.rs
├── main.rs
├── cli.rs
├── head.rs
├── query.rs
├── query_bcf.rs
├── query_format.rs
├── validate.rs
├── variant_type.rs
├── format/
│   ├── reader.rs
│   ├── record.rs
│   ├── value.rs
│   └── text.rs
├── validation/
│   ├── definitions.rs
│   ├── header.rs
│   ├── record.rs
│   └── v44.rs
├── expression/
└── commands/
    ├── head.rs
    ├── index.rs
    ├── query.rs
    ├── validate.rs
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
| `rsomics-vcf-filter` `93d91c114d2ce0fc31a6b1c7176280f558c06f3c` | Test and expression-integration seed | Later `filter`; discard whole-file input |
| `rsomics-vcf-filter-summary` `f8323af72303498bcc59f16c4a9feb897b992d3f` | Merge report fixture | `stats filters` |
| `rsomics-vcf-fixref` `d6efd2bd79067b2b7b2f738703e428ca40dc56f1` | Refactor then merge | Later `fixref`; retain reference-access performance seed |
| `rsomics-vcf-head` `0297fa20cb271124c9ccc15d51fff973f1df50b6` | Refactor then merge | First-slice `head`; add BCF |
| `rsomics-vcf-indel-stats` `a7774e648149a7b12dbfbbb60870d54d1cf2a373` | Refactor then merge | `stats indels` |
| `rsomics-vcf-index` `5eafb949d64a101c1c4e2d21e9a311ad9379ac65` | Refactor after dirty-diff attribution | First-slice TBI/CSI `index` |
| `rsomics-vcf-isec` `86bedb28892ccbcb6137bfb3c82925fe931609f1` | Test and merge-loop seed | Later `isec` |
| `rsomics-vcf-merge` `571af0688ac61b857b529b0db20ae886999e04fa` | Test asset | Replace incomplete header, allele, and FORMAT reconciliation |
| `rsomics-vcf-norm` `c4eeb5026199141a08ddd7b710be14488887edc2` | Test and split seed | Later complete `norm`; add reference realignment and 1.24 semantics |
| `rsomics-vcf-query` `1bd16a4562e931010d6138e71c3a6112040edd29` | Refactor merge complete; partial parser replaced | First-slice streaming `query` |
| `rsomics-vcf-reheader` `e25a2942b13b912fefc21e739d3f10876a59ac74` | Refactor then merge | Later transactional `reheader` |
| `rsomics-vcf-sample` `3217323c7e6a22f2086367f8bdf9cc8bde6abd88` | Merge sample-selection fixtures | First-slice `view --samples` |
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
| `rsomics-vcf-view` `d0c187ec2c85033f721ac135be874cf0aa48eb02` | Test and predicate seed | Replace whole-file VCF-only command |

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

### Publication decision

Do not publish `rsomics-vcf` yet. The target repository and complete `head`,
current-contract `query`, and strict `validate` operations now exist with
bcftools 1.24, hts-specs, and EBI oracles plus performance records, but `view`
and `index` remain absent. No placeholder command is exposed. Publication
waits for the complete first slice, exact-head four-native-platform CI over
that slice, the representative region/index gate, and a fresh public API
review.

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

### First release slice

All three commands ship together with:

- multiple alignment inputs and samples;
- SNP, indel, and multiallelic likelihoods;
- BAQ, overlap handling, mapping/base quality policy, depth caps, strand
  evidence, and current annotations including `FORMAT/QM`;
- indexed regions and target files;
- reference-only and gVCF-compatible positions where declared;
- explicit ploidy and sample metadata;
- consensus and multiallelic calling modes that are individually named and
  oracle-tested;
- VCF, BGZF VCF, and BCF output with transactional named files.

A single-sample SNP-only command is not a publishable slice.

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

`rsomics-bamio` supplies validated SAM/BAM/CRAM streams and indexed access.
`rsomics-pileup` supplies a fallible sorted projection kernel, overlap
handling, BAQ, and bounded column state. `rsomics-call` is the second concrete
pileup consumer beside `rsomics-bam`; its consumer tests justify public API
items one by one.

Calling likelihoods, allele selection, ploidy policy, priors, annotations, and
VCF output remain in the product. `rsomics-stats` receives a numerical kernel
only if another product demonstrates the same contract.

Compatibility uses pinned bcftools 1.24 `mpileup`, `call`, and their composed
pipeline, plus adversarial BAM/CRAM and reference fixtures. Performance
compares both individual stages and the fused `run`; the fused path should
provide a material I/O or memory advantage without changing calls.

Do not publish `rsomics-call` until the complete slice, consumer-driven
foundation APIs, four native exact-head CI classes, and current oracle and
performance evidence pass.

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
