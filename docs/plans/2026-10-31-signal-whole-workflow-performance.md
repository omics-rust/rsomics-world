# Signal whole-workflow performance contract

## Scope and evidence

Adopted before preparing performance derivatives or observing signal timings.
This authorizes deterministic corpus preparation and functional qualification;
timing follows only after the corrected native candidate, same-file source
interoperability and resource budget qualify. No performance result is claimed.

The complete read-only proposal is retained externally at
`/Volumes/KIOXIA/Developments/tmp/signal-performance-contract-20261031.80CKBD/`:

- REPORT.md: `2ce56c5062492efe84a2954d8cb63260d0280fe4c0da25cce746b17e5790ae22`;
- evidence-bindings.json: `51ff199d1af320360fc331c0beb74f94cacb067d7b17cfc2cf4e0e007000a02d`.

Parent reads both completely, reads the retained qualification and freshly
checks all 11 small-file bindings. Large BAM/archive hashes below are the
previously verified retained identities, not a new whole-file hash this turn.

## Corpus and preparation

Reuse the existing RSeQC tutorial evidence under
`/Volumes/Zane's HDD/rsomics-fixtures/evidence/rnaseq-qc-real-performance-2026-10-28/run-40592936910/`.

| Input under artifact/benchmark/inputs | Bytes | SHA-256 |
| --- | ---: | --- |
| paired.bam | 437662663 | `38174082a8f36ae76a6852b9b970fcb334ab5c8186c133993a0fd652d1d147cd` |
| paired.bam.bai | 2723112 | `05076af3c06906d3ecdf1c448e0f0b5b2d4e1243a3d63f585c004d772f782957` |
| genes.bed12 | 9018094 | `5132086afd5b33bb4d5b2724ef5c4e8275aaba7854a7ec5af0d1902171b88ada` |

This is the unchanged whole public BAM: 9,810,963 mapped paired records,
6,959,152 proper-pair records, 25 hg19 references and real splicing. The original
has 36-base reads and no duplicate FLAGs; positional duplicates remain to be
characterized. It does not cover modern long reads or insertion/deletion-rich
alignments. Existing other full copies are the same specimen, not new samples.

Make exactly three engineering partitions in one streaming pass. For exact
QNAME bytes, bucket is the first eight SHA-256 digest bytes interpreted as
unsigned big-endian, modulo three. Hash the bytes
`rsomics-signal-perf-v1`, then one NUL, then QNAME. Preserve all records sharing
a QNAME, the full header and record payloads, original within-bucket order and
all records exactly once. Do not strip mate suffixes or sample individual
records. Require at least two million mapped records per partition; freeze
actual populations, coordinate checks, exact-QNAME membership, record-multiset
union, files and indexes before timing.

Split the existing qualified BED12 by strand, preserving all twelve columns,
order and duplicate names: plus 29,903 rows and minus 28,481. Together these are
58,384 rows, including 54,030 multi-exon rows and 62 duplicate-name rows.
Keep the existing 2,793 absent-reference exclusions and original annotation
provenance. Do not duplicate transcripts to inflate workload. Require at least
50,000 retained rows in each final performance matrix.

No new download is needed. The retained public access receipts do not establish
a dataset redistribution license; raw data must not enter a public repository
or package. Existing software licensing does not imply a data license.

## Connected useful outputs

Use independent source/native paths and real intermediates throughout. Source
is the unmodified pinned deepTools3.5.6 runtime. Native is the exact accepted
release candidate, with binary, source and lock identities frozen before trials.
Run the same graph at one and four workers:

1. Filter each partition at MAPQ20, exclude FLAG2816 and positional deduplication;
   retain passed/rejected BAMs and metrics. No extension, centering or RNA-strand
   selection in the main graph.
2. Explicitly sort and index every passed BAM with the same qualified pinned
   samtools1.21 or pysam1.21 dispatcher and worker budget. Include this necessary
   work and dispatcher startup in both end-to-end timers.
3. Produce three bin50 CPM bigWigs from those sorted BAMs.
4. BAM0/BAM1 compare ratio with readCount scaling and pseudocount1:1;
   track0/track1 BW compare ratio with scales1:1 and pseudocount1:1;
   three-track average with scales1:0.5:2. All use bin50.
