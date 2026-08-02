# Count product dossier

Status: first release slice published as `rsomics-count 0.1.0`. Later slices
remain audited but unimplemented.

## Boundary

`rsomics-count` is one genomic feature-counting and count-normalization
product. Its primary path assigns reads or fragments from SAM/BAM inputs to
annotated features and meta-features, then emits a multi-sample count matrix
and assignment summary. A secondary `normalize` path derives TPM, FPKM, or
FPKM-UQ from an existing gene-count matrix and validated lengths. These are
one count-table workflow, not operation-sized wrappers.

The primary behavior sources are:

- [featureCounts 2.1.1](https://subread.sourceforge.net/featureCounts.html) and
  its installed command-line help;
- the
  [Subread/Rsubread user guide](https://bioconductor.org/packages/release/bioc/vignettes/Rsubread/inst/doc/SubreadUsersGuide.pdf);
- the [featureCounts method](https://doi.org/10.1093/bioinformatics/btt656);
- the public SAM, BAM, GTF, GFF3, and SAF format contracts.

The product is not transcript-abundance estimation. Salmon, kallisto, RSEM,
transcript compatibility classes, and effective-length correction belong to
other workflows. `normalize` performs declared arithmetic on supplied gene
counts and lengths; it does not infer transcript abundance from reads.

## Operation and option map

The normal command remains `rsomics-count [options] <alignments>...`. A
synthetic `features` subcommand is not introduced merely to leave room for
future operations.

| Contract area | featureCounts operations | Target decision |
|---|---|---|
| annotation | `-F`, `-t`, `-g`, `--extraAttributes`, `-A` | GTF/GFF3 and SAF, multiple feature types, grouping attributes, extra output attributes, chromosome aliases |
| summarization level | default meta-feature, `-f` feature level | one typed level option |
| overlap | `-O`, `--minOverlap`, `--fracOverlap`, `--fracOverlapFeature`, `--largestOverlap`, `--nonOverlap`, `--nonOverlapFeature` | one assignment-policy model with checked thresholds |
| read projection | `--readExtension5`, `--readExtension3`, `--read2pos` | explicit pre-assignment projection |
| multi-mapping | `-M`, `--fraction` | exclude, full, or fractional alignment contribution |
| filtering | `-Q`, `--splitOnly`, `--nonSplitOnly`, `--primary`, `--ignoreDup` | composable filters with mutually exclusive split modes |
| strandedness | `-s` per input | unstranded, sense, or antisense per library |
| pairing | `-p`, `--countReadPairs`, `-B`, `-P`, `-d`, `-D`, `-C`, `--donotsort` | explicit read-versus-fragment mode, mate validation, and sort contract |
| junctions | `-J`, `-G` | optional junction-count output from the same scan |
| execution | `-T`, `--byReadGroup`, `-L` | measured thread count, read-group stratification, and long-read mode |
| assignments | `-R`, `--Rpath` | optional per-record CORE, SAM, or BAM assignment evidence |
| outputs | counts table and `.summary` | one transactional matrix, summary, and any requested auxiliary outputs |

featureCounts accepts multiple alignment inputs and emits one column per input.
`rsomics-count` preserves that model. It does not run the same single-input
writer repeatedly or require a separate matrix-merging command.

`rsomics-count normalize <counts> --lengths <lengths>` accepts a strict
gene-by-sample count matrix and emits TPM, FPKM, or FPKM-UQ. Missing lengths,
duplicate genes, negative or non-finite counts, zero denominators, the
FPKM-UQ biotype population, and percentile interpolation are explicit
contracts. A fused counting run may request the same derived units only after
the raw counts and effective feature lengths are fixed.

## Data and assignment model

- GTF/GFF3 coordinates are validated and converted once into checked half-open
  intervals. SAF receives a separate strict parser.
- A feature has a stable identity, interval, strand, and selected attributes.
  A meta-feature groups feature identities by the requested annotation key.
- Meta-feature length is the union of its component feature bases, not the
  unchecked sum of overlapping exon spans.
- An alignment projects to checked reference blocks from its complete CIGAR.
  Invalid operations, overflow, missing required fields, truncated BAM, and
  inconsistent headers fail the command.
- Read, fragment, long-read, and read-group units are explicit types. Pairing
  and strandedness cannot be accepted by the CLI and then silently ignored.
- Candidate overlap records covered read bases, covered feature bases, strand,
  feature identity, and meta-feature identity before assignment policy is
  applied.
- Multi-overlap and multi-mapping contributions use an exact rational or
  otherwise deterministic representation until output conversion.
- Each input library has one complete accounting: assigned units plus all
  mutually exclusive unassigned categories equals the processed total.
- Annotation and alignment contig mismatches are summarized and can be made
  fatal. They are never hidden by an empty-looking success.

## Target structure

```text
src/
├── cli.rs
├── annotation/
│   ├── gff.rs
│   ├── saf.rs
│   ├── attributes.rs
│   └── aliases.rs
├── features/
│   ├── model.rs
│   ├── index.rs
│   └── length.rs
├── alignments/
│   ├── blocks.rs
│   ├── pairing.rs
│   └── projection.rs
├── assignment/
│   ├── policy.rs
│   ├── candidate.rs
│   └── summary.rs
├── count.rs
├── normalize.rs
└── output/
    ├── matrix.rs
    ├── summary.rs
    ├── junctions.rs
    └── assignments.rs
```

Modules remain private unless a second product demonstrates the same
policy-free API. `rsomics-common` owns errors, exit mapping, reports, aliases,
and multi-output transactions. `rsomics-help` owns the CLI presentation.

`rsomics-count` is a concrete consumer of the validated reader and record
contracts reconstructed in `rsomics-bamio`. `rsomics-bam`, `rsomics-call`,
`rsomics-rnaseq-qc`, and `rsomics-signal` are its other named consumers.
Counting policy, GTF attributes, feature grouping, and assignment categories
do not enter `bamio`.

The checked coordinate type in `rsomics-intervals` is reusable. The
feature-to-meta-feature index stays private unless another product needs the
same query and metadata contract. `rsomics-count` does not depend on the Layer
B `rsomics-annotation` product; both may use a standards-focused external GFF
parser without creating another public rsomics foundation.

## Historical asset disposition

The four routed source candidates are clean repositories:

- `rsomics-featurecounts` at
  `5571384b250761eaf7368a124dca6a5d05962f64`;
- `rsomics-count-matrix` at
  `9a92e84f4470baa5d62a6ebcde81d56e452ee86d`;
- `rsomics-fpkm-count` at
  `370954dae656dce4d1f5a60fdd121bca57b0baa8`;
- `rsomics-tpm` at
  `f97156b8af6201d5923c6faef556168ca5ed4d12`.

| Asset | Disposition |
|---|---|
| CIGAR block projection | refactor then merge; make parse failures and coordinate overflow fatal |
| COITrees feature lookup and allocation-free single-hit path | refactor then merge behind checked intervals and full overlap metrics |
| GTF/GFF attribute and feature loader | tests and fixture asset only; replace permissive row skipping and incomplete grammar |
| single-end default assignment | refactor then merge; retain secondary, supplementary, duplicate, NH, MAPQ, gap, and ambiguity cases |
| counts and summary writer | tests and golden asset only; rewrite for multiple input columns, union lengths, fractional counts, and transactions |
| duplicated `HelpSpec` and inherited `Tool` runtime | discard; migrate to current `rsomics-help` and `rsomics-common` |
| small and adversarial BAM/annotation fixtures | retain and expand |
| Criterion harness | benchmark recipe only; replace tiny cases with direct end-to-end comparisons |
| `rsomics-count-matrix::merge_counts` | test, fixture, and fallback implementation asset; accept a public collation mode only if separate htseq-count or legacy featureCounts files remain a demonstrated workflow after the main multi-input writer is complete |
| `rsomics-fpkm-count` | assignment, strandedness, BED12, and RSeQC output fixtures; use the common counting engine rather than retaining a second gene counter |
| `rsomics-tpm` | refactor normalization arithmetic and oracle tests into `normalize`; remove the silent 1000 bp missing-length default and permissive row skipping |

The count product already emits a multi-input matrix, so the historical
collator does not automatically become another subcommand. It remains inside
this product's source pool rather than creating an expression utility product.

## Existing implementation gaps

The historical implementation is valuable but is not a product-ready
featureCounts replacement.

- Multiple BAM inputs overwrite the same counts and summary paths; only the
  final input survives instead of one multi-sample matrix.
- Strandedness is exposed in the CLI and report model but is never used.
- Paired fragments, multi-overlap, fractional assignment, feature-level
  counting, SAF, aliases, read projection, long reads, read groups, junctions,
  and detailed assignments are absent.
- The CIGAR iterator silently drops malformed operations.
- The index is rebuilt for each input BAM.
- Meta-feature length sums overlapping exon lengths rather than taking their
  union.
- Short annotation rows are skipped; bounds, strand, interval order, duplicate
  features, empty selections, aliases, and header compatibility are
  insufficiently checked.
- Output files are created directly and can leave a counts table without a
  matching summary.
- The committed differential compares normalized gene counts and summary
  categories, not the complete output contract. Its live oracle may skip.
- The only CI job is Linux `x86_64`; repository metadata points at the control
  plane; source contains extensive narration and historical comments.

## Retained evidence

Two always-run synthetic fixtures preserve featureCounts 2.1.1 gene counts and
all summary categories for the implemented single-end default. Live tests can
run the same fixtures when the oracle is installed. The later adversarial
fixture protects spliced CIGAR blocks, ambiguity, multi-mapping, MAPQ, and
secondary, supplementary, and duplicate default behavior.

The historical 23.7-times report is invalid as a hot-path claim because the
featureCounts command ran through `conda run`, whose startup dominated its
5.1-second result. A separate parent remeasurement on a 201 MB BAM with two
million reads and 500 genes reports 1.208 versus 1.514 seconds at one thread,
or about 1.25 times. That is the useful baseline, but consolidation, `bamio`,
multi-input output, and complete assignment policy require a fresh target-head
measurement.

## First release slice

The first release contains the normal short-read counting workflow:

- strict GTF/GFF3 and SAF annotations;
- SAM and BAM input, including stdin for one library;
- multiple input libraries and one matrix;
- feature and meta-feature levels;
- single reads and paired fragments;
- unstranded, sense, and antisense assignment;
- minimum MAPQ, primary-only, duplicate, split, and pair-validity filters;
- multi-overlap and multi-mapping exclusion, full counting, and fractional
  counting;
- minimum and fractional overlap rules;
- chromosome aliases;
- complete summary categories and transactional outputs;
- measured thread control only if the implementation provides real parallel
  work.

Junction counts, read-group stratification, per-record assignment files,
read-position projection, and long-read mode are later slices. They remain
absent from help and documentation until complete.

## Compatibility gates

- Pin featureCounts 2.1.1 initially and refresh the oracle decision against the
  current Subread release before publication.
- Run the real binary in Linux and macOS compatibility jobs. Frozen,
  provenance-recorded goldens run on all four native target classes.
- Compare every gene or feature row, selected attributes, union length, every
  library column, fractional formatting, and every summary category. Normalize
  only tool name, version, command line, and intentionally different paths.
- Exercise GTF, compatible GFF3, SAF, gzip annotation, aliases, multiple
  feature types, duplicate and missing attributes, overlapping exons, empty
  selections, mismatched contigs, and malformed annotation.
- Exercise CIGAR `M`, `=`, `X`, `D`, `N`, `I`, clipping, padding, malformed
  encodings, secondary and supplementary records, duplicates, QC failure,
  unmapped records, missing and typed NH tags, minimum MAPQ boundaries, and
  coordinate limits.
- Cross product the core assignment modes: feature or meta-feature, single or
  paired, three strand modes, multi-overlap, multi-mapping, fraction,
  largest-overlap, minimum overlap, and primary or duplicate filtering.
- Verify name- and coordinate-sorted pairs, orphan mates, interchromosomal and
  opposing-strand pairs, fragment-length limits, and truncated inputs.
- Assert the accounting invariant and atomic failure of the complete output
  set.

## Performance gates

- Compare direct executables without `conda run`, shell startup, or hidden
  format conversion inside the timed command.
- Measure one and multiple libraries separately. Build the annotation index
  once per invocation and report its time and memory separately from scans.
- Use at least two million realistic spliced reads, overlapping multi-exon
  genes, ambiguous and multi-mapped records, and enough libraries to exercise
  matrix output.
- Compare one thread and equal multi-thread counts where supported. Record
  scaling, total CPU, peak RSS, and output equality rather than only wall time.
- Include single-end, paired-fragment, stranded, multi-overlap, fractional, and
  long-read workloads as their slices become stable.
- Record exact revisions, binary hashes, commands, fixture generator and
  hashes, warmups, timing distribution, CPU time, peak RSS, and output hashes.
- The stable primary hot path must be strictly faster or demonstrate a
  material resource advantage over featureCounts.

## License and attribution

The retained Rust code and synthetic fixtures are team-owned and remain MIT OR
Apache-2.0. Subread and featureCounts are GPL-3.0. Their source is not copied,
translated, linked, or vendored; the paper, public documentation, format
specifications, and separately installed executable define the compatibility
contract. The featureCounts name, pinned version, paper, and Subread project
remain in product attribution.

## Explicit exclusions

- No publication under the retired `rsomics-featurecounts` name.
- No option that is parsed but not enforced.
- No silent malformed annotation, CIGAR, BAM, or pair skipping.
- No one-output-per-loop overwrite for multiple inputs.
- No transcript-abundance inference, effective-length correction, or
  differential expression.
- No speculative public annotation, assignment, or interval-index crate.
- No direct dependency on another Layer B product.
- No CRAM, junction, read-group, detailed assignment, or long-read claim in the
  first release unless its complete slice passes independently.
