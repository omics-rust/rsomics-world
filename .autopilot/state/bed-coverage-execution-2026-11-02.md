# BED coverage implementation

Reviewed source integrated and pushed as
`88169954cce59986c58d4afeca9f903b8705cd20`.
Actual exact-head native CI `41440046744` is in progress. No performance,
native-four or release acceptance yet. Current immutable BED0.2 assets remain
unchanged; split-plus-fraction compatibility choice is still held.

Plan: `docs/plans/2026-11-02-bed-coverage.md`.
Packet: `/Volumes/KIOXIA/Developments/tmp/bed-coverage-20261102.apqCDc`.
FROZEN.json SHA256:
`9c8f34707774e5a8e91161f360ebb7a15bd3dabbd5f015b52b3b4b62cad4a4de`.
Root fully reads all five changed/new files and independently verifies every
frozen/source body hash. One fresh integrated review finds no P0/P1/P2 issue.
Root then shortens one unsupported-selection error to remove internal process
narration; no algorithm change after freeze. No extra review/test ladder.

Summary, counts, mean, histogram and per-base depth share one private engine.
Existing interval index retains pooled multi-B multiplicity; clipped sorted
events avoid per-base allocation. Existing parser/common/help/output contracts
are reused, including all-B alias protection. Counts bypass depth reduction.
No dependency/version/foundation change, no existing-operation rewrite.

The initial compact absent-command tests fail before implementation. Retained
final debug/release each103 pass, zero failed/ignored; formatting, strict
Rust1.91 Clippy and rustdoc pass. Seventeen complete outputs match actual
local BEDTools2.31.1 at `/opt/homebrew/Cellar/bedtools/2.31.1/bin/bedtools`,
SHA256 `fc994e1fcfdce8ce8591630d71f21fe67e2275c2a8d967dc0e3dfb4cfa4677a3`.
This is distinct from the retained source-built remote2.31.1 witness, not
relabeled as that executable. All project build/cache/tmp paths are external.

Split with explicit fraction selection returns an explicit unsupported error;
this temporary hold does not decide whether to fix or emulate the upstream
defect. Ordinary blocks and non-split filters are implemented. Sorted execution,
header passthrough and BAM are excluded, with unsupported flags rejected.
No stable delivery documentation/lifecycle/version is expanded before the
remaining qualification gates. Representative performance has not run.
