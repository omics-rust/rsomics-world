# inferCNV matched performance — execution ledger

## User-directed pause boundary

The user now requests a pause after inferCNV is finished and will resume next
Saturday. Do not advance unrelated product families or automatically resume,
schedule or notify. A nonblocking clarification is pending about whether the
completion boundary is the whole usable inferCNV workflow or the current
prepared-input correctness/performance slice. Current measurement verification
continues because both interpretations require it; a preprocessing-only result
must never be labeled a completed inferCNV workflow.

## Current status

Task 1 is accepted: private Rust measurement support is committed and verified
on four native targets. Task 2 (R trial and checked Python driver) is in progress.
No speed or memory advantage has been measured or accepted. This follows accepted
shipped correctness, not another correctness reconstruction.

- Product baseline: `bdcc8a3a55596be8d46a774d39b0335845c75175`.
- Accepted four-native shipped conformance: run `36222541338`, source head
  `070dd67118745a50f31d08717b20ec1ea7e63609`.
- Accepted R oracle: run `36221554790`, receipt
  `../oracles/infercnv-shipped-2026-09-26.json`.
- Correctness documentation head: `282d2171dc799e5ef7e94962e7b0b8c53a4a922e`;
  exact-head Control plane run `36222894885` passed.
- Design: `../../docs/plans/2026-09-26-infercnv-matched-performance-design.md`.
- Plan: `../../docs/plans/2026-09-26-infercnv-matched-performance-plan.md`.
- Private work ledger: `.superpowers/sdd/2026-09-26-infercnv-matched-performance-plan/progress.md`.

## Environment boundary

Fresh preflight: boot APFS total 245107195904, free 5162151936 bytes
(97.89% occupied), KIOXIA 59 GiB free, HDD 254 GiB free. Local Cargo/R/builds
remain stopped; hosted native runners carry all compilation and measurement.
Source, scratch and evidence remain on the prescribed external disks.

## Scope and decisions

Only full-example prepared-input preprocessing through step 14 is measured.
Production core stays unchanged. One fresh-process smoke pair is excluded;
seven alternating pairs are accepted only after every result passes unchanged
identity and numerical criteria. Region clocks and baseline RSS are separate
from whole-process timing/peak RSS. This is not full CNV calling, large-cohort
scaling, biological validation, a public CLI or a release decision.

Independent design review required and now confirms both: full RDS group/hidden
state checks, and binding timed source/lock to verified four-native bytes.
Routine design/execution decisions use the user's explicit delegation; no
additional approval pause, destructive action or automatic publication.

## Test-first candidate

Plan commit `1f28ece08eab71d9fde6322c4e0647286b89d8fd` passed exact-head
Control plane run `36223517522`. Local external-temp standard-library checks:
84 tests passed and control-plane validation passed. All ten candidate workflow
shell blocks passed Bash 3.2 syntax; independent CI review has no important
finding. Resolver stderr must additionally be preserved from original Actions
logs; artifact-only logging improvement is a deferred minor.

Frozen `red-4` contains 140 files; only Cargo.toml and the new six-test
measurement contract differ from baseline. Support implementation is absent.
Production bytes and original lock remain unchanged. Archive SHA256:
`33d91da411d1518688ffb0b318fdded730fe6b1340834fc921eac05c42bba02c`;
manifest SHA256:
`bb8f6e6ef31668fa62a5a62a2826a4f0ce3b330ff02cd7e13cc0312cdd12719d`.
Hosted red/remote lock resolution remain pending; no measurement was run.

## Actual red and resolved lock accepted

World head `c1530ace5c09b777c8536cead3eb2254a9a4abed` passed Control plane
run `36223712402`. Native expected-red run `36223760183` failed only the new
measurement-support debug compilation because `benches/support/mod.rs` is
absent. The ordinary 38 tests passed. This is an intended missing-support
compile failure, not an executed assertion failure or dependency/runner issue.

Controller verified original run/jobs/artifact metadata, ZIP digest and CRC,
extracted bytes, frozen source, 140 before/139 after source entries (the
intentional lock update excepted), test counts and complete lock difference.
Only `nix` 0.29.0 and `cfg_aliases` 0.2.2 were added; every prior package's
version/checksum/dependencies remained unchanged and root gained only nix.
The exact generated lock was copied into the product for locked green builds.

Evidence directory:
`/Volumes/Zane's HDD/rsomics-fixtures/evidence/infercnv-matched-performance-2026-09-26/run-36223760183/`.

| Evidence | SHA-256 |
| --- | --- |
| Original artifact 10900295452 | `b06c043d5161ec2ab1489411413aa2bab55b2ad63bda8cd9590946c4090634ea` |
| Original Actions logs | `663234e2154474a145d1e3d233de7cfebbc904c5eed9b4826160e22dc33a8cc7` |
| Resolved Cargo.lock | `e469fc5d773632a5893adfd3156ea816204b2208a3090fbd432fc37112d11f46` |

Product implementation is now in progress; four-native green, R execution and
actual performance measurement remain pending. The source-only R preparation
review confirms ordered groups, exact count.data, NULL hidden spike, genuine
run flags and accepted TSV pins; it is not an executed RDS/measurement result.

## Native measurement harness and review correction

