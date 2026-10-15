# Private ingestion trial process lifecycle

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development.

Spec: `2026-09-30-infercnv-raw-ingestion-measurement-design.md`.
This is only the process-owner prerequisite for the later offline paired
launcher, not a completed campaign or a public component.

## Global Constraints

- Work only in the world control plane, directly on main as authorized.
- No Rust, R, existing prepared driver, workflow or public API changes.
- Never install, compile, download project data or run R locally. Python
  standard-library tests use external TMPDIR and disable bytecode writes.
- Before tests, inspect APFS container occupancy and external paths/free space;
  boot occupancy above80% prohibits builds, not these bounded stdlib checks.
- Preserve all originals and failures; never delete or overwrite evidence.
- No shell command strings; launch exact argv with a fresh process session.
- Scope is process-group containment for frozen trusted trial children. It is
  not a security sandbox or containment of descendants that deliberately call
  setsid. The later launcher must state this scope and authenticate its inputs.
- A surviving group after any parent exit invalidates that trial even when
  cleanup succeeds. No subsequent trial may run after an owner error.
- Keep source comments rare, stable and contract-focused. No Co-Authored-By.

## Task 1: Checked serial process owner

Files: add `scripts/infercnv_trial_process.py` and
`scripts/test_infercnv_trial_process.py` only. No other tracked file edits.

Provide `run_child(argv, prefix, timeout=1200, env=None, cleanup_timeout=5)`.
Return a frozen `CompletedTrial` with exact argv tuple, positive pid,
zero returncode, monotonic start/end nanoseconds and `group_empty=True`.
The later launcher owns GNU-time wrapping, typed schedule, input/code/runtime
authentication and final sample acceptance; do not add those here.

Validate argv is a nonempty sequence of strings (not a string), timeout and
cleanup_timeout are positive finite numeric scalars (not bool), and evidence
prefix has an existing real parent. Use exclusive files named
`PREFIX.stdout`, `PREFIX.stderr`, `PREFIX.process.json`; reject preexisting or
symlink destinations without reading them. Reserve the report before launch
so even Popen failure has retained machine-readable diagnostics. Never dump
the complete environment or credentials.

Start Popen with supplied env, stdout/stderr raw files and
`start_new_session=True`. Record monotonic launch/end, exact argv, pid when
available, parent exit status, timeout/interruption/launch error, whether
leftovers were detected, cleanup actions and final group liveness in the
process report. Check process-group existence after *all* parent exits,
including zero and nonzero. A successful return requires zero parent status
and no surviving group. Detect lingering descendants, terminate the group,
escalate TERM to KILL within a bounded cleanup interval, reap the parent,
preserve diagnostics and raise rather than returning an accepted sample.
Never signal an unrelated group or the owner's own group. Timeout, nonzero,
launch failure and cleanup failure must all propagate nonzero/domain errors.
SIGINT and SIGTERM during a real main-thread run must execute the same cleanup
before propagating interruption; restore any temporary signal handler.
Do not claim that killed orphan zombies can be reaped by this non-subreaper
owner. A nonempty group still fails closed, never proceeds to the next trial.

Write actual harmless-child tests first and retain a genuine missing-module
or missing-behavior RED before implementation. Cover exact arguments and
paths with spaces, fresh sessions, zero exit/report chronology, nonzero exit,
Popen failure, finite timeout, successful-parent/background-child,
nonzero-parent/background-child, TERM-resistant child, SIGINT and SIGTERM
cleanup, raw output preservation and exclusive evidence files. Tests must
clean up only their own known child processes and must not leave live ones.
Use real children rather than mocks for lifecycle claims; pure argument
validation may be tested directly. Platform-specific assertions may skip on
unsupported systems, but Linux CI must exercise all lifecycle cases.

Run focused tests while iterating and full controller suite once when done.
Commit only these two files after tests/self-review; root owns push, exact-head
CI and independent task review. Report test commands/output, real RED/GREEN,
source scope, files/commit and limitations to the task report. Do not spawn
agents or update other task ledgers.
