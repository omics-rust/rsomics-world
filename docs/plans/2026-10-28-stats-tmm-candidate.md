# Default TMM from two actual consumers

Adopted after complete native/performance baselines for limma and the existing
edgeR normalization contract. This is one candidate in existing rsomics-stats,
not a new crate/general expression model or permission to release edgeR.
Read-only assessment0e9fbe39... and actual identical149-line kernel/call sites
are recorded in tmm-shared-kernel-2026-10-28.md.

## Verified base and ownership

Local stats/main and fresh remote main both equalbac010ed3abf1e003729ec6189845d0d50b99578.
Fresh crates.io API (meaningful User-Agent) reports non-yanked0.2.4/checksum
fa3a27a3004b94df8e54f053fd71c052795ed08fd2382d42e596c75132189fcc,
matching cached published source. Initial anonymous default-UA403 is an access
failure, not evidence of a newer release. Candidate version is0.2.5, unpublished.

Owner Cargo.lock is inherited, hash
4b664c523c043985df8f5a3cd1eea135fe0390039b87e8c2dfa4739869038dc7,
self0.3.0/removedcommon dependencies despite manifest0.2.4. PRESERVE EVERY BYTE.
No Cargo/metadata/fmt invocation in that dirty owner repository. No reset,
stash, file replacement or staging of inherited contents. Production source/
manifest/docs edits belong only to owning stats; compilation/formatting/tests
use an external clean verification copy with its own generated candidate lock.
Keep existing common0.6/statrs/thiserror dependencies; unrelated cleanup stays
out of this concern.

Parent may later stage the separately validated NEW canonical lock blob into
Git's index without modifying the working inherited file, just as partial
staging preserves unrelated hunks. It must be generated from the tracked/
published dependency base and candidate manifest, not copied from the inherited
lock. Review its scoped diff first; preserve owner lock hash before/after.
Worker must not stage/commit the owner or perform that parent-only operation.

## Minimal candidate API

One numerical module/function and named result:
`normalization::tmm_factors(retained_rows: &[&[f64]], library_sizes: &[f64])`
returns reference_sample (zero-based caller sample order) and factors. Validate
its public borrowed-row shape/finite-nonnegative/nonzero-row domain and positive
finite supplied libraries once, with narrow existing StatsError style. No IDs,
serde, full library state, filter/design/QL/voom policy, public quantile/rank,
generic matrix trait or configurable normalization framework.

Move current team-owned default arithmetic without changing reduction/tie/
NaN behavior: reference normalized75thquantile/fallback,30%/5% trims, fixed
weights/Acutoff, first-sample ties, NaN-only weighted-sum skipping and geometric
centering. Do not revive old donor finite-weight skipping. Private helpers
remain private. Reject invalid centered factors; no fabricated success.

Important size correction to the initial assessment: `&[f64]` is a fat pointer,
16bytes on64bit, so8066borrowed rows use129056bytes, not64528. No count-matrix
copy is needed; the allocation and public validation are NOT zero-cost and
must be included in checks.

## Two real candidate consumers, no production adoption yet

Prepare isolated exact-head copies of edgeR3a97e8f4... and limmaabb4a01... .
Replace BOTH actual normalization::prepare calls with the candidate function;
remove only superseded TMM/rank functions, retain product filtering/median/
library recomputation/effective offsets/QL/voom. Add one assertion to each
existing consumer lifecycle contract for reference/factors/effective sizes/
offsets/retained mapping. Use actual original source normalization fields
under unchanged TMM/offset abs+rel1e-10 and library abs1e-8+rel1e-10.

Owner products remain clean on their accepted registry0.2.4 baselines until
publication/adoption is separately qualified. Save exact real consumer patches
and their test/evidence records externally; do not pretend they are already
adopted production commits. Temporary verification copies may use a local Git
candidate revision as an immutable Git source (no Cargo path dependencies,
B-to-B dependencies, permanent local URL or public registry/name reservation).
Only those new scratch Git repositories may be initialized/committed by the
worker; no owner Git/index/remote mutation or push/publication.

Run candidate library/real consumer existing tests, strict checks and actual
normalization contracts. A real full before/after consumer path may compare
unchanged native normalization/state; do not rerun upstream R or repair the
held edgeR P/FDR gate. Passing shared TMM never means edgeR full release passed.

## Proportionate hot-path check and handoff

Use original retained8066×7 rows/libraries, no tiling/prefiltering/timing tuning.
Compare the current verbatim team private kernel against candidate kernel
INCLUDING borrowed-row adapter/public validation. Scratch-only test harnesses
are not new public crates. Fixed warmup and5alternating old/new trials with
100calls each; record all distributions/compiler/host/inputs/flags/output
equality. No claim of whole-product speed from this primitive benchmark.
Flag >10%median regression for investigation before promotion; do not retune
data/reps/criteria afterward. Report129056-byte adapter arithmetic, not a
separately measured RSS win. No local R/new source campaign.

One bounded candidate/consumer review follows working code. Public promotion
still needs two demonstrated consumer contracts, policy-free API/hot-path
review and native4/release gates. Product registry adoption/new final-binary
qualification follows actual foundation publication; existing accepted
performance cores remain attributed to their original binaries.

All scratch/caches/build paths external, fresh APFS/resolved-target guards on
EVERY test/build/download. Verify in the correct clean copied manifest, never
dirty stats owner; a common guard invoked with that copy as cwd resolves its
actual target. Stop affected build if boot>=80% (latest79.1%). No deletion,
global toolchain/environment change, CI dispatch, publication or actor-message
approval workarounds. Source comments stay rare; record provenance in docs.
