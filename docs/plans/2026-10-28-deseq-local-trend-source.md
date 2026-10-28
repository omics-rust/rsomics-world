# DESeq local-trend source witness implementation plan

**Goal:** Capture one source-only, complete local-trend/default-fallback/fitted-VST witness before choosing a native kernel or persistence implementation.

**Architecture:** One private R driver has fit and reload modes. The hosted workflow invokes each in a separate R process. Existing private DESeq source-identity, design-preparation, table-export and fit-state helpers are reused; no public foundation or product dependency changes.

**Stack:** Released DESeq2 1.52.0, locfit 1.5-9.12, R 4.6.1, Bioconductor 3.23; Linux x86_64 source oracle. This is not four-native acceptance or performance evidence.

## Approved specification and pins

The approved proposal is `/Volumes/KIOXIA/Developments/tmp/deseq-local-source-plan-20261028.H0F1FW/REPORT.md`; official locfit source qualification is `/Volumes/KIOXIA/Developments/tmp/locfit-official-source-20261028.pWLEjE/REPORT.md`. The executable contract is recorded here so these temporary reports are not needed to resume the workflow.

- DESeq2 source commit `16aeab6d6158bd8cbb8e98764c2f33399b1a3fd3`, archive SHA-256 `8c91699286336350e66eec132ce6fdf5bb4af78e2a4d015a5a61224f62a95984`.
- Official locfit release URL `https://cran.r-project.org/src/contrib/locfit_1.5-9.12.tar.gz`, SHA-256 `fa31b9d407f3557a6fcb7105cbd7b16a32a6fb868a7c60bc38a3e33cb7a39343`, 207112 bytes. Its source member is `locfit/R/locfit.r`, not `locfit.R`; the corresponding Archive URL currently returns404.
- Preserve actual installed source functions, complete package file hashes, registered S3 prediction methods, loaded locfit/DESeq2/stats/libR paths and hashes, session/packages, numerical-library threads and compiler configuration. locfit's unexported methods are read through its namespace/S3 registration, not invalid `locfit::predict.locfit` access.

## Four frozen cases

| ID | Input/normalization | Requested fit |
|---|---|---|
| `pasilla-local-ratio` | Full shipped pasilla, 14599 genes ×7 samples, original identity join/levels, `~type+condition`, ratio SF | local |
| `increasing-default-ratio` | Frozen increasing-dispersion NB matrix, `~condition`, ratio SF | parametric |
| `increasing-local-nf` | Same NB counts and supplied NF only | local |
| `increasing-local-sf-nf` | Same counts/NF plus supplied SF | local |

Pasilla original count/annotation pins remain `4b5570008936786da365d110df1c83c8cffc646d6c667d6b15ecbfb3d9ee735d` and `1844f871ca5d750f41685c73b87bd7dda291ec0d52304c1afb46a17ae88db745`.

Synthetic generation is fixed, not selected after a fit: seed20261028, Mersenne-Twister/Inversion/Rejection; six control then six treated samples. Genes1..1184 use `t=(g-1)/1183`, baseline `exp(log(2)+t*log(2500))`, NB dispersion `.03+.55*t`; final16 rows are zero. Exposures are `.70,.85,1,1.15,1.30,1.45,.75,.90,1.05,1.20,1.35,1.50`. Treated genes divisible by4 have a1.4 mean multiplier; all others have no condition multiplier. Actual `rnbinom` draws use column-major order and size1/dispersion. NF is `E[j]*exp(.18*sin(g/11)*((j-6.5)/5.5))` for every row, including zero rows; supplied SF is `2*E`, without recentering. Retain generation parameters/means and RNG states.

Freeze all input bytes, design/coding/contrasts, generation state, options, controller/source identities and query rules before the first fit. Positive extra queries are `.1,.5,1,2,5,10,20,50,100,200,500,1000,2000,5000,10000,50000,100000`; additional queries follow the fixed rules of all nonzero gene means and the actual VST grid. Do not query the logarithmic trend at zero. Source-derived SF/grid values are results of those frozen rules, not tuning choices.

## Source and persistence contract

Use real public Wald DESeq calls: betaPrior FALSE, useT FALSE, minmu .5, minReplicatesForReplace7, parallel FALSE, ratio SF when not supplied; results alpha .1/BH/independent filtering/omitted real Cook cutoff. Full fitted VST is `varianceStabilizingTransformation(blind=FALSE)`, never subsampled `vst()`.

