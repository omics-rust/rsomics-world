# Current edgeR QL native implementation plan

Parent adopted 2026-10-28 after reading the complete architecture/math reports,
actual pinned QL/deviance source and the primary method. The comparison bounds
below are fixed before creating or running the new native product. Source
report SHA99e3a9180f945560a582220768c4294a2859c510d4b02632eaa8f4436db31b61;
source-receipts SHA c3bfa20da09b7194374166e190dd713343b293b37ca9fad0333cd573676684d4.
This acceptance authorizes implementation, not numerical/performance acceptance.

> **For agentic workers:** Use `superpowers:executing-plans` for the one isolated complete implementation. The parent owns delegation and integration; do not create subagents or independently publish. Checkboxes below are future implementation work, not actions performed by this report.

**Goal:** Implement one private, functioning integer-count → filter/TMM → current QL fit → saved/reloaded contrast/export workflow, using independently evaluated modern deviance moments rather than upstream approximation tables.

**Architecture:** One `rsomics-edger` product with the already accepted named numeric-design route and existing common/help/stats. NB fitting, modern QL, unequal-df moderation and result policy stay private. This is one integrated implementation assignment, not five public helper projects.

**Tech stack:** Rust 1.91+, Clap/common/help, one Rayon pool, typed owned matrices, small dense QR/Cholesky, serde/JSON, existing stats BH, permissively licensed numerical primitives where appropriate.

**Spec:** `docs/plans/2026-10-28-edger-first-workflow-oracle.md`, the edgeR section of `docs/10-products/bulk-expression.md`, and the accepted architecture report `/Volumes/KIOXIA/Developments/tmp/edger-native-architecture-20261028.VhhQd6/REPORT.md`.

## Global constraints and the actual decision

- edgeR 4.10.7, commit `27ba7b39d819e68df2ecdea9f32a4b6039b2a0b5`; limma 3.68.5, commit `825d1c8330d024adb6c54af66d79b0f4ec258530`; R 4.6.1/Bioc 3.23. The first oracle is accepted run 40603806303, not the older failed run or legacy-QL donor.
- Initial scope is the approved default `legacy=FALSE`, `dispersion=NULL`, `top.proportion=NULL`, abundance trend, non-robust EB, no observation weights, user offsets or alternative normalizations. One fully named nonzero contrast; no joint multi-contrast API, formula engine or new public foundation.
- Start from the six already classified team-owned donors only. Retain their actual revision/disposition in a merge record. Do not revive their repositories or parser/CLI boundaries. Existing common/help/stats contracts remain as previously reviewed.
- Public-source GPL code is a behavior reference. Do not transplant `ql_weights.c` coefficients, polynomial partitions, fitted constants or generated oracle answers into MIT code. The paper describes moment matching, not a proof of the accuracy of the package's fitted approximations.
- Mathematical resolution: evaluate NB unit-deviance moments independently. An optional private, analysis-local interpolation curve is generated only from that evaluator, never from edgeR outputs. The same small engine is reused through both global updates and the final adjustment. Direct summation remains its authority.
- The numerical-error target of that evaluator is distinct from compatibility with edgeR's approximate tables. The complete oracle is the gate; no claim of exact table parity or achieved performance is made before running it.
- All build/test/data/scratch paths remain external: Cargo home `/Volumes/KIOXIA/Developments/cargo-home`, target `/Volumes/KIOXIA/Developments/cargo-target`, scratch `/Volumes/KIOXIA/Developments/tmp`, fixtures `/Volumes/Zane's HDD/rsomics-fixtures/`. Fresh resolved-target/APFS checks precede execution; stop at boot APFS ≥80%.

## Review focus

The one integrated gate must cover these five risks, not create a helper-by-helper campaign:

1. The final raw deviance and null fit use **working** NB dispersion, while stored biological/common dispersion remains unscaled.
2. Small means/fractional adjusted df change EB weights and denominator df; legacy raw-deviance/raw-df moderation is not a substitute.
3. Display logFC uses augmented/shrunken coefficients, while means, deviance and testing use unshrunk fits.
4. Reload reuses the full filter/TMM/full-QL state but really fits the contrast null model; no hidden full refit or frozen contrast answers.
5. Solver exhaustion, bad designs and failed moments produce a nonzero error, never an apparently successful result with stale or fabricated diagnostics.

