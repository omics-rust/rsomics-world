# One real Python identity capture for paired ingestion

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development.

Spec: `2026-09-30-infercnv-raw-ingestion-measurement-design.md` and the accepted
actual shell/R runtime mismatch in `2026-10-15-infercnv-python-r-runtime-identity.md`.
The paired owner must use the same real capture inside and outside R after
dependency installation and around trials. This is private control-plane code,
not a foundation or another R factory/profile producer.

## Global Constraints

- Only the2 new files below; no existing workflow, Rust, R, metadata checker,
  process owner or public API edits. Later integration owns their call sites.
- Direct main authorized, commit only assigned files, no push/deletion.
- Check APFS/external paths/free space before bounded stdlib tests; external
  TMPDIR, bytecode disabled. No local Cargo/R/build/install/data download.
- Preserve originals/failures/unrelated user edits; no Co-Authored-By.
- Rare stable comments, focused functions, no fake successful capture mode.

## Task 1: Real argumentless identity capture

Add `scripts/infercnv_python_identity.py` and
`scripts/test_infercnv_python_identity.py`. Provide
`capture_identity() -> dict` and argumentless CLI. It must run using
`python -I -B ABSOLUTE_SCRIPT` without project imports or site dependencies.
Use only stdlib. CLI always captures its actual process; no maps override,
mock-runtime input, caller-supplied executable or permissive fallback option.

Return exactly the existing probe's7 fields: executable,
executable_sha256, version, prefix, base_prefix, stdlib_json,
libpython_sha256. Preserve actual sys.version/sys.prefix/sys.base_prefix and
json.__file__; executable is actual canonical sys.executable. Hash actual
executable and each mapped libpython file with SHA256. Do not dump environment.
Capture itself need not enforce Python3.12: the paired owner/profile gate
enforces that separately, while this component reports actual identity.

Read actual `/proc/self/maps`; fail nonzero if missing, unreadable, empty or
malformed. A private pure parser accepts text only for unit tests; it is not
exposed as a successful CLI replacement for actual procfs. Parse the5 fixed
columns plus optional pathname without losing spaces in paths. Validate row
shape; accept ordinary anonymous/bracket mappings and deduplicate mapped
libpython paths across segments in sorted order. Empty libpython mapping is
legitimate (observed native Ubuntu Python), not empty procfs. Consider a file
whose basename starts libpython a named runtime mapping. Require absolute
paths; reject deleted/unreadable/nonregular runtime files rather than silently
drop them. No claim is made about renamed libraries or full OS/stdlib supply
chain. No copied executable wrapper may be mislabeled as the interpreter.

Emit exactly one deterministic sorted JSON object plus LF on stdout. On
OSError/invalid maps, emit a concise diagnostic to stderr and exit nonzero;
never emit partial successful JSON. Unexpected CLI arguments are rejected.
No files are written by capture. Pure parser tests must not be confused with
actual runtime evidence.

Write tests first and retain genuine missing-module RED. Cover normal/multiple
segments, sorted/deduplicated libs, pathname spaces, anonymous mappings,
no-libpython mapping, empty/malformed maps, deleted/nonabsolute named libs,
CLI argument rejection and clean failure on unsupported local procfs. Use
real local temporary files for file hashing/type/error tests without changing
actual process identity or adding injectable fake capture modes. On Linux,
exercise actual CLI under -I/-B and compare exact executable/hash/version/
prefix/stdlib fields to the actual test process, verify emitted mapped-file
hashes, and require one JSON line. A statically embedded runtime may emit {}.
Actual Linux tests may skip only where procfs is absent locally; Linux CI must
exercise them. Do not install Python/R or recreate the old mismatch via a
fabricated version string.

Focused tests while iterating, full controller suite once before commit,
self-review and report exact RED/GREEN commands/output plus limitations.
Commit only assigned files; root owns review, Linux CI and later real R
invocation/runtime comparison. No agents/reviewers may be spawned by worker.
