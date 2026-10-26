# Integrated denoise execution

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

Pending: Linux candidate compilation/tests/default-output regression,
installed-package workflow comparison, one overall review and four-native
acceptance. No feature or release is accepted from source inspection alone.

Storage: APFS boot81.29%, KIOXIA262GiB and fixture HDD272GiB free. All project
files/scratch/data remain external; no local Rust/R build/install/download.
Standalone Rust1.91 formatting only. Preserve inherited VCF state and root
.csv/crlf.txt/floatreads.txt/ws.txt. No deletion or automation is introduced.
