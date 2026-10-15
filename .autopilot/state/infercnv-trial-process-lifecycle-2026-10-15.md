# SDD ledger — plan: docs/plans/2026-10-15-infercnv-trial-process-lifecycle.md

Spec: docs/plans/2026-09-30-infercnv-raw-ingestion-measurement-design.md.
Base world0aff9f8eca362b93be734758a5ecfd2f1208ab57.
Status: task1 source/fix reviews pass through761e3eeb.18 focused/301 full
stdlib tests pass locally; root push/exact-head Linux CI pending. This is not
the complete paired launcher or a performance acceptance.

| Task/interface | Producer/consumer | Preflight finding |
| --- | --- | --- |
| Task1 self | two new files; genuine child tests against process owner | consistent; no existing prepared-driver mutation |
| Task1/later launcher | CompletedTrial or propagated failure | failure cannot become Sample; launcher must abort before next slot |
| Task1/spec no overlap | fresh process group and final group liveness | bounded trusted-child containment, not setsid sandbox |
| Task1/evidence | exclusive raw/report files | preserves Popen, timeout and cleanup failures; no replacement |
| Task1/platform | stdlib POSIX process behavior; Linux CI | local no-build/APFS constraint preserved; Linux native evidence required |

Ruling: choose bounded process-group ownership for frozen trusted factory and
validator children, not a new subreaper/sandbox framework — no accepted source
detaches sessions, and the owner rejects leftovers on every exit. Cost if
wrong: a future detaching child requires stronger containment before use;
this component must never advertise escaped-descendant protection.

Future launcher runtime requirement: rerun and compare actual shell-to-R
Python identity after dependency installation and around campaign execution.
The current initial probe precedes apt/R installation and cannot alone prove
unchanged dynamic-library identity throughout a later32-process campaign.
This is not waived by matching executable hashes or a Python minor version.

Current installed-contract run38700520489 proceeds independently. No raw
paired performance, CLI, public release or full inferCNV acceptance is implied.

Installed-contract run38700520489 now accepted after original evidence audit;
root records it in separate530b2250 documentation/receipt commit. This does
not change the process owner's source or scope. Root will push only after the
new task's source review and exact final-head controller verification.

Skill scratch deviation: invoking the workspace helper from the plugin's own
checkout created a small brief/marker on boot storage. Root immediately moved
that new directory intact to external tmp; no evidence deleted. Reinvoking
from world creates the task workspace on the allowed HDD. Use world as helper
working directory thereafter; no local build/R/install was performed.

Task1 source commitaf273ff70a07e9044eca9641eb1cf6e9952b3f64 reports genuine
missing-module RED,13 final focused/296 full stdlib tests passing. Fresh task
review requests changes on2 Important findings: late SIGINT/SIGTERM during
final report flush can be swallowed after the last pending flag check, and
signal-test fallback can kill its owner before readiness without retaining a
discoverable child-group PID. No Critical findings. Root confirms both are
load-bearing interruption/cleanup gaps. Fix round1/5 returns both findings to
the original implementer with targeted regressions; no push/acceptance yet.

Task1 fix round1/5:9b794802d18fe9465ccacccd1e21b31bb40e9ef6 reproduces3
targeted REDs then passes16 focused/299 full tests. Scoped review confirms2
original findings addressed but finds1 new Important test-safety gap: the
launch witness is visible before its PID write completes, and its read-error
finally can kill an owner with no known group, bypassing cooperative cleanup.
Fix round2/5 is a test-only complete-witness/unconditional-cleanup correction;
source handoff/PID publication is accepted by the scoped review, not yet CI.

Task1 fix round2/5:761e3eebd87a513477cd1eef856dd3a7c1e161c4 changes tests
only. Actual empty/directory witness REDs precede complete atomic publication,
bounded valid reads and unconditional cooperative owner cleanup.18 focused/
301 full tests pass. Scoped review confirms the new finding ADDRESSED, with
no new Critical/Important issues or out-of-scope observations. Source scope
remains the2 assigned files; production stops still cannot return accepted
samples, and test cleanup no longer relies on file existence as complete PID.

Root accepts task source after these reviews; Linux exact-head execution and
the later aggregate launcher review remain distinct. No escaped-session
containment, all-platform release or actual campaign claim is made.
