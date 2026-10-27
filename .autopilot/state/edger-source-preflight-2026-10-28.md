# Current edgeR source preflight

Read-only next-family qualification is complete; no edgeR R/native execution,
product repository, publication or performance evidence yet. Plan:
docs/plans/2026-10-28-edger-first-workflow-oracle.md.

Official current edgeR4.10.7 and limma3.68.5 replace the dossier's historical
4.10.1/3.68.4 anchors for new work. Bioc3.23/R4.6.1 are retained. Exact source
commits/archive hashes/locations are recorded in the plan. Official branch tips
agree with archive DESCRIPTION short commits; no independent full archive-tree
versus Git-tree comparison is claimed. Downloads passed the external/APFS guard.

Source review confirms modern QL legacy=FALSE differs materially from legacy
donor code: default dispersion is internally estimated common NB, not tagwise
or a fixed0.05; average QL scaling, adjusted deviance/fractional df and current
limma unequal-df EB are required. Explicitly supplied NB dispersions are capped
at4 in the modern supplied branch; do not infer that branch applies to the
internally estimated default. Some Rd prose disagrees with actual release code.

The first source workflow reuses original pasilla counts/identity metadata,
uses design filtering, recalculates filtered library sizes and TMM, fits modern
QL, saves state and derives contrasts through their legitimate null GLMs.
It does not use DESeq fitted/normalized values or prohibit the null GLM.

Six concrete donor trees match dossier revisions and are clean. Filter/TMM/
CPM/aveLogCPM supply numerical seeds; legacy QLF explicitly uses legacy=TRUE,
abundance.trend=FALSE and cannot be the modern workflow skeleton. The old
estimate-disp local/moving-average paths are not an adequate QL dispersion
model. Existing stats BH is a candidate consumer contract, not grounds for a
new public numerical crate. No second broad source-pool inventory is needed.

The source-only driver/hosted workflow is implemented and parent-read. It
reuses private hash/table helpers, pins and builds both exact released sources,
matches installed R bodies to release definitions including assign-based
subsetting, and retains actual loaded DLL bytes/hashes. Original pasilla raw
inputs/options are frozen before actual design filtering/TMM/modern QL calls.
Complete DGEList/DGEGLM state and initial/reloaded contrasts/exports survive,
with explicit filtered-row mask and NA/NaN/Inf encoding. Iter/failed are
source-reported diagnostics, not invented C cap-exit convergence evidence.

The full489-test control suite passes with two existing skips; the compact
new route/Bash test passes. No local R parse/run or install has occurred.
Actual source outcomes, prior degrees of freedom and retained feature counts
are pending the first hosted run, not filled with predicted values. No native
product or public numerical API exists yet. Source-task report:
/Volumes/KIOXIA/Developments/tmp/edger-workflow-source-20261028.0XaUNy/REPORT.md.
