# Paired ingestion workflow integration ledger

Plan: docs/plans/2026-10-15-infercnv-paired-workflow-integration.md.
Status: root implementation in progress alongside isolated two-file owner
implementation. No full campaign dispatched or performance result claimed.

Genuine workflow regression RED before editing YAML: six tests ran,
17 failures/one error. The real old selector accepted11 forbidden nonempty
raw combinations; new input and four route steps/always-postcheck were absent.
Embedded-copy and Bash syntax tests passed at baseline. This is an expected
missing-route RED, not scientific/campaign evidence.

Implemented the optional raw selector, original native source admission,
exact bench build, context/owner invocation and always raw-source/code/exe
postchecks in the worktree. Selector48-combination execution now passes;
all literal Bash blocks parse; complete embedded workflow copy synchronized
mechanically and byte-equality test passes. The independent freeze/authenticate
step awaits completed owner review and its exact import-closure hashes. Thus
the new source/test worktree remains deliberately uncommitted and not pushed.

No changes to old prepared/clustering/default step bodies; the new route is
guarded by a nonempty explicitly admitted raw snapshot. Existing contract12
pins and accepted R/Rust bytes remain unchanged. Root's worktree files are
separate from the delegated owner's two files. Its full-suite run is being
coordinated to avoid treating this known temporary integration RED as a
regression in its isolated owner.

Copy synchronization check: the first mechanical command added an extra LF
because Perl's input separator was locally undef and chomp removed nothing.
The byte-equality test caught it. Replaced chomp with explicit single final-LF
removal and reran the actual test: one test, OK. No executable workflow bytes
were affected by that documentation-only intermediate failure.

Before local stdlib tests: APFSContainerSize245107195904,
APFSContainerFree18627567616 (92.40% occupied), external KIOXIA40GiB/HDD54GiB.
No local Cargo/R/build/install/download. Tests use bytecode disabled/external
TMPDIR, only actual selector Bash rather than install/build bodies.