The default-request case counts as fallback only when actual saved fitType is local and the actual public-call substitution message was captured. If not triggered, retain the complete result as uncovered; no reseeding, changing dispersion, filtering, manual fallback or tolerance changes.

Save the full fit before derived exports, including the local trend closure and captured locfit model/environment. Retain original counts, all assays/row diagnostics, prior/trend state, coefficients, source NA/warnings and results metadata. Source local fitting uses the original ordered `dispGeneEst >100*minDisp` subset and mean weights; the inner local filter is >=10*minDisp, not a replacement outer filter. Prediction uses the actual saved locfit tree/grid/coefficients and its registered method.

Keep source local-VST intermediates:999 positive sinh-grid points; local trend, SF choice/xim, variance/integrand;998 midpoint/trapezoid knots; FMM spline coefficient state and serialized closure; h1/h2/eta/xi and full matrix. Public VST remains authoritative; the trace must reproduce it without fitting. RDS and text tables retain nonfinite values; nullable JSON summaries do not replace numeric/model evidence.

NF controls normalized counts, fit offsets and arithmetic-column moments on nonzero rows. Local VST instead uses existing SF when present, otherwise NF geometric-column means over all original rows. Compare the same-NF pair for equal inference/normalized counts and actual different xim/finite VST values; preserve supplied SF unchanged.

One fresh R-process reload reads all available full saved models and repeats forward/reverse/mixed results, normalized counts, trend queries and fitted VST. It may not call DESeq/dispersion estimation/locfit fitting or regenerate inputs. Compare complete numeric/control state and outputs, not environment pointer identity. Failed cases and missing triggers remain visible and non-green; all artifacts are retained even on failure.

## Owned files and execution

- Create `scripts/deseq_local_trend_workflows.R`: source identity, frozen input preparation, four fits, local/FMM traces, fresh-process reload and coverage witness.
- Create `.github/workflows/deseq-local-trend-oracle.yml`: manual hosted source acquisition/install with exact archives, then the two source processes and unconditional artifact retention.
- Create `scripts/test_deseq_local_trend_oracle_workflow.py`: one compact routing/failure-propagation test and shell-syntax check; no local R execution.
- [x] Implement the coherent source-only route and driver.
- [x] Run the compact routing/shell check and parent source-structure review against pinned source.
- [x] Parent freshly passes both routing tests and the full495-test control suite (two existing skips).
- [ ] Commit and dispatch the hosted run after exact-head CI; actual R syntax/numerics remain unverified locally.

Rulings: parent explicitly forbids Git/worktrees, local R/install, unrelated files and agents, so skill worktree/commit/reviewer automation is not run. Tests are deliberately bounded to the requested routing/CLI/shell contract; source execution and numerical qualification belong to the later hosted witness. Scratch/test fixture outputs stay under `/Volumes/KIOXIA/Developments/tmp`, with fresh external-path/APFS<80% guard before testing or downloading. No temporary evidence is deleted.

Verification on2026-10-28: `PYTHONDONTWRITEBYTECODE=1 python3 -B -m unittest discover -s scripts -p test_deseq_local_trend_oracle_workflow.py -v` passed2/2. The routing test substitutes only the R process boundary, checks distinct processes and retained stdout/stderr/statuses for success, fit failure, uncovered reload and log-write failure; every workflow shell block also passes `bash -n`. Initial absent-workflow RED and a log-write-failure RED were observed before their fixes. This does not execute or parse R, install packages, prove a fallback trigger or provide numerical acceptance. Parent read the complete driver/helpers/actual source and identified the unnamed-vector exporter issue; its bounded fix supplies positional labels, including empty vectors, while full RDS state remains unchanged.

## Review focus and exclusions

Review freeze-before-any-fit ordering; full original/captured model retention; non-exported locfit method qualification; source FMM rather than locfit Hermite; SF/NF priority; absence of fitting/input generation from reload; failure retention/non-green uncovered coverage. A missing fallback remains a missing witness, not an implementation decision.

Synthetic cells each have six samples, intentionally excluding new positive local×Cook-replacement coverage. The actual source preserves its existing original-trend/prior/refit lifecycle; this driver must not alter it. No new low-df, glmGamPoi, blind/subsampled VST, combinatorial outlier campaign, Rust dependency, public API, native correctness/speed or release claim is included.
