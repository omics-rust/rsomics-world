# Real RNA-seq complete-report performance

## Current state

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

Official full original BAM size437662663 and published MD5
ba014f6b397b8a29c456b744237a12de remain unverified until hosted download.
Only a properly bounded original HTTP206 prefix was acquired locally. Its
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

The actual complete BAM count, chr21 derivative, source eligibility and real
five-metric compatibility still require hosted execution. The driver first
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
