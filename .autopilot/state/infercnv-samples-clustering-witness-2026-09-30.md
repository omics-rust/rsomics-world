# inferCNV samples/Ward clustering witness

Status: candidate dispatched; installed-package execution and artifact acceptance pending.

The user explicitly resumed unattended work on 2026-09-30. InferCNV remains
first; the goal tool's historical paused status does not override that request.

## Accepted prerequisites

- Creation-edge witness: run `36694464090`, receipt
  `../oracles/infercnv-creation-edges-2026-09-30.json`.
- Strict native raw reader: product commit
  `e62fc960aeeb9dc543298885213cda6a57196fe9`, four-target run `36696131908`,
  receipt `../oracles/infercnv-raw-reader-native-2026-09-30.json`.
- World prerequisite head `3a38092adf92b0abae88a7789036a230d267bc41`;
  exact-head control-plane run `36698410571` passed.

Neither prerequisite accepts native clustering, default Leiden, raw-ingestion
performance, a whole inferCNV replacement, CLI delivery or publication.

## Candidate scope

The exporter invokes infercnv 1.28.0 from source commit
`b421d9405c97a309b081ef86d455e976df93eae4` with R 4.6.1.
Sixteen workflow cases compare the actual namespace function on step 14
against the actual complete upstream run from creation through step 15.
Twenty deliberately isolated probes characterize small groups, ties,
ordered maps, reference filtering and pooled-name collisions. Probe failures
are preserved, not relabeled as successful full workflows.

The independent checker binds source and checkpoint inventories, runtime,
arguments, unchanged matrix/identity state, filtering, ordered Euclidean
distances, tree structure and memberships. Near-threshold reference-filter
genes remain explicit characterization ambiguity; an artifact arithmetic
screen is not a native numerical tolerance.

Design and implementation plan:

- `../../docs/plans/2026-09-30-infercnv-samples-clustering-witness-design.md`
- `../../docs/plans/2026-09-30-infercnv-samples-clustering-witness-plan.md`

## Execution boundary

Controller verification: 56 focused checker tests and all 200 script tests
passed with external TMPDIR. Ruby parsed the workflow YAML and Bash parsed all
22 run blocks. The unchanged exporter passed R 4.6.1 syntax parsing on SSH
4090 (33 top-level expressions); this is not package-execution evidence.

Candidate source hashes:

| File | SHA256 |
| --- | --- |
| `scripts/infercnv_samples_clustering_witness.R` | `17e2fe9b027eaeae2efec07702d0724003f7bebfc00f405672dd7f6f1d0000ec` |
| `scripts/validate_infercnv_samples_clustering_witness.py` | `b23297496b5a7c65fd1a5a792070690035b66deca339c3d1fca51be05a9872b4` |
| `scripts/test_infercnv_samples_clustering_witness.py` | `1188184e6a2e9d3f3de08d4ab5a0e301fd1d1c92e3b3c8587f2b1f86a2314bf2` |

Fresh review found two concrete checker gaps before dispatch: distances were
not independently bound to the filtered matrix, and the installed-package
inventory rejected valid duplicate names across library directories. Both
were corrected with mutation regressions. The full package inventory is
preserved; the three runtime packages must match their actual loaded
namespace's version and parent library, not another installed copy.

Current storage check: boot APFS occupancy 88.53%, KIOXIA available 41 GiB,
Zane's HDD available 250 GiB. No storage cleanup is authorized by this gate.

The Mac boot disk remains above the 80% gate. No local Cargo/R build or
dependency installation is permitted. Controller tests use the external
TMPDIR; R syntax checks use SSH 4090 with `/dev/shm`. Actual package execution
uses the existing installed-package GitHub Actions oracle workflow.

Next: finish independent review and controller regressions, commit the
complete candidate, verify exact-head CI, dispatch the shipped dataset with
`samples_clustering=true`, and preserve and independently inspect original
logs/artifacts before accepting any witness. Only accepted evidence can
justify a native clustering model or implementation.

## Exact-source dispatch

Fresh review approved dispatch after the corrected package-library regressions.
Candidate world head `e38c5f3250748890f321dba0c9cfc85d07bba672` passed
exact-head control-plane run `36701542024`. Installed-package oracle run
`36701645992` was dispatched with `dataset=shipped` and
`samples_clustering=true`; its API-reported head matches the candidate.
No successful package outcome or artifact acceptance has yet been established.
