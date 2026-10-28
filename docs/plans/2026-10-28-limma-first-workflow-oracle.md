# First current count-input limma source workflow

Prepare one source-only witness before native implementation. Current limma
3.68.5 recommends edgeR::voomLmFit, not limma::voomLmFit or the older manual
voom+lmFit composition. Source package ownership does not change the product
boundary: rsomics-limma owns its weighted linear-model/moderation lifecycle
and never depends on the Layer B rsomics-edger product.

## Anchors and inputs

Use already-qualified limma3.68.5/Bioc3.23/R4.6.1, commit
825d1c8330d024adb6c54af66d79b0f4ec258530, archive
40b1d2059a76ad98db4bf80139760acf8e8bae92a8265aaaf16ac199831533f4;
edgeR4.10.7 commit27ba7b39d819e68df2ecdea9f32a4b6039b2a0b5, archive
a3cc0f66be29f1a61a74021f9848ec940794e653905c4332327a719dd9ad8393.
Both are GPL>=2 behavior references, not vendored Rust source. Current source
trees are /Volumes/KIOXIA/Developments/tmp/edger-source-preflight-20261028.sDtAJY/.

Reuse only DESeq2 1.52.0's original pasilla raw data: source carrier archive
8c91699286336350e66eec132ce6fdf5bb4af78e2a4d015a5a61224f62a95984,
counts gzip4b5570008936786da365d110df1c83c8cffc646d6c667d6b15ecbfb3d9ee735d,
sample CSV1844f871ca5d750f41685c73b87bd7dda291ec0d52304c1afb46a17ae88db745.
No DESeq statistics or package installation is involved. Retain all14,599
original rows/seven samples before filtering and the declared fb-suffix sample
mapping. Join samples explicitly. Fix typepaired-end/single-read and condition
untreated/treated factor levels, design~type+condition with rank3. Sequencing
layout is not donor/pairing; block=NULL and sample.weights=FALSE.

## Actual calls and fitted-state contract

Freeze inputs, mapping, design/coding, options and fully named forward(0,0,1),
reverse(0,0,-1), mixed(0,-1,1) contrasts before fitting. The mixed vector is an
algebraic saved-state diagnostic, not a biological pairing claim.

Run DGEList, filterByExpr(y,design=X), subset keep.lib.sizes=FALSE,
normLibSizes(method=TMM), edgeR::voomLmFit(y,design=X,save.plot=TRUE,keep.EList=TRUE),
limma::contrasts.fit on the full base fit, eBayes and topTable(coef=1,number=Inf,
sort.by=none,adjust.method=BH,p.value=1,lfc=0,confint=FALSE). Record all actual
defaults. Voom keeps no block/prior/sample weights, no supplied offsets/library
sizes, normalize.method=none, adaptive.span=TRUE; the public derived span
replaces nominal .5 and must not be invented as a returned fit field.

Actual source preserves full weighted coefficients/unscaled SE while structural
zero correction replaces only sigma and residual df. Its weighted LOWESS and
corrected averages must be observed, not replaced by legacy all-zero removal.
eBayes uses proportion=.01, stdev.coef.lim=(.1,4), trend=FALSE, span=NULL,
robust=FALSE, winsor.tail.p=(.05,.1), legacy=NULL. Positive-df equality chooses
the legacy versus unequal-df moderation path; do not force edgeR QL's policy.
Contrasts.fit's nonorthogonal weighted mixed-contrast SE is its documented
approximation; an exact per-gene refit is not identical behavior.

Save complete original/filtered/normalized DGELists, keep mask, X/term coding,
full uncontrasted MArrayLM, contrast and moderated fits, original identities,
options and mappings. Export counts/library factors/effective sizes/offsets,
EList logCPM/weights, voom.xy/line, Amean/targets, base coefficients/SE/sigma/df/
covariance/rank/pivot/design, fitted(base), residuals(base,E), contrast fits and
all moderation fields, including t/p/lods/F and coefficient prior variance.
Targets$lib.size is raw post-filter library size, not effective TMM size.
Voom E is not ordinary edgeR logCPM. Do not call fitted on a contrast object.

Reload and derive forward/reverse/mixed contrast -> eBayes -> full result tables
and observation/model exports from the saved base fit, with no filter/TMM/voom/
linear-model rerun. Contrast moderation is legitimate. Verify full base-state
and original bundle-byte preservation, identical repeated primary outputs and
original feature order. Filtered rows have an explicit mask/status, not fake
statistical estimates. Keep scalar/vector/matrix shapes in the full RDS.

Retain actual df distributions, zero/NA locations, conditions and branch
coverage. NA sigma at zero residual df and infinite prior df can be legitimate
upstream states; do not impose a universal finite-everything gate or fabricate
convergence/iteration fields. Record NA/NaN/Inf distinctly in tables/metadata.
If no actual df loss/unequal-df branch occurs, call it uncovered, not accepted
coverage or a reason to change raw data after inspecting outcomes.

## Bounded delivery

One new R driver, hosted workflow and compact route/Bash test reuse existing
private source/hash/table helpers and exact edgeR/limma hosted installation
recipe. Check archive member spellings case-sensitively, not through the Mac
filesystem. Qualify relevant installed function bodies against the source,
retain DLL/runtime/source identity and all outputs/failed partial witnesses.
All hosted storage is RUNNER_TEMP; all local scratch/data stays external with
fresh path/APFS guards. No local R or native product/placeholder is created.

Historical voom, voom-quality-weights, limma-ebayes and ebayes-core are reviewed
numerical/fixture seeds, not skeletons or public foundations to revive. Existing
common/help/stats may be reused where their actual contracts match. Filter/TMM
and variance arithmetic are shared-contract candidates only after both products
have concrete consumers; do not lock a public API or share fitted-state policy.

Exclude log-expression/NA input, sample quality weights, donor/block/correlation,
prior weights, supplied offsets/library sizes, other normalizations, robust/
trend/forced-legacy moderation, TREAT, public multi-term F operation, plotting
UI, batch removal, splice/gene-set workflows and cross-product bundles. Source
success is not native compatibility, performance, biological or release evidence.
