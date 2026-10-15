# SDD ledger — plan: docs/plans/2026-10-15-infercnv-trial-metadata.md

Spec: docs/plans/2026-09-30-infercnv-raw-ingestion-measurement-design.md.
Status: task1 dispatched to ingestion_trial_metadata_impl after process-owner
reviews and exact-head Linux CI38704356107 pass. Base58dd7922. Only its2 new
files are assigned; source/producer/workflow changes remain excluded.

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

Ruling: profile.Python is the existing probe's full7-field identity, projected
to R trial's3 emitted fields — mapped-library equality remains the external
owner's proof, not invented metadata. Cost if wrong: profile schema integration
rework, not a public API commitment. The apparent mirror-shape ambiguity in
the original brief is resolved in the plan and sent to the implementer.

Ruling: binding paths are absolute original-run identifiers, not local files
to open during offline auditing; only the actual result root is lstat/read.
Cost if wrong: an owner that fails physical input/code authentication could
feed unauthenticated expectations. That separate admission remains mandatory.
Same-run untimed installed-package contracts may supply the current-host
profile after independent source/function/clock/runtime proof; none of32 timed
samples may be chosen as its own metadata oracle. Reuse that preflight rather
than invent another R factory/profile producer.
