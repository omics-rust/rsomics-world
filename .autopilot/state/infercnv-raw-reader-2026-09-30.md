# inferCNV raw reader — resumed candidate

Spec: `docs/plans/2026-09-30-infercnv-raw-reader-design.md`.
The user explicitly resumed on 2026-09-30 at 16:35 CST. The earlier pause
receipt is historical and does not instruct the controller to stop again.
The goal tool still exposes the old `paused` state and has no agent-side resume
operation; continuation follows the user's explicit instruction, not a second
or completed goal.

## Storage and scope

Fresh APFS size/free: 245,107,195,904 / 29,228,732,416 bytes (about 88% used).
No local build or dependency download. Cargo/test execution remains on hosted
native runners, with externally stored controller source and evidence. Product
is unpublished at `3715e55b`; no CLI, release, ingestion-speed or full-CNV
claim follows from this reader candidate.

## Accepted test-first failure

World `47dc2475db4e1538e90a524767b1a1b391a0f205` passed exact-head control
run `36688482518`. Frozen `red-6` adds only the 21 input contract tests.
Actual Linux x86_64 run `36688501838` failed solely at debug compilation with
E0432: missing `CreationConfig`, `CreatedInput`, `InputPaths`, `create_input`.
This is an absent-feature compile failure, not an executed assertion failure.
Dependencies and resolved output paths passed; both before/after source checks
verified all 143 files. The original artifact's size/API digest/CRC and source
archive/manifest/checksum/README bytes were independently verified.

Original evidence is preserved at
`/Volumes/Zane's HDD/rsomics-fixtures/evidence/infercnv-ingestion-2026-09-30/run-36688501838/`.
Artifact `11084363788` has SHA-256
`bb8e7e70652dc900621af9e4fb10ac2501b561248b095f9fe27c36fab6bd2366`;
original logs SHA-256 is
`b05c38b7b42dcedd1f598a1b09747eee31c70c4b47ddcfd426b0285c4d30ea28`.

## Current work

The bounded product implementer resumed after red acceptance. The controller
owns frozen source, dependency-lock resolution, Git and native CI. A separate
agent owns the three creation edge scripts; source-derived failure classifiers
and input-path provenance are being strengthened before actual package tests.
The controller's new gzip lock verifier has five targeted tests passing after
the missing-module red; remote lock resolution and workflow integration are
pending. Existing packages may not change: only flate2 1.1.9 and its Rust gzip
dependency nodes are allowed to be added.

Ruling: no local compile despite external Cargo paths — APFS remains above
the manual's 80% gate — hosted execution preserves the native-platform gates
at the cost of longer source snapshot and artifact round trips.

Ruling: keep ingestion policy internal to `rsomics-sc` — one actual consumer,
inferCNV-specific joins/filter/group semantics — no speculative foundation API
or revival of a deleted micro-crate.

The inherited VCF preflight and loose test data remain untouched and unstaged.

## Implementation snapshot and acquisition retry

Pure-Rust gzip lock expansion and five verifier tests are committed at world
`d2f34fd4`; exact-head Control plane run `36691346587` passed. The three edge
scripts and installed-package integration are committed at `e61c6c46`, with
exact-head run `36691438844` passing. Remote R syntax passed; the whole Python
suite passed 144 tests. Actual edge run `36691614700` is still pending.

The product-local reader, 25 input tests and raw-to-step-14 external test are
frozen in `green-8` at world `2a90fe02`. Production numerical core and baseline
lock bytes remain unchanged. Standalone Rust 1.91 formatting ran on the
controller without compiling. Independent whole-reader review is pending.

The first snapshot push returned HTTP 408 and did not update remote main.
Dispatch `36692180206` accidentally used old head `e61c6c46`, where green-8
did not exist; it failed source acquisition, not Rust tests, and is excluded
from implementation evidence. Retrying push with HTTP/1.1 and an explicit
post buffer delivered `2a90fe02`; authoritative remote-head confirmation
precedes the replacement dispatch. Do not classify that acquisition failure
as a test-first red or run against an assumed latest head.

The Linux-only lock-resolution selector uses `expected_red=true` solely to
select one native host; green-8 does not require or claim a test failure. A new
locked immutable snapshot and four-native exact-head evidence remain required.

## First execution, review and corrections

Run `36692452334` compiled the candidate and verified the gzip lock expansion.
Its one failure was test-fixture setup, not decoder behavior: the compressed
record-limit branch set the common cap to 5 bytes, so the preceding position
record failed before the intended count header. Other input cases passed
23/24, including damaged gzip checks; the existing core suite also passed.
The corrected test uses an 8-byte valid position record and a genuinely longer
compressed count record, asserting count-path/record-2 failure. No production
limits or validation were relaxed.

Artifact `11086192495` SHA-256
`f1b58b31ea297aadc067e32e04f924602950063eeaa74d713ebef7f2b53203a0`
was checked against API size/digest, CRC and frozen archive bytes. Its original
`Cargo.before.lock` matches the product baseline; all old nodes are unchanged.
The remote-generated lock was copied exactly into the product.

Independent whole-reader review found no critical defect. Two important gaps
were addressed in the next candidate: actual synthetic grouped/no-reference
raw inputs now join the shipped raw-to-checkpoint tests, and two `V1` contract
tests/documentation make the strict headerless behavior explicit. Ordinary
input contracts now count 26 (the initial handoff's 25 was a count error; the
actual first run had 24). Group-name lookup remains O(cells × groups): deferred
minor pending representative high-group-count scaling, not a throughput claim.

Ruling: preserve literal first `V1` annotation cells rather than R's silent
deletion heuristic — native input is explicitly strict headerless TSV — this
intentional divergence costs compatibility with files relying on that heuristic
and must remain documented/tested.

The review set aside arbitrary R fractional lexing/thresholds, R general table
dialects and locale ordering, RDS/sparse/sampling/downstream/CLI, whole-process
memory ceilings, and unmeasured throughput. Those remain explicit exclusions
of this slice, not claims of inferred correctness. Four-native runtime/locked
dependency evidence remains mandatory before accepting the actual reader.

Actual edge run `36691614700` executed every R probe but failed independent
checking of the null-limits provenance. Root cause: assigning NULL with R's
`$<-` removes the named argument instead of retaining JSON null. A remote
base-R reproduction confirmed this; bracket assignment preserves the field.
The exporter now preserves it and asserts the complete argument-name set.
Expected-error classifiers and every original numerical/output check remain
strict. Original failed evidence is retained; no edge acceptance is claimed yet.

## Locked four-native candidate

`green-9` freezes the verified remote-generated lock, 26 ordinary reader
contracts and three actual external tests (prepared shipped, raw shipped,
raw synthetic grouped/no-reference). Source checks cover 144 files. Both
numerical kernels and the prepared-only measurement boundary remain unchanged.
Four-native debug/release, measurement-support tests, bench compilation and
strict Clippy remain pending; the reader has not been committed in its product.

Original failed edge artifact `11086935354` was independently checked against
API size and SHA-256 `8076731cb254ede5b36c94a4aca88875d8d6ed49b04c471c809a7f7afa9d08d9`.
It confirms the missing null argument, long-double precision 64 bits / storage
16 bytes, and the three source-derived failure classes/calls. Those are
diagnostics from a failed run, not an accepted oracle receipt.

The null-field fix at world `3c1970b8` passed exact-head control run
`36693720465`. Replacement actual-package run `36694464090` is queued at that
exact source head; acceptance still requires its terminal success and an
independent original-artifact audit.
