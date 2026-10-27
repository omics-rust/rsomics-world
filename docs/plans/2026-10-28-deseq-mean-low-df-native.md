# Native mean trend and low-df fitted workflow

Extend clean owning8bb14ad after schema3/green3 was accepted on all four
native classes at40602456359. Preserve its frozen artifacts. Implement one
coherent mean-trend/low-df fit -> saved state -> contrasts/normalized/VST path;
no new public numerical component, dependency or duplicate analysis pipeline.

## Qualified source precondition

Actual source40603312183 at1f3f7a002ac2d6ac10016eb9a68536657b1367e4 is
SUCCESS. Its original20350457-byte artifact12780580470 is retained and freshly
API/ZIP/CRC/safe-path/full-manifest/input-freeze verified at:
/Volumes/Zane's HDD/rsomics-fixtures/evidence/deseq-low-df-2026-10-28/run-40603312183/artifact/qualification/.
ZIPb5323154769ee97fe524b0c4e6a016fbcaa160dbeac0e8690286f7b799ac9cb1,
manifest43318049466200c3521858daefca65a378c8707daced0f803dc5339826953919,
witness0421f6dc922ab6a5ef1898399ec16c07f6fa2df7de5ff48059980a21d5f4efa4.
All284 manifest entries and frozen inputs verify. No local R ran.

All four1,200-gene paired designs complete with actual fitType=mean and
converged beta states, at residualdf1/2/3/4. All three Monte Carlo priors are
strictly above the .25 floor. Reference counts reproduce actual source
histograms/densities/KL; the signed prediction operator passes all five
independent probes and all source prediction/argmin checks. Largest probe
absolute residual is1.78e-15, far below the unchanged2e-11+5e-13*abs(reference)
bound. Source/operator first-min indices agree exactly; no input/seed/bound
tuning occurred. The original metadata-only failed run remains retained.

This qualifies fixed algorithm constants for this pinned R4.6.1 default RNG
profile, not universal bitwise R equivalence. Native consumer evidence and
four-platform checks are still required before accepting their implementation.

## One typed model and actual runtime math

Read core.R estimateDispersionsFit, estimateDispersionsPriorVar and the fitted
VST source again as needed. Add explicit mean fit selection while retaining
the current default parametric request and clear rejection of not-yet-supported
local fallback. Do not silently substitute mean for a failed parametric fit.
Represent the actual trend as a typed product-local variant rather than fake
parametric coefficients. Persist requested fit policy, actual trend and prior
method/profile in the private analysis schema; bump/reject older private
schemas clearly instead of an implicit conversion.

Mean trend first preserves the source's useForFit >100*minDisp nonempty gate,
then averages all nonzero-gene estimates >10*minDisp with trim=.001 and the
actual floor(trim*n) order statistic rule. Its fitted value is constant. Prior
MAD residuals and outlier/MAP/Wald logic remain in the existing shared pipeline.
Mean VST is (2*asinh(sqrt(alpha*q))-log(alpha)-log(4))/log(2), using original
normalized counts and saved trend without any refit.

For df>=4 preserve the existing analytic MAD-minus-trigamma prior and floor.
For df1/2/3 use only the qualified data-independent reference-count tables,
exact grid/break IEEE doubles and signed fixed prediction weights. Copy only
those constants and compact algorithm/source provenance into the product;
no source RDS, dataset IDs, prior answers, expected minima, source fit outputs
or fixture-specific branches. Compile-time dimensions and consumer checks
are preferable to hashing every constant again on each fit.

Runtime must derive the real observed residual histogram from actual native
gene dispersions/trend, using >=100*minDisp selection, strict(-10,10) outer
filter and actual right-closed/include-lowest histogram boundary/fuzz policy.
Use retained totals and unfuzzed widths for densities; per-grid small and all
200 KL values are data-dependent. Apply signed weights in ascending scalar
multiply/add order, without FMA/pruning/normalization/clamping, then use first
minimum on the exact fine grid and source .25 floor. No epsilon tie rule.
Keep this policy internal and record enough method/grid provenance to audit
the prior. An empty/invalid actual histogram fails clearly, not with a fixed
fallback answer. Do not implement a new R RNG merely to reproduce fixed data.

All normalizations, row ordering, original/refitted states, Cook lifecycle,
optimizer statuses and the existing single Rayon pool remain shared. Refit
reuses the saved original trend/prior; it does not run another prior estimator.
Use Rust parallelism on the real gene work, not another private thread layer
or speculative per-helper microbenchmark.

## Integrated acceptance

Extend the existing private source comparator to consume this separate source
witness and all four complete mean/low-df cases at1/4 workers, with original
numerical bounds, exact state/missingness/convergence/decisions and prior-grid
selection. Check save/reload, forward/reverse/mixed results and original-count
normalized/VST exports. Compare actual full source outputs, not only the four
prior scalars. Keep source arrays and all earlier failed evidence immutable.

Retain pasilla and all15 general/Cook workflows as regressions, deterministic
fitted state except workers and byte-identical exports. One compact integrated
new lifecycle test is enough; no exhaustive per-bin/per-helper RED campaign or
new exact iteration-count gate. One fresh whole-feature review, strict local
checks and a later exact four-native snapshot follow stable behavior.

If native histograms/priors or statistical decisions disagree, trace actual
kernel/data differences without changing source inputs, tolerances, seeds or
selection. Constants are not permission for source-answer lookups. Public
foundation APIs stay unchanged. Local trends/fallback and formal performance
remain separate release gates; keep publish=false and no placeholders.

All local build/data/scratch remains in the authorized external paths with
fresh Cargo resolution/APFS guards. No local R/install, world edits, Git or
publication belongs to the isolated implementation task; parent integrates.