## 1. NB solver, Cox–Reid and predictive coefficients

Use `mu_i = exp(o_i + x_i beta)` with the stored log effective library sizes. For dispersion `phi`, score and Fisher information are

`g = X'[(y-mu)/(1+phi*mu)]`, `I = X'diag(mu/(1+phi*mu))X`.

The initial null start is the least-squares projection onto X of a constant log abundance. With `E_i=exp(o_i)`, first compute `c = sum(y_i/(1+phi E_i)) / sum(E_i/(1+phi E_i))`, then project `log(c)*1` by QR. This is the actual default start, not DESeq's ridge or `minmu=.5`.

Solve `(I + lambda Id) delta = g`. Initial damping is `max(1e-6*max(diag(I)),1e-13)`; failed positive-definite factorization increases damping tenfold; a deviance-increasing candidate doubles it and retries from the previous accepted state. Accept a decreasing deviance or a relative deviance below `1e-13*ymax`. Stop for accepted low deviance or `g'delta < 1e-6`; reduce damping tenfold after an immediately successful step. The source keeps `max_info` as a running maximum within each gene's iterations, not necessarily the current iteration's maximum. Use a bounded, explicit retry loop, not its potentially unbounded inner loop.

`glmFit.default` sets 250 iterations for a general design. All-zero rows have a separate boundary result. One-way designs need the real scalar Fisher-scoring route (`maxit=50`, step tolerance `1e-10`), including the zero-group boundary, followed by solving the representative group design for coefficients. This matters now: the pasilla full design is general but the ordinary condition-null design is one-way. Detect identical design rows directly, rather than copying the source's numeric hash `(e+pi)/5`. Keep the source's finite zero-group coefficient convention in the adapter, not inside the statistical distribution.

The unit deviance as a mathematical function is

`D0(y;mu,phi) = 2[y log(y/mu) - (y+r)log((y+r)/(mu+r))]`, `r=1/phi`, with the continuous `y=0` limit and Poisson limit `2[y log(y/mu)-y+mu]`.

For fitted-observation compatibility, the pinned release evaluates a numerical version after adding `1e-8` to both y and mu, uses a small-phi expansion below `1e-4`, a gamma limit for `mu*phi>1e6`, and clips roundoff below zero. Express these as documented numerical policies using independently written `log1p`/series arithmetic. They are not the fitted QL-weight tables. Keep this observed-data deviance separate from the ideal unshifted distributional deviance used for the moment expectation below.

One concrete source-arithmetic trap: `src/compute_nbdev.c:109` contains integer `2/3`, not floating division. Its small-phi correction therefore evaluates as `-.5*(y-mu)^2*phi*(1-phi*y)` after the epsilon shift. A source-compatibility adapter must preserve that actual expression rather than unknowingly “fixing” it while porting; this does not alter the mathematically defined D0 used for independent moments. A deliberate correction would be a separately recorded incompatibility, not a hidden change in this first slice. No material effect on this witness is asserted without evidence.

Cox–Reid common dispersion is data-driven:

- Hold the DGEList AveLogCPM computed before QL fitting (default prior count 2, default abundance-dispersion .05); do not recompute it from final fitted means or replace common NB dispersion by .05.
- Set `nu=n-p`, `t=min(.02+.98*(20/(M*sqrt(nu)))^(1/3),1)`, select `ceil(t*M)` highest held-abundance genes in stable feature order.
- `dispCoxReid` removes selected rows with total count <5; its optional 10,000 systematic subset applies only when the selected population is at least 20,000. Keep that actual selection rule if the numerical input route accepts that larger population.
- For each candidate `phi=z^4`, independently fit the selected rows with no coefficient augmentation and sum `ell_NB(y;mu_hat,phi) - .5 log det(X'WX)` in fixed feature order. Use Cholesky for positive-definite information. For an exact zero-expression subspace with constant structural rank across the objective, use the positive-subspace log-determinant: its omitted constant cancels from the maximizer. Any changing rank or unexplained negative pivot is a numerical-fit error, not permission to repair a bad X. Record that this differs from the source C's small LDL-pivot replacement; the unchanged complete oracle must still pass.
- Bounded Brent maximization on `z in [0,4^(1/4)]`, absolute tolerance `1e-5`, gives the common NB dispersion. The null/start fit must be evaluated at the candidate dispersion. No estimateDisp/tagwise/moving-average pipeline is inserted.

