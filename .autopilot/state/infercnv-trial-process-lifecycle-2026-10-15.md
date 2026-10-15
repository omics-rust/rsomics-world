# SDD ledger — plan: docs/plans/2026-10-15-infercnv-trial-process-lifecycle.md

Spec: docs/plans/2026-09-30-infercnv-raw-ingestion-measurement-design.md.
Base world0aff9f8eca362b93be734758a5ecfd2f1208ab57.
Status: task1 accepted through761e3eeb and root58dd7922, exact-head Linux
CI38704356107 successful.18 focused/301 full stdlib tests pass locally and
all301 pass on Linux. This is not
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

Root pushes58dd79223c1e89562bb75bf6122e8fa4d460f37d; exact-head Control
plane38704356107 succeeds. Original run/jobs and full log ZIP are retained
externally at run-38704356107-process-owner-ci. Root verifies head/event/path,
successful unique test/architecture steps and log CRC; actual Linux log says
301 tests/21.640seconds/OK without FAILED or ResourceWarning. Logs SHA256:
`d4503c28f080c818a9e3aa49bb76c6d9f4f4a6f651f000f4a7525066d7da268a`.
Accepted source SHA256:
37c1d558524ebc6ec48dd90d16dc36d231ccece431e894e2a534a69f088b2444
(process owner),
59171af29413f0f911886ef5f6504f2920a50796846c4e65cfdc1941bffa3b90
(tests). Source is accepted only for documented trusted process-group scope;
metadata/paired launcher/integration and whole-launcher review remain open.
