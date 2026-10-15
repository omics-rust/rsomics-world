# Paired raw-ingestion owner — durable task ledger

Plan: docs/plans/2026-10-15-infercnv-paired-ingestion-owner.md.
Status: plan authored after reviewed actual Python capture and Linux
controller CI38708622261 at e1f6fdb90c99891a7e4cd9bd6c8798f860a7a41e.
No paired campaign or speed advantage yet.

Task1 is the real offline owner: existing lifecycle/metadata/exact-state
components, externally frozen workflow code pins, original input/native run
records, actual Cargo executable and untimed installed-contract profile.
The worker owns only its controller/test files. Root owns independent review,
Linux CI and later distinct workflow selector/full32-process integration.

All original failures/partial outputs stay retained. Boot APFS gate still
blocks local Cargo/R/build/install; bounded stdlib tests use external scratch.
No production Rust/R/public crate boundary changes are authorized by this task.
