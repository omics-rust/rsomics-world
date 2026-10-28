# Native mean-trend/low-df execution

Owning committed140114900b37557673368494b6a4f4cb1a43b985 implements
schema4 explicit --fit-type mean, typed actual trend and analytic/fixed-reference
prior through the existing fit/refit pipeline and one Rayon pool. Default
parametric remains unchanged; unsupported local fallback still fails clearly.
Plan: docs/plans/2026-10-28-deseq-mean-low-df-native.md.

Six qualified algorithm tables are embedded, not source fitted answers or
dataset IDs. Actual residual histogram, retained totals,200 small/KL values,
signed1000-point predictions and first minimum are computed from real input
and retained with the R4.6.1 default-seed2 profile. Mean filtering/trim/VST and
original trend/prior reuse match primary source. No dependency, Layer A API
or Layer B dependency is added. Cargo/lock remain unchanged.

All4 new actual-source mean/low-df cases plus15 old general/Cook and original
pasilla pass at1/4 workers:20 full workflows, zero differences outside original
bounds, full saved-state equality except workers and byte-identical exports.
New observed histograms/selected indices52/96/134 are exact; all predictions
satisfy the frozen bound, max absolute difference5.55e-16. The df4 analytic
control, reload contrast contract and144 mixed-zero rows pass. These are
bounded profile/case results, not universal bitwise R equivalence.

One fresh whole-feature review reports PASS with no Important/Critical issue;
all17 owned file hashes and six source-generated table hashes match. Parent
freshly runs all12 release tests, strict all-target release Clippy and absolute
Rust1.91 formatting successfully. Guards verify external paths/metadata target,
boot APFS77.18% and KIOXIA237GiB free. Local executable
16f0ed3d8c2a8d2442c16d7d17b5c9ab024a824dc8f48de6bf78d2592adc3a70.
Report: /Volumes/KIOXIA/Developments/tmp/deseq-mean-low-df-20261028.kL20uU/REPORT.md.

Green4 freezes36 files; archive
8e93150fe806c84a5b7bac2067af59dc216e3ec25715d6ff0f2ec1e67c93ff2f,
manifest03007d66b69208115e7192ac148c5b92e1349f5127fdf17f1908d5af9e64f2f6.
Existing snapshot guard verifies exact inventory. Updated wrapper keeps one
source-acquisition mechanism for55+705+284 files, then compares all20 cases
and full1/4-worker state/exports. Two compact routing/determinism/Bash tests
and one actual retained-data acquirer replay pass; the replay is explicitly
fresh_api=false and executes no R/native fit.

Exact four-native40609417746 at world3768a323 is terminal SUCCESS on all four
classes, and original evidence is now retained/verified. Per platform all12
debug/release tests,20 full workflows at1/4,236 ordinary commands and7128530
numeric comparisons pass; all20 full states agree except workers and98 exports
match bytes. Low-df indices52/96/134 are exact. The three accepted55/705/284
source witnesses are bound without reopening their nested ZIPs. Receipt:
/Volumes/Zane's HDD/rsomics-fixtures/evidence/deseq-native-2026-10-28/run-40609417746/verification.json,
SHA2e2e8f94589814d23059b2d571a69d236bfe15be0f4b12d1293e197094b96a88.
Report49d12bd74bd049b510eceb1e5674b4ef17a9c539ad5e2f3a88e5089c859cb82f
records all four original API/ZIP/log/source/binary identities. Subsequent
owningc9f99b7 changes only buffered save, frozen separately as green5; this
green4 run does not attest it. Prior green3 and its completed resource-only
performance40605899832 remain bound to their original source/binary; this wave
does not inherit that measurement as if it were its own. Local trends/default
fallback, final whole-path performance and publication gates remain. Keep
publish=false and continue the full portfolio without speculative components.
