# Grouped i6 whole-feature review

One fresh read-only reviewer inspected all ten new HMM modules, changed
CLI/report/preprocessing code, comparison controller/tests, receipt and CI
wiring against the actual pinned upstream source. All180 worktree source
files matched green25. No builds, downloads, mutations or nested reviewers.

Verdict: pass for committing the bounded unpublished functional slice,
conditional on root confirming the corrected four-native gate. No Critical
or Important findings. The reviewer confirmed depth-normalized calibration,
shared preprocessing, separate expression/states/denoise, draw and bootstrap
order, adjusted emissions, deterministic region rules, fallible workspace
allocation and disjoint chromosome/group workers.

One Minor finishing item: explain the six-state array order, integer state
mapping, regression intercept/slope order, transformed-spike raw SD versus
group-fitted decoder SD, and zero-based half-open gene ranges. Root resolves
this in the product README, not by adding code-narration comments or another
validation campaign. It does not change candidate code or numerical behavior.

Declined-to-judge rulings:

- Final green25 live CI: root confirms run40484702873 success on exact
  worlda133ad5d2a9dfaffee6facfba313ac6db73445af, all four native jobs success.
  Root also verifies all four original archives, source/binary identities,
  complete six-trial summaries and worker fingerprints; see the acceptance
  record. This conditional review gate is closed.
- Strict speed advantage: open. Native process observations and retained
  R in-process timings are unpaired; do not infer a speedup qualification.
- Full inferCNV, biological validity, excluded modes and persisted state:
  outside this slice, still required by the full active portfolio goal.
- Universal numerical parity: not claimed. The retained workflows and
  focused numerical tests establish bounded evidence, not every seed/input.

No separate per-helper review or repeated denoise review was run.
