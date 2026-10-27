# Integrated denoise execution

Status: accepted integrated non-HMM denoise, private/unpublished product.
All native and real-package workflow gates below pass. Full inferCNV and the
whole portfolio objective remain incomplete and active.

Previous goal turn: progress. Product7495e3e implements accepted matrix
workers, docs3a43638 is clean at this turn's start. Four-native run40309311228
passes; real GBM whole-CLI five-pair scaling is131.53/53.32s at1/4 workers,
with identical outputs and retained external records. Do not redo those gates.

Current plan: docs/plans/2026-10-26-infercnv-denoise.md. Full portfolio goal
remains active. Inline execution, direct main, one overall feature review.
Ruling: the user's delegated scope/no repeated approvals and consolidated
testing override skill approval/per-helper RED/reviewer/deletion rituals.
Preserve all scratch/evidence; no generic workspace cleanup.

Observed integrated RED on the retained accepted4090 binary: cnv samples
--denoise exits2 with unexpected argument, before input reads/output creation.
New real CLI tests were written first. Six critical numerical tests cover
global center/open bounds, SD axis, logistic values, zero-width distinction,
workers and real invalid/budget cases. Hand values were independently checked
with base R4.6.1; this is not an installed infercnv acceptance claim.

Candidate implementation now exists in the product worktree, uncommitted:
private validated DenoiseConfig, NoiseFilter, Denoised values/bounds, pooled
reference statistics, explicit all-cells fallback, hard/logistic transforms
and reused scoped workers. CLI validates config before inputs and always
persists optional step22 expression, with step14/tree identities retained.
Default new fields are omitted. Serialization moved to src/cli/report.rs
without exposing private CLI arguments. No dependency/public foundation added.

Actual source reviewed includes R/inferCNV_heatmap.R:2783-2810 and
R/SplatterScrape.R:210-212: logistic is1/(1+exp(-20*(deviation-width))).
BSD3 attribution/conditions are retained in THIRD_PARTY_NOTICES.

Initial gates were Linux candidate compilation/tests/default-output regression,
installed-package workflow comparison, one overall review and four-native
acceptance. They are now resolved below; source inspection alone was not used
as acceptance.

Update: Linux green22/run40319047320 passes124 tests per debug/release,
strict Clippy and unchanged default report fingerprint198e3beb...49fb868d.
Original run/artifact/log/ZIP are retained externally under this feature's
evidence/run-40319047320/. Binary SHA
aa14dfd802016e1bb57259373e8306dc896f033b1ac526a7193533ca3c6c840e;
artifact12647838216 SHA7720af4c12c4965aa6dc0a33c0f1b81621395a397e17080245fea51c5a1ae8fb.
The immutable native receipt selects these bytes for actual R comparison.

An actual parse probe on this binary found --noise-filter -1 rejected by
Clap before required-input checks. Added a real CLI bypass regression first,
then allow_negative_numbers on that argument. Green23 will contain only this
CLI parsing fix/test beyond green22; numerical, public library and report
source bytes are unchanged. The R comparison's positive/default cases on
green22 remain applicable; final four-native gate exercises the new parser.

The installed-package driver reuses existing export/trace/parameter helpers,
now import-safe, plus the already-proven complete C locale rule. It records
15 real raw-input workflows (grouped, proxy, full ×off/fixed/SD/hard/logistic)
and the native comparator runs actual1/4-worker processes. Four comparator
behavior tests and14 workflow tests pass; R syntax/import/locale pass on4090.
Actual installed-package execution remains pending, not inferred from these.

One obsolete control test compared a919-line YAML copy in an old plan to the
live workflow. Its failure was unrelated to production/compatibility. Replace
that duplicate with an exact historical source link and real closed-selector
behavior tests. All fixed-code hash checks and executable old-mode gates
remain unchanged. Full controller suite passes445 tests/46.880s, two native
skips, after this focused cleanup.

Storage: APFS boot81.29%, KIOXIA262GiB and fixture HDD272GiB free. All project
files/scratch/data remain external; no local Rust/R build/install/download.
Standalone Rust1.91 formatting only. Preserve inherited VCF state and root
.csv/crlf.txt/floatreads.txt/ws.txt. No deletion or automation is introduced.

