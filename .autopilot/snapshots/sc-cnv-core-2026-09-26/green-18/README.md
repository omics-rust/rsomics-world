# Integrated private samples/Ward CLI candidate

Based on productbe35e65 working tree plus actual
samples ownership and CLI. This is a development snapshot, not an accepted
workflow or release. Product changes remain uncommitted pending native tests.

Raw plain/gzip TSV creation and existing preprocessing now feed product-local
filtering, ordered grouped/pooled samples and Ward. The single binary uses
rsomics-help/common, reports identities/maps/selection/trees and optionally
expression, protects input aliases and writes named reports transactionally.
Scoped distance workers partition condensed pairs without altering per-pair
reduction. No HMM, denoising, plotting or speculative foundation is added.

Existing original tests/fixtures remain; one distance-parallel contract,
8 group tests and5 CLI tests (one opt-in canonical raw case) are added.
Canonical case checks8508 retained genes/184 cells, originalR selection
counts6187/2321, global member coverage and identical1/4-worker reports.
It does not assert exact current-Rust-to-R tree heights or general R precision.

One independent whole-workflow static review found no Critical/Important
defect. Its temporary-fixture cleanup minor was fixed. Standalone Rust1.91
formatting and whitespace checks pass; builds/runtime tests and performance
are pending. Four-native debug/release, strict Clippy and the original witness
checks use the existing candidate workflow. The only metadata policy change
allows exactly one rsomics-sc binary while still requiring publish=false.

The lock adds only the existing registry rsomics-help0.4.0 node and direct
root dependency links; its checksum is reused from the real seq consumer.
No local dependency download/build occurred under the boot-disk gate.
Plan: docs/plans/2026-10-23-infercnv-samples-workflow.md.
