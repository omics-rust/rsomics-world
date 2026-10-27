# Real RNA-seq shared-report measurement

The accepted semantic candidate is green4, four-native run40546148508 at
world1224fbf18d7af1b4b76869fc6e41b1611d7bb43b. Measure that retained Linux
binary, not a fresh unqualified build. This plan does not accept a performance
result before the real complete report agrees with the actual source calls.

## Inputs and scope

Use the official RSeQC tutorial's paired, non-strand-specific human hg19,
36nt TopHat1.2.0 BAM. Its complete original is437662663bytes with published
MD5ba014f6b397b8a29c456b744237a12de:

<https://data.cyverse.org/dav-anon/iplant/home/liguow/RSeQC/Pairend_nonStrandSpecific_36mer_Human_hg19.bam>

The official hg19 RefSeq BED gzip is2567192bytes, SHA-256
eb7070cfb52b9011c132346fd27baffb8a7acf3c034ebf996cebd52fbb52aa19,
MD5b7df344f3787415eaa4f24c23f498632:

<https://sourceforge.net/projects/rseqc/files/BED/Human_Homo_sapiens/hg19_RefSeq.bed.gz/download>

Bounded external-disk qualification retained the original1MiB HTTP206 prefix,
the original BED gzip and metadata in
`/Volumes/KIOXIA/Developments/tmp/rnaseq-qc-chr21-qualify-20261028.egJWCT/`.
The BAM has25 references. Of61177 valid BED rows,2793 are on23 references
absent from this BAM. Do not alias contigs or silently drop annotation rows.

Predeclare two separately hashed profiles before observing native results:

1. `chr21`: every chr21 BAM record and every original chr21 annotation row,
   preserving original order, flags, pairing fields and coordinates. The
   annotation has754 transcripts,737 at least100nt,673 multi-exon, both strands.
   The complete BAM record count and observed eligibility remain to be measured.
2. `whole`: the complete, unchanged original BAM with all58384 annotation rows
   whose exact reference name is present in its header. Retain an explicit
   exclusion ledger containing every absent-reference row and reason. Both
   implementations receive the same derived BED. No read filtering or annotation
   deduplication is permitted during acquisition.

Generate required source BAM indexes before timing; the native report must not
require one. Keep originals and derivatives, size/SHA-256/MD5, HTTP receipt,
header, record/flag/CIGAR/length counts and annotation row identities. Acquisition,
decompression, indexing, input scans and source eligibility tracing are untimed.
The first campaign may use chr21 to establish a nontrivial real-data contract;
it cannot substitute for the separately required whole-input measurement.

The first chr21 campaign40556139574 acquires and fully verifies the complete
original BAM and BED gzip. Subsequent profiles use that immutable Actions
artifact12758010802, independently API/ZIP-verified and rehashed before use,
instead of requesting the public data endpoint again. That endpoint returned
an IP-unblock/Turnstile HTML page to first whole run40556744875; no challenge
is handled or bypassed. The failed response and original receipt are retained.
The source/input provenance chain remains explicit. Avoid uploading duplicate
cache ZIP/extractions and BAM hardlink aliases: each new artifact retains its
actual measured BAM/BAI, original BED gzip, derived annotation, complete source
and native outputs, cache API/verification and original HTTP receipts. The
first input artifact and verified external copy retain the complete originals.

## Whole operation

Use exact installed RSeQC5.0.5 script bodies and the dependency versions from
accepted oracle40520155303. Preserve the wheel/source acquisition and installed
identity comparison. Run the real five public scripts serially on one input:
mapping, strandedness, distribution, gene-body coverage with plotting disabled,
and TIN without background subtraction. Source startup, annotation loading,
repeated input work and numeric report output are included. No source algorithm
replacement, cached per-transcript result or intercepted return value is used.

Compare with one `rsomics-rnaseq-qc report --profile rseqc-5.0.5` call at
1 and4 inflate workers. Defaults are MAPQ30, first200000 eligible strandedness
records, minimum mRNA100nt, TIN strictly more than10 starts and100 samples.
Use the accepted complete-report comparator for exact counts, source-rounded
fractions/densities, ordered TIN rows/summary and all100 coverage bins. A separate
untimed call to the actual source's check_min_reads records eligible transcript
membership, including zero scores and duplicate names. Do not infer eligibility
from a positive TIN value. Verify report/manifests and exact worker equality.

Run the complete functional comparison before collecting repeated measurements.
Any disagreement fails that profile and prevents performance acceptance; retain
the original outputs for diagnosis instead of fitting tolerances or filtering
the input after observing results. Global versus restarted depth admission
remains an explicit unproved boundary unless the actual data prove otherwise.

## Measurement and decision

Use one otherwise idle Linux native GitHub runner with a fixed four-CPU affinity
and record CPU model, available CPUs, OS, runtimes, binary/source/input hashes,
flags and storage. No concurrent benchmark process. OMP/BLAS thread pools are1.

After one complete warmup per implementation, retain3 whole serial source
trials and5 paired native1/4-worker trials, alternating native worker order.
Warm caches and source index availability are explicit. GNU time wraps each
real source script and the native CLI, retaining elapsed/user/system CPU,
process maximum RSS, filesystem input/output blocks, faults and exit status.
Report the source workflow's summed wall/CPU/I/O and maximum script-process
RSS; do not sum peak RSS or claim aggregate fork memory. Check every result,
not only the fastest trial. Keep complete raw logs/numeric outputs.

Require a strict throughput or resource-use advantage on the complete operation,
with observed timing distributions and memory/I/O scope. Report the worker gain
honestly if decoding is not the bottleneck. These older short-read hg19 data do
not establish modern long-read, very-high-depth, cohort-wide, cold-I/O, biological
or diagnostic equivalence. No publication follows from a chr21-only result.

All project downloads and scratch stay on hosted runner-temp or the configured
external disks. No local build or dependency installation is needed. The full
30-family goal stays active while this product's evidence is collected.
