# Paired raw-ingestion owner — durable task ledger

Plan: docs/plans/2026-10-15-infercnv-paired-ingestion-owner.md.
Status: plan authored after reviewed actual Python capture and Linux
controller CI38708622261 at e1f6fdb90c99891a7e4cd9bd6c8798f860a7a41e.
No paired campaign or speed advantage yet.

Candidate e25e96f54209092724408aabb3844f00d98865bf implements only the
owner/test. Root read both complete files/report; fresh independent full
spec/code review found three Important, no Critical/Minor:

1. R system2 FALSE discards stdout/stderr, so actual identity JSON is lost.
2. Direct exec/R --version needs validated R_HOME even for version handling.
3. Plan/implementation conflated target.test=false with profile.test=false.
   Root independently read original retained Cargo1.91 bench JSONL line63
   from run36229683578: target.test=false/profile.test=true, opt3 and debug
   assertions false; line64 successful build. Both admitted green14 bench
   targets use identical harness=false/test=false configuration. Plan now
   narrowly requires that actual distinct-field shape, not either flag.

Original implementer is fixing each concern with genuine RED regressions;
first stream-preservation fix1c018827467335a1df4627ac372b331b3666cb3a is
not yet accepted pending scoped re-review. Its opt-in actual installed-R test
will be explicitly run after dependencies in the native workflow, never on
this boot-disk-blocked Mac. No real campaign has been launched. All candidate
and failed tests remain retained, with full-suite-before-final-guard and
focused-after-change scope distinguished in the implementation report.

Task1 is the real offline owner: existing lifecycle/metadata/exact-state
components, externally frozen workflow code pins, original input/native run
records, actual Cargo executable and untimed installed-contract profile.
The worker owns only its controller/test files. Root owns independent review,
Linux CI and later distinct workflow selector/full32-process integration.

All original failures/partial outputs stay retained. Boot APFS gate still
blocks local Cargo/R/build/install; bounded stdlib tests use external scratch.
No production Rust/R/public crate boundary changes are authorized by this task.
