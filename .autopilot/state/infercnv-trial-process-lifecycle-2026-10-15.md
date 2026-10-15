# SDD ledger — plan: docs/plans/2026-10-15-infercnv-trial-process-lifecycle.md

Spec: docs/plans/2026-09-30-infercnv-raw-ingestion-measurement-design.md.
Base world0aff9f8eca362b93be734758a5ecfd2f1208ab57.
Status: task1 dispatched to ingestion_trial_process_impl with isolated two-file
scope. Implementer reports14 real-child tests GREEN after genuine absent-module
RED; full suite/self-review and root task review/CI remain pending.

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
