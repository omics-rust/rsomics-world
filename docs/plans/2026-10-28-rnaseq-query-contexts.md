# RSeQC query-local pileup contexts

## Demonstrated requirement

The first real whole-report comparison exposed shared-library overlap-state
defects, now corrected at rsomics-pileup1bc989e (0.9.1, unpublished). Its
exact-head four-native CI40579417757 and local27 units/three actualsamtools
modes pass. A repaired complete consumer removes the original13 TIN/18-bin
differences but exposes two other TIN rows and six positive coverage bins.

Actual original-data source/native evidence at
`/Volumes/KIOXIA/Developments/tmp/rnaseq-qc-remaining-chr10-2_vxlhi5/` proves
the remaining two TIN differences are query context, not another library
projection error. For NM_001321027.1 and NM_205845.2, native on each original
query subset equals retained RSeQC exactly. Actualsamtools1.24 on a whole chr10
stream equals the global native result instead. The only sampled depths differ
at5030063/5030072: query1/global2. The first diagnostic used the generator's
shorter final insertion-order point as its stop; that experiment is preserved,
but it is not the executed TIN range contract. Actual process_bam sorts the
picked positions before calling genebody_coverage; confirm sorted min/max
query bounds in the implementation and corrected diagnostic. The original
full source outputs and failed native outputs remain preserved.

## Implementation ruling

Preserve the CLI, no-index input contract, one physical BAM read, all metric
and numerical comparison contracts, and the fast canonical global profile.
For explicit rseqc-5.0.5, give each source query its own pileup context over
original unmodified records. A global adjusted-depth map is not a faithful
substitute when read-name cache history differs. This is product-local source
policy; do not alter the now-correct shared engine or add public foundation
items to disguise the discrepancy. No tolerance or input-cohort relaxation.

Buffer/reuse records once across active annotation contexts on external-memory
paths only if needed, without requiring a BAM index or rereading input. Use
shared owned records/references internally rather than duplicating raw bytes
for every transcript. Retire completed contexts/records to bound memory; avoid
holding the full whole genome just for convenience. Contexts with identical
reference/start/stop can share one replay with union sample points. Start with
clear correct code, then measure the actual whole workflow before claiming
performance or further optimizing.

Read actual pinned source definitions before framing:
geneBody_coverage.py genebody_coverage/percentiles and tin.py
genomic_positions/genebody_coverage. Plain source files are under
`/Volumes/KIOXIA/Developments/tmp/rnaseq-qc-source-20261027.tH3KY1/`.
The wheel is retained in actual source run40520155303/acquisition. Source
coverage queries positions[0]-1..positions[-1], unique points then genomic
sort and strand reversal. TIN process_bam sorts picked_positions before the
pileup call, so query bounds use the sorted minimum/maximum. Its denominator
remains len(picked_positions), including the existing short-transcript duplicate
denominator behavior; do not replace it with the deduplicated point count.
Preserve coverage identity last-winner and every TIN row/eligibility rule.

Per-context replay must clone unadjusted originals into PileupEngine; never
reuse qualities already mutated by another context. Original-record mapping,
strandedness, distribution and start-eligibility observations still happen once.
Use bounded deterministic product-owned parallel query work through --threads;
update its help to describe actual worker usage. Do not advertise scaling until
the whole report is measured. Errors still propagate and output stays atomic.

## Acceptance

Retain one compact workflow regression that distinguishes window/global
contexts, existing9 consumer tests and actual first single/paired source
comparisons. Then compare the complete original9810963-record report at1/4
workers against original retained source-warmup metrics with unchanged bounds,
all58384 TIN rows and100 coverage points. Worker report bytes must agree;
manifest still reports one BAM traversal. This is integrated validation, not a
new per-helper oracle/review campaign. Keep failures. Representative CPU/RSS/I/O,
four-native exact-head gate and publication remain separate work.
