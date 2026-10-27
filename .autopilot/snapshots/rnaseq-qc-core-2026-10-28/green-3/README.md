# Fixed-lock complete RNA-seq QC report candidate

Use the normal four-native gate, bootstrap_dependencies=false.
This private source imports the generated Cargo.lock from the initial Linux
bootstrap40542614508, SHA
c09dea6b5397ca673d7052042d403fa55d2663e45c99672dc6f5ea6f51f308e2.
That run passes compilation and all eight tests in debug/release; its strict
Clippy fails two test-only single-range Vec literals. Both literals now use
explicit Vec::from arrays without suppressing lints or changing behavior.

The prior full oracle comparator was skipped after the lint failure, so
this source is not yet native five-metric upstream acceptance. The gate now
attempts the full comparison whenever metadata succeeds, retaining real
outcomes even if an independent lint/test fails. No algorithm or dependency
version is changed after the initial resolution. publish=false remains.

Same one-scan five-metric report, focused projections, mandatory help,
shared foundations, source provenance and explicit high-depth limitations
remain described in the product README/MERGE_RECORD and the shared-report
plan. Snapshot naming does not imply a passing gate or public release.
