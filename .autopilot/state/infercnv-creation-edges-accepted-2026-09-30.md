# inferCNV creation edges — bounded acceptance

Actual installed-package run `36694464090`, attempt 1, succeeded at world
`3c1970b8c0d5b76738bd4766d25d8f088838e94f`. That head passed control run
`36693720465`. The pinned runtime is Linux x86_64, R 4.6.1, infercnv 1.28.0,
source commit `b421d9405c97a309b081ef86d455e976df93eae4`, C.UTF-8; long-double
precision/storage are 64 bits / 16 bytes. This accepts the seven fixed R
creation probes only, not native raw ingestion or complete inferCNV.

## Original evidence

Original run/jobs/artifact JSON, logs and ZIP are retained under
`/Volumes/Zane's HDD/rsomics-fixtures/evidence/infercnv-ingestion-2026-09-30/run-36694464090/`.
The [receipt](../oracles/infercnv-creation-edges-2026-09-30.json) identifies
artifact `11087846273`, size 287,475,164 bytes. The controller independently
checked terminal run/head/status, API digest and size, safe unique ZIP paths,
CRCs and the shipped manifest before extraction. The checker passed on those
original extracted bytes; a separate reviewer recomputed the tiny fixture
and checked source scripts against the executed Git revision.

| Evidence | SHA-256 |
|---|---|
| Original ZIP | `fbe97024e71e5b1736f91db2f2707cad771cdf9f83b9b268d3cc9c43b67f7b14` |
| Original Actions logs | `91bb60a17ff0e781a46521cc2c10ba08ec5131d1759c497fa19771bd2e125606` |
| Shipped manifest | `524bf2544af1098968311dcd3d358a7a8439a2f1cde56fb378cb1d8114f3cd97` |
| Edge JSON | `06046dbe8df35790642d94a741a971cb1d4a86ad4b5c22c421a4d7940fa1b054` |
| R session | `d80333b2ae1e5e8e6bd39376ae5b4596b0f90c437c7b41f971c9043fced9d1e8` |

## Accepted observations

- The shipped original gzip uses 184 header fields and 185 row fields. Actual
  creation produces 9,939 genes × 184 cells. The driver checked five slots
  with `identical` against stage-1 RDS; expression/genes/cells exports are
  independently byte-identical to stage 1. The reviewer did not decode RDS
  locally: slot identity is supported by the executed driver and original logs.
- Tiny reduced column totals are `2,3,5,6,0,1,4`. Inclusive limits `[3,5]`
  retain min/max, after exclusion of the million-count unmatched/excluded/
  zero-position genes. Minimum zero and NULL both clamp to one; their four
  output exports agree exactly and retain low/min/max/high/unit.
- Gene order is `C,B,A,D`: chromosome ranks include position-only records,
  and complete coordinate ties preserve expression order. Matrix columns,
  annotations and ordered group maps agree with the independent expectations.
- Missing annotated cells produce the actual membership error. A removed
  reference triggers upstream `!all.equal(...)` invalid-argument error; no
  common genes trigger the source's length-zero `if` error. Those failures
  are recorded with original class/call, not replaced by successful outputs.

R condition-warning arrays are empty; original logs separately contain four
logger `scipen=100` advisories. They are retained and concern later clustering,
not swallowed or reported as absent warnings. The previous failed run
`36691614700` remains unchanged: its null argument was removed by `$<-` in
the exporter. Bracket assignment fixed only provenance serialization.

## Exclusions and continuation

No arbitrary fractional lexical/threshold parity, sampling, RDS/sparse input,
general R table dialect, locale-independent R ordering, ingestion throughput,
native original implicit-header support, downstream clustering/HMM/reporting,
CLI or release follows from these observations. Native strict TSV currently
requires an explicit gene-column header; its four-platform candidate is
recorded in the [reader ledger](infercnv-raw-reader-2026-09-30.md).
The [samples-clustering witness design](../../docs/plans/2026-09-30-infercnv-samples-clustering-witness-design.md)
is the next independent source/evidence slice, not an accepted implementation.
