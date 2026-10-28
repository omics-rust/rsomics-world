# Native count-input limma implementation plan

> Adopted by the parent on 2026-10-28 before native output. Execute one complete private implementation using `superpowers:executing-plans`; no per-helper campaign. The numerical bounds below are frozen before implementation.

**Goal:** One functioning original-count → filter/TMM → current `voomLmFit` → named contrast → default eBayes/topTable → complete save/reload/export lifecycle.

**Architecture:** One private `rsomics-limma` product, named numeric design and one fully named contrast per result command. Copy reviewed team-owned kernels as implementation assets where appropriate, never a Layer B dependency or a revived micro-crate. Keep all fit/zero-loss/moderation/result policy private.

**Tech stack:** Rust1.91, common0.12.3/help0.4.0/stats0.2.4, one Rayon pool, owned typed matrices, small QR/Cholesky, buffered common atomic JSON/TSV output, existing permissively licensed numerical dependencies.

**Spec:** World `docs/plans/2026-10-28-limma-first-workflow-oracle.md`, `docs/10-products/bulk-expression.md` limma/shared/release sections; external source preflight `limma-first-workflow-preflight-20261028.TXxtak/report.md`. Parent reads/adopts this proposal before coding.

## Constraints, current evidence and review focus

- Actual source: edgeR4.10.7/commit27ba7b39d819e68df2ecdea9f32a4b6039b2a0b5 and limma3.68.5/commit825d1c8330d024adb6c54af66d79b0f4ec258530, R4.6.1/Bioc3.23. Both are GPL>=2 behavior references, not permission to transplant new upstream implementation text.
- Accepted source run40606516204/artifact12780957275/head d5e136a87c681e53095293638441a5a26c4dc503:203 verified files. ZIP SHAfac61c690c3f742335446c141ac2ad78c84fe3df33d20df8e0181cc31e43f5a2; manifest SHAaf682fed901644550411e6ae74e22b6bda45a7e567ca15e2dddc2c6b17027e97; witness SHA091f476ca726487d23e83de7d62b5bffa7cea9021b533a4dda2043c89289a577. Reuse this unchanged witness, not a new source campaign.
- Original input14599×7, source-derived8066 kept/6533 filtered; actual residual df:7 genes at2 and8059 at4. Source discoveries at BH<=.05: forward1131, reverse1131, mixed1215. These are comparator observations, never production constants.
- `/Volumes/KIOXIA/Documents/omics-rust/rsomics-limma` is absent; not created/reset. Frozen edgeR26-file tree was not modified. Its failed P/FDR gate remains failed; neither its QL weight method nor a numerical-contract waiver enters limma.
- All build/data/scratch external: Cargo home `/Volumes/KIOXIA/Developments/cargo-home`, target `/Volumes/KIOXIA/Developments/cargo-target`, scratch `/Volumes/KIOXIA/Developments/tmp`, fixtures `/Volumes/Zane's HDD/rsomics-fixtures/`. Fresh path/resolved-target/APFS guard before scratch/build/test/download; stop boot APFS>=80%.
- Exclude log-expression/NA count input, sample/prior weights, supplied offsets/libraries, blocks/correlation, other normalizations, robust/trend/forced-legacy modes, TREAT, public multi-contrast F, plotting UI, batch/splice/genesets and cross-product bundles. Numeric designs do not imply a formula engine or donor pairing.

Review these five things in the one whole-feature review/integration gate:

1. Structural-zero fits replace sigma/df only; full weighted coefficients/SE remain from all original logCPM observations.
2. Voom uses four-pass weighted LOWESS here; edgeR's one-pass EB smoother is not interchangeable.
3. `legacy=NULL` branches on positive df; this source uses unequal-df with no abundance trend, not edgeR's forced/trended QL moderation.
4. Mixed-contrast SE preserves upstream's approximation using global coefficient correlation and row-specific unscaled SE.
5. Reload never repeats filtering/TMM/voom/linear fitting; tagged legitimate NA/Inf, shapes, raw identities and the full base fit survive unchanged.

