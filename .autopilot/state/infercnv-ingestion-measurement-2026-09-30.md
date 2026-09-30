# Canonical raw-ingestion measurement ledger

Status: exact controller helper implemented, independently reviewed and
accepted by exact-head CI. No actual timed factory or speed/resource claim.

Design:
`../../docs/plans/2026-09-30-infercnv-raw-ingestion-measurement-design.md`.
First independently testable controller step:
`../../docs/plans/2026-09-30-infercnv-creation-export-checker-plan.md`.

The helper verifies both warm/measured exports independently against stage 1:
binary64 count bits including signed zero, literal TSV identities/coordinates,
roles and ordered maps. Each phase must contain exactly four regular files;
symlinks, directories and missing/extra files fail. Trial-root metadata remains
outside this helper and belongs to the subsequent complete-trial driver.
Source/input/receipt pins and timings are not established by this helper.

## Test and review evidence

- Eleven tests first failed explicitly because the checker file was absent,
  then passed after implementation.
- Three genuine phase-entry mutation tests failed before the inventory guard:
  empty directory, directory symlink and broken symlink; all passed afterward.
- Final focused suite: 21 tests. Full controller suite: 223 tests, passed by
  both worker and root with external TMPDIR and bytecode disabled.
- Literal quoted IDs/groups remain literal; added CSV quotes or quoted numbers
  cannot alias the expected values. The existing generic CSV table reader is
  intentionally not reused for this different literal TSV contract.
- Root smoke-checked both original plain/gzip creation exports from run
  `36703475039` against its shipped full stage-1 tables and witnessed maps:
  9939 genes × 184 cells; four hashes exactly match accepted creation exports.
  This only exercises checker loading/comparison; it is not a timing run or
  acceptance of that run's incomplete samples/Ward evidence package.

Frozen controller SHA256:

| File | SHA256 |
| --- | --- |
| `scripts/validate_infercnv_ingestion_measurement.py` | `0fb1c968e46ff4dd041a42907ab70efaee9fae8beadf1254c81b1edf3c19cb36` |
| `scripts/test_validate_infercnv_ingestion_measurement.py` | `da5f3d79593cabbe8b0c9337f4fdb6fbe7a05efb28fef40768a4f382cebae879` |

Product `rsomics-sc` is still clean at
`e62fc960aeeb9dc543298885213cda6a57196fe9`; production Rust, Cargo.lock and
old prepared measurement support have not changed. No local build is permitted
while boot APFS remains above the 80% gate. Controller scratch stays external.

Fresh independent code review approved this exact-state helper after reading
both complete files and independently passing 21 focused and 223 controller
tests. Review accepted only the helper contract, not a source-pinned or timed
measurement. Its frozen hashes remain unchanged.

Controller commit `afb398e992664adfbeaa5a7b89c4cfb7a55e7800` is pushed to main.
Exact-head Control plane run `36710205465` completed successfully. This gate
accepts the helper alone, not a timed trial or product performance.

## Remaining gates

1. Complete: independent helper review, main push and exact-head controller CI.
2. Private Rust creation comparison/export support with its own implementation
   plan, test-first failures and four-native debug/release gate.
3. Actual installed-package factory trial, source/runtime pins, paired process
   driver, immutable input/candidate receipts and complete provenance tests.
4. Thirty-two fresh processes yielding 64 checked factory outputs; 28 measured
   samples with frozen plain/gzip strict wall advantage criteria.
5. Original artifact/log/metrics/source audit before any bounded performance
   acceptance. Full inferCNV and publication remain separate.

## Fresh actual-factory source review

Root read the actual pinned archive factory, reduction and object validator;
an independent read-only reviewer verified their input/source hashes against
original run `36707344893`. Creation functions need a new installed-body/formals
check: the accepted samples witness pins four different run/cluster functions.
All three factory functions are in `R/inferCNV.R`, SHA256
`2b25f5fdf9665f382073255e44729ff38cca32f3862c158f91ebf58d43e475a2`:
`CreateInfercnvObject`, `.order_reduce` and `validate_infercnv_obj`.

The canonical plain count file is 36,378,958 bytes, SHA256
`539ea675832047cc77dc550c648dd421f2f5370aa2e7b52e0907b7dc690237c5`.
Its existing witnessed gzip is 4,407,153 bytes, SHA256
`489e4ea3ae98886eee1fbba5c01484b1ba0f10989d5e79b4c01873a0f1e15e12`.
Use those original bytes; do not regenerate gzip for the measurement.

The [input-only receipt](../oracles/infercnv-ingestion-measurement-input-2026-09-30.json)
freezes those cases, auxiliary files, exact expected state/maps/checkpoint,
factory source and accepted original artifact. Root verified every physical
hash/size against that receipt and streamed gzip to verify its exact decoded
size/hash equals plain. This is input authentication, not trial acceptance.

Keep intrinsic `digest(raw.data)` and object validation inside the timer.
Resolve absolute input/source/expected paths first, then run from the new
trial output directory: the upstream validator writes `broken.infercnv_obj`
to its working directory on a failure. Do not silence actual vanilla logger
advisories by changing scipen/logging to improve a measurement.

