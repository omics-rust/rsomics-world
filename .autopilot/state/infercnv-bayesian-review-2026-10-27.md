# Bayesian whole-feature code review

Ruling: the integrated candidate and comparison are now complete as code;
perform the single fresh-context feature review while the installed R witness
finishes, not a new review of each helper. Actual posterior compatibility,
four-native execution, performance and final acceptance remain open. Review
cannot replace those gates. Cost if wrong: repair this unpublished candidate
and rerun the whole-flow gate, not publish unverified behavior.

Product base7853e92282c43b01d785df40fd19b6a933d035d9, current uncommitted
integration frozen as green26 under sc-cnv-core-2026-09-26. Archive
93e0e287bebdf8dfb0e804eccc9cf32e97b2634e49becbc55f276daefd945d22;
manifest2392df2566f1b45bac147fb55ac834a98656f8ae5aa5a716d8ae1703c1076b24,
182 files. Root standalone formatting/parser check passes; no local Cargo
build/test is allowed because boot APFS91.74% occupied. Rust tests are unrun.

Review product src/cnv/hmm/bayes/{mod,sampler}.rs plus changed HMM/cnv exports,
CLI/report and CLI contract test. Review control-plane Bayesian driver,
uncertainty reducer, whole-flow comparator and optional CI routes from
9dd9dfc6; current world5077d6bd adds only continuation/RNA-seq docs. Consult
docs/plans/2026-10-27-infercnv-native-bayesian.md and the execution ledger for
contract/rulings. Preserve unrelated inherited VCF/root scratch changes.

Source model and installed methods are retained in fixture
evidence/infercnv-extensions-2026-10-27/run-40491608980/artifact/bayes-workflows/
metadata; upstream commit is b421d9405c97a309b081ef86d455e976df93eae4.
That run's actual first Bayesian call/filter/report passed, then the collector
failed on NULL calibration names. The original failed artifact is preserved,
not accepted retrospectively. Root's unnamed-vector record test observes
RED then GREEN on existing4090 R4.6.1; corrected Bayesian-only run40499067515
is live on world9dd9dfc6. Do not review or rerun accepted grouped-HMM/denoise
helpers as separate campaigns. Ward policy is independently held for a user
decision after actual membership corruption was measured.

Review focus: true shared Dirichlet/categorical target; raw spike likelihood;
mixture versus per-cell probabilities; original IDs/retained neutral maxima;
strict threshold; immutable original expression/HMM states; memory budget and
indexed worker reproducibility; useful complete CLI/report integration; actual
comparison coverage and stochastic limits. Source1st-repeat probabilities
include near-tied maxima, so root must diagnose actual repeat variability
rather than silently relaxing exact-decision criteria. Code review is not a
convergence, arbitrary-input, biological or replacement-speed proof.

Reviewer is read-only, no subagents, compilation, test/install, downloads,
staging, commits, deletion, publication or global memory writes. Return
strengths, Critical/Important/Minor findings with exact paths/lines, any
declined-to-judge behaviors and a clear conditional code verdict. Findings are
graded by user effect, not by whether an input was enumerated in the plan.

## Review outcome and single fix pass

Fresh reviewer bayes_whole_feature_review found no Critical issue. One
Important: native refinement omitted the source stage prerequisite that skips
an entirely single-valued HMM matrix, including a non-neutral state. The real
full fixture satisfies that prerequisite, so its comparison cannot expose the
missing branch. Root independently reads pinned ops.R:1364/1407 and verifies
the exact predicate in existing4090 R4.6.1: constant2/3 skip; distinct constant
groups and mixed paths run.

Ruling: re-grade the help finding from Minor to Important: zero could be read
as maximal filtering but actually disables the entire refinement. That affects
interpretation of analysis output, not just prose polish. Fix both findings
in the same pass; no remaining Minor was reported. Cost if wrong: one explicit
CLI sentence and regression assertion, with no extra reviewer campaign.

Root writes the global-non-neutral/distinct-constant regression and help
assertion before changing production code. Candidate now checks the global
state prerequisite before active workspace sizing; bypass copies original
groups without posterior work. Zero behavior is explicit in help. Formatting/
parser checks pass. Rust regression RED/GREEN and full suites remain unrun:
local builds are forbidden and the user rejected per-helper native campaigns.
Do not report the fix verified until the integrated native gate executes it.
Freeze green27 and use it instead of green26 for that first complete gate.
No second reviewer is requested; root owns one fix-pass runtime verification.

## Rulings on declined-to-judge boundaries

- Native compile/Clippy/tests/four-platform execution: hold acceptance pending
  the actual integrated gate; static review does not replace runtime proof.
- Convergence and near-tied/near-threshold decisions: inspect actual two-repeat
  coda/chain evidence; descriptive MCSE alone is not a convergence claim.
- Performance/worker scaling: retain observations but keep representative paired
  replacement-speed and useful-worker gates open; no inferred speed claim.
- Exact posterior ties/all-regions-removed: native deterministic/empty-output
  handling is coherent, but upstream parity on those edges is unestablished.
  Do not claim it without actual source outcomes; resolve any demonstrated
  intent contradiction before accepting that affected behavior.
- JAGS bit streams: intentionally excluded. Native keyed-worker reproducibility
  remains required; this does not imply a shared JAGS seed.
- Ward/i3/cell/subcluster modes/biological validity: absent from this feature's
  acceptance, not dropped from the active portfolio or full-CNV work.

Final code verdict: with the above fixes, conditional on root's outstanding
runtime/statistical/performance gates. This is not native acceptance or release.