## Exact supported calls and the numerical path

Source calls are `DGEList(counts,samples)`, `filterByExpr(...,design=X)`, subset `keep.lib.sizes=FALSE`, `normLibSizes(method="TMM")`, `edgeR::voomLmFit(...,design=X,save.plot=TRUE,keep.EList=TRUE)`, then `contrasts.fit(base,C)`, `eBayes(contrast)` and `topTable(coef=1,number=Inf,sort.by="none",adjust.method="BH",p.value=1,lfc=0,confint=FALSE)`.

Keep factor/term provenance from the oracle (~type+condition, treatment coding, type paired-end/single-read and condition untreated/treated), but accept X as an explicitly named numeric matrix. Join sample rows by name, preserve count feature order, require every contrast coefficient including zero. Proposed count boundary is finite nonnegative f64 (upstream permits fractional counts); the accepted real witness only covers integers. Reject globally rank-deficient X and n<=p at input. Internal masked variance fits may lose rank legitimately and need rank-aware QR, not the global rejection policy.

### 1. Shared library state, separate voom expression

Default design filter: min.count10, min.total.count15, large.n10, min.prop.7. Retain raw mask/order, recompute postfilter library totals, TMM reference/trim/rank/factors and effective sizes `L_i=lib_i*TMM_i`; offsets `ln(L_i)`. Reuse the source-qualified edgeR implementation assets for this arithmetic, including count*1e6/library operation order and NaN-only TMM sum skipping. Do not run its unused held-AveLogCPM/.05 abundance fit in limma.

Voom defaults: block/prior.weights/var.design/var.group/lib.size/offset/offset.prior=NULL, sample.weights=FALSE, prior.n10, normalize.method="none", nominal span.5, adaptive.span=TRUE, plot=FALSE. Actual span is `min(.3+.7*(50/M)^(1/3),1)`, observed.42858845222626446; it is derived provenance, not a returned fit field.

Expression is `E_gi=log2((y_gi+.5)/(L_i+1)*1e6)`, not ordinary edgeR logCPM. Targets retain group="1", raw postfilter lib.size, TMM factors and joined sample metadata; targets$lib.size is not L. Save both meanings explicitly.

### 2. Initial linear fit and source zero-loss correction

Unweighted initial QR gives beta0, SE0, sigma0=sqrt(RSS/(n-rank)), df0 and Amean=rowMean(E). The raw design's own column space, without adding an intercept, gives leverage h. Candidate rows have `sum(y<1e-4)>max(2,1/max(h))-1e-4`. This differs from the filtering `stats::hat` policy that augments a constant column.

For candidates, actually fit a Poisson GLM (`dispersion=0,prior.count=0,lib.size=L`), independently using the reviewed team NB/Poisson solver, source start/250 general or50 scalar iteration limits and explicit failure. Mark an observation only when fitted mean<1e-4 AND count<1e-4. Temporary missing E observations are an internal variance correction, not accepted missing-count input. Refit the affected rows on remaining observations; set df=observations-rank and sigma=sqrt(masked RSS/df), or explicit NA sigma at df0. Replace only initial sigma/df. Leave beta0/SE0 intact. The source fallback with fewer than two replicated genes cannot form this complete EList workflow; return a clear unsupported/no-replication error, never invent a trend.

The actual seven source df-loss identities are FBgn0004188, FBgn0052011, FBgn0052318, FBgn0052495, FBgn0052939, FBgn0053909 and FBgn0261359. Compare these exact identities/df and all sigma values, not only the distribution. The accepted exports do not expose the temporary IsZero mask; retain native mask diagnostics, but do not claim source-mask equality from an unavailable field.

### 3. Trend, weights and final full linear model