Warm validation/export runs in a local scope; retain only scalar/storage
metadata afterward. Drop both warm and expected objects and run full GC before
baseline RSS. Do not retain the old prepared benchmark's `prepared_identity`
list, which deliberately holds matrix references for a different lifecycle.
After the measured factory returns, collect retained-result RSS before any
oracle reload/comparison, hash/export, delta formatting or reclamation.
R options contain Inf: preserve typed options or dput text, not an implicit
JSON null. Record actual class/type and clock resolution without inferring
double storage, duplicate allocation, or nanosecond accuracy.

Private Rust support is specified in
`../../docs/plans/2026-09-30-infercnv-private-creation-support-design.md`.
Fresh independent spec review found no load-bearing gap after checking the
actual typed state/group constructor and old fixture/export contracts.
No factory trial has run and no new performance claim follows this review.

## Private Rust support test-first freeze

Root read the complete helper and test proposals, reused the product's checked
count constructor instead of maintaining duplicate invariants, and saved the
complete exact-code [plan](../../docs/plans/2026-09-30-infercnv-private-creation-support-plan.md).
The product now has only the new opt-in test and Cargo.toml registration;
the helper remains absent until the genuine remote RED is inspected.
Rust 1.91 standalone formatting passed without any local Cargo execution.

Candidate `red-7`: 145 source files, archive SHA256
`74025cd4c05f992d8521a1529841f70b359b300acf033b758b4268095d32cf2d`,
manifest SHA256
`0c83801bf5af66403e5da1a8f3ab9d2e45d8488d1a7d3d8021b864ddfff84626`.
All accepted green-10 source hashes excluding only changed Cargo.toml still
match, including production/old tests/support/bench and Cargo.lock.
The new test SHA256 is
`e9d9d450cb701c6ba03301dca0b4aeb31c6733d9634234ff9fd60f9216729df2`.

Candidate workflow adds only a default-false ingestion-support selector,
requires the existing measurement flag and runs the dedicated test in debug
and release. YAML, all 12 Bash blocks and the actual four-case selector guard
passed; all 223 controller tests and architecture checks passed. Native RED
dispatch, original failure inspection and four-native GREEN remain pending.

## Genuine remote RED accepted

World `dc4a051e9a5a70f26ed15e42095ec0fff8267892` passed exact-head control run
`36715756810`. Native test-first run `36715831817`, attempt 1, failed only the
new ingestion-support debug step after valid Linux x86_64 source/dependency
metadata. Ordinary debug tests passed 69 without the three external-oracle
tests; old measurement support passed 11. Source-before/after, archive/lock,
target-directory and upload steps succeeded. The helper is absent from the
original 145-file source snapshot; new test bytes match the frozen hash.

Original APIs, logs and ZIP are retained at
`/Volumes/Zane's HDD/rsomics-fixtures/evidence/infercnv-ingestion-measurement-2026-09-30/run-36715831817/`.
Artifact `11096815340` is 624,670 bytes, SHA256
`b3699deaf56eb4d07191db2f233cf3a072ce6b1c289a34e7eb862e41ddd30cec`.
Original logs SHA256:
`447ae5fb3bc059f90c52b460e538d876838d7662075864476635a93d82e2a4f7`.
Root verified API status/head/digest/size, CRC/safe unique ZIP paths, all source
checks, archive/manifest/lock hashes and exact compiler diagnostic.

Ruling: the plan predicted E0583, but the explicit Rust `#[path]` attribute
produces `couldn't read tests/../benches/ingestion_support/mod.rs` with OS error
2 instead. Accept only that exact missing-helper diagnostic plus the verified
successful prerequisites, not a generic compilation failure. This corrects
the plan's diagnostic spelling; no code, source or gate is waived. Cost if
wrong would be accepting an unrelated compile failure, prevented by original
path/location, one-error count, source and dependency checks. Original RED
README and original upload remain preserved unchanged.

Fresh independent complete-code review found no critical/important defect and
approved code only. One minor test gap is deferred: existing regular-file
output target is not a separate sentinel test, although `create_dir` rejects
it before writing. Existing empty/nonempty directories, repeated export and
missing-parent tests cover the known tested preservation paths. The injected
callback error after a successful flush is not an OS flush-failure proof.
Four-native GREEN and actual timing remain pending.

## Private implementation frozen for native GREEN

After accepting the exact missing-helper RED, root added only the reviewed
private helper. External Rust 1.91 formatting passes for both new files.
All green-10 source hashes excluding Cargo.toml still match. No local build,
production change, old measurement change or Cargo.lock change occurred.

Candidate `green-11` contains 146 source files. Archive SHA256:
`cce4714dc2364f0747c7034e15e9cfc91f6d35a26f220df88011c53e0f93e54b`.
Manifest SHA256:
`7c67f2b40cf30051539ca6250dae23b3af2912335d386801f47afc27f399adba`.
The new test and Cargo registration remain byte-identical to red-7; helper
SHA256 is `62c66bd97dc855614fcbf906f3559316ca030e2fd556e7db277cf65d64252b60`.
Archive/checksum verification passed. Independent code review covers these
same helper/test bytes, not an untested subsequent edit. Native correctness,
four-target strict Clippy/format gates and original artifact inspection are
still pending; there is still no timed factory or performance claim.

