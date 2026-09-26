# SDD ledger — plan: docs/plans/2026-09-26-infercnv-shipped-data-plan.md

Spec: `docs/plans/2026-09-26-infercnv-shipped-data-design.md`.
Execution follows accepted native core commit
`13961bf6a42cb4cbbf36c528331aafb98a52937b`; see its separate acceptance receipt.

Independent source-informed design review identified and resolved original
matrix dialect, gene-ID subset selection, independently pinned source/input
hashes, preservation of synthetic semantics, bounded validator memory and
exact-artifact delivery to four-native CI. A follow-up explicitly required
stage-1 raw-value equality and unchanged retained stage-2 values; both are
now binding spec requirements. No real-data R checkpoint is accepted yet.

## Preflight

| Task/interface | Producer and consumer | Check |
|---|---|---|
| Task 1 internal | Pinned originals → canonical cases → real R checkpoints → independent checker | Original dialect, gene/cell joins, original hashes and stage-1/2 numeric identity are explicit; predictions are not truth |
| Task 1 shared exporter | Two common R writers → synthetic and shipped generators | Existing synthetic schema/settings/negative tests remain unchanged and actual synthetic oracle reruns after extraction |
| Task 1 → Task 2 | Accepted run/head/artifact/manifest receipt → explicit external native test | No latest lookup or missing-data skip; large data stays outside Git and runner-temp acquisition is digest-checked |
| Task 2 internal | One native execution per prepared case → all stages and returned state | Exact identities, fixed numerical gate, unchanged input and four-native debug/release evidence remain mandatory |

Ruling: Retain independent repositories and direct main commits — these are
the user's operating rules — a mistake is corrected through ordinary commits,
not by altering unrelated history.

Ruling: Keep source/evidence and skill workspace on external disks without
cleanup — local compilation remains prohibited above 80% boot APFS — remote
validation latency and small extra external-storage usage are accepted costs.

Ruling: Treat the shipped example as engineering evidence, not biological
ground truth or large-cohort performance — provenance is limited to the pinned
upstream distribution — any later claim needs its own stronger evidence.

## Execution

Task 1 implementer: `/root/infercnv_shipped_oracle_impl`, fresh context.
World base `58044e6aab6d0b798eafea0bd0cbca33a76208d5` passed exact-head
Control plane run `36219378654`. The agent owns only the named harness
scripts and oracle workflow; the controller owns Git, plans, snapshots,
remote execution and evidence. Product source remains unchanged.

Fresh storage check: boot APFS 97.86% (5,240,377,344 bytes free of
245,107,195,904); KIOXIA 59 GiB free; external HDD 256 GiB free. Cargo, rustup,
target and TMPDIR resolve under the required KIOXIA paths. No local builds,
dependency installs or R execution are permitted. Pure Python `-B` checker
tests use external scratch only. No task is complete yet.

Independent original-input review is recorded in
`infercnv-shipped-input-review-2026-09-26.md`. It predicts exact stage-1/2
ordered gene identities, all cell identities/order and chromosome counts
without reading the implementer's files. Both cases include a retained gene
with exactly three positive cells, but neither isolates rejection by the
detection threshold. These predictions await comparison with actual R output.

Task 1 source candidate is frozen in the six harness/workflow files, with
hashes and a complete diff under this plan's `.superpowers/sdd/` workspace.
The implementer recorded test-first failures and 13 new checker tests. The
controller independently ran all 50 Python tests (1.108 s, clean), control-plane
validation and `git diff --check`, then verified all six frozen file hashes.
All three R scripts parse successfully under remote R 4.6.1 on `4090`, using
`TMPDIR=/dev/shm`; only source bytes were streamed to `parse()`, without
executing either generator, installation or remote source-file creation. No
local R or Cargo ran.

Task-scoped independent review: `/root/infercnv_shipped_task_review`.
The real installed-R execution, raw bundle inspection and acceptance remain
pending; the new native external-oracle task has not started.

Task 1 initial review: needs fixes. Two Important findings: metadata file
contents were not semantically checked, and negative tests did not change a
retained stage-2 value or remove an artifact from a valid full-schema bundle.
One Minor finding: a pre-existing sorted manifest was accepted without
verification and could be overwritten. Fix round 1 includes all three because
immutable preserved-bundle revalidation is the next operation. The original
six-file candidate is retained in the plan workspace for a scoped fix diff.

Task 1: fix round 1/5 (3 addressed, 1 new Important finding). The controller
tested the new metadata check against actual accepted run `36216764707` and
found legitimate duplicate package names: Matrix 1.7-6 in the runner library
and Matrix 1.7-5 in `/opt/R/4.6.1/lib/R/library`. The check incorrectly rejected
that original metadata. Scoped re-review confirmed the issue: uniqueness must
use package-plus-library identity, while the loaded infercnv version remains
checked separately. The v2 candidate is preserved before fix round 2.