Let Amean2 use the mean of remaining nonmasked E observations only for corrected rows; leave saved Amean unchanged. For df>0, `sx=Amean2+mean(log2(L+1))-log2(1e6)` and `sy=sqrt(corrected sigma0)` (fourth root of residual variance). Structural-zero presence selects `weightedLowess(sx,sy,weights=df,span=derived,output.style="lowess")` with DEFAULT iterations4 and npts200. Without that branch use ordinary `lowess(...,f=span)` with three robustness updates. Do not silently use the edgeR EB one-pass wrapper, even if weights happen equal.

Independently extend the team's local-linear frequency-weight implementation: sort x; choose delta from cumulative sorted gaps/minimum cluster-spacing rule; fix seed windows using original df weights and span*sum(weights), nearest-distance expansion (right on equal distance), extended ties. At each of four passes, local tricube regression multiplies original weights by bisquare robustness weights; linearly interpolate between seeds. Robust scale is six times the ORIGINAL-weight weighted median absolute residual (exact half-weight uses three times the two neighboring residuals). Stop if scale<=1e-7*mean(abs residual); retain source1e-7 local distance/variance and interpolation thresholds. These are independently written mathematical operations, not copied C text.

Preserve original-order voom.xy points, sorted full-length voom.line and delta. Interpolation is linear, constant outside the range (`rule=2`), averaging ordered tied x. Predict with initial ALL-observation beta0: `fitted_logcount=log2(1e-6*2^(X beta0)*(L+1))`; weights are inverse fourth power of the interpolated trend. Avoid algebraic reassociation until the unchanged whole oracle establishes it harmless.

Final WLS uses each row's full `sqrt(w)X,sqrt(w)E`. Store full beta, row-specific unscaled SE=sqrt(diag((X'WX)^-1)), sigma, df and Amean. Repeat only the masked weighted variance fit for zero-corrected rows and overwrite sigma/df, NOT beta/SE. Global `cov.coefficients`, rank and pivot come from UNWEIGHTED raw X QR as source lm.series does. Model exports are `beta*X'` and raw `E-fitted`, not weighted residuals or a contrast fit's fitted values. Use rank-revealing column-relative1e-7 QR for internal masked rows; preserve source column order/estimable pivot rather than a speculative SVD threshold.

### 4. Contrasts, automatic EB and complete statistics

For each fully named c, coefficient=`beta*c`, global covariance=`c'V0c`. With `R0=cov2cor(V0)`, upstream mixed SE is `sqrt((SE_g*c)' R0 (SE_g*c))`; for orthogonal columns it is the sum-of-squares special case. This is NOT `sqrt(c'(X'W_gX)^-1c)`. Retain source rank/pivot/design metadata, sigma/df/Amean; do not call a new full linear fit or inherit old moderated statistics. Forward/reverse and mixed(0,-1,1) are separate scalar derivations; mixed is algebra, not paired biology.

`eBayes` defaults are proportion.01, stdev.coef.lim(.1,4), trendFALSE, spanNULL, robustFALSE, winsor.tail.p(.05,.1), legacyNULL. Positive df all equal selects legacy fitFDist; otherwise current fitFDistUnequalDF1. The source witness exercises the latter. Default auto routing must exist; do not hardcode the observed branch. Equal-df/default and legitimate Inf branches remain explicitly uncovered by this source witness, even if old numerical fixtures and the integrated native test exercise them.

For the current unequal-df, trendFALSE case: set variance zero at df0; missing variance/small df<.01 gets zero likelihood weight and internal df1. Floor variances at1e-12*median(informative positive variance). With d=df/2, `e=ln(xpos)+g(d)`, `w=1/trigamma(d)`, `g(d)=ln(d)-digamma(d)`, take the precision-weighted SCALAR e-mean (no abundance smoother). Minimize `-2*sum(pw*(-(d+delta)*log1p(d*xpos/(delta*s0))-d*ln(delta*s0)+lgamma(d+delta)-lgamma(delta)))` over theta[.5,.9998], delta=theta/(1-theta), tolerance2^-13; `s0=exp(emean-g(delta))`, dfprior=2delta. Reuse the edgeR private Brent/g/trigamma arithmetic, not its policy wrapper. Finite posterior is `(df*x+dfprior*s0)/(df+dfprior)`; legitimate infinite prior gives posterior=s0. Unestimable prior and the source weighted-two-informative anomaly fail explicitly, not fabricated estimates.

