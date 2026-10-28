# Private DESeq fitted-workflow checkpoint

## Current state

Current clean private main is f53f7136f9a8563bddd6273768efce856717ea43,
publish=false. Schema5/local trends and source-faithful local VST are complete
locally; see deseq-local-trend-native-2026-10-28.md. Green6/all47 files now
passes all four native classes, with original evidence retained/verified.
Final green7/all49 files closes accidental public modules, rehomes tests and
adds project licenses/private complete-case reuse, without math/schema/runtime
CLI changes. Its final native gate is pending. No current-feature performance or release
acceptance is inferred from the earlier green5 result.
Previous C9 adds only buffered fitted-bundle save to schema4/1401149;
see deseq-buffered-save-2026-10-28.md. Green5/run40611588271 at exact42f378fe
now passes all four native classes and20 full workflows; original evidence is
retained/verified, receiptc86fec10a22f4b69a6470b99562f0bdc8d4de12496e488c227a69d962b767bf4.
No mathematical/schema/dependency change is involved in this sink fix.

Schema2 green2/run40597122905 at exactworldec72d1ba passes all
four native classes; original APIs, ZIPs, binaries,23 source files and both
complete source witnesses are independently retained/verified. Final receipt:
/Volumes/Zane's HDD/rsomics-fixtures/evidence/deseq-native-2026-10-28/run-40597122905/verification-final.json,
SHAfb69144f9be90a364363002bf5c9d2016c00587ab10e160860bb669d2e138e7f.
This accepts seven general plus four disabled-Cook cases, not positive refit.

New schema3 green3 implements positive Cook replacement/refit, preserving
original assays and explicit final state. All15 source workflows pass locally
at1/4,11 tests/strict checks/fresh whole-feature review and bounded P2 fix pass.
Exact four-native40602456359 atc598ffaf is SUCCESS and original evidence is
retained/verified; see deseq-cook-refit-2026-10-28.md. Low-df source40603312183
at1f3f7a00 is also SUCCESS with all284 manifest entries, independent probes and
three positive MC priors qualified. Native mean/low-df1401149/schema4 now
passes all20 full workflows at1/4 locally,12 tests/strict checks and one fresh
feature review. Four-native green4/run40609417746 is SUCCESS at exact3768a323;
original evidence is retained/verified, all20 workflows at1/4 pass on every
platform. Authoritative verification SHA
2e2e8f94589814d23059b2d571a69d236bfe15be0f4b12d1293e197094b96a88, under
/Volumes/Zane's HDD/rsomics-fixtures/evidence/deseq-native-2026-10-28/run-40609417746/.
See
docs/plans/2026-10-28-deseq-mean-low-df-native.md.
The completed green3 performance40605899832 accepts a strict process-RSS
advantage, not strict source-inner throughput/scaling. Its original binary
attribution is immutable. Fresh green5 benchmarking follows its accepted
native gate and measured save fix:40614312003 now passes strict source-inner
throughput/RSS/scaling, with original evidence retained/verified. Actual local
source40614317043 also passes all four cases. Native schema5 now passes all24
workflows locally at1/4 workers and fresh whole-feature review; its four-native
and current-feature performance gates remain. Declared scope remains a release
gates; no complete DESeq2 replacement is claimed. Full30-family goal stays active.

## Initial schema1 checkpoint

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

## Historical schema1 gates

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
40589627680 at that source with green1 is terminal SUCCESS on all four native
classes. All four original API/ZIP/job-log artifacts are retained and verified
under `/Volumes/Zane's HDD/rsomics-fixtures/evidence/deseq-native-2026-10-28/run-40589627680/`.
Each source/archive/lock identity matches green1;55 oracle files/19 source
files, both complete1/4-worker comparisons,144 mixed-zero rows, unchanged
source and deterministic outputs pass. Top-level verification.json records
the four original ZIP/binary identities. Linux artifact12773187129 ZIP
984aaa3dd19b3d42557197b1fc91941a3b79706187d78ff9bc030b5a45fe0558,
binaryb2923bc2fed4f248cce09a00128fe1cec25efd6d6d6bda970f5c6f1266149966.
This accepts the bounded checkpoint, not complete DESeq2 replacement or formal
performance. New schema2 general-design work follows the separate plan below.
Evidence: /Volumes/KIOXIA/Developments/tmp/deseq-native-ci-check-2m2__eg9/.

Initial schema1 checkpoint explicitly rejects residual df<=3, design cells
size>=7 and unsuccessful parametric trends. Formula coding, supplied
normalization, positive Cook replacement, low-df and nonparametric trend paths,
broader compatibility and same-host formal performance remain release work.
Iteration-path differences are retained diagnostics, not a reason to widen
numeric bounds or declare exact optimizer trace equivalence. Do not publish
this private checkpoint or expose placeholder commands. Continue coherent
workflow work after its native gate rather than stopping at this boundary.

Following-wave plan: docs/plans/2026-10-28-deseq-design-normalization.md. Its source
case driver and private product implementation proceed in differentiated
files. Preserve green1/schema1 artifacts while extending the new checkpoint.