Task 1: fix round 2/5 (1 addressed, 0 open; scoped review clean). Distinct
libraries are accepted, duplicate package/library identities still fail,
installed-version row order does not change the verdict, and loaded infercnv
1.28.0 remains required. The actual preserved R metadata now passes.

Harness commit `424f2bae` contains only the six owned source/workflow files.
Fresh controller verification on the frozen candidate: 62 Python tests passed
in 1.706 s, control-plane validation and whitespace checks passed, and all
six source hashes matched the reviewed candidate. R files are unchanged since
the successful remote syntax check. Exact-head CI and actual installed-R
execution are the next gates; Task 1 is not yet complete.

Parallel source review of the future measurement contract is preserved in
`infercnv-performance-boundary-review-2026-09-26.md`. It clarifies native wrapper
costs, exclusion of early clustering, all save/plot controls, effective thread
settings and warm-up/RSS accounting. This is not a performance result.

World head `68dcf7c4525a4e87581743f96e0e09dfb6d453d3` passed exact-head
Control plane run `36220412157`. Shipped-data oracle run `36220439929` was
dispatched at that exact head with `dataset=shipped`; it also reruns the four
synthetic profiles before the two shipped cases. No output has been accepted.

Ruling: overlap only Task 2's test-first reader preparation with the reviewed
Task 1 harness's remote execution — the reader contract does not depend on
unaccepted numerical values, and no production algorithm changes are needed —
a schema discrepancy may require test rework, but no oracle or conformance
claim is advanced. The test-first agent must freeze before reader
implementation so the controller can capture an actual remote red. Native
large-oracle acceptance still waits for the Task 1 receipt.

Task 2 implementer: `/root/infercnv_external_reader_impl`, fresh context.
Its initial freeze adds only `tests/cnv_fixture_reader.rs`: six malformed
reader cases plus non-square explicit-root and checked-in entry-point positive
controls. Snapshot `red-3` preserves all 137 source files, verified against
its archive and manifest. The unchanged lock SHA-256 is
`636ead1961f573b303be8c02f6068201acd11dde1d0cb9bab3d81ccf86bf5554`;
new test SHA-256 is
`b7eefe530334a6ef9bf4abb6e1a64a4557d58464937e5625a1b437427b476e54`.
The missing explicit-root API should fail compilation remotely. This is an
expected red, not yet an observed result; loader implementation is held until
the actual failure is captured. Existing production and fixture bytes match
the accepted product commit.

World `77b595978a53dc4762cd7d14e120c225af6b30f3` passed Control plane
run `36220765158`; expected-red Linux run `36220790633` then failed only
`Test debug` with two E0599 errors for missing `Fixture::load_from_root`.
The controller checked exact head/job/failed-step identity, all 137 source
hashes before and after, unchanged Cargo.lock, artifact API digest, both ZIP
CRCs and extracted bytes. The real failure permits test-only reader
implementation; no product numerical change or oracle acceptance is implied.

Raw evidence: external fixtures
`evidence/infercnv-native-core-2026-09-26/run-36220790633/`.
Artifact ZIP SHA-256:
`d73629dedeeaa6c2b3d71c3c94610e08c68c328c7b577e51227afefc7b6909e3`.
Full logs ZIP SHA-256:
`335747d705e1783aa04c21e672ab1cae70ee1784e546e9a220d53340c7fbfefe`.

The first shipped oracle run `36220439929` failed only the independent
validator after both R cases completed. An independent entrywise audit and a
separate R 4.6.1 diagnostic identified exactly five one-ULP decimal-parser
differences among 1,902,192 original values. The complete evidence and reviewed
canonical hashes are in `infercnv-shipped-parser-audit-2026-09-26.md`.
The corrected contract isolates this input bridge and pins its entire
canonical byte output; exact stage-1/2 checks and downstream tolerances remain.
No complete shipped oracle is accepted yet. The Task 2 loader is frozen for
later review and remote green; no production code has changed.

Task 2 reader implementation candidate `green-4` is frozen for four-native
verification. Compared with red-3, only `tests/support/mod.rs` changes among
137 source files; archive and file hashes were verified. The original entry
point delegates to an explicit-root loader with contextual parse/I/O errors,
checked dimensions and finite numeric values. The eight reader tests remain
unchanged. No large-oracle feature or production change is in this snapshot.