Displayed coefficients need the .125 prior-count fit: `p_i=.125*E_i/mean(E)`, `y_i*=y_i+p_i`, `o_i*=log(E_i+2p_i)`. Refit with the same working dispersion and no nested augmentation. Save the resulting natural-log coefficients separately from unshrunk coefficients; output `c'beta_shrunk/log(2)`. A preliminary fit used only for its means need not compute unused shrunken coefficients; the final observed state must.

## 2. Modern adjusted moments and the two global updates

Write `a` for average QL dispersion, `phi` for the original common NB dispersion, and `mu` for the current fitted means. For each observation define `m=mu/a` and `Z~NB(mean=m, dispersion=phi)`. With `e=E[D0(Z;m,phi)]` and `v=Var[D0(Z;m,phi)]`, solving `alpha*e=kappa` and `alpha^2*v=2*kappa` gives

`alpha=2e/v`, `kappa=2e^2/v`.

This derivation, not the upstream coefficient arrays, is the implementation authority. It is consistent with the first-two-moment scaled-chi-square construction described by [Chen et al., 2025](https://pmc.ncbi.nlm.nih.gov/articles/PMC11754124/) in “Correcting quasi-likelihood for bias.” The paper does not establish a numerical error bound for the shipped approximation tables.

With `W_i=mu_i/(1+phi*mu_i/a)`, obtain leverage `h_i` from QR of `sqrt(W)X`. Compute

`dev_adj = sum(alpha_i * D_numeric(y_i;mu_i,phi/a))`,

`df_adj = sum((1-h_i)*kappa_i)`, `s2 = dev_adj/df_adj`.

If `1-h_i < 1e-4`, that observation contributes zero to **both** adjusted terms, but its stored count/mean/raw NB deviance is not discarded. If `df_adj<1e-4`, raw QL variance is zero. No observation-weight extension is implemented in this slice.

Actual order:

1. Fit all retained genes at common phi, producing initial unshrunk means `mu0`.
2. Set `a0=1`. Compute adjusted state at `(mu0,phi,a0)`; retain genes with `df_adj>1e-8`; regress the fourth-root `s2` on held AveLogCPM by ordinary Cleveland LOWESS, span .5, **three robustness iterations after the initial pass**, delta .01 times abundance range. Set `a1=max(quantile_type7(fitted_trend,.9),1)^4`.
3. Repeat that adjustment and trend using the **same mu0** and `a1`, obtaining `a2`. There are exactly two updates, not a convergence loop and not two new NB fits.
4. Refit all retained genes at `phi_work=phi/a2`, including the final predictive-coefficient augmentation. Store raw NB deviance from this working fit, not from step 1.
5. Recompute final adjusted deviance/df at the final fitted means, original phi and `a2`; feed its raw QL variances and fractional df into current unequal-df EB.

Reuse the team's existing ordinary LOWESS implementation privately only if its span/iteration/tie/delta contract matches this source. The EB smoother in the next section is a different weighted algorithm.

### Independent moment arithmetic, with a practical acceleration

Do not call R/edgeR, source tables, source-only interpolation constants or oracle values to construct the weights. For `phi>0`, `r=1/phi`, `q=m/(m+r)` and

`p[k+1]/p[k] = q*(k+r)/(k+1)`.

Start relative mass at one at the NB mode `max(floor(m*(1-phi)),0)`, recur up/down, then normalize accumulated mass. This avoids subtracting large log-gammas and underflow in a tiny `p[0]`. For phi=0 use Poisson mode/ratio. Accumulate mass, first and second deviance moments with compensated sums; derive the variance and the two weights. At m=0 the adjusted contribution is zero.

Stop on **omitted moment contribution**, not only omitted probability. Above `K>=m`, NB `D0(k)<=2k log(1+r/m)`. For the remaining upper tail choose `rho=max(q,p[K+2]/p[K+1])<1`; geometric sums of `(K+1+j)` and its square bound omitted first/second moments. Below the mode for r>1, downward ratios similarly bound missing mass and `D0(k)<=D0(0)` bounds both deviance moments; r<=1 has mode zero. For Poisson use its decreasing mass ratios and the polynomial bound `D0(k)<=2(k^2/m+m)`, requiring geometric power sums through degree four. Normalize the resulting moment intervals and require the induced alpha/kappa relative uncertainty ≤`1e-10`, not a hardcoded number of standard deviations. A finite iteration/work limit returns an error, not normalized truncated answers advertised as exact.

The first production implementation may use direct evaluation for sparse/small means. For the expensive repeated range, create one private `MomentCurve` for this analysis's single common phi, with x=log(m) and responses log(alpha), log(kappa). Generate degree-16 then degree-32 Chebyshev approximations from **this direct evaluator**, subdividing intervals when nested coefficient-tail and interlaced holdout errors exceed `1e-8` in log space. Derive interval bounds from the actual requested positive m values and extend downward when an update changes the range. No source knots are copied. Reuse this curve across both updates and the final adjustment, because phi is unchanged.

Nested/holdout interpolation error is an estimator, not a theorem bounding an entire interval. Retain this distinction in numerical diagnostics; use direct evaluation on nonconvergent intervals, and fail if direct evaluation exhausts its work budget. Initial implementation limits: four million recurrence terms per direct point, 256 curve intervals, 64 bounded degree/subdivision attempts per interval. These are native resource limits, not edgeR behavior, public CLI options or permission to return best-effort numbers. The complete workflow gate is still required. Do not turn this into a broad approximation-generation project; begin with this one common-dispersion path.

## 3. Current unequal-df empirical Bayes and special functions

`squeezeVar(...,legacy=FALSE)` calls `fitFDistUnequalDF1` even if a particular dataset's df happen to be similar. `robust=FALSE` does **not** remove its precision weights or abundance-dependent trend.

For each raw variance x and adjusted df nu:

- Set variance to zero at exact zero df. Treat `nu<.01` as uninformative for hyperparameter likelihood: zero its prior weight and use df=1 in that internal calculation. NAs also receive zero prior weight. Preserve original adjusted df for posterior moderation.
- Fewer than two informative positive variances gives an unestimable prior. The first integrated path must report this clearly. The source's two-informative route disables the trend. Its weighted two-informative branch also clears `prior.weights` without recomputing its branch flag; do not quietly reproduce or repair that anomalous branch. Treat that condition as explicitly unsupported until separately source-qualified; the ordinary unweighted scalar fallback remains well-defined. Tiny retained universes without enough points for the two global LOWESS/quantile updates likewise fail clearly rather than replaying source out-of-range indexing.
- Let `xpos=max(x,1e-12*median(informative positive x))`, `d=nu/2`, `e=log(xpos)+g(d)`, `w=1/trigamma(d)`, where `g(d)=log(d)-digamma(d)`. Apply any internal zero prior weights to w.
- Trend e against held AveLogCPM, span `min(.3+.7*(500/M)^(1/3),1)`. Normalize weights by their type-7 .75 quantile and clip to `[1e-8,1e2]`, including the tiny nonzero weight this gives otherwise zero-weight trend points. The likelihood still uses the original zero prior weights.
- `loessFit` routes unequal weights to `weightedLowess`, npts=200, **one fit pass** (iterations=1). It is a local-linear tricube fit whose neighborhood expands to `span*sum(weights)`, extending ties, not to a fixed number of rows. Choose seed delta from sorted x gaps as the pinned R wrapper does; linearly interpolate seed fitted values. Preserve the `1e-7` small-distance/variance thresholds and the small-population weighted-linear fallback. Equal weights route to ordinary LOWESS with zero robust updates. Do not substitute quadratic R loess or the average-QL ordinary LOWESS route.

For `theta in [.5,.9998]`, set `delta=theta/(1-theta)`, `s0_g=exp(trend_g-g(delta))`. Minimize

`-2 sum[pw_g * {-(d_g+delta) log1p(d_g*xpos_g/(delta*s0_g)) - d_g log(delta*s0_g) + lgamma(d_g+delta)-lgamma(delta)}]`.

Use bounded Brent with R optimize's default absolute tolerance `epsilon^(1/4)=2^-13`, not the Cox–Reid tolerance. Then `df0=2*delta`, `s2_prior=s0`, `s2_post=(nu*x+df0*s0)/(nu+df0)`. This current non-robust optimizer produces a finite prior df on its bounded interval; do not invent Inf from reaching its endpoint. The bundle representation should nevertheless distinguish legitimate positive infinity from missing values if encountered in a supported future path.

Implement g directly to avoid log/digamma cancellation: `g(x)=g(x+1)+1/x-log1p(1/x)`, then a Bernoulli asymptotic expansion after recurrence to a sufficiently large argument. Trigamma has the positive recurrence plus its Bernoulli expansion. These are mathematical identities, with [DLMF §5.11](https://dlmf.nist.gov/5.11) and [§5.15](https://dlmf.nist.gov/5.15) as independent references. The actual statmod 1.5.2 `logmdigamma` source was inspected to confirm the target function, not transplanted. Universal rational Bernoulli coefficients are not the fitted NB weight coefficients.

## 4. Named contrast, real null fit and F survival

For one fully named contrast c, align by raw X-column identity, require all finite weights and c!=0. Form a complete orthogonal basis with its first direction proportional to c, fit the null on `X*Q[:,1..]`, with the **stored working dispersion phi/a2**, no predictive augmentation, original offsets and counts. This is a real null fit; the original full fit, filter, TMM, average-QL and EB stages are not repeated. Changing c can require a new null design/fit. Forward and reverse null spaces are equivalent.

`LR = dev_null - dev_full_working`, `q=1`, `F=LR/(q*s2_post)`.

`df_total_g=min(df0+df_adj_g, sum(full_unadjusted_residual_df))`.

The cap sums raw residual df over the **kept** feature population, not adjusted df and not raw/filtered input rows. Compute it from state, never insert the witness's resulting number. For positive F use direct survival

`P=I_x(df_total/2,q/2)`, `x=1/(1+q*F/df_total)`.

Use a direct incomplete-beta tail/log-domain branch, not subtraction of an almost-one F CDF. [DLMF §8.17](https://dlmf.nist.gov/8.17) supplies independent beta definitions/identities. Existing team-owned tail routines are seeds, but their comments calling R routines “public domain” are not provenance proof. Prefer an already available permissively licensed numerical primitive or independently derived, bounded continued fractions; do not import GPL nmath implementation text. F<=0 has upper-tail probability one as in the source; do not silently fabricate a different LR statistic. The modern path disables the legacy Poisson-bound branch. BH uses existing `rsomics-stats`, with feature order and finite-value population decided here.

## 5. State, failure semantics and provenance

Create no new public math interface. The private modules from the accepted design are sufficient: `input`, `normalization`, `nb`, `ql`/`ql::moments`, `moderation`, `results`, `artifact`, plus `cli` using help/common. Keep helpers with the owner; do not create a generic solver framework.

The versioned analysis bundle saves original counts/metadata and keep mapping; raw/postfilter library totals, TMM/effective sizes/offsets; named X; held AveLogCPM; common phi and top selection policy; a1/a2 and working phi; final shrunken/unshrunk coefficients, means, raw and adjusted deviances, raw and adjusted df; raw/prior/post QL variances and prior df; requested contrast/results; actual solver statuses and numerical-engine provenance. Do not store three copied count matrices or an unused preliminary mean matrix. Atomically save/load using common; validate product/schema/dimensions/names once at the load boundary. NA, NaN and infinity need explicit tagged representations, not undifferentiated JSON null.

Expose the same diagnostic/export fields as `fit/row-diagnostics.tsv`, both coefficient tables, fitted means, library state/filter mask, result/test-df tables, observed CPM/logCPM and held AveLogCPM in the accepted oracle. `deviance` means final working NB deviance. Observed CPM comes from kept counts/library state, not fitted means. Reloaded results may report `full_fit_reused` and `contrast_null_fit`; do not claim that no fitting happened.

Source failure facts: the scalar one-group routine writes a coefficient only on convergence, while its R wrapper can suppress the convergence warning by default; the general C solver can exhaust 250 iterations without assigning all outgoing status fields; failed decompositions can error directly, and excessive damping sets failure. None is a reason to treat a returned buffer as proof of convergence.

Native statuses must be real: converged (reason/iterations), all-zero boundary, iteration limit, damping exhaustion, matrix failure, moment-budget failure, unestimable EB prior. Nonconverged stages abort the successful analysis/export transaction, return nonzero, and retain gene/stage/error context. Initialize every status before iteration. Proposed private ceilings are 250/50 source iteration limits, 128 damping retries and 1,000 Brent evaluations; beta continued fractions must also have a finite iteration cap. Do not add public “ignore failure” or force-output switches. Reject rank-deficient X, nonfinite/negative counts, zero effective libraries, n<=p, empty kept data and invalid contrasts at their actual boundary. These are whole-workflow failure checks, not repeated per-inner-loop validation.

Provenance record: exact upstream package/commit/archive/source files, primary paper, numerical method/version/error goals, optional runtime-generated curve domain/estimated error, six donor revisions/dispositions, actual dependencies/licenses, input/option hashes and executed native binary. Keep explanatory provenance in this record/docs, not audit/history comments throughout production code. This is an engineering provenance plan, not a legal clearance opinion. If a proposed numerical dependency is not demonstrably license-compatible, do not silently vendor it.

## Frozen integrated comparator contract

These engineering bounds are ratified **before native outputs**. They are acceptance criteria, not proof of the upstream approximation's accuracy. For numeric x/reference r use `abs(x-r)<=atol+rtol*abs(r)`; keep the full failure list and maximum scaled error by field.

| Whole-oracle fields | atol | rtol |
|---|---:|---:|
| TMM factors/log offsets | 1e-10 | 1e-10 |
| Library/effective sizes, observed CPM | 1e-8 | 1e-10 |
| Held AveLogCPM/observed logCPM | 1e-7 | 1e-7 |
| Common NB dispersion | 1e-6 | 1e-4 |
| Shrunken/unshrunk coefficients, fitted means, raw NB deviance | 1e-5 | 1e-4 |
| Working NB dispersion, average QL, adjusted deviance/df, raw/prior/post QL variance, QL F | 1e-5 | 2e-3 |
| Prior df and capped denominator df | 1e-2 | 5e-3 |
| PValue/FDR | 1e-8 | 2e-3 |

Counts, masks, feature/sample/coefficient identities/order, raw residual df, contrast numerator df, finite/missing/infinite classification and FDR<=.05 discovery identities are exact gates. Raw inputs are all14,599 original genes/seven samples; compare the source-derived mask rather than hardcoding8,066 into production. Iteration counts remain diagnostics; convergence failure is not excused by numerical closeness.

One native run at each of1/4 workers, both complete bundles and forward/reverse/CPM/logCPM/AveLogCPM exports after reload, must match the **unchanged** accepted source oracle. Compare all stored fields above; require exact native fitted-state/export determinism between workers/reloads, ignoring only workers and explicit timing metadata. Validation must inspect/copy the measured fit, not refit or replace the originals.

Moment integration diagnostics distinguish direct truncation uncertainty, curve error estimate and source-compatibility error. If independent moments disagree with the source approximants beyond the frozen whole-workflow bands, retain that failed comparison, inspect the responsible mathematical layer, and return the choice to the parent. Do not tune bounds, use source coefficient answers or calibrate against pasilla until it passes. No new per-helper oracle matrix is required before the complete path exists.

## Execution phases — one isolated complete assignment

### A. Owning product and NB/library path

- [ ] Parent creates the private owning repository only at implementation start. Add one typed input/library/analysis model and existing common/help/stats dependencies; port the selected filter/TMM/CPM/abundance seeds with their provenance. No old micro-crate parser or advertised empty command.
- [ ] Implement `nb` for general and one-way designs, Cox–Reid selection/optimizer, source-compatible observed deviance, and final predictive-coefficient fit. A small integrated harness can already exercise named-design/count/library construction and explicit bad-rank/solver-limit failures; no publication yet.

### B. Current QL and moderation in the same pipeline

- [ ] Implement the independent direct moments and the small analysis-local curve if direct evaluation is materially costly; record its actual numerical work/error diagnostics. Implement exactly two average-QL updates, working refit, adjusted state and weighted unequal-df EB.
- [ ] Wire the complete analysis with one Rayon pool: indexed independent gene/null fits and per-worker workspace; fixed-order global objectives/quantiles/smoothing/BH. Do not parallelize reductions nondeterministically or create nested pools. Omit unused initial augmentation rather than prematurely changing the statistical objective.

### C. Real saved/reloaded results and one compatibility gate

- [ ] Finish `results`/`artifact`/CLI, the complete state/export mapping and actual null fits. Add one `tests/workflow.rs` integration with the supported named numeric route and boundary errors, plus private `scripts/check-workflow.py` for the unchanged whole source witness and frozen comparison contract above.
- [ ] Run formatting, strict Clippy and the integrated1/4-worker/reload comparison. Resolve failures in the implementing layer; retain output/binary/source/contract hashes. A later small general continuous/mixed contrast source workflow is needed before advertising that broader input capability; do not block the first pasilla lifecycle on a speculative formula API.

### D. Delivery and actual performance

- [ ] Parent reviews the full numerical/provenance path, then exact-head native CI on Linux/macOS x86_64/aarch64. No new public item is promoted here.
- [ ] After compatibility, one original-input full count-to-export CPU/RSS/I/O campaign with fixed CPUs, BLAS1, source inner versus whole timing, actual native CLI reload costs and honest RDS/JSON sizes. Require a disjoint strict throughput or process-resource advantage; do not call overlapping1/4-worker ranges a scaling win. If only memory wins, say so. Publication remains a separate review of a complete declared slice.

## Source-preflight record before parent adoption

This is the historical preflight record. Parent initialized the private
owning main repository only after adopting the contract and fixing its bounds.

Read the pinned source paths whose hashes are listed in `source-receipts.json`, the primary method sections, the existing architecture/dossier/source plan, and only targeted files in already-audited donor paths. EdgeR/limma archive hashes match the approved pins. The owning `/Volumes/KIOXIA/Documents/omics-rust/rsomics-edger` path still does not exist.

For actual dependency behavior, downloaded only the 93,227-byte CRAN statmod1.5.2 source archive to this external report directory, not an installation; SHA-256 `5f83d82043c5e26f1048fc0139580da1b7e40becf95fb4cb26ed3afcd46a581d`. The source identifies GPL-2|GPL-3, reinforcing that its implementation text is a reference, not an automatically reusable MIT component. Existing edgeR/limma source trees were reused. The DOI redirect was inaccessible; the primary PMC article was read instead.

Fresh guard before that download verified external paths and Cargo's resolved target, boot APFS77.2606% (<80%), KIOXIA237GiB free. No native repository, source/world/Git edit, build, R execution, test, install, dispatch, deletion, publication or subagent was performed. No completed DESeq/RNA campaign or full88-file oracle audit was repeated. The remaining empirical gate is whether this independently implemented full path meets the frozen source comparison and measured resource/throughput requirements; this report does not claim that it already does.