Intermediate run `36226155239` at `f104fa7b97d5f19cc54fcb2a69c31bf1b6c8f7e9`
passed all four native targets. Its original artifacts and Actions logs were
preserved and independently verified: 39 ordinary and 7 measurement tests per
profile and target; 142 source entries unchanged; all 60 checkpoint report lines
identical to accepted conformance. This is retained intermediate evidence only.

Independent product review found that malformed environment values could be
misreported or corrupt runtime TSV provenance, plus a minor CPU sampling-order
issue. Four regression tests preceded the fix. Expected-red run `36226422538`
at `4b8f5665db4137a77b6f46bfe724dd84ebf79ecc` failed only with four missing
`format_env_value` errors; 38 ordinary tests passed. Original artifact and log
proof is preserved in the corresponding external evidence directory.

The fix rejects non-UTF-8 and TSV separators, distinguishes absent values, and
samples child CPU before self CPU. Scoped re-review approves both corrections
with no new finding. Frozen `green-7` has 142 files, unchanged production and
reviewed lock; the controller independently matched every current file and tar
entry to its manifest and all eight production files to `bdcc8a3a`.

- Archive SHA-256: `d8898fcc887a10c5e7aaba389bb1ce9f80a72f5b0a17926b83ee1de5ecc4e8e7`.
- Manifest SHA-256: `7267ad15a96a84b34b1bdaae07350093ca0f55ce6d9dfed66a32097b89bc72dc`.
- Native run: `36226732979`, attempt 1, head `a87d0e1544393612dd0a5214f14c8e583c862964`.
- Exact-head Control plane run: `36226671941`, success.

All four native jobs report success. Original Linux artifact 10901276804 has
verified API/ZIP SHA-256
`25227eda8e80b3835aac52fba888185b213067a75ea5b81c6f6014b955aa45b6`;
11 measurement tests pass in debug and release, including all four regressions,
with successful bench compilation, format and strict Clippy.

## Task 1 accepted

Final independent original-artifact verification and scoped product re-review
are complete. Every target passed 39 ordinary and 11 measurement tests in each
profile; all 142 source-before/after entries passed and all 60 checkpoint report
lines match accepted conformance exactly. Numeric tolerance is unchanged:
`1e-12 + 1e-12*abs(expected)`; maximum shipped absolute/scaled deltas remain
`1.27329258248209953e-10` / `1.06420983204087941e-2`.

| Target | Artifact ID | Original ZIP SHA-256 |
| --- | --- | --- |
| Linux x86_64 | 10901276804 | `25227eda8e80b3835aac52fba888185b213067a75ea5b81c6f6014b955aa45b6` |
| Linux aarch64 | 10901167302 | `0615295c3481d18bf82a3d541751397c8dc286c3e62da407c3d07100c6365c91` |
| macOS x86_64 | 10900164222 | `7eb9567b145cd4e92c8b82904234b7d1ffbd1723a20a879cfea3baedca3765a2` |
| macOS aarch64 | 10901007788 | `01ce9ee7eb8236503197f21b5a32296565ddb5a3e591d092dabb8c57ca518486` |

Evidence is retained at
`/Volumes/Zane's HDD/rsomics-fixtures/evidence/infercnv-matched-performance-2026-09-26/run-36226732979/`.
Original Actions logs SHA-256:
`dea68daeef1c3d8f5f8fc734c75bf85193fd041b751b3f310d4e6857381e2ce6`.
The four unchanged upload-artifact Node 20 deprecation annotations remain
nonblocking; saved test logs contain no warnings. The deliberate Linux missing
oracle-root error is a passing negative guard, not a hidden test failure.

Product commit `3715e55b8c06c4bb6b1605f42942805da29a4de7` contains exactly
seven reviewed measurement/test/manifest files. Production bytes are unchanged.
The unpublished product has no configured remote; no repository or release was
created. The next task implements the actual R trial and paired driver. This
acceptance is not an executed performance trial or a complete inferCNV product.

## Task 2 implementation and review

The five R/Python files are implemented. The controller independently verified
94 passing Python tests and architecture consistency, and streamed all three R
sources through existing R 4.6.1 syntax parsing on 4090 using RAM-backed
`TMPDIR=/dev/shm`. No source was copied there, no package was installed, and
infercnv is absent from that diagnostic runtime. Actual R semantics remain
unverified until the pinned hosted package run.

Independent review requested two corrections before acceptance: pass
`bundle/full` to the Rust case reader (bundle root remains correct for R), and
terminate the entire GNU-time/trial process group on timeout. Regression tests
and both bounded fixes now pass scoped re-review. The controller independently
reran all 12 focused tests successfully; the implementer reports all 96 Python
tests and architecture checks passing. The plan explicitly substitutes owned
`Popen` sessions and checked waits for its earlier `subprocess.run` prescription.
Post-warm/post-call thread observations are a deferred minor for final review.
Exact-head Control plane validation is pending, including checking whether the
hosted job supplies the test suite's required TMPDIR.

Both new prerequisite receipts were independently checked against accepted
original bytes and metadata with no discrepancy. They bind the accepted full
input and the four-native-tested `green-7` source, not timing results. Actual R
negative tests, smoke, seven measured pairs and their original-artifact audit
remain outstanding; nothing has been published or claimed faster.
