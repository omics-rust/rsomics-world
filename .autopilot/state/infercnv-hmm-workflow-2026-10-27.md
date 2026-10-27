# Integrated HMM execution

Whole portfolio goal active; previous goal turn made progress on actual
denoise acceptance and owning-product commits. No phase-boundary stop.
Plan: docs/plans/2026-10-27-infercnv-hmm-workflow.md.

One bounded read-only source/asset inventory completed. Root also read the
actual pinned HMM, hidden-spike, mean/variance and pipeline code. Calibration
starts at step3 and follows preprocessing; adjusted Viterbi is not Gaussian
density decoding. Source expression and state paths are different results.
No new foundation, native HMM API or placeholder CLI flag is introduced.

The real-R workflow driver now runs grouped-reference, no-reference proxy and
shipped-full raw workflows through step17 with actual i6 calibration and four
original region/report tables. It retains transformed spikes, calibration
coefficients/model inputs, actual input/state replay objects and source/runtime
provenance. The same calibrated objects exercise the actual pooled namespace
route separately, recording success or errors plus actual grouping. No copied
decoder/simulation equations stand in for the installed package.

An explicit optional HMM workflow selector reuses the existing R4.6.1/source
installation and base bundles while excluding unrelated ingestion probes.
It rejects mixed campaigns and non-shipped selections. Two meaningful workflow
tests first failed on missing routing, then pass after implementation. All447
controller tests pass49.188s, with two existing native-only skips. R4.6.1 syntax
parsing passes on4090. Actual installed-package HMM execution is not yet run;
these checks do not establish HMM compatibility, native implementation,
performance or release readiness.

Boot APFS remains above80%; no local Rust/R compile, package installation or
dependency download. All project paths/fixtures/scratch stay external. Preserve
inherited VCF/root scratch and all evidence. No deletion or automation.

World2ba75b48e1b2ff15578ef4a8cb7d32c40e46295a was pushed. Exact-head control
run40468399041 passes controller tests and architecture validation. Real HMM
oracle run40468601684 was dispatched on that head, job122069956538,
initially installing its pinned R/Bioconductor dependencies. Its terminal
failure and root cause are recorded below; it is no longer live. Source commit/archive
pins and the old executable compatibility/measurement gates are unchanged.

Denoise collection is complete; exec session46488 has exited successfully and
must not be treated as live. Its full original archive and summary are verified
in the denoise ledger. Productc5db7c74 is clean after codeab5465ac plus README.

Run40468601684/attempt1 fails in the first synthetic grouped raw workflow at
step3, before any HMM decoder invocation. Original run/artifact API, job.log
and ZIP are retained under external evidence/infercnv-hmm-2026-10-27/
run-40468601684/. Artifact12717453408 is270039658 bytes; verified ZIP SHA
4548eecaa388e865a6a68175324a0e4af880db1872b5102fb91c440f76eb4af7.
Its actual failure.json and call.log report "smoothing parameter value too
small" at smooth.spline(logv~logm). No HMM parity or successful calibration
follows from that failed execution.

Root reproduced this error using the original stage3 object and unmodified
source .get_mean_var_table plus actual stats::smooth.spline in R4.6.1 on4090:
1784 mean/variance rows,16 unique log means, x IQR0.09820742. Read-only copied
assets remain in /tmp/rsomics-hmm-20261027.P1SX57 and external scratch
hmm-spline-diagnosis-20261027/. No package was installed or source patched.

Ruling: keep that verified synthetic error as an explicit negative case.
Three required positive real-count workflows now cover default reference
groups, all-cells proxy and aggregated simulated normals. An unrelated
message/call, unexpected positive failure or missing result still aborts.
This changes fixture suitability, not the upstream algorithm or acceptance
tolerance. The R outcome regression first failed on the non-import-safe
driver, then passes after the guarded entry point and strict outcome policy;
it rejects unexpected success and unrelated error messages/calls.
Full controller suite again passes447 tests/51.738s, two existing skips;
architecture validation passes. The real example has two distinct normal
groups (19 microglia/macrophage and23 non-malignant oligodendrocytes), so
default-versus-aggregated calibration exercises real multiple-reference
handling, not duplicate single-group cases.

Fresh boot APFS92.17%, KIOXIA247GiB and fixture HDD272GiB free. Local builds,
installs and dependency downloads remain prohibited; all project artifacts
and scratch are external. No deletion is authorized or performed. Parallel
read-only work now resolves exact spline/GCV and seeded-calibration contracts
for the native implementation, not a duplicate overall HMM inventory/review.

World090049142b2e75b2c6edd8f6d94a71b6b4a769e9 is pushed; exact-head controller
run40471422874 passes. No replacement R workflow has yet been dispatched.
The two bounded implementation-research tasks completed. Live4090 R4.6.1
confirms MT/Inversion/Rejection and the mixed stream; official R4.6.1-tagged
sgram/sbart/sslvrg sources confirm the spline/GCV details. The plan records
the internal R-aligned profile and independent native spline architecture.
MIT AS241 and original permissively licensed MT references are read before
adapting selected numerical assets; no GPL source is mechanically imported.

World5ddc3dcdd6324921acda444ce234b8eeda592357 passed exact-head controller
run40473014716. Replacement R run40473486230/attempt1, job122085950218,
is now terminal failure, not live. It confirms the exact known synthetic
negative, then the actual real-count full workflow succeeds through step17
and writes all four reports. The collector then fails its expression/state
contract, so this is not a complete accepted witness.

Original API/job.log and artifact12720142658 are retained under external
evidence/infercnv-hmm-2026-10-27/run-40473486230/. Verified ZIP270309513 bytes,
SHA86092064fc12004d8cd33781481da0a20ae1ecd1692a419f480b5267bf2e0973.
The actual full call.log reaches HMM prediction, all six reporting groups,
the four table writes and "Reached up_to_step". Upstream succeeds; collector
validation does not. Do not relabel this as an upstream HMM failure.

Root cause: the predictor replaces its local infercnv_obj@expr.data before
returning (pinned HMM.R:561), so reading that argument in an exit trace captures
the state matrix rather than its original expression input. Capture the
object on function entry and retain fitted models/groups on exit. Existing
whole-workflow assertions detected this; do not weaken them. A small installed
real-object hook regression now checks preservation across mutation before
the expensive raw workflows. Base policy tests pass on4090; the installed
hook test and corrected whole R execution still require hosted execution.

Controller suite passes447 tests/42.159s, two existing native skips, and
architecture validation. Native candidate source/remaining wiring is recorded
in infercnv-hmm-native-2026-10-27.md, explicitly uncompiled/unaccepted.
