# Methyl product dossier

Status: source and upstream-operation audit complete. The target repository has
not been created.

## Boundary

`rsomics-methyl` is one bisulfite-sequencing methylation extraction and bias-QC
product. It converts coordinate-sorted alignments and an indexed reference into
per-site, per-context, per-read, and positional-bias methylation evidence.

The primary behavior source is
[MethylDackel 0.6.1](https://github.com/dpryan79/MethylDackel), including its
MIT-licensed implementation and installed command-line help. The public SAM,
BAM, CRAM, FASTA, BED, bedGraph, and BigWig contracts define the surrounding
formats.

This boundary does not include differential methylation, DMR calling,
segmentation, imputation, array normalization, or epigenomic visualization.
Those are separate statistical and signal workflows.

## Upstream operation map

| Target subcommand | MethylDackel operation | Decision |
|---|---|---|
| `extract` | `extract` | CpG, CHG, and CHH calls with filters, context merging, region selection, and alternative output forms |
| `mbias` | `mbias` | read-position methylation bias metrics, inclusion suggestions, TSV, and SVG |
| `merge-context` | `mergeContext` | merge sorted per-cytosine CpG or CHG metrics from a file or stdin |
| `per-read` | `perRead` | informative CpG count and methylation fraction per alignment record |

`--mergeContext` inside extraction and the standalone `mergeContext` operation
share one context-merging engine. The operation is not split into a fifth
public crate.

### Extract contract

The complete operation map includes:

- indexed BAM and CRAM, coordinate regions, BED inclusion, and BED strand
  filtering;
- MAPQ, base quality, SAM ignore/require flags, duplicates, singletons,
  discordant pairs, NH multi-mapping, and thread controls;
- CpG, CHG, CHH, and merged CpG/CHG contexts;
- OT, OB, CTOT, and CTOB inclusion and fixed-end trimming bounds;
- conversion-efficiency and opposite-strand variant filters;
- paired-overlap quality adjustment without double counting;
- percentage/count output, fraction, total counts, logit, methylKit, and
  exhaustive cytosine report;
- minimum depth and region-aware transactional output;
- optional BigWig or BBM mappability filtering.

Mappability and BBM support remain a later slice until their I/O and retained
format value are justified. No public `rsomics-bbi` crate is recreated for a
single consumer.

## Calling model

- Reference bases and coordinates use checked types. BAM/CRAM header sequence
  names and lengths must agree with the indexed reference for every processed
  contig.
- Input coordinate order is validated. A contig cannot reappear after it has
  been finalized, and positions cannot move backward within a contig.
- Bisulfite strand is an enum with OT, OB, CTOT, CTOB, and undetermined states.
  Aligner tags, read number, orientation, and single-end fallback are tested
  independently.
- CpG, CHG, and CHH classification is explicit at contig boundaries and for
  ambiguous reference bases.
- CIGAR projection, base and mapping quality, pairing, overlap adjustment,
  region selection, trimming, conversion efficiency, and variant filtering
  produce one typed call decision.
- Overlapping mates contribute at most one call per molecule and position.
  Agreement and disagreement quality transformations are overflow-safe and
  compatibility-tested.
- Counts use a width that cannot overflow on realistic deep coverage. The
  calling path is sparse or chunked; memory is not proportional to an entire
  chromosome's base count.
- Missing reference contigs, truncated records, invalid CIGAR, malformed aux
  tags needed by the selected protocol, and header mismatches fail loudly.
- All requested context and auxiliary outputs stage successfully before any
  final path is replaced.

## Target structure

```text
src/
├── cli.rs
├── reference.rs
├── context.rs
├── strand.rs
├── calling/
│   ├── projection.rs
│   ├── overlap.rs
│   ├── filters.rs
│   └── metrics.rs
├── extract/
│   ├── window.rs
│   ├── region.rs
│   └── output.rs
├── mbias/
│   ├── metrics.rs
│   ├── bounds.rs
│   └── render.rs
├── merge_context.rs
├── per_read.rs
└── report.rs
```

`rsomics-common` owns errors, exit mapping, execution reports, aliases, and
multi-output transactions. `rsomics-help` owns the complete command tree.

`rsomics-methyl` is a concrete consumer of validated BAM/CRAM readers and
records from `rsomics-bamio`. It is also a concrete driver for
`rsomics-pileup`: sortedness validation, checked CIGAR projection,
low-allocation column views, and generic overlapping-mate evidence are shared
with BAM and VCF products. Bisulfite strand, cytosine context, conversion,
methylation calls, bias policy, and output formats stay inside this product.

Indexed FASTA access may use the aligned external noodles implementation.
`rsomics-seqio` is not expanded from streaming FASTA/FASTQ into speculative
random-access reference policy.

## Historical asset disposition

The one routed source candidate is the clean `rsomics-methyldackel` repository
at `9d32057f7ec5eb8bb241d53c115280f9d6acbdea`.

| Asset | Disposition |
|---|---|
| bisulfite strand classification | refactor then merge as an enum with aligner-tag goldens |
| CpG calling and overlap-quality algorithms | direct algorithm merge after checked-coordinate and pileup adaptation |
| SAM flag filters | refactor then merge; replace raw bit constants with validated record APIs |
| CIGAR-to-reference projection | tests and behavior asset; use the shared checked pileup contract |
| active mate window | refactor then merge only if shared pileup overlap handling does not supersede it |
| full-reference loader and per-contig counts vector | discard; replace with indexed reference access and sparse or chunked accumulation |
| output writer | tests and formatting asset; replace direct file creation with transactional context outputs |
| synthetic BAM, reference, index, and golden bedGraph | retain and expand |
| Criterion subprocess benchmark | recipe only; the tiny fixture is not representative |
| duplicated `HelpSpec`, inherited `Tool`, and narrative comments | discard during current common/help migration |

MethylDackel is MIT licensed, so its implementation may be read and adapted
with attribution. The historical repository already records the directly
consulted upstream modules.

## Existing implementation gaps

The historical implementation covers a narrow `extract` default and must not
be published as the target product.

- It allocates one `PosCounts` value per base of the current contig. Despite
  comments describing bounded memory, a large human chromosome can require
  multiple gigabytes before active reads are counted.
- It assumes coordinate sorting but never validates contig or position order.
- A reference contig missing from FASTA silently drops all of its reads.
- Reference and BAM header lengths are clamped to the smaller value instead of
  reporting a mismatch.
- Header sequence identifiers are used for unchecked vector indexing.
- The entire FASTA is loaded despite requiring an index in help text; the FAI
  is not used.
- Only CpG standard bedGraph output is implemented. CHG, CHH, merged context,
  bias, per-read, alternative outputs, trimming, conversion, variant,
  mappability, region, NH, and CRAM behavior are absent.
- Threads control BGZF workers only; the calling path itself is not parallel,
  and help does not accurately expose the execution contract.
- Output is created before alignment validation and is not transactional.
- The live compatibility oracle may skip, and the frozen fixture exercises
  only one tiny reference and MAPQ difference.
- The source uses unchecked integer casts, suppresses Clippy cast lints, and
  contains extensive code narration.
- CI covers only Linux `x86_64`.

## Retained evidence

The committed synthetic fixture has an authentic MethylDackel 0.6.1 CpG
bedGraph golden and an optional live differential. A separate historical
reverification reports byte-identical data lines on a 92 MB, four-million-read
BAM producing about 2.5 million CpG rows.

The useful historical performance baseline on that larger input is about 1.59
times at one thread. A later 4.43-times record uses an 839 KB, 50,000-read
fixture with 31 versus 139 millisecond means and is too small to define a
release claim. Neither large fixture is retained in the current fixture store,
so both results are migration evidence that must be reproduced.

## First release slice

The first release is the coherent extraction and QC workflow:

- `extract` for BAM and CRAM with CpG, CHG, and CHH contexts;
- standard, merged-context, fraction, counts, logit, methylKit, and cytosine
  report outputs;
- MAPQ, base-quality, flags, duplicate, singleton, discordant, NH,
  conversion-efficiency, variant, trimming, region, and BED filters;
- `mbias` TSV, suggested bounds, and SVG;
- `merge-context` from a path or stdin;
- `per-read` without the upstream's documented 10 kb span limitation;
- indexed reference access, strict order and header validation, sparse or
  chunked memory, transactions, unified help, and execution reports.

Mappability BigWig and BBM input is deferred. It remains absent from help until
the private I/O implementation or a justified shared format contract passes
compatibility, memory, and throughput gates.

## Compatibility gates

- Pin MethylDackel 0.6.1 and its HTSlib version for the initial oracle. Review
  current master changes before publication rather than assuming the 2021 tag
  is the complete modern contract.
- Run live differentials for all four subcommands on Linux and macOS. Frozen,
  provenance-recorded goldens run on all four native target classes.
- Cover CpG, CHG, CHH, merged contexts, contig edges, ambiguous reference
  bases, zero and minimum depth, percentage rounding, fraction, counts, logit,
  methylKit, and exhaustive cytosine reports.
- Cover OT, OB, CTOT, CTOB, undetermined strands, Bismark tags, missing or
  malformed tags, single and paired reads, both mates, overlapping agreement
  and disagreement, equal and unequal qualities, and clipping.
- Cover every reference-consuming CIGAR operation, invalid encodings,
  supplementary and secondary records, duplicates, QC failure, singletons,
  discordant pairs, NH values, conversion thresholds, and likely variants.
- Cover regions and BED selections at exact boundaries, strand-aware BED,
  contig aliases, missing contigs, header-length mismatch, out-of-order input,
  truncated BAM/CRAM, output aliases, and interrupted multi-output writes.
- Compare `mbias` numeric metrics and suggested bounds before SVG bytes.
  Rendering receives deterministic structural or image regression tests.
- Compare `per-read` fragments below and above 10 kb and document the deliberate
  improvement over the upstream limitation.

## Performance gates

- Use a retained or reproducibly generated WGBS-like input with at least four
  million reads and enough output to dominate startup and filesystem noise.
- Add an RRBS-like sparse-coverage input, a high-depth targeted input, long
  contigs, overlapping pairs, all contexts, and CRAM.
- Compare direct binaries at equal thread counts, with identical regions,
  filters, contexts, overlap behavior, and output bytes.
- Record index and reference-cache setup separately from calling. Report wall
  distribution, CPU time, peak RSS, output size and hashes, and thread scaling.
- Assert that peak live accumulation memory follows active depth or chunk size,
  not reference contig length.
- Benchmark `extract`, `mbias`, `merge-context`, and `per-read` separately.
- Record exact revisions, binary hashes, commands, generator and fixture
  hashes, warmups, and machine details.
- The stable extraction hot path must be strictly faster or use materially
  fewer resources than MethylDackel.

## License and attribution

The retained Rust code and synthetic fixtures are team-owned and remain MIT OR
Apache-2.0. MethylDackel is MIT licensed. Adapted algorithms retain its
copyright notice and attribution at the appropriate source or product level.
MethylDackel 0.6.1, HTSlib, relevant format specifications, and any directly
consulted upstream modules are recorded with compatibility results.

## Explicit exclusions

- No publication under the retired `rsomics-methyldackel` name.
- No differential methylation, DMR, segmentation, array, or imputation
  workflow.
- No silent missing contig, reference mismatch, malformed record, or unsorted
  input handling.
- No whole-chromosome per-base allocation.
- No parsed but unenforced context, strand, filter, or thread option.
- No speculative public methylation, reference, or BigWig foundation.
- No direct dependency on another Layer B product.
- No mappability or BBM claim until that complete later slice passes.