Green-4 world head `0c96a8a9b1be882959cd5a784a1dc0c96fdf94e4` passed
Control plane run `36221298670`. Four-native run `36221334797` was dispatched
at that exact head; results and evidence review are pending.

Task 1 parser fix round 3 passed scoped independent review. Controller reran
all 67 Python tests (1.805 s), architecture validation, whitespace and reviewed
file hashes successfully. Fix commit `4afd7558a2407a5c43582ab473137d3271a9ad9f`
passed exact-head Control plane run `36221478265`. A fresh shipped oracle
execution is being dispatched; the earlier failed run remains unchanged.

Task 2 acquisition preparation is bounded to an offline byte/provenance
verifier and its standard-library tests. It receives downloaded exact API
records/ZIP plus a committed receipt; only the controller performs downloads,
chooses the accepted receipt and changes CI. No placeholder real receipt may
be created. The product reader remains frozen while this independent
control-plane component is implemented.

The reader candidate's four-native evidence has now been verified: all targets
passed 38 tests in debug/release and all 40 synthetic comparisons, with the
same maximum delta as the accepted core; formatting and strict Clippy passed.
Receipt: `infercnv-external-reader-native-2026-09-26.md`. Full Task 2 review and
large-oracle conformance remain open; product changes are still uncommitted.

Fresh shipped oracle run `36221554790` is executing at exact fixed head
`4afd7558a2407a5c43582ab473137d3271a9ad9f`. Its output must be inspected and
accepted independently; the old failed run is not substituted for it.

World documentation head `bf55c77adf7dd7d71f23ce7ce6a1626c70a95e5f` passed
exact-head Control plane run `36221728812`. The fresh shipped oracle run
`36221554790` completed successfully, including both real R cases and both
bundle validators. Original API records and ZIPs are being preserved; success
alone is not yet acceptance.

Task 2 acquisition review found one Important integration defect: the offline
verifier restricted all archive entries to `shipped-bundle`, although the
workflow uploads the complete evidence directory. The original ZIP includes
synthetic evidence, source and logs. Fix round 1 is required before live use.

Ruling: validate and preserve the complete original ZIP, with manifest file-set
equality scoped only to `shipped-bundle` — this matches the existing upload
contract and retains the independently pinned original bytes — the cost is
additional safe extraction of provenance files, not weaker bundle validation.

Fresh storage check: boot APFS 97.89% (5,168,803,840 free of 245,107,195,904
bytes), KIOXIA 59 GiB free, HDD 255 GiB free. No local builds, dependency
installation or R execution; remote correctness work continues.

Task 1: complete (harness commits `424f2bae` through `4afd7558`, scoped review
clean, fresh successful upstream evidence independently accepted).
Receipt: `infercnv-shipped-oracle-accepted-2026-09-26.md`; exact machine pin:
`../oracles/infercnv-shipped-2026-09-26.json`. All original ZIP/extracted/manifest
bytes and actual dimensions were checked, preserved validators passed, all
60 shipped TSVs match independently diagnosed data, and all 120 synthetic
TSVs match the previously accepted oracle. This is not native conformance.

Task 2 acquisition fix round 1: original whole-ZIP scope finding addressed,
no new breakage in scoped independent re-review. Controller verification:
84 Python tests passed (2.170 s), control-plane validation and whitespace
checks passed. The reviewed verifier also successfully processed the genuine
accepted API records/original ZIP into a fresh external directory and checked
the committed receipt's manifest. CI integration remains a separate gate.

Task 2 product continuation is assigned to the original reader implementer.
It now uses accepted shipped evidence, explicit feature
`external-infercnv-oracle`, test target `cnv_external_oracle` and required
`RSOMICS_INFER_CNV_ORACLE_ROOT`. The controller is wiring the exact receipt
into four-native CI, including an actual missing-configuration failure check.
No shipped-data native result is claimed yet.

Control plane run `36222234178` at `798986240b6798d47966f80927e869e7c4ec3f98`
failed all 17 newly added verifier tests during setup: their hardcoded Mac
temporary directory does not exist on hosted Linux. The other 67 tests passed.
Original logs are preserved under external shipped evidence
`control-36222234178/logs.zip`. Acquisition fix round 2 removes that hardcoded
path and uses runtime `tempfile` selection; local commands still explicitly
set KIOXIA TMPDIR. Independent scoped review is clean; 17 focused and 84 full
tests pass locally. Fresh hosted CI recovery remains required.

The candidate workflow review separately found Bash 3.2 empty-array expansion
under `set -u` would break no-oracle mode on macOS. A focused local shell-only
diagnostic reproduced exit 127. That integration fix is in progress; no local
Cargo or build was invoked.
