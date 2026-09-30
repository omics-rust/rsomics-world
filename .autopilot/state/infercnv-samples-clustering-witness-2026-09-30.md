# inferCNV samples/Ward clustering witness

Status: replacement execution succeeded, but original artifact acceptance
failed because two installed-function records were omitted during upload.

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

Original candidate source hashes:

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
This first dispatch did not establish a successful package outcome or artifact
acceptance.

## First actual failure and environment repair

Run `36701645992` failed in `Capture samples-clustering witnesses` at the
exporter's C-locale enforcement, before trace installation or any clustering
case. The 56 checker tests and preceding package/source, synthetic, shipped,
creation and edge-probe steps passed. This is not a clustering mismatch.

Preserved original API metadata, logs and artifact:
`/Volumes/Zane's HDD/rsomics-fixtures/evidence/infercnv-ingestion-2026-09-30/run-36701645992/`.
Artifact `11090851598`, 291413181 compressed bytes:
`238f21c15fdd39a11692b50b846e0fd8b490e70f6f0f40e4080fd7486b312089`.
Logs SHA256:
`2cd80a54e53b22efcf0c2440d9a8feaaa2e3013ca3186a34db88a8e03668ea55`.
API digest/size, CRCs, unique safe paths and both source manifests were checked
before extraction; 481 original files are retained. There is no
`samples-clustering/witness.json`. The preserved exporter matches the original
candidate hash above.

Package-free R 4.6.1 on SSH 4090 reproduced the failure: setting `LC_ALL=C`
left `LC_MESSAGES`, `LC_PAPER` and `LC_MEASUREMENT` at `C.UTF-8`, yielding a
composite return rather than `C`. Explicitly setting these categories to C
produces the actual `Sys.getlocale() == "C"`. The repair initializes LC_ALL,
sets the three omitted categories individually and retains the final exact
C hard gate. It changes no clustering algorithm, filtering, checker or
comparison tolerance.

The actual exporter locale AST block failed before the repair and passed
afterward on R 4.6.1; all eight supported category queries returned C. Updated
R syntax still parses 33 top-level expressions. Replacement exporter SHA256:
`5ee4c3f4b5b02750b4c719675e9e6c26670a4044092bc92570f396e90cafe3e9`.
Python source hashes remain unchanged. Replacement package execution and
original-artifact acceptance are still required.

The locale repair at world `2b51418b28b16fd067ce81d14ebea7368c2b0e9e`
passed all 200 controller tests and exact-head CI `36703413152`. Replacement
oracle run `36703475039` was dispatched with the same shipped/sample-clustering
arguments; its API-reported head matches that repair. Do not treat the parent
failure's synthetic/shipped exports as replacement clustering evidence.

## Successful execution, incomplete uploaded evidence

Run `36703475039` succeeded, including all 16 workflows and 20 probes and the
complete checker on the runner. Original APIs, logs and ZIP are retained under
`/Volumes/Zane's HDD/rsomics-fixtures/evidence/infercnv-ingestion-2026-09-30/run-36703475039/`.
The [incomplete-artifact receipt](../oracles/infercnv-samples-clustering-incomplete-2026-09-30.json)
identifies original bytes only; it is not a samples-clustering acceptance.
Artifact `11091785736` has 1,572,350,571 compressed bytes and SHA256
`0f6144ad4ccd58d1db996569a941d5321211bca635dc1db8903ab1818b54af26`.
Original logs SHA256 is
`08f577b70aa7e0142d379dc569ef355e05f10ac4138e8e347786914e732f804b`.
Witness JSON SHA256 is
`177bae2d65e99eab71c6aa2c36bed16ca062c0554e5dd7cf9b12762781c42598`.

The controller checked run/head/status, API digest/size, CRCs, unique safe ZIP
paths and every synthetic/shipped manifest file before safe extraction. The
offline witness checker then rejected the original file inventory. Two paths
recorded in witness.json are absent from both ZIP and extraction:

- `metadata/.get_relevant_args_list.function.txt`
- `metadata/.single_tumor_subclustering.function.txt`

The [uploader migration contract](https://github.com/actions/upload-artifact/blob/main/docs/MIGRATION.md#hidden-files)
defaults to excluding hidden files. The exporter named installed
function text directly from dot-prefixed R function names, so the two records
existed for the runner's successful check but were not uploaded. Exactly 938
witness files arrived, versus 940 expected including witness.json; there are
no extra files. Do not reconstruct those texts from source, waive inventory,
or label this green run accepted.

The repair uses visible deterministic filenames, retaining the original
semantic function column and all algorithm, source-pin and tolerance contracts.
A controller regression rejects hidden installed-function paths before upload;
another preserves rejection of an omitted file even if its inventory exists.
Package-free R exercises the actual naming AST before and after the repair.
Fresh exact-head CI, actual package execution and original-upload verification
remain required. No native clustering or performance acceptance follows.

Independent diagnostic review found all 937 physically present inventoried
files hash-matched, and all 52 routes individually passed unchanged state,
group/tree and Euclidean artifact checks: 160 traced groups, 152 actual trees.
All 16 full routes logged step 15 and agreed with their isolated groups. All
52 captured warning/error condition arrays were empty; original per-route
logs remain separately retained, including logger advisories.
Zero-gene positive-filter probes produced zero distances and tree heights;
the boundary probe's genes 3 and 6 remain unresolved characterization. The
pooled-name collision uses observation indices twice, matching actual source.
These observations do not repair the incomplete original evidence inventory.

The actual R filename AST regression failed on the old two hidden names and
passed on all four visible unique names after the one-line repair. R 4.6.1
syntax still parsed 33 top-level expressions. Replacement exporter SHA256:
`2d437e9393e51c32dc49baefe1593300b94ff1cb9141fd7e08206c5bdf2de9bc`.
The hidden-path controller regression failed before its guard and then all
202 controller tests passed, including 58 focused witness tests. The strict
offline inventory gate remains unchanged and still rejects omitted records.

Replacement checker SHA256:
`682d2a41bbe901a02b6687756e64cc1359fa1e3881d49af40eaa5fc725211222`;
test SHA256:
`8d53a373f261110d6b204ae0d62b139fcd724bffe90e5599603c8ab4455e7f8e`.
Fresh boot APFS occupancy is 88.65% (25.91 GiB free), KIOXIA 41 GiB free,
Zane's HDD 246 GiB free. The local-build gate still applies; the replacement
uses remote installed-package execution and external-only controller scratch.

Fresh independent repair review approved another actual-package dispatch and
reran all 58 focused tests successfully. This is dispatch approval only;
run `36703475039` remains incomplete-evidence NO-GO.

The visible-name repair is committed at world
`8ee2ed9301c347b970f59ee5646de197535b7db6`; exact-head control run
`36707224867` passed. Replacement actual-package run `36707344893` is running
with `dataset=shipped` and `samples_clustering=true`; its API head matches.
Original-artifact acceptance remains pending, regardless of runner status.
