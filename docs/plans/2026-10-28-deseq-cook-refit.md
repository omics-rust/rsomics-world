# DESeq positive Cook replacement and refit

Extend private owning a3274cc after freezing green2. Reuse the existing fitted
dataset, normalization, original trend/prior and one Rayon pool; do not create
a public component or a second dispersion/GLM pipeline. The actual source
15-case witness40592944209 already contains four positive replacement cases.

## Released contract

Read DESeq2 1.52.0 core.R:416-427,2089-2137,2504-2581, the called gene/MAP/
Wald functions, methods.R:1-28,60-67 and results.R:551-607,791-850. Invoke
replacement only when a full-model cell has at least seven samples, unless
the existing --no-replace-outliers opt-out selects Inf. Cells use the complete
design, not just the condition factor.

The default cutoff is F(.99,p,n-p). A gene's replace flag reflects any original
Cook value above cutoff, including samples not eligible for replacement; it
does not mean a count changed. All-zero/absent diagnostic states remain
explicit. The replacement center is the .2-trimmed mean of all normalized
samples. Replace only above-cutoff entries in eligible samples.

Both SF and NF truncate mean*effective factor. In the NF expression,
as.integer encloses the entire matrix*normalizationFactors product; its closing
parenthesis is after the multiplication. Use one scalar replacement kernel
over the existing normalization row. Preserve checked integer input and
replacement invariants. The earlier parenthesis interpretation was wrong and
is corrected before implementation; it is not a distinct upstream behavior.

For flagged nonzero-after-replacement genes, re-estimate gene dispersion,
evaluate the ORIGINAL trend at the new base mean, MAP-shrink using the ORIGINAL
prior, and refit Wald using the same design/beta-prior policy. Do not estimate
size factors, fit another trend or estimate another prior. NF moments during
the subset refit use the actual subset's nonzero rows. Refit flagged genes
even when no eligible entry changed, matching the source lifecycle.

Global base means/variances/allZero describe replacement counts after a
positive flag. Replacement-to-zero is not an originally all-zero gene.
Preserve source conditional clearing of result columns and the special final
LFC/SE/stat=0,p=1 override, after Cook masking. This differs from mixed-sign
contrast-zero handling, which retains SE. Preserve the independent-filter
population and original feature order; never drop failed genes.

## Durable state and observables

The bundle must distinguish original input counts/diagnostics, replacement
eligibility/flags/count changes, and the new fitted parameters/diagnostics.
Use a typed product-local replacement state and bump the private schema if
needed; reject incompatible old private schemas clearly. Do not flatten these
states into an unexplained optional vector or fake a fit for a zero row.

The released public object retains original mu/H/Cook assays even after its
coefficient/result columns are refit. This is independently confirmed on both
7+7 and7+3 actual marker rows. Keep source-observable diagnostic views separate
from newly fitted GLM means/hat/covariance; do not label one as the other.
Numeric contrasts use released getContrast semantics, reconstructing from
fitted betas/dispersion and replacement counts, not a new iterative fit.

When a nonzero flagged subset is refit, all-replaceable samples clear maxCook
globally; partial replacement recomputes it from original Cook values with
eligible sample columns zeroed. Preserve the conditional branch when every
flagged gene becomes zero. Record whether replacement assays actually exist.
Normalized counts and VST exports use preserved ORIGINAL counts and the saved
normalization/trend; deriving outputs after reload never refits the analysis.

## Integrated validation

Extend the existing source comparator to the four currently excluded default
Cook cases, verifying full mcols, source-observable assays, changed counts,
flags/eligibility, original-count preservation, contrasts/NA/filter/reload and
transforms at1/4 workers. Keep all11 accepted cases and pasilla as regressions,
with the SAME frozen bounds. Preserve original failures and actual fallback
statuses; no seed tuning, table patching or tolerance widening.
Retain dispersion/GLM iteration fields as diagnostic provenance, following the
original native plan. Do not add exact iteration-count equality as a new
cross-BLAS gate when convergence and the declared statistical outputs and
decisions pass. A failed convergence or changed result still requires repair.

Reuse the qualified normalization contract and shared SF/NF replacement kernel.
Keep evidence scope honest: ordinary NF fits and positive SF replacements do
not constitute a separately executed NF-outlier fixture. Do not create a new
source campaign merely to defend the corrected parenthesis interpretation.
One fresh integrated review and exact four-native gate follow the stable
workflow. Local trends, low-df prior and formal performance remain later
release work; keep publish=false and no placeholder subcommands.

All build/data/scratch paths remain external, with fresh APFS/path/free checks.
No local R install or execution. The source profile runs on the existing pinned
hosted R4.6.1/Bioc3.23/DESeq1.52 environment and retains its complete evidence.
