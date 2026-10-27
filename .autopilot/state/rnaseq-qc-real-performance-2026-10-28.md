# Real RNA-seq complete-report performance

## Current state

Controller40555913726 at58a926c944f39e4538edf089bf299d59dd3a5ce6 passes:
483 tests, two existing skips, architecture validation. At this exact source,
chr21 run40556139574/job122372589460 is terminal SUCCESS. The
original455305682byte artifact12758010802 (SHA
de578dfd196a822cbe981d38d8f9f741bb44e45bb021a5b49a6313e224247e87)
is retained, API/size/digest/CRC/safe-path verified and independently reread
under external evidence/rnaseq-qc-real-performance-2026-10-28/run-40556139574/.
Summary SHA67b7d0169155d5c06c59eb960b79c5bc9f7d0f15fa8ded59b04379dcd18fb909.
The separately required whole run40556744875/job122374722150 is terminal
FAILURE during initial download, before any source/native call. Its request
redirects to unblockme.cyverse.org and receives13283bytes of HTTP200
Turnstile/IP-unblock HTML, not BAM. Identity validation correctly rejects it.
Original failed response/API/log/ZIP artifact12757741690, SHA
b48705e558d28fc81017aa1c2e10daef2c4c8b7d32331d748a6feac45b3b427b,
is fully retained/verified under run-40556744875/. No performance result is
inferred from that failure. Do not process or bypass the challenge.

The updated route reuses the first campaign's verified complete original BAM
and BED gzip from immutable artifact12758010802; no public data request is
needed. Both original identity and API/ZIP chain are checked. Native/source
algorithms, input cohort, flags, thresholds and numerical tolerances are
unchanged. Whole acceptance and publication remain pending.
The cache-route exact-head controller40558375098 at
7f1e7c1e0857feeb5f69ee1ccba67101cff5d6c1 is SUCCESS. Whole retry
40558683154/job122381526636 runs at that head and is authoritatively live
in Measure the real complete report after the verified cache/runtime steps.
Local actual input-artifact extraction/full-original MD5+SHA checks and all16
RNA-focused control tests pass. No algorithm/input population change is used
to make the previous acquisition failure disappear. Poll this same live run,
not failed40556744875, before deciding the next action.

## Accepted bounded chr21 result

Complete original:9810963 records,437662663bytes, official MD5 matched;
SHA38174082a8f36ae76a6852b9b970fcb334ab5c8186c133993a0fd652d1d147cd.
Chr21:102098 records,4627545bytes, all36nt/M-or-N CIGAR;
SHA736414382b9c139de6b295c88913e9e724470490188818a87cc975efabece34a.
The754 BED rows include737 coverage-eligible and546 TIN-eligible transcripts,
including30 eligible-zero scores. All four source workflows and twelve native
whole reports agree;32 timed processes exit zero with no leftovers.

Same Xeon6973P-C host, four CPU affinity, warm cache, complete startup/output:

| Cost | Source five-script serial | Native1 worker | Native4 workers |
| --- | --- | --- | --- |
| wall seconds | 9.44/9.36/9.40 | .21/.20/.19/.20/.20 | .18/.20/.18/.19/.18 |
| median user CPU seconds | 9.25 | .18 | .20 |
| median system CPU seconds | .12 | .00 | .00 |
| peak process RSS KiB | 51316/51188/51276 | 9364/9416/9316/9384/9252 | 9344/9768/9520/9540/9604 |
| filesystem input blocks | all0 | all0 | all0 |
| filesystem output blocks | all176 | all528 | all528 |

Serial median47x faster, strict range separation; observed median process
peak RSS5.48x lower, also strict separation. Four-worker median gain1.11x
has overlapping ranges, not strong scaling. Native output I/O is higher,
not an I/O advantage. These accept only the predeclared real chr21 whole
five-metric operation, not whole-input, modern/long-read/high-depth/cold-I/O,
aggregate-fork-memory or biological equivalence. Publication is not accepted.

Plan: docs/plans/2026-10-28-rnaseq-qc-real-performance.md. The complete private
candidate has four-native semantic acceptance, not real-input/performance
acceptance. Owning initial main commit6fdda78; publish=false, no public repo.
Green4 binary/source identities are retained in rnaseq-qc-shared-report state.

New hosted workflow rnaseq-qc-performance.yml uses exactly accepted Linux
run40546148508/artifact12751813613/binary SHA
ae968a1a490433086b6e50867a15b9797bea47e903faf032967b5a3b377fa840.
The original native ZIP carries accepted RSeQC wheel/source/dependency freeze;
the new preparation path passes offline against that actual API/ZIP evidence.
Historical inferCNV verifier code and pins are unchanged. A trial extension
hit independently pinned code hashes; that extension was reverted exactly,
not accepted by refreshing old pins. Failed and successful local controller
logs remain external;482 tests pass with two existing native skips before
the final performance-result status correction.

One bounded whole-performance review finds one issue: a no-advantage summary
could say complete despite nonzero exit. The driver now explicitly records
advantage_passed=false/status=rejected_no_advantage while retaining raw costs;
a compact regression covers an apparently faster median with overlapping
ranges. No source numerical tolerance or workload selection changes.

## Qualified inputs, not measured results

The original bounded local qualification used only a properly bounded HTTP206
prefix; the subsequent hosted full original identity is accepted above. Its
header has25 exact hg19 reference names; no alignment records were parsed.
The original BED gzip size2567192/SHA
eb7070cfb52b9011c132346fd27baffb8a7acf3c034ebf996cebd52fbb52aa19
was fully acquired and verified on external scratch.

Real retained-BED offline derivatives preserve row order and duplicate names:

| Profile | Rows | Bytes | SHA-256 |
| --- | --- | --- | --- |
| chr21 | 754 | 114113 | a3db0ae1a2a7ad539e22a50dbcbb7f7179e07564d471eea64a835d0faf3000f7 |
| whole exact-present references | 58384 | 9018094 | 5132086afd5b33bb4d5b2724ef5c4e8275aaba7854a7ec5af0d1902171b88ada |

Both record2793 missing-reference rows explicitly; chr21 separately excludes
57630 outside-profile rows. Whole retains62 duplicate-name rows. No aliases,
deduplication or read/flag filtering is used. The raw derivatives and complete
exclusion JSONL are in
`/Volumes/KIOXIA/Developments/tmp/rnaseq-qc-inputs-offline-i8om7owt/`.

For the still-pending whole profile, the driver first
checks all five actual public source CLI outputs against native1/4-worker
reports, including actual untimed source eligibility, before any repeated
timing acceptance. Every measured result must match. Three serial whole
source trials and five paired native trials follow warmups, fixed four CPUs,
exact dependencies and full startup/output scope. CPU/process-RSS/filesystem
blocks are explicit; no logical-I/O or aggregate-process-memory claim.

Proceed chr21 first, then whole; a chr21 pass cannot complete the full-input
performance or publication gate. Preserve failed evidence rather than fit
the input/thresholds. Full30-family goal remains active; the separate held
inferCNV stochastic contract does not block this work.