Final green23 four-native run40324825127/attempt1 passes at exact world head
3fdab08bd159f871d4aa6e30326d5e1c1dbe685c: Linux/macOS x86_64/aarch64,
125 tests per debug/release profile, formatting and strict Clippy. All four
tested source manifests have SHA
5a0ae8e071fde4ae664f55e5d80420405a752bc95cca285fb374e492a961f274.
Original run/jobs/artifact API responses, job logs and ZIPs are retained under
external evidence/infercnv-denoise-2026-10-26/run-40324825127/.
The ZIP byte counts and SHA-256 match the GitHub artifact API:

- Linux x86_64 artifact12650742942:
  1b7d23f986d6cbf067c9884b977e8e72adfcd876f64e41a95cd24354ee641edc.
- Linux aarch64 artifact12651087021:
  37e2735105290114389275380deee14dfe3b951e09bf50874a103655bfcbc422.
- macOS x86_64 artifact12651257052:
  bfb871cdc9b86414c9e4924c5110935230131ad317ebe8bc4e5d3162213005e1.
- macOS aarch64 artifact12651012453:
  7aeab31580dc27f64f695d8166cb08a0b04b416fc4e94e8e8a44e039de13fd2e.

One whole-feature fresh-context read-only review found no actionable Critical,
Important or material minor issue. It checked the exact green23 source,
upstream math, config/CLI/report integration, deterministic workers and real-R
comparison. Do not repeat that review. Runtime installed-package evidence,
not review confidence, was the final outstanding feature gate and now passes.

Actual R run40324825110/attempt1 completes on the same world head, using
the retained green22 numerical/report bytes and actual infercnv1.28.0. The
only green23 differences are negative-number CLI parsing and its regression.
Synthetic/shipped base exports and actual raw step15/22 comparisons pass.
Do not rerun merely because an observation times out.

The full portfolio goal was verified active with all30 products still in
scope. No objective narrowing or new goal is needed. Parallel HMM work is
bounded read-only source/data-flow inventory, not an implementation or a
reason to stop at denoise. Fresh boot APFS81.31%, KIOXIA261GiB free: no local
build, package install or dependency download is allowed.

Installed-package run40324825110/attempt1 now completes successfully at the
same exact head. Its original job log records all15 real R workflows and all
30 actual native processes (one/four workers), with no missing/duplicate
case/worker pair. Maximum logged expression scaled error is0.00134056 under
the unchanged1e-12+1e-12*abs(expected) criterion. Source/CLI selection, exact
identities, tree merges/order and worker-report equality checks all execute
inside that successful step. Timings are explicitly not a paired speed test.

All170 current product files matched green23's tested manifest immediately
before committing. Product code commitab5465ac40714ef81eb245709c62477c1362a2d9
contains the six owned implementation/test/notice files; following docs-only
commitc5db7c7 describes the accepted operation and remaining limitations.
The product is clean, publish=false, and still has no remote. No release or
full inferCNV acceptance is implied.

Original run/jobs/artifact API and job.log are retained externally in
run-40324825110/. Artifact12651626826 is608616250 bytes, API ZIP digest
5686bc3c362b42ffa74a8644ab5a8b4df5298acfc057f7de5a6575012157d977.
The original ZIP is now fully retained and its byte count/SHA matches the
API. Its718 files total1725727128 uncompressed bytes. Direct inspection of
the complete summary/witness confirms the binary/source identities,15 cases,
30 processes and equality of all15 worker pairs. Witness SHA
60f50167c166f98a745d4da68fdb2271ed0695615281ef251d018723c41c2ad8.
Maximum before/after expression scaled errors are0.0008477393156599713 and
0.0013405626739539781. The original job log and actual summary agree.

Single correctness-run full-case native wall times are0.54/0.44s without
denoise and0.64-0.65/0.53-0.55s with denoise at1/4 workers. All denoise modes
include the additional complete step22 JSON matrix; peak RSS is60383232-
60661760 bytes. This is proportional overhead/provenance, not repeated paired
performance or an R speed comparison. The one additional value vector and
existing worker plumbing require no new dependency/foundation. No further
helper benchmark campaign is needed for this acceptance.

Previous goal turn: progress, not a blocker. Native gates, real R outputs,
verified source identity and owning-product commits changed authoritative
state. Next HMM work follows docs/plans/2026-10-27-infercnv-hmm-workflow.md;
its source calibration, grouping and adjusted emission semantics are real
implementation dependencies, not reasons to start another helper-audit loop.
