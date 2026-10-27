# Current edgeR source preflight

Current source witness40603806303 is SUCCESS, with all88 files retained and
verified; modern QL/save/reload/exports pass on actual raw pasilla data. No
native product, publication or performance acceptance exists yet. Plan:
docs/plans/2026-10-28-edger-first-workflow-oracle.md.

The entries below retain source preflight and execution provenance in order.

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

First hosted source run40602956264 at exactfcc8c003 fails before any fit:
both exact source builds install successfully, but the identity check spells
R/glmfit.R as R/glmFit.R. The actual archive uses lowercase f; case-insensitive
Mac reads hid the spelling error, while Linux correctly reports missing file.
Correct only that source-file mapping, retaining the same file SHA and actual
function bodies. No model/options/input/output/bound changes occur. Artifact
12778584754 is17521047bytes, SHA
a7a960067718d79415d6a73bd0fb1a0325634fb01192d27c43dbd0038316ecb4;
all698 original files are retained/verified under
/Volumes/Zane's HDD/rsomics-fixtures/evidence/edger-oracle-2026-10-28/run-40602956264/.
The two-entry partial manifest verifies; witness remainsfailed with no fitted
outcomes. This is not labelled a native/math failure. Controller40603400276
at1bc92a1eee3b9941276fa97af07bd19935f04edc passes; fresh source run40603806303
is terminal SUCCESS at that corrected head with unchanged source calls/options.

Its43901911-byte artifact12779298937, ZIP
a5685a6c012a5b4494cd75c4d0be06da67168c2e5854e624842da0cefc47f3ff,
is freshly API/ZIP/CRC/safe-path/full88-file/input-freeze verified under
/Volumes/Zane's HDD/rsomics-fixtures/evidence/edger-oracle-2026-10-28/run-40603806303/.
Receipt: .autopilot/oracles/edger-first-workflow-2026-10-28.json.
Manifestdb6a6e6be77d8b65a4fb0a9aeb3afb21f562b83ec5d7e1b1c28a55dc130c1ce3;
witness0fbc8a249f286367713cb375bf48b9f954eff46410c079fdd7936aeaf18d052f.
Actual design filtering keeps8066 of14599 genes, filters6533; TMM/current QL
and full saved-state/reloaded forward/reverse contrasts/CPM exports pass all
state, order, byte and frozen-input checks. Actual solver failures0, iterations
2..22;1152 rows per direction have FDR<=.05. These numerical source outputs
are not biological conclusions or native acceptance. The next implementation
must consume this real modern QL lifecycle, not the retired legacy skeleton.