Controller recheck first passed 222 tests and failed the pre-existing wrapped
descendant timeout test with macOS `killpg(SIGKILL)` PermissionError after
termination. Root found no surviving named descendant, read the complete
process cleanup/test path, and reran the unchanged test successfully. The
unchanged complete 223-test suite then passed, retained at external scratch
`rsomics-ingestion-support-proposal-20260930/controller-green11-recheck.log`.
No skip, weakened assertion or controller-code change was used. Architecture
and whitespace checks pass; exact-head hosted CI is still required.

## Native green-11 code tests pass; archive packaging rejected

World `9bdf3952156f6ad6c108fbaffaa7019176b23c2f` passed control run
`36717967981`. Native run `36718150977`, attempt 1, succeeded on Linux/macOS
x86_64/aarch64. Each target/profile passed 55 private creation-support tests,
11 old measurement tests and 72 ordinary tests including three full external
oracle tests. Linux format/strict all-feature Clippy and missing-oracle failure
check passed. Old bench checks passed on all targets.

Root retained all four original artifact API replies/ZIPs, run/jobs APIs and
original log ZIP externally under `run-36718150977`. ZIP API digests/sizes,
safe unique inventories/CRCs, source before/after, actual archive/manifest/lock,
native rustc 1.91 hosts and original successful test summaries were verified.
Original logs SHA256:
`f3a602b30be0d31ed195b34dd3d4d429944e4e6f11be07e0e046f34664c56bfb`.

The subsequent strict tar inventory assertion failed: root's default macOS
tar produced 292 regular members, 146 of them undeclared AppleDouble xattr
metadata. red-7 similarly has 145 extra metadata entries; older green-10 and
accepted measurement green-5 do not. Both new checksum files also omitted
README.md, which the existing downstream snapshot verifier requires.
This is root's source-packaging error, not a production algorithm failure or
source hash disagreement. Do not accept green-11 as a downstream measurement
archive or ignore extras in the strict verifier. Preserve both original
snapshots/uploads; rebuild clean green-12 with identical 146 source bytes,
authenticate all three snapshot files and add a regression-tested reusable
pre-extraction source guard before fresh four-native dispatch/acceptance.

## Shared source guard and clean green-12

Root preserved genuine missing-module RED before adding the shared private
guard. Focused GREEN passes 19 tests; the unchanged old verifier suite passes
12; all 242 controller tests and architecture/whitespace checks pass. YAML
and all 12/22 Bash blocks in the two modified workflows pass. Complete raw
logs are retained under external scratch
`rsomics-source-freeze-guard-proposal-20260930/root-*.log`.

The candidate gate now verifies the same strict logical archive contract as
the measured-source receipt verifier before Rust setup. It preserves copied
guard/test bytes and hashes in the uploaded evidence. The oracle workflow
also includes the new imported helper in its existing controller-code hashes.
No receipt schema, timer, Cargo policy, production or numerical rule changes.

Guard SHA256: `095ebb67a2d9a050c20b6e876f2b0c9cfd26fcbf8f0ce3601a8124b292b553f2`.
Tests SHA256: `4034bd9fde999458c900a6e7d0eff72e1607e6f555bf869d4b68bbcde5baf2fe`.
Root verified green-5 through the generic guard and the active accepted
prepared receipt's green-7 through the full updated verify_snapshot entry,
using original saved native-run/jobs APIs. Both pass unchanged. Original
green-11 rejects its unsorted two-entry checksums; its original tar separately
rejects undeclared logical members. The sorted two-entry mutation test proves
missing README rejection without rewriting original evidence.

Clean green-12 was packaged with COPYFILE_DISABLE=1, --no-xattrs and ustar.
Its 146 manifest/source hashes are byte-identical to green-11; only archive
representation and snapshot documentation/checksums differ. Archive SHA256:
`a5dc9b50f9e521e70351d77506ed1de3ccc15b394475dd39f7a00f7c5a97fd90`.
Manifest SHA256:
`7c67f2b40cf30051539ca6250dae23b3af2912335d386801f47afc27f399adba`.
README SHA256:
`605bb074544fd8a2b1b618a523d9607cd0d2154a8e39aa17b63c927a910c55d3`.
All three sorted checksum entries and exact 146 logical regular members pass
the actual new guard. Fresh exact-head controller and four-native candidate
gates remain pending; green-11 is not retroactively accepted.

Independent proposal review found no critical/important defect. A minor
coverage extension is deferred: PAX representation is tested with mtime,
not a separate overridden final-path/long-path case; actual logical names
are checked after tarfile resolution. This is not a physical-header audit.

Fresh independent review of the actual owning-world integration also found
no critical/important defect. It verified unchanged receipt/run/Cargo/
post-trial ASTs, both workflow gates and exact clean archive/member bytes.
Its approval is code-level only, not an executed native or performance gate.
