# Phylo WPGMA execution ledger

Plan: `docs/plans/2026-11-03-phylo-wpgma.md`.
Owning base8971f75d6649776ecad54c8e0f0ee8427f0abc63, clean before edits.
Existing dossier explicitly includes WPGMA as an inference method.
Root reads complete pinned scikit-bio0.7.3 `_upgma.py`: weighted=True selects
SciPy weighted linkage and the common linkage-to-tree constructor.
Official SciPy documentation specifies equal weights for the two clusters.
Current implementation already uses nearest-neighbor-chain linkage and a
common typed matrix/tree model. No new foundation or public model needed.

Pre-flight: single task, no cross-task interfaces. Existing UPGMA and new
WPGMA share private linkage; retain exact existing size-weighted expression
for UPGMA to avoid numerical regression. The WPGMA expression is `(a+b)/2`.
User's unattended/minimal-test/direct-main instructions override repeated
approval gates, per-helper matrices and isolated-worktree scaffolding.
Production implementation and local correctness gates below; performance,
native4 CI and release gates are not yet qualified.

## Implementation and evidence

One integrated CLI test is added, not a helper matrix. Root observes RED
session96306 exit101 because wpgma is not recognized. Native implementation
adds private PairGroupWeight::{Taxa,Clusters}, shared reconstruction and typed
InferenceMethod::Wpgma/CLI mapping, preserving the existing UPGMA arithmetic
and NJ implementation. No dependency or package-version change.

Fresh whole-feature reviewer `/root/phylo_wpgma_whole_review` (astra/high)
finds one Important caller contract gap: the NJ-only negative-branch override
was rejected for UPGMA but accepted/ignored for WPGMA. No Critical or Minor.
The existing integrated test is extended; RED94555 exit101 confirms that gap.
One fix pass changes the existing CLI guard to reject the override for any
non-NJ method. No second review or per-helper test ladder.

The first actual source-oracle run54238 exits1 because our new two-tip
expectation was wrong: scikit-bio0.7.3 `_validate_dm` requires at least three
identities. Native two-tip extraction had succeeded immediately before the
source exception. Root reads the actual validator and corrects only the new
WPGMA method to return typed TooFewWpgmaTaxa for two identities; old NJ/UPGMA
contracts remain unchanged. The source oracle now checks both two-tip
rejections and named-output preservation, not an invented source tree.

Final session86950 exits0 after fmt, complete debug and release all-target/
all-feature tests, and strict Clippy. Each profile has17library+4CLI passing,
zero failed/ignored, plus both benchmark smoke programs. Final source-oracle
session7783 exits0:135sequence-distance comparisons,64inference checks,
11tree checks and Hommola statistic. Maximum inference branch score difference
4.2631696775044585e-15. The inference set is20random matrices for each of
NJ/UPGMA/WPGMA plus3valid deterministic WPGMA cases and1two-tip rejection.
trim_checks=0: no available trimAl supplied; do not claim fresh trim coverage.

Actual existing runtime is external environment
`/Volumes/KIOXIA/Developments/venvs/rsomics-composition-skbio-0.7.3`, with
scikit-bio0.7.3, DendroPy5.0.8 and SciPy1.18.0. Its existing interpreter
resolves to read-only Homebrew Python; bytecode disabled, all tmp/cache and
MPL/Numba paths external, OMP/OpenBLAS/MKL threads1. No dependency downloads.
Every invocation uses the BED physical-external guard: USB disk12s1,27GiB
free and actual Cargo target `/Volumes/KIOXIA/Developments/cargo-target/rsomics-bed`
verified even though the owning manifest is phylo. This shared external
target was deliberate reuse, not a boot-disk fallback or path dependency.

Packet `/Volumes/KIOXIA/Developments/tmp/phylo-wpgma-20261103.s2TgfAf5` retains
the failed oracle logs and distinct final logs; no failed evidence overwritten.
Final stdout SHA256:
debug5a14d1cee266de4d34eacf66b651a868577adb97209de0bdfa9a754a78401bbb;
releaseec20e9712c9ca942b93e11f5f36ac7ac85c5fe8f91b8ae6d62c9bcff5a64f860;
oracle3436d4031b6c4d42dd41c1f8ec44c2ca954fb2c121f463c338c5ecfe0c2ab8a9.
Clippy stderrb0b6d99cba1aa3dec30e7a3adfb5e327dd788939f9d8b04fa406e1c779d5a2d2.

Review declined compiler/oracle/native/performance/release/publication claims.
Root closes only actual local compiler/oracle items above. Reviewer inspected
SciPy1.16.3 algorithm source; actual SciPy1.18.0 runtime differential, not
that source version alone, owns compatibility evidence. Native4/performance/
release remain open. Existing timing and combined NJ+UPGMA RSS do not qualify
WPGMA. Historical fixture directory cited in phylo benchmark README is
currently absent; use a new shared input, do not assert it was recovered.

## Native and representative resource gates

Owner784cef7cd42640880a1d24f35954edea54008fdb is committed/pushed and clean.
Exact-head native4 CI41586653945 passes Linux/macOS x86_64/aarch64, debug
and release, with17library+4CLI tests and both benchmark smoke programs.
Linux also passes fmt, strict Clippy, docs, package and actual full source
oracle including trimAl. World1660b7be CI41586673846 passes.

Root reads complete external performance proposal/runner before execution,
replaces an unnecessary source full-table string/float-list representation
with NumPy loadtxt before freezing/running it. No predicates or measured
results exist at that point. One actual prepare44884 exits0; one campaign
39138 exits0, no retry. Physical-external guard is rechecked before each.
Packet and canonical pins are in
`.autopilot/oracles/phylo-wpgma-performance-2026-11-03.json`.

Same119918537B labeled2500-taxon matrix, one retained warmup/tool then five
alternating pairs. Each output is checked for rooted RF, all tip identities
and full branch score. Root's independent replay99562 exits0: input/helper/
binary/output hashes, exact alternating order/status, original raw RSS and
stdout records, all12complete trees; clade maps including tip branches have
maximum absolute difference0.0. This is not a sampled checksum-only check.

Median maximum-process RSS native224673792B versus source275070976B, lower
50397184B/18.3215%; all measured native RSS values are below every source
value. Accept this representative workflow resource gate. Median full-process
wall0.290964458s versus2.372631208s is secondary: Python imports/startup are
included and do not establish a kernel throughput advantage. Source inner
times0.0676–0.0759s have a different scope; native core is not measured.
All raw vectors and both warmups are retained, with no cold-cache, affinity,
parallel-scaling or universal-platform performance claim.

This qualifies local source784cef7 resource use, not another binary/version.
No package-version bump, GitHub release or registry publication is performed.
The next owning change is release metadata/documentation and actual native/
source delivery preparation; do not advertise an already published WPGMA.