Equal-positive-df fallback is the actual no-covariate fitFDist moment route: valid variances floor1e-5*median (median0→1), mean corrected log variance and sample variance minus mean trigamma; positive spread gives dfprior=2*trigammaInverse(spread), s0=exp(emean-g(dfprior/2)); nonpositive spread gives Inf and arithmetic mean of floored variances. Internalize reviewed historical kernel/fixtures, replace its unbounded inverse loop by bounded mathematically equivalent solve; no revived ebayes-core dependency. Preserve n<3 source no-shrink semantics where the complete voom trend is estimable.

Moderated t=`coef/SE/sqrt(s2post)`; `df.total=min(df.residual+dfprior,sum(df.residual))`; two-sided P from a direct Student-t/beta tail. Coefficient prior/B requires full row-specific SE and unequal-df tail-to-quantile mapping to maximum df BEFORE ranking. Set ntarget=ceil(.01*M/2), p=max(ntarget/M,.01); select largest absolute mapped t, compute p0 and ptarget=((rank-.5)/M-(1-p)*p0)/p. If ptarget>p0, `v0_i=SE_i²*((t_i/qt_upper(ptarget_i/2,MaxDF))²-1)`; otherwise0. Clamp each to (.1²,4²)/median(s0), then mean; missing estimator uses actual1/s0 fallback with a diagnostic. For r=(SE²+v0)/SE², B=`ln(.01/.99)-ln(r)/2+kernel`; finite kernel=`(1+df.total)/2*ln((t²+df.total)/(t²/r+df.total))`, or `t²*(1-1/r)/2` when dfprior>1e6. Do not reuse the donor's constant-SE or equal-df shortcut.

Single-contrast F=t². IMPORTANT: source classifyTestsF supplies UNCAPPED dfprior+df.residual to F.p.value, whereas the t P uses capped df.total. Do not alias these probabilities merely because the cap is inactive here. Legitimate infinite uncapped df uses `stats::chi2_sf(F,1)`, not a finite-df beta call with Inf parameters; the t-test cap remains finite. BH applies to the entire supplied contrast family before thinning; stable all-row topTable order is logFC,AveExpr,t,P.Value,adj.P.Val,B. Existing stats BH matches this witness's all-finite family; any future NA wrapper must preserve R's full n rather than silently BH-adjusting a smaller finite subset.

Use existing MIT statrs0.18 direct beta tail and independently bounded tail inversion for t quantiles (positive finite bracket, finite expansion/iteration ceiling, explicit exhaustion); no GPL nmath implementation text. The source P range here bottoms near1e-11, but retain log-tail semantics for mixture df remapping. g/trigamma are independent recurrence/Bernoulli mathematics, not empirical coefficients.

## State, files and concrete seams

Proposed private files and interfaces (not public foundation APIs):

| File | Responsibility / narrow output |
|---|---|
| `model.rs`,`input.rs` | Dataset{Counts,Design,joined samples,input hashes}; `read_inputs(counts,samples,design)->Result<Dataset>` and complete named `contrast(names,terms)->Result<Vec<f64>>` |
| `normalization.rs` | `prepare(&Dataset)->Result<LibraryState>` with mask/kept indices/raw+filtered libraries/TMM/effective/offset/targets; no held abundance |
| `linear.rs`,`poisson.rs` | rank-aware QR/WLS; `fit(E,design,weights,mask)->Result<LinearFit>`; actual zero-probe Poisson means; errors carry gene/stage |
| `smoothing.rs`,`voom.rs` | ordinary and weighted four-pass LOWESS distinct policies; `fit(&Dataset,&LibraryState)->Result<BaseFit>` |
| `numerics.rs`,`ebayes.rs`,`results.rs` | independent functions, automatic variance-prior route, contrast-specific mixture/B; `derive(&BaseFit,&[f64])->Result<ContrastResult>` |
| `artifact.rs`,`cli.rs`,`lib.rs`,`main.rs` | versioned complete Analysis; run/results/export; common buffered atomic output, alias rejection and error classes |
| `tests/workflow.rs`,`scripts/check-workflow.py`,`scripts/guarded-cargo.sh` | one integrated native lifecycle test; unchanged full source comparator; external-path/target/APFS execution guard |

