# Transcript and gene quantification

Status: product-operation survey. The authoritative counting boundary and
release evidence are in the [`rsomics-count` dossier](../10-products/count.md)
and [recorded first-release gate](../10-products/count-gate-2026-08-02.md).
The command surface below was checked against owning source
`5bc764d8af75171c4ac2e4a9e4a153d0e2d05b49`. Publication and performance results
are the gate's historical accepted evidence, not a fresh registry check.

## Workflow boundaries

| Workflow | Current product decision |
|---|---|
| Assign genomic reads or fragments to annotated features | `rsomics-count [options] <alignments>...` |
| Derive TPM, FPKM, or FPKM-UQ from supplied gene counts and lengths | `rsomics-count normalize` |
| Infer transcript abundance from reads or transcriptome alignments | upstream scope; not an implemented count-product operation |

Genomic feature counting and transcript-abundance inference are different
contracts. The count product emits one multi-library matrix and assignment
summary; its normalization arithmetic does not infer transcript compatibility,
effective lengths, or transcript abundance. Differential-expression fitting
belongs to the [stateful expression products](differential-expression.md).

## Implemented counting and normalization

The current `rsomics-count 0.1.0` source exposes one product binary, with
default counting and a `normalize` subcommand:

```console
rsomics-count sample.bam -a genes.gtf -o counts.tsv
rsomics-count normalize counts.tsv -o tpm.tsv --mode tpm
```

The implemented counting slice follows the supported featureCounts 2.1.1
contract:

- SAM and BAM libraries, including stdin for a single library;
- GTF, GFF3, or SAF annotations, gzip detection, selected feature types and
  grouping attributes, and feature or meta-feature counting;
- multiple libraries in one matrix with complete assignment summaries;
- single reads or paired fragments and unstranded, sense, or antisense modes;
- exclusion, full, or fractional contributions for multi-overlap and
  multi-mapping, with minimum-base and fractional-overlap thresholds;
- MAPQ, primary, duplicate, split, mate, chimera, and fragment-length filters;
- chromosome aliases and bounded parallel counting;
- transactional outputs and nonzero exits for parse, annotation, alignment,
  pairing, normalization, and I/O failures.

`normalize` accepts a strict gene-by-sample matrix with validated positive
lengths and computes the selected TPM, FPKM, or FPKM-UQ arithmetic. It is not
an alternative transcript quantifier or a differential-expression method.

CRAM, read-position projection, junction counts, read-group stratification,
long-read mode, per-record assignments, largest-overlap selection, non-overlap
limits, and extra annotation output attributes are not implemented in this
release. Planned dossier options are not current CLI promises.

HTSeq union, intersection-strict, and intersection-nonempty semantics are not
implemented or advertised as a counting mode. There is no `htseq-strict`
flag; `--mode` belongs to normalization. A future HTSeq-compatible slice would
need its own complete semantic and oracle contract inside the count product,
not another operation-sized public crate.

## Compatibility and measured resource use

The [recorded gate](../10-products/count-gate-2026-08-02.md) retains frozen and
live featureCounts 2.1.1 comparisons, failure-atomicity coverage, and exact-head
four-native-target CI. Compatibility is limited to the declared slice, not
complete featureCounts or HTSeq parity.

The recorded Apple M2 benchmarks used two million single-end reads or one
million paired fragments. The measured implementation was slower than
featureCounts: single-read wall time was 1.67 times at one thread and 2.52
times at four threads. Four-thread single and paired cases used about 87%
less peak RSS. That resource advantage, not a throughput advantage, supported
the first release; these measurements remain bound to the gate's exact
implementation and binary.

## Wider upstream scopes

These official upstream links are retained from the historical survey as
scope references, not a current inventory of versions, implementation
languages, FFI dependencies, registry availability, or adoption decisions.

| Upstream reference | Scope to evaluate separately |
|---|---|
| [HTSeq](https://github.com/htseq/htseq) | alternative read-to-feature assignment semantics |
| [Salmon](https://github.com/COMBINE-lab/salmon) and [kallisto](https://github.com/pachterlab/kallisto) | transcript-abundance inference from reads |
| [RSEM](https://github.com/deweylab/RSEM) | transcript-abundance inference from transcriptome alignments |
| [StringTie](https://github.com/gpertea/stringtie) | transcript assembly and abundance; see [assembly survey](assembly-isoform.md) |
| [oarfish](https://github.com/COMBINE-lab/oarfish) and [piscem](https://github.com/COMBINE-lab/piscem) | upstream quantification or mapping/indexing adoption candidates |

These scopes and adoption candidates do not authorize additional public
`rsomics-*` installation identities, dependencies, or shared-foundation APIs.
Any later implementation or adoption decision must establish the actual
workflow, source, compatibility, license, and performance contract first.
