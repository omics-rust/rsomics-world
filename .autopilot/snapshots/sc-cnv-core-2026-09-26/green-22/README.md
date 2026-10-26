# Integrated denoise candidate

Adds fixed and reference-cell-SD denoising, hard/open-bound or logistic,
to the same samples CLI. Configuration is validated before input loading.
The optional step22 matrix is always persisted when requested; the original
step14 expression and sample trees retain their meanings. Default reports
omit new optional fields. Report-only serialization moves to cli/report.rs.

Uses pooled reference values or all cells, and mean per-cell sample SD
across genes, following pinned infercnv1.28.0 source. Independent columns
reuse existing product-local worker dispatch. Only one value buffer is
added; metadata maps are not copied. No dependency or public foundation.

Includes two real CLI cases and six critical numerical tests. The retained
accepted Linux binary demonstrably rejects --denoise before this change.
No per-helper RED campaign, old oracle change or tolerance relaxation.

This is an unaccepted candidate. Linux checks/benchmark, installed-package
workflow comparison, one overall review and four-native gates are pending.
The product remains unpublished and not a full inferCNV replacement.
