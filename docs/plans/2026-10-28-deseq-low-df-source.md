# Low-df prior source and fixed-operator qualification

This is a source-only qualification, not a native feature or adopted constant
table. Keep the accepted15-case source artifact and current green2 untouched.
Use the same exact DESeq2 1.52.0/R4.6.1/Bioc3.23 source/runtime pins. No local R.

## Actual workflow inputs

Generate one independently frozen1,200gene×10sample, five-pair count matrix:
seed20261030, explicit Mersenne-Twister/Inversion/Rejection, base means
2^runif(3,10), true dispersions (0.08+2/base)*exp(rnorm(0,1.2)), per-gene
condition effects rnorm(0,.3) and pair effects rnorm(0,.15). Record all draws,
mean/dispersion parameters, library exposure and generator provenance before
any fit. No gene filtering or native-driven input tuning.

Use nested2/3/4/5-pair identity subsets, drop unused pair levels explicitly,
design~pair+condition and named condition contrasts. These give residual
df1/2/3/4. Use actual public DESeq Wald calls, explicit fitType=mean and disabled
replacement to isolate prior estimation. The mean trend is a real upstream
mode, not a fallback invented by the native code. Record full fit, all source
diagnostics, initial and reloaded forward/reverse/mixed results, original
normalized counts and fitted VST, preserving every gene/NA/convergence state.

Freeze all input bytes, contrasts, options and existing numerical bounds before
the first fit. At least one df1..3 case must exercise a prior strictly above
.25; otherwise positive-prior coverage is incomplete, not an accepted fixture.
Retain failed or uncovered cases without adjusting a candidate's bounds.

## Data-independent constants

Read released core.R:1136-1209 and actual R4.6.1 loess/hist implementation.
Independent read-only qualification finds the fixed Gaussian one-dimensional
loess predictor mathematically linear: one fit, weights1, X-only geometry,
no one-dimensional normalization and no residual scale in se=FALSE prediction.
This does not imply floating-point bitwise equality or universal R equivalence.

Generate an independent constant bundle using only fixed algorithm settings:
seed2/RNGkind, df1/2/3,200 source variance-grid doubles,10,000 chi-square then
normal draws per grid point, strict(-10,10) filtering and source histogram
breaks. Store3×200×40 reference histogram COUNTS, not fitted priors. Each density
uses its actual retained count total, not10,000; small remains data-dependent.

Generate the1000×200 fixed loess prediction operator by basis responses using
the exact source grids/span/default Gaussian/interpolated controls. Preserve
signed weights and IEEE f64 bits; no pruning, absolute values, normalization,
clipping or epsilon tie rules. Store grids/breaks/row-major layout/endianness,
hashes, generator, exact package/runtime/RNG and algorithm provenance. Constants
contain no gene/sample/dataset IDs, expected prior values or fixture branches.
Do not copy them to production until consumer qualification passes.

Freeze independent operator probes before reading workflow outcomes: constant,
linear, quadratic, localized peak and mixed smooth responses. Compare all1000
predictions to direct R loess with abs<=2e-11+5e-13*abs(reference); do not widen
after a mismatch. Record linearity residuals and preserve complete values.

For df1/2/3 workflows, derive real observed residual histograms, per-grid small
and all200 KL values, then direct and operator1000-point predictions, first-min
index/gap and prior floor. Compare to the actual package prior, preferably
reinvoking the unmodified qualified private estimator on its actual fit.
Freeze/check that function body against source as well as the existing source
identity checks. No installed source function is patched or monkey-patched.
The df4 control must use the ordinary non-Monte-Carlo prior path.

Require exact source/operator argmin and prior agreement on the qualified
workflows, with the prediction bound above and recorded nearest-minimum gap.
A near-tie failure rejects this operator qualification; do not fix it by
changing seeds, inputs or selection rules. Reference histograms may still be
useful if a factorized source-order predictor is subsequently needed.

## Delivery and boundaries

One bounded new source driver/workflow and compact routing check reuse the
existing source/export helpers. An optional explicit-contrast hook in the
private case-preparation helper is allowed if it preserves existing defaults;
do not duplicate the complete driver or add a generic framework. Retain all
raw RDS/tables/constants/metadata/conditions/witness/manifest through failure.
Separate actual source-case success from operator qualification in the witness.

No native compatibility, four-platform, performance or publication claim is
made by this source run. Runtime native code must still calculate real observed
histograms, KL values, predictions and argmin from the actual input. All
source/data/scratch stay in authorized external paths or hosted runner-temp.
