# Peak product dossier

Status: source and upstream-operation audit complete. The target repository has
not been created.

## Boundary

`rsomics-peak` is one chromatin-enrichment peak workflow for ChIP-seq,
ATAC-seq, CUT&RUN, and CUT&Tag. It calls enriched genomic regions, refines or
annotates those regions, and builds per-sample peak count matrices.

The primary behavior sources are:

- [MACS3 3.0.4](https://macs3-project.github.io/MACS/), especially
  [`callpeak`](https://macs3-project.github.io/MACS/docs/callpeak.html) and the
  complete
  [subcommand ledger](https://macs3-project.github.io/MACS/docs/subcommands_index.html);
- [SEACR 1.3](https://github.com/FredHutch/SEACR) and its CUT&RUN method;
- [ChIPseeker](https://bioconductor.org/packages/release/bioc/html/ChIPseeker.html)
  `annotatePeak` for nearest-feature and genomic-category behavior;
- [bedtools 2.31 `multicov`](https://bedtools.readthedocs.io/en/latest/content/tools/multicov.html)
  for interval-overlap count behavior;
- SAM/BAM, BED, bedGraph, narrowPeak, broadPeak, gappedPeak, GFF3, and GTF
  specifications.

The family boundary is the user workflow, not one upstream binary. Different
callers remain explicit modes because their statistical models and input
contracts are not interchangeable.

Generic one-dimensional local-maximum detection is not a chromatin peak
workflow. The routed `rsomics-find-peaks` asset has no concrete product
consumer and does not become a public operation in this product.

## Operation map

| Target subcommand | Upstream operation | Decision |
|---|---|---|
| `call` | MACS3 `callpeak` | narrow or broad enrichment calling from treatment and optional control alignments or fragments |
| `call-track` | MACS3 `bdgpeakcall`, `bdgbroadcall` | narrow or broad regions from a continuous score bedGraph |
| `call-sparse` | SEACR 1.3 | control-derived or numeric sparse-signal calling with stringent or relaxed thresholds |
| `call-atac` | MACS3 `hmmratac` | ATAC-specific HMM calling; later slice with its own oracle |
| `model` | MACS3 `predictd` | inspect and report the fragment-length model without calling peaks |
| `refine` | MACS3 `refinepeak` | refine summits from aligned tags and candidate regions |
| `annotate` | ChIPseeker `annotatePeak` | nearest transcript or gene, signed TSS distance, and prioritized genomic category |
| `count` | bedtools `multicov`; DiffBind-style counting | preserve peak identity and emit a multi-sample count matrix plus assignment summary |

MACS3 helper operations are routed by their user-visible contract:

| MACS3 operation | Decision |
|---|---|
| `callvar` | variation calling belongs to `rsomics-vcf` |
| `bdgcmp`, `bdgdiff`, `bdgopt`, `cmbreps` | general track arithmetic belongs to `rsomics-signal`; peak calling may keep private equivalents |
| `pileup` | public track generation belongs to `rsomics-signal`; the exact MACS background model stays private to `call` |
| `filterdup`, `randsample` | alignment filtering and sampling belong to `rsomics-bam`; `call` may apply equivalent private policy during ingestion |

ChIPseeker comparison, GEO retrieval, enrichment analysis, and visualization
are not folded into `annotate`. Statistical comparison belongs to the relevant
analysis workflow, and signal/profile visualization belongs to
`rsomics-signal` or reporting.

### `call` contract

The stable MACS-compatible path covers:

- one or more treatment files and one or more optional controls;
- single-end BAM or SAM, paired-end BAMPE, BED, MACS-style BEDPE, and 10x FRAG
  input, including gzip where the text format supports it;
- explicit input format and safe auto-detection where upstream supports it;
- effective genome size, duplicate policy, barcode selection, model and
  no-model fragment handling, shift and extension, local background windows,
  scaling, cutoff type, minimum length, maximum gap, and summit behavior;
- narrow and broad outputs, model and cutoff reports, and optional treatment
  pileup and control-lambda tracks;
- complete transactional publication of every requested output.

An option is not accepted until its statistical and output effects match the
pinned oracle. Multiple treatment or control files are pooled, never ignored
after the first path.

### `call-sparse` contract

SEACR accepts a nonzero paired-fragment bedGraph plus either an IgG control
bedGraph or a numeric top fraction. Normalization and stringent or relaxed
thresholding are explicit typed choices.

SEACR 1.3's shell pipeline skips an initial data record and fails to flush
certain final accumulated blocks. The historical Rust code deliberately
reproduces those behaviors. Before release, each quirk receives a focused
oracle fixture and one explicit policy:

- exact `seacr-1.3` compatibility with the behavior documented as such; or
- corrected strict behavior under a separately named mode whose output is not
  described as byte-identical to SEACR.

The product never silently selects between bug compatibility and corrected
input handling.

### `annotate` and `count` contracts

`annotate` preserves every input peak row and stable identifier while adding
the requested nearest-feature and category fields. GFF3, GTF, transcript
level, gene level, TSS windows, feature priority, strand, coordinate
conversion, overlapping isoforms, missing contigs, and tie behavior are
explicit.

`count` preserves every peak row, including identical coordinates with
different identities. It emits one column per alignment input and supports
the relevant mapping-quality, flag, strand, fragment, overlap, and paired-read
policies. The result is a peak count matrix, not a duplicated spelling of
every `rsomics-bed multicov` option.

## Data and execution model

- Genomic coordinates are checked half-open intervals. Conversion from GFF3
  or GTF occurs once at a validated boundary.
- Alignment records use validated `rsomics-bamio` readers. Invalid CIGAR,
  reference identifiers, truncated input, header mismatches, and required
  auxiliary-tag failures propagate.
- MACS treatment tags, paired fragments, sparse signal blocks, called peaks,
  annotations, and count rows are distinct types.
- Non-finite signal, score, p-value, q-value, fold change, or normalization
  inputs fail before ordering or threshold selection.
- Required coordinate ordering and continuity are validated rather than
  assumed.
- Named outputs are staged as a set and replace destinations only after the
  complete operation succeeds. Aliasing inputs or sibling outputs is rejected.
- Thread counts reach measured work. A compression worker count is not
  reported as parallel peak calling.
- Execution reports identify algorithm mode, oracle version, input format,
  filters, statistical options, thread counts, output hashes, and deliberate
  compatibility quirks.

## Target structure

```text
src/
├── cli.rs
├── formats/
│   ├── peaks.rs
│   ├── fragments.rs
│   └── tracks.rs
├── call/
│   ├── tags.rs
│   ├── model.rs
│   ├── pileup.rs
│   ├── background.rs
│   ├── statistics.rs
│   ├── narrow.rs
│   └── broad.rs
├── sparse/
│   ├── blocks.rs
│   ├── threshold.rs
│   └── compatibility.rs
├── track_call.rs
├── annotate/
│   ├── model.rs
│   ├── nearest.rs
│   └── category.rs
├── count/
│   ├── assignment.rs
│   └── matrix.rs
├── refine.rs
├── output.rs
└── report.rs
```

`rsomics-common` owns errors, exit mapping, execution reports, aliases, and
multi-output transactions. `rsomics-help` owns the complete command tree.

`rsomics-peak` is a concrete consumer of validated alignment readers and
records from `rsomics-bamio`, and of checked interval geometry and indexing
from `rsomics-intervals`. Consumer tests must cover `call`, `annotate`, and
`count`, not only compile a dependency.

The exact MACS pileup, Poisson tail, q-value, background, broad-region, and
SEACR threshold policies stay inside this product. They do not justify
expanding `rsomics-pileup` or `rsomics-stats` without a second identical
consumer contract.

`rsomics-peak` does not depend on the Layer B `rsomics-annotation` product.
Both products may use the same standards-focused external syntax parser. A
new public annotation-model foundation is considered only after concrete
consumer-side call sites demonstrate a policy-free transcript graph shared by
both products; this audit does not yet justify a tenth foundation.

## Historical asset disposition

All five retired GitHub repositories and crates.io packages are deleted. Their
clean external-disk clones remain the source pool.

| Source asset | Revision | Disposition |
|---|---|---|
| `rsomics-macs` | `f914cdcad4c1ef63ac9930d13257518fd23531e3` | refactor then merge; retain the single-end no-control algorithm, model, goldens, and measurements |
| `rsomics-seacr` | `d6ed2f0890989fa2de2007a1763bed3d51e365b2` | refactor then merge; retain threshold algorithms and all-mode oracle fixtures, but isolate deliberate shell quirks |
| `rsomics-peak-annotate` | `e3435660b4ab6219661ece133b8f7dd52c0203db` | refactor then merge; retain the verified ChIPseeker default semantics and goldens |
| `rsomics-peak-count` | `147ed9b4a3d2d58210d28b8be268ac25bd537b07` | refactor then merge; retain overlap fixtures and the sequential-scan performance hypothesis |
| `rsomics-find-peaks` | `20c65c54462aaf1f92dd86242e8f9e7ff2afb3a7` | discard from this product; retain only as an unpromoted algorithm, test, and benchmark asset if a concrete future consumer appears |

The source code and fixtures in these repositories are team-owned and may be
reused. Upstream behavior and license provenance remain recorded independently.

### Reusable evidence

- MACS: byte-identical narrowPeak and summit outputs on the committed
  single-end no-control fixture, plus larger synthetic no-model and model-path
  comparisons.
- SEACR: all six control/fraction, normalization, and threshold combinations
  compared live against SEACR 1.3 on one fixture.
- Annotation: a committed ChIPseeker 1.46.1 golden covering categories,
  strands, multiple transcripts, equal-coordinate isoforms, wide peaks, and
  downstream behavior.
- Count: a committed bedtools `multicov` golden and an optional live
  differential.
- Generic find-peaks: SciPy value and property goldens remain useful only if a
  real consumer later needs that exact one-dimensional contract.

## Existing implementation gaps

### MACS asset

- It reads only the first treatment path even though the CLI accepts many.
- Control paths are rejected; BAMPE, BED, BEDPE, FRAG, gzip, broad peaks,
  p-value cutoff, local-window controls, scaling, automatic duplicate caps,
  shifts, cutoff analysis, and most MACS outputs are absent.
- Effective genome sizes, q-value cutoff, local window, model parameters, and
  peak gap or length rules are partly hard-coded.
- Output files are created independently and are not transactional.
- Reference length and coordinate narrowing use lossy clamps or casts.
- CI covers only Linux `x86_64`; the product does not use the current unified
  `rsomics-help` layer.

### SEACR asset

- It deliberately skips the first data line, drops the final signal block, and
  drops the final merged peak to mirror shell-pipeline omissions.
- It accepts inverted intervals, overlapping or unsorted records, non-finite
  values, and cross-file chromosome inconsistencies without a complete
  validation contract. Non-finite values can reach partial comparisons that
  unwrap.
- Numeric stringent and relaxed threshold paths share calculations whose
  intended distinction requires a focused source and oracle review.
- Control input is reparsed, output is created directly, JSON discards the
  result, and the live oracle can skip entirely in normal CI.
- The historical performance comparison is dominated by Bash, R, and bedtools
  startup on a 257 KB fixture.

### Annotation asset

- Peak and GTF inputs are loaded wholly into memory.
- Transcript builders are keyed only by transcript identifier, so identifiers
  reused across genes or contigs can merge unrelated models.
- Coordinate-identical and overlapping feature resolution relies on the first
  stored region and needs broader real-annotation tie tests.
- GFF3 graph semantics, alternate attribute conventions, stdin, gzip, headers,
  and streaming output are absent.
- Named output is created before validation and is not transactional.
- The 160-times performance claim compares a complete ChIPseeker TxDb build
  and annotation pipeline with a narrow Rust parser; useful component-level
  attribution is missing.

### Count asset

- It accepts one BAM and emits only four BED-like columns. Multiple samples,
  peak names and extra columns, strand, paired fragments, duplicate policy,
  and full flag filters are absent.
- Counts are keyed only by chromosome and coordinates, so duplicate peak rows
  collapse into one identity.
- Malformed CIGAR operations and missing alignment fields are skipped.
- Reference sequence identifiers index a vector without an explicit checked
  boundary.
- Named output is created directly; threads only configure the BAM reader.
- The 99.6-times historical record needs target-head reproduction with exact
  commands and output hashes.

All four merge candidates contain extensive implementation narration and
inconsistent command plumbing. Their algorithms and evidence are inputs; their
file structures are not the target structure.

## Retained performance evidence

The historical figures are migration baselines, not release claims:

- MACS single-end no-control: about 9.8 times faster and 2.9 times lower peak
  RSS than MACS3 3.0.4 on a 400,000-read, 20 Mb synthetic fixture.
- SEACR: a 4,000-block correctness fixture and a separate 10,000-line
  measurement show large wall-time differences, but the oracle is dominated
  by interpreter and shell startup.
- Annotation: 20,000 synthetic peaks and 2,000 genes were value-exact to
  ChIPseeker 1.46.1; the reported 160-times wall and 88-times RSS ratios include
  ChIPseeker's TxDb construction.
- Count: a 188 MB BAM and 20,000 regions produced a 99.6-times historical
  advantage over bedtools 2.30 `multicov`, but the wrapper revision was dirty
  and the fixture and exact output evidence are not retained in the target.

Each result must be reproduced after consolidation. Startup-heavy comparisons
do not prove a hot-path advantage.

## First release slice

The first release is a complete common chromatin-peak workflow:

- `call` for narrow MACS-compatible peaks from one or more single-end or
  paired-end BAM treatment files, optional pooled controls, model or explicit
  fragment handling, duplicate policy, p- or q-value cutoff, pileup and local
  background, narrowPeak, summits, and optional bedGraph;
- `call-sparse` for all six SEACR 1.3 control/fraction, normalization, and
  stringent/relaxed combinations, with an explicit compatibility-quirk policy;
- `annotate` for the pinned ChIPseeker default transcript contract, strict
  GTF/GFF3 input, configurable TSS window, stable peak identity, and
  transactional output;
- `count` for multiple BAM inputs, stable peak rows, mapping and flag filters,
  paired-fragment policy, one matrix, and a complete summary;
- unified help, structured execution reports, strict failures, transactional
  outputs, and all four native CI target classes.

Broad MACS peaks, FRAG barcode selection, `call-track`, `call-atac`, `model`,
and `refine` are later slices. They stay absent from help until each complete
operation passes its own compatibility and performance gates.

## Compatibility gates

- Pin MACS3 3.0.4, SEACR 1.3, ChIPseeker 1.46.1 for the retained annotation
  contract, and bedtools 2.31 initially. Review current upstream changes before
  publication.
- Compare complete MACS output sets for single and pooled treatments, with and
  without controls, single-end and paired-end input, model and no-model paths,
  duplicate modes, p- and q-value cutoffs, and track output.
- Cover no peaks, one peak, contig ends, equal summits, broad candidates,
  secondary and supplementary reads, duplicates, QC failure, missing mates,
  invalid CIGAR, inconsistent headers, truncated input, and output aliases.
- Run SEACR live differentials for all six mode combinations. Add focused
  fixtures for the first record, final block, final merge, zero values,
  non-finite values, sorting, overlaps, chromosome changes, ties, empty input,
  and normalization degeneracy.
- Compare every annotation field across both strands, isoforms, equal-distance
  ties, repeated transcript identifiers, GFF3 and GTF, coding and noncoding
  transcripts, UTR derivation, missing contigs, and configurable feature
  priority.
- Compare every peak row and sample column for `count`, including duplicate
  coordinates, extra BED columns, zero counts, overlaps, split reads, pairs,
  MAPQ boundaries, flags, and unsorted or unindexed inputs.
- Frozen, provenance-recorded goldens run on native Linux and macOS for
  `x86_64` and `aarch64`. Live oracle jobs fail if an expected oracle is
  unavailable rather than reporting a skipped compatibility gate as success.

## Performance gates

- MACS uses retained realistic or reproducibly generated treatment and control
  BAMs with millions of reads, narrow signal, single and paired fragments, and
  enough output to dominate startup. Broad workloads join this gate when that
  later slice is implemented.
- SEACR uses large sparse and dense bedGraph pairs. Record parser, block
  construction, threshold computation, and output separately from process
  startup.
- Annotation measures model construction and repeated annotation separately,
  then compares the complete one-shot workflow. Use a real large gene model
  and peak set in addition to synthetic edge cases.
- Count compares sequential scan and indexed interval-query strategies across
  sparse and dense peak sets, one and multiple BAMs, single reads and paired
  fragments.
- Record wall distributions, total CPU, peak RSS, output size and hashes,
  input hashes, exact revisions, commands, warmups, thread counts, machine, and
  storage.
- Each stable hot path must be strictly faster than its relevant upstream
  oracle or demonstrate a material resource or composed-workflow benefit.

## License and attribution

The retained Rust code and fixtures are team-owned and remain MIT OR
Apache-2.0.

MACS3 is BSD-3-Clause, ChIPseeker is Artistic-2.0, bedtools is MIT, and the
current SEACR repository declares GPL-2.0. The historical SEACR README and help
incorrectly say GPL-3.0 and must be corrected. No upstream code contamination
is presumed, but every source-informed algorithm retains accurate project,
version, method, and license provenance before publication.

## Explicit exclusions

- No publication under any of the five retired micro-crate names.
- No generic SciPy-style `find-peaks` command without a concrete
  bioinformatics product consumer.
- No silent treatment/control truncation, malformed alignment, invalid
  interval, unsorted track, non-finite score, or missing-contig handling.
- No parsed but unenforced statistical, filtering, format, or thread option.
- No unlabelled SEACR bug compatibility.
- No direct dependency on another Layer B product.
- No speculative public peak, annotation-model, pileup, statistics, or signal
  foundation.
- No broad, HMMRATAC, track-calling, model-only, or refinement claim in the
  first release.
