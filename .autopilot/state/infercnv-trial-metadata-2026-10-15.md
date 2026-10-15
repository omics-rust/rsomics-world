# SDD ledger — plan: docs/plans/2026-10-15-infercnv-trial-metadata.md

Spec: docs/plans/2026-09-30-infercnv-raw-ingestion-measurement-design.md.
Status: read-only producer/actual-artifact inventory and plan prepared; no
implementation dispatched while the process-owner task is in its fix loop.

| Task/interface | Producer/consumer | Preflight finding |
| --- | --- | --- |
| Task1 self | two new files, pure metadata gate and mutation tests | no producer/network/launch/public API changes |
| Task1/existing validators | reuse literal12 counter parser, no exact-export duplicate | inventories protect reads before existing exact-export gate |
| Task1/native source |65 typed literal fields from frozen green14 producer | wrapper PID differs from native PID; no invented PID authentication |
| Task1/actual R |20 runtime/13 provenance fields,15 root metadata files | caller's current-host profile, not first timed trial, anchors dynamic environment |
| Task1/source admission | input/code paths and hashes independently supplied | metadata checker never authenticates its own trust root |
| Task1/future launcher | checked file hashes, lifetime I/O, observed threads | caller retains process chronology/full Python identity/source postchecks |
| Task1/test execution | small synthetic profiles and regular phase files | exact expression parsing stays in existing validated component |

Ruling: add one private metadata checker, not alter frozen runtime producers
or create a foundation — the missing contract is closed output identity and
metadata, distinct from launch ownership and exact state comparison. Cost if
wrong: too much caller-supplied profile policy requires integration refactoring,
but no public API or source/numerical behavior is locked in.

Ruling: strict shape and typed equality against independent current-host
profiles, not historical hostname/BLAS/library paths — hosted runners vary.
Constant source/receipt/function/clock contracts remain fixed. Cost if wrong:
a mis-authenticated caller profile can invalidate metadata acceptance; the
later launcher's independent preflight is therefore a required gate, not optional.

Read-only reviewer confirms actual native65 fields, R20/13 fields, root
inventories, exact source/hash/options paths, and observed two R threads.
No build, R, dependency installation, source edit or performance claim follows.