Analysis schema stores raw counts ONCE plus mask/kept mapping, joined metadata/raw X/options/source+input identities and all library stages. Full BaseFit retains E/observation weights, beta/unscaled-SE matrices, corrected sigma/df, Amean, global covariance/rank/pivot/design binding, targets, voom.xy/line/delta/derived-span policy. Keep native zero-mask provenance, not unused copied initial mean/count matrices. Store the requested primary contrast AND full derived/moderated fields including scalar-vs-row shape for s0/dfprior/v0, t/P/B/F/Fp and capped/uncapped df semantics. No successful QR fit gets fictional convergence/iteration fields. Represent finite, NA, NaN,+Inf,-Inf explicitly; distinguish scalar from per-gene arrays, not JSON null.

CLI proposal: `run --counts ... --samples ... --design ... --bundle ... --contrast NAME=WEIGHT...`; `results --bundle ... --output ... --contrast ...`; `export --bundle ... --output ... --kind log-cpm|weights|fitted|residuals`. Source-equivalent base/contrast/moderation diagnostic fields are in the full bundle and comparator mapping; no placeholder operations. All named '-' files rejected; alias errors stay common ConfigError. `results`/exports load the saved base and may recompute only contrast/moderation; no full fit or state mutation. Save JSON through `common::write_output(Some(path),...)`, not unbuffered tiny writes.

Concrete reuse map; initially keep copies private with exact source-file hash/provenance, not a B→B dependency:

| Seam | Existing actual caller | Limma concrete target / promotion judgment |
|---|---|---|
| identity join/named numeric contrast | edgeR input.rs:62/127/222; CLI read/derive | limma input::read_inputs/results::derive; share lexical IDs/joins only, NOT integer-vs-fractional policy, factor/results names or bundles |
| design filtering/TMM/library state | edgeR normalization.rs:8-78,165/229 | limma normalization::prepare before voom; strongest exact second-consumer candidate once BOTH workflow tests establish contract |
| scalar one_group | edgeR normalization.rs:91 and nb.rs:146 | Poisson grouped-design branch may call it, but this witness's full general X does not establish an actual second scalar call; do not promote on that claim |
| general Poisson fit | edgeR nb::PreparedDesign::fit/general_fit | limma poisson::zero_mask at phi0; reuse only necessary solver/subspace arithmetic, not QL/moments/dispersion/predictive policy |
| ordinary LOWESS | edgeR ql average update→smoothing::lowess | limma voom no-zero branch→ordinary lowess; policy-compatible primitive candidate, but this accepted witness exercises weighted, not ordinary branch |
| weighted local-linear / special functions | edgeR moderation.rs→weighted_trend/numerics | limma voom requires4 passes, EB here needs NO trend; share local-linear mathematics only after consumer contracts, not wrappers or fitted state |
| public foundations already sufficient | common runner/output/ThreadArgs; help::parse; stats::bh_adjust/chi2_sf | same explicit limma CLI/artifact/results call sites, including the existing chi-square tail for Inf F denominator df; no new public item needed to implement this slice |

Legacy assets read selectively: voom-quality-weights revision18ad63de6ed12827da6edc2625fd2f99cd7c36cc (Householder/voom fixture seed, replace non-rank-aware/raw-library/zero-row shortcuts); limma-ebayes8fbfa0b8fcb9444dcf5740a4c4feab7a398e9c30 (QR/contrast/mixture fixtures, replace shared df/SE and missing state); ebayes-core only internal numerical/fixture reference. Never revive their parsers/CLI or old performance claims. The current edgeR asset snapshot is final-source SHA9f98b1172f721e507b8c5b8365c49bb82befa23161d4a2bda4149714a384765e, not an assertion that its failed QL oracle passed.

