# Variant format, calling, and copy-number product dossiers

Status: boundary, upstream-operation, and historical-source audit complete.
None of the three target repositories exists, and none is published.

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
├── format/
│   ├── header.rs
│   ├── reader.rs
│   ├── record.rs
│   ├── value.rs
│   └── writer.rs
├── expression/
├── query_format/
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
| `rsomics-vcf-extract` `3bca5d5a6d2dbec187a00a29620c8c04b2fabe0d` | Merge fixtures and field selection | First-slice `query` |
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
| `rsomics-vcf-query` `1bd16a4562e931010d6138e71c3a6112040edd29` | Refactor then merge | First-slice streaming `query` |
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
| `rsomics-vcf-validate` `e6fef96f3cdfde5d5740d57cb8c5185cfc5285ff` | Test seed after dirty-diff attribution | Replace the eight-column line checker |
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

1. VCF 4.2 through 4.5 headers and records, BCF2, BGZF, TBI, and CSI;
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

### Publication decision

Do not publish `rsomics-vcf`. The target repository and shared internal format
model are absent, the first slice is not implemented, and no current
four-native-platform or 1.24 performance gate exists.

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
