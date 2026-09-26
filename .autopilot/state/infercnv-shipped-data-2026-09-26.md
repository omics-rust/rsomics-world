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
