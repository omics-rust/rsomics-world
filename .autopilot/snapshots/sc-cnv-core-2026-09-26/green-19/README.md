# Integrated samples/Ward lock repair

Identical to green18's167-file integrated source except Cargo.lock. That
actual native run39818103394 failed before compilation because --locked
rejected an incomplete dependency graph. No test or numerical outcome was
produced. Preserve that original failure; do not infer a code failure or pass.

rsomics-help0.4.0 enables Clap wrap_help, which requires terminal_size0.4.4.
The previous lock added help but omitted the newly enabled optional node/link.
Add the checksum-pinned node already used by the actual seq consumer and its
clap_builder dependency link. Existing rustix/windows-sys nodes suffice.
Cached official registry metadata confirms the feature/dependency requirement.
All other source/test/fixture/expected bytes remain unchanged.

Native compilation, integrated CLI/canonical1/4-worker checks, four-native
debug/release and strict Clippy remain pending. No performance, full inferCNV
or release claim. Plan: docs/plans/2026-10-23-infercnv-samples-workflow.md.
