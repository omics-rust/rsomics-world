# Private DESeq fitted-workflow checkpoint

## Current state

Owning repository `/Volumes/KIOXIA/Documents/omics-rust/rsomics-deseq` has
clean local main718f12a2815157755118dd435e65c76dd18565c8, publish=false,
no public remote. The full30-family goal remains active, not inferCNV-only.
Plan: docs/plans/2026-10-28-deseq-native-fitted-workflow.md.

The actual unfiltered DESeq2 1.52 source witness has14,599 genes/seven samples,
rank-three type+condition design,2,240 all-zero rows and12,359 converged
nonzero fits. Native one/four-worker fits pass all pre-fixed bounds for
normalization, gene/trend/raw-MAP/final dispersion, full coefficients/SE/Wald,
p/padj, raw mu/hat/Cook diagnostics, normalized counts and fitted VST.
All gene/sample order, missingness,72 dispersion outliers,5 real optimizer
fallbacks,5,800 missing padj,1,330 discoveries and selected filter grid16 agree.
Outputs are deterministic across workers; reloaded commands preserve fit bytes.

Fresh complete-workflow review found mixed-sign numeric contrast zero-count
semantics and serde I/O classification issues. Both are fixed and the scoped
follow-up passes. The mixed(0,1,-1) contract checks144 affected genes against
the actual source selection X %*% binary(contrast!=0) !=0, retains covariance
SE, and applies Cook/filter after LFC/stat0/p1. Read/write JSON I/O errors keep
their underlying error type; malformed data remains InvalidInput. Seven tests,
strict all-target release Clippy and format pass. Original bounds, input and
source outputs are unchanged; no R refit is used to debug native behavior.

Evidence and raw commands:
`/Volumes/KIOXIA/Developments/tmp/deseq-native-first-workflow-20261028.yBqTXJ/REPORT.md`.
Corrected executable SHAa9cdd33d9546ff11f39526afb323c3a0bf1fe9e619824aae1169558cd91974d3.
Initial failed LOWESS/filter comparison and pre-review mixed-contrast failure
remain retained, not overwritten. Equal quantile endpoints are preserved;
the existing team-owned voom LOWESS kernel is reused rather than creating a
new public crate or adjusting the source acceptance threshold.

## Source, reuse and dependency decision

Actual source witness40561801963, receipt and55 verified files are recorded
in deseq-first-workflow-2026-10-28.md. Production code never reads witness
outputs or branches on fixture/gene identity. Integer counts, metadata,
model matrix and complete named contrasts belong to the fitted product.
Historical size-factor/dispersion/LRT/VST kernels are refactored into typed
private modules; the unsafe old two-group Cook policy and LOWESS path are not
reused. Existing common/help/stats are consumed without expanding their APIs.

Registry rcompat-lbfgsb=0.2.2 supplies bounded optimization, not an external
product dependency. Its MIT archive/VCS/tree provenance is independently
checked; checksum2d1c33c96f4c4739597c80105feb37a9f4f98c689db2c95ada956d2b4fc0c769.
The tag and published VCS commit have the same source tree. The finite-box
three-variable path uses true bounded Cauchy/subspace optimization; production
checks convergence status rather than treating every Ok as convergence. No R
optimizer source is vendored and no independent legal certification is claimed.

## Next gates

Frozen green1 has19 source files, archive
e39cbf72e89d084dc804461fcca1bc5894562d155c63433e9cf9a59cd3c9b67a,
source manifestfbb3285b708bfbe036eca98fe564302de28d02442d485f587aaffd75d37b7279.
The existing exact-byte source guard verifies/extracts it. The bounded hosted
deseq-candidate.yml wrapper is complete and reuses the actual retained source
witness, without installing/refitting R, to exercise whole fits/reload at1/4workers.
Its original26MBZIP verifier passes55files locally; its determinism leaf also
passes against the actual complete one/four-worker fits. Two compact workflow
tests and all486 control tests pass (two existing skips), using the configured
HomebrewPython3.15. The initial Apple3.9 invocation fails due to unsupported
Python APIs; its log is retained, not treated as a product defect or repaired
through unrelated code changes. Exact-head control-plane40589407371 at
60111087e49136e46a5e0460bd14b3f4845fed28 passes. Four-native candidate
40589627680 is dispatched at that source with green1; native acceptance is
pending, not inferred from local results.
Evidence: /Volumes/KIOXIA/Developments/tmp/deseq-native-ci-check-2m2__eg9/.

Current private checkpoint explicitly rejects residual df<=3, design cells
size>=7 and unsuccessful parametric trends. Formula coding, supplied
normalization, positive Cook replacement, low-df and nonparametric trend paths,
broader compatibility and same-host formal performance remain release work.
Iteration-path differences are retained diagnostics, not a reason to widen
numeric bounds or declare exact optimizer trace equivalence. Do not publish
this private checkpoint or expose placeholder commands. Continue coherent
workflow work after its native gate rather than stopping at this boundary.
