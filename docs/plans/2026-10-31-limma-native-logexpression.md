# Native ordinary log-expression lifecycle

Execute as one coherent product extension with one independent whole-feature
review and one bounded fix pass. Parent owns Git, source acceptance and hosted
dispatch. No new foundation or operation-sized crate.

## Scope and prerequisite

Complete the existing limma first-release input pair: current voom counts and
supplied ordinary log-expression. Source plan is
2026-10-31-limma-logexpression-oracle.md; pinned limma3.68.5 source is
825d1c8330d024adb6c54af66d79b0f4ec258530. Run41184814043 at world06837d86
is source-qualified: parent reads complete retained REPORT2f5f036f854b869089db5164b10449cf96a573d119cec999f2bb7cad1a73e302,
verifies original ZIP/all1164member bytes and reads successful capture plus
independent no-refit reload. All324result/reload file pairs match. Parent has
authorized implementation. The failed original41182610397 stays immutable RED.

Owning repository: /Volumes/KIOXIA/Documents/omics-rust/rsomics-limma,
clean policy-only headb54ad8c. Cargo home/target/rustup and all cache/scratch
stay under /Volumes/KIOXIA/Developments/. Use scripts/guarded-cargo.sh for
every Cargo invocation; no global boot occupancy gate. Final reports and
command logs use a fresh external scratch directory. Do not alter unrelated
repositories or any accepted source/performance artifact.

## Product architecture

- Preserve `run --counts` and schema1 serialized count semantics. Add mutually
  exclusive `run --expression` for supplied already-logged finite signed E;
  never infer the input kind or log/normalize E. All commands retain common,
  help, ThreadArgs and transactional-output plumbing.
- Keep counts and expression as distinct typed inputs. Reuse lexical TSV,
  identity joins, named full-rank numeric design and contrast parsing rather
  than copying those parsers. Missing/duplicate IDs, nonfinite E/X, rank loss,
  n<=p and malformed complete contrasts fail at the existing input boundary.
- Move the shared semantic full linear fit out of the voom-specific model:
  design, beta, unscaled SE, sigma, residual df, covariance, rank, pivot,
  method, Amean and expression. Count-only targets, weights, zero-loss state,
  LOWESS points/curve/span remain count-specific. Flatten the shared part in
  the existing count encoding so its field layout and numerical values stay
  compatible; do not populate imaginary voom fields for ordinary input.
- An explicit schema2 ordinary bundle records input_kind=log-expression,
  the supplied E/digests/metadata/X, complete shared base fit, fixed source
  defaults, initial complete contrast/moderation and source provenance.
  Product-owned dispatch/load validates the actual schema and type. Retain
  schema1 loading and count commands; no public cross-product object model.
- Ordinary fitting prepares `System::new(X,None,None)` once, shares it across
  independent feature fits in one Rayon pool, and retains every input row.
  Do not run filtering, TMM, Poisson probes, voom/vooma, weighting, duplicated
  probes, blocking or missing-data fitting. General named X and mixed contrasts
  use the existing algebra, not special two-group code.
- Both inputs derive contrasts from saved uncontrasted full base state.
  Reload never preprocesses/refits or mutates the bundle. Preserve the actual
  default equal-df legacy moderation, infinite prior df and uncapped F-test
  denominator; the accepted unequal-df count path remains unchanged.
- Retain legitimate one/two-feature behavior and zero-variance nonfinite
  statistics using the existing explicit Number representation, extending
  result fields where needed. Numeric JSON null is not an encoding. Keep
  NA/NaN/+Inf/-Inf distinct when the retained source distinguishes them.
  Finite count results retain numeric JSON and existing TSV values. Follow
  actual t-mixture/BH missing-value population semantics, not a blanket
  finite-output assertion or a special fixture-only fallback.
- `export --kind expression` emits supplied/model expression for either
  input; fitted/residual exports use the full base. Existing log-cpm/weights
  remain valid for counts and fail explicitly for ordinary input. Native
  output keeps stable supplied feature IDs; record the source's single-row
  positional mapping/name loss as provenance, never repair source artifacts.

## Compact implementation and validation

1. Read actual source report, exported source states and relevant retained
   lmFit/lm.series, squeezeVar/fitFDist, eBayes/tmixture, topTable and contrast
   bodies. Read the complete owning input/model/linear/voom/EB/results/artifact/
   CLI path and existing integrated workflow before editing.
2. Add one integrated ordinary lifecycle test covering signed E, reordered
   metadata/design, shared full fit, forward/reverse/mixed saved contrasts,
   supplied/fitted/residual exports, workers1/4, tiny positive and one-zero
   outcomes. Include only meaningful input/output boundary failures in this
   test; no exhaustive per-helper defensive campaign. Observe real RED before
   implementing the coherent extension. Keep existing count tests intact.
3. Add product-local scripts/check-logexpression.py using the fully retained
   successful ordinary source witness, exact manifest/witness/input/source
   identities and original output mappings. It runs all five frozen cases at
   workers1/4, complete base/contrast/EB/table/export checks and no-refit reload;
   preserve raw argv/stdout/stderr/status and all mismatches. It does not run
   R, synthesize answers or depend on another Layer B crate.
4. Fixed bands are those adopted before native results in the source plan:
   E abs/rel1e-10; linear1e-7/1e-6; EB1e-6/1e-4; df1e-3/1e-4;
   odds1e-5/5e-4; probabilities1e-12/5e-4 and positive log error<=1e-3.
   Identity/order/dimensions/classes and worker equivalence are exact. No
   post-outcome tolerance tuning, rounding-to-source, epsilon variance fixes
   or output-dependent source changes. Contradictions stop the affected gate.
5. Run fmt, strict all-target Clippy, debug/release tests, actual complete
   ordinary comparisons and one cached accepted count-source comparison.
   The last is a local regression of shared-state extraction, not a replay of
   the accepted count source/native4/performance campaign. No local R install,
   import or upstream execution. Retain failures honestly; no skip-as-pass.
6. Update owning README with the two truthful input lifecycles, schema/export
   contracts and exclusions. Sparse stable source comments only. Return one
   compact REPORT, complete source/fixture/script manifest, frozen candidate
   archive and raw verification logs; no Git staging/commit/push/publication
   by the implementer. Parent reads the actual code and one independent
   whole-feature review, fixes material findings once, then adopts.

## Gates after adoption

Freeze the reviewed owning commit for one four-native new-path gate, retaining
actual hosts, binaries, all tests, source comparisons and worker evidence.
Then measure one fair complete supplied-E workflow against pinned source on
the representative8066x7 input: equivalent named outputs and state reuse,
warmup/distributions, CPU/thread controls and peak RSS. Do not use small
synthetic cases or count timings as ordinary-path performance. No published
speed claim, registry retry or release is authorized merely by local success.