## Adopted before-native whole-oracle bounds

The parent ratifies these bounds BEFORE any native output; no observed native errors exist at adoption. Use `abs(native-reference)<=atol+rtol*abs(reference)` over ALL source fields, not a sample. Preserve full failures and maximum scaled error per field; never tune these after results.

| Fields | atol | rtol |
|---|---:|---:|
| TMM factors/log offsets; derived span/delta | 1e-10 | 1e-10 |
| library/effective sizes | 1e-8 | 1e-10 |
| voom E/Amean; voom point/line x | 1e-10 | 1e-10 |
| beta/unscaled SE/global covariance/fitted/raw residuals | 1e-7 | 1e-6 |
| initial trend sy, voom line y, corrected sigma | 1e-7 | 1e-6 |
| observation weights | 1e-6 | 1e-5 |
| s2.prior/s2.post; t/F | 1e-6 | 1e-4 |
| df.prior/capped and uncapped test df | 1e-3 | 1e-4 |
| coefficient var.prior/B | 1e-5 | 5e-4 |
| P, BH-adjusted P, F.p.value | 1e-12 | 5e-4 |

For strictly positive probabilities additionally require `abs(log(native/reference))<=1e-3`; zero/missing/infinite class remains exact. This prevents tiny-P absolute tolerance from concealing large relative errors. The df/EB allowance reflects the pinned bounded optimizer tolerance; it does not permit a different EB policy. No source fitted QL approximation enters this pipeline.

Exact gates: original counts/identity/order/hashes, metadata/X/contrast weights, source mask/kept mapping, targets meanings, all dimensions/scalar-vector shapes, base rank/pivot/method, every residual df and the seven df-loss identities, zero/NA/NaN/Inf locations, auto moderation branch, complete FDR<=.05 discovery identities for all three contrasts. Compare all source base/contrast/moderated fields and all observation/model exports, initially AND after reload. One native original-input run at each1/4 workers must have exact complete state/exports ignoring workers and timing metadata only; preserve measured bundle bytes. Source RDS byte equality is NOT a native JSON requirement.

## One complete implementation assignment and gates

- [ ] **A — coherent input/library/linear/zero-loss path.** Parent creates owning repo only after adoption. Build the files/interfaces above using concrete shared foundations/assets; wire one end-to-end integration test, not helper stubs. The test uses named reordered metadata/design, structural-zero df correction and full coefficient retention; includes the auto-equal-df/Inf codec behavior and one alias/rank failure within that same lifecycle test. Internal masked-rank loss is valid, global bad rank is an input error.
- [ ] **B — finish actual current voom/contrast/EB/state.** Implement weighted4-pass trend and final row WLS, mixed approximate SE, unequal/default-equal prior routes, per-row t/B/mixture, base save/load and four exports. One Rayon pool, indexed gene fits and per-worker workspace; fixed global reductions/weighted medians/sorting/optimizer objective; no nested pools. Reuse QR for identical masked subdesigns where safe; do not share statistical policy to save a few lines.
- [ ] **C — one integrated acceptance gate.** Fresh guarded fmt, offline tests, strict all-target Clippy and release build; unchanged accepted203-file witness comparator at1/4 and full reload/forward/reverse/mixed exports. Source shape/nonfinite rules and the frozen bands above apply. Retain exact binary/source/lock/contract and all failures. Stop the affected numerical action on contradiction, diagnose the real layer; do not change input, source, mixed SE or tolerance to pass. Parent performs one independent whole-feature review, then exact-head four-native Linux/macOS x86_64/aarch64 CI.
- [ ] **D — complete real workflow performance then release decision.** After compatibility, fixed4CPU/BLAS1 hosted source inner count-read→fit→primary contrast→durable save/reload→all three actual contrasts/tables plus logCPM/weights/fitted/residuals exports, excluding only package/identity setup; also GNU-time whole-process wall/user/sys/RSS. Native1/4 runs the same complete declared CLI lifecycle. Proposal:one warmup then3 source and5 native trials per worker setting, original input every fit, numerical checks after timing on retained outputs without refit/replacement. Report distributions, sum native command costs/max single-command RSS, actual JSON/RDS sizes/I/O and strict disjoint throughput or resource advantage; overlapping1/4 ranges are not a scaling win. No release before parent API/provenance/hot-path review and complete stable-slice qualification.