5. Summarize the three BAMs by bins10,000 and exon-aware BED12, retaining NPZ,
   raw counts and scaling factors. Summarize the three arithmetic BWs by the
   same bins and annotation with default approximate statistics, NPZ/raw counts.
6. Feed the three actual CPM tracks and both annotation groups to metagene TSS
   matrices (before500/after1500/bin50) and scale matrices
   (body3000/before500/after500/bin50/unscaled ends0). Keep order, default mean,
   normal missing cells, no skipZeros or threshold selection. Emit gzip,
   matrix TSV and sorted BED from equivalent decisions.

Keep the optional sorted-BED policy decision separate. It does not authorize
inventing source-compatible invalid BED or changing matrix cohorts. If that
choice remains open, prepare and qualify the numerical graph without claiming
the optional export accepted.

Two separately reported useful branches use the whole original BAM: bin20 CPM
track; and proper-pair filter plus ATAC shift, followed by timed explicit
sort/index and MACS fragment BED. The latter stresses a transform on RNA-seq;
it is not evidence that this specimen is an ATAC experiment. Record actual
normalization, retained/rejected and shift-drop populations.

## Correctness and resource qualification

Before timing, freeze all inputs, flags, dictionaries, identities, source/native
builds, comparison bands and named incompatibilities. Use the adopted native
plan's exact categorical/nonfinite rules and fixed numeric bands. Validate
every trial's products, including approved corrected zeros and signed outputs;
do not patch source, delete corrected native data or widen bounds to pass.
Default statistics remain approximate/default, never silently exact.

An additional untimed lane lets actual pinned pyBigWig/deepTools read the same
native-produced BWs, including default zoom and exact diagnostic queries.
Original imported BWs are only a separately labelled compatibility lane, not
substitutes for native-connected intermediates. A source writer applied to a
corrected run stream is likewise diagnostic, not a replacement input.

The bin50 graph spans about 61.9 million possible tiles; the separate bin20
case about 154.8 million. Upper raw matrix payloads are about 56MB and 112MB,
before metadata/compression. Do not allocate chromosome-sized per-base arrays
or estimate runtime/scratch from compressed BAM size alone.

Qualify actual source/native output and task-spool sizes first. Campaign limits:
16GiB live-tree memory, one active benchmark workflow, 160GiB combined output
and scratch, at least 32GiB free external headroom. Per-trial retained budgets
are 5GiB for the main graph and 1GiB for each additional branch. These are
qualification/abort bounds, not measured sizes. If exceeded, stop that action
and revise the workload explicitly before timing; do not silently discard
outputs, delete evidence or turn an OOM into a speed result.

## Measurement and decision

Use one otherwise-idle host with four fixed allocated logical CPUs: one of
those CPUs for T1 and all four for T4. Bind hidden source OMP/BLAS/NumExpr pools
to one. Record host, affinity, versions, input hashes and real worker settings.
One complete warmup per tool/worker setting, then five measured source/native
pairs with predeclared order S/N, N/S, S/N, N/S, S/N. No concurrent trials.
This is warm-cache evidence, not a cold-I/O claim.

Retain all wall/user/system samples, exits, process scope, peak RSS, faults,
raw filesystem block counters and actual available I/O byte counters. Normalize
Linux KiB versus macOS byte RSS explicitly. Maximum individual-process RSS is
not aggregate multiprocess memory; only claim matched-scope resource advantage.
Missing counters are unavailable, not zero. Retain complete useful outputs,
hashes and byte counts. Stable-storage durability requires an identical timed
publication/fsync barrier; otherwise do not make that claim.

Functional/source correspondence is mandatory. The fixed strict criterion is
native maximum wall below source minimum wall, or strict peak-resource range
separation with genuinely matched scope. Report all five samples, medians and
ranges, each worker setting, full graph and per-stage distributions. The graph
must have a material strict advantage. A losing replacement hot path remains
an optimization gate; do not hide it by averaging over faster operations.
Four-platform correctness and existing RSeQC performance are separate gates.
