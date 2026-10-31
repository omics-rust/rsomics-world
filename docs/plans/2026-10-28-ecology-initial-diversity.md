# Ecology: one count-table diversity workflow

Adopted for source qualification before native implementation. The full
ecology dossier and read-only six-asset report are the starting point, not
the old operation-sized crate interfaces. No target repository or public API
is created yet.

## First complete slice

One `rsomics-ecology` binary with `alpha`, `beta`, `rarefy`, one strict
feature-by-sample TSV count model, shared identities/output and one prepared
phylogeny. Start with nonnegative integer counts only; continuous abundances,
BIOM import, ordination, inference and association remain later slices.
Accept explicit `feature`/`#OTU ID` first-header labels; preserve IDs/order,
allow numeric padding only, reject duplicate/empty/ragged/missing/nonfinite/
fractional/overflow input. Do not build six parsers or expose empty modules.

Alpha selects observed, Shannon, Simpson/dominance/inverse/evenness, Pielou,
bias-corrected Chao1, Good coverage, Faith PD and generalized PD. One selected
multi-metric invocation shares its per-sample summary. Beta selects Bray-Curtis,
Jaccard, Euclidean, Canberra, city-block and three UniFrac modes, one complete
labelled matrix at a time. Rarefy is without replacement with an explicit
recorded NumPy-PCG64 compatibility profile, fixed depth/seed and independent
same-seed vector calls per sample; parallel scheduling does not change draws.
Capture actual source integer/RNG limits before finalizing the native input
range. Depth0, empty features, zero samples and depth>total are explicit cases.
The user explicitly chooses same-dimensional zeros for empty/all-zero depth0
inputs. The captured upstream IndexError remains the source observation;
native successful zeros are a documented bug fix/known incompatibility.

Use existing common/help/phylo-tree0.2/Rayon. The current immutable tree API
already supplies topology/traversals/tip identities and has actual phylo and
composition consumers. No Layer A addition, generic matrix/count/RNG crate,
B-to-B dependency, permanent path dependency or revived historical repository.
Phylogenetic accumulation/root/nonnegative-length policy stays product-local.

## Source gate before native results

One bounded hosted Python source witness, not a separate campaign per helper.
Pin scikit-bio0.7.3 sourcea3beaa632077a532330343d5db0d8e30e58b6705 and
biom-format2.1.17 source2b81a013aae29004d9b82d13847450b27fd6b161. Record
actual installed package/wheel/source/function identities, NumPy/SciPy versions,
runtime/BLAS/thread controls, full inputs and original outputs. Missing oracle
is failure. Future performance uses exactly that qualified runtime/profile;
do not infer RNG stability from the generator name alone.

Reuse compact historical count/tree goldens after checking their meaning,
selected multi-alpha/each named beta driver, both exact-seed rarefaction
sampling regimes, and one integrated invalid/zero/root/identity group. Named
source drivers are the baseline, not artificially slow callable pair loops.
Retain all selected output fields and unavailable-value classifications;
capture source failures as actual expected statuses, never swallowed success.
The source may emit NaN for undefined beta pairs; preserve that observation
without deciding the pending native public boundary or substituting zero.
Freeze numerical bands/exact identities/nonfinite rules from mathematical
and source precision requirements before native output, not after mismatch.
Initial alpha/phylogenetic/beta bands are abs1e-10+rel1e-10; identities,
rarefaction integers and unavailable classifications are exact. Source input
profile conservatively limits each cell/sample total to2^53-1; wider source
observations do not silently enlarge the native contract.

Known root profiles are source-specific: Faith/UniFrac validator accepts root
degree<=2; generalized PD auto means exactly two children. Name the profile,
do not describe these heuristics as biological proof of rootedness. Extra
tree tips are allowed; table features map exactly once, required non-root
lengths exist and are finite/nonnegative. Preserve source stored-root-length
effects. Missing root length follows its declared zero convention.

## Affected public-boundary hold

An explicit user choice is pending for undefined Bray-Curtis/normalized
UniFrac pairs. Dossier requires finite reusable distances; source can return
NaN. Recommendation is an informative fail-loud error and atomic destination,
documented as a stricter input domain. The alternative is a distinct nullable
distance report, not admitting NaN silently into a reusable matrix. Do not
implement/advertise either branch before the answer. This does not block
source capture, other ecology semantics, current performance or the portfolio.

## Implementation and acceptance

After source qualification and boundary resolution, merge selected team-owned
formulas/RNG assets into private modules behind shared validated models. The
six assets are direct/refactor inputs, not six binaries. Keep source comments
rare and explanatory. Historical alpha `all`, silent beta header skipping,
empty-rarefaction indexing and duplicate mutable-tree stacks are not retained.

One integrated lifecycle rarefies a table, reuses its output for multi-alpha/
phylogenetic metrics and beta modes, checks exact identities/draws and source
numeric results at1/4 workers, and covers the compact invalid/atomic group.
One whole-feature review, fmt/strict Clippy/existing tests and actual native
Linux/macOS x86_64/aarch64 follow on the same exact source. No per-helper
review or artificial exhaustive section campaign.

Use one genuinely observed public community table with matching phylogeny for
representative whole alpha/beta/rarefy timings and RSS, not synthetic tiling or
an easier subset. Dataset scouting is read-only until identity/license/size
and acquisition are explicit. Record original versus frozen-export formats.
Fixed warmup/repeats, source defaults/BLAS1, native1/4, command/CPU/memory/I/O/
bytes and unchanged outputs are required. Each advertised hot-path class
needs strict throughput OR material resource advantage; scaling is separate.
Do not borrow legacy README speed claims or time an unfinished mode.

The source task owns only its workflow/helper/compact routing tests. Parent
owns world state/Git, adoption/dispatch, product creation and public gates.
Local scratch/downloads/builds remain external with APFS/resolved-Cargo-target
guards; hosted scratch/package caches remain RUNNER_TEMP. No local package
installation or benchmark is needed to prepare the source witness.