## Read-only execution record and self-review

Read the full approved source plan/preflight and relevant dossier, actual pinned voomLmFit.R, lmfit.R lmFit/lm.series/fitted/residual methods, contrasts.R, ebayes.R including mixture estimator, squeezeVar.R, fitFDistUnequalDF1.R, fitFDist.R, weightedLowess.R/C, loessFit.R, chooseLowessSpan.R, classifyTestsF and topTable. Inspected only named historical assets and actual current edgeR/common/help/stats call sites. Bindings are in adjacent source-references.json; accepted manifest/witness were rehashed, not all203 artifacts re-audited.

Self-review: all source calls/default branches/state/exports have owning files; signatures use the same Dataset/LibraryState/BaseFit/ContrastResult across tasks. Five review risks have one integrated/whole-oracle gate; absent equal-df/Inf/ordinary-no-zero source coverage is labeled, not inferred. Weighted voom4-pass and no-trend EB are separate policies. Mixed-SE and uncapped Fp nuances are explicit. No new public API, source coefficient table, incomplete scaffold or speculative family boundary is proposed.

Fresh guard before creating this external directory: all external homes/target/TMP resolve correctly; APFS occupancy77.2731%, boot free55,705,370,624bytes, KIOXIA236GiB available. This assignment ran no builds/tests/R, benchmark, install, dependency/data download, Git/CI dispatch, deletion, publication or agents; no world/product/source file changed. Only this external plan and its reference receipt were written. Await parent adoption before native implementation.

## Parent adoption, 2026-10-28

The full report was read before adoption. The specified original-count lifecycle,
private state and module boundaries, source-derived math, explicit exclusions,
whole-oracle bounds and one integrated review/native/performance gate are adopted.
The fractional nonnegative-count boundary follows the actual upstream contract;
the retained real source witness only establishes integer-input compatibility.
No formula engine or new public foundation is implied. Equal-df/Inf and ordinary
LOWESS paths need honest coverage labeling rather than fabricated source proof.

The immutable proposal is retained externally at
`/Volumes/KIOXIA/Developments/tmp/limma-native-plan-20261028.UyFrMk/REPORT.md`,
SHA256 `3299fdf34fef03e66c897466fbdbd5aabe45b87111b5974d2979c6168d667787`.
Its source-reference receipt is retained alongside it, SHA256
`5dbbd4ced64a7d764dee7ffe52ce6a3d27862e31084f7442e7c00c8d755beec6`.
The source receipt documents the original read-only state; the adoption here
now authorizes creation of the absent owning repository and implementation.
The edgeR source failure is unaffected by this adoption.

Implementation ruling before final native freeze: `results` may explicitly
accept `--state JSON` with its table output to retain the required complete
derived contrast/moderation diagnostics. Use the existing common atomic pair
and alias contract; no implicit sidecars, large stdout dump or hidden full fit.
This completes the specified saved-state lifecycle, not a new statistical mode.

First release supports the CLI/artifact/table contract, not accidental public
solver/model/cache modules. Keep internals crate-private, exposing only a
run_cli binary bootstrap if needed; do not invent a Rust analysis facade or
hide a still-public tree with doc(hidden). Existing lifecycle tests inspect
the public files or run inside the crate where appropriate. Supply the full
team-standard project license texts. These surface/packaging rules do not
change any numerical bounds, source inputs, math, state schema or CLI results.

